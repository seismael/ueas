//! UEAS Debug Adapter Protocol (DAP) server with `Content-Length` framing, breakpoints, and `stepBack`.
#![allow(missing_docs)]

use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
use ueas_hir::check::linker::NullModuleResolver;
use ueas_hir::compile_to_hir;
use ueas_vm::{compile_program, StepOutcome, VmConfig, VmSession};

/// Run the DAP server over stdin/stdout with standard `Content-Length` framing.
pub fn run_dap_stdio() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    let mut session: Option<VmSession> = None;
    let mut program_path = String::new();
    let mut entry_chunk = String::new();
    let mut seq = 1u64;

    while let Some(msg) = read_dap_message(&mut reader)? {
        let req_seq = msg.get("seq").and_then(Value::as_u64).unwrap_or(0);
        let cmd = msg.get("command").and_then(Value::as_str).unwrap_or("");
        let args = msg.get("arguments").cloned().unwrap_or_else(|| json!({}));

        match cmd {
            "initialize" => {
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({
                        "supportsConfigurationDoneRequest": true,
                        "supportsStepBack": true
                    }),
                )?;
                send_event(&mut writer, &mut seq, "initialized", json!({}))?;
            }
            "launch" => {
                program_path = args
                    .get("program")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let src = std::fs::read_to_string(&program_path)?;
                let prog = compile_to_hir(&src, &NullModuleResolver)
                    .map_err(|e| anyhow::anyhow!("HIR compile error: {}", e.join("; ")))?;
                let compiled =
                    compile_program(&prog).map_err(|e| anyhow::anyhow!("VM compile error: {e}"))?;
                entry_chunk = args
                    .get("entry")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| compiled.tests.first().map(|(n, _)| format!("test::{n}")))
                    .or_else(|| compiled.chunks.first().map(|c| c.name.clone()))
                    .ok_or_else(|| {
                        anyhow::anyhow!("no runnable algorithm or @Test in {program_path}")
                    })?;
                let vm = VmSession::new(
                    compiled,
                    &entry_chunk,
                    Vec::new(),
                    VmConfig {
                        max_snapshots: 512,
                        ..VmConfig::default()
                    },
                )?;
                session = Some(vm);
                send_response(&mut writer, &mut seq, req_seq, cmd, json!({}))?;
            }
            "setBreakpoints" => {
                let mut bps = HashSet::new();
                if let Some(arr) = args.get("breakpoints").and_then(Value::as_array) {
                    for bp in arr {
                        if let Some(line) = bp.get("line").and_then(Value::as_u64) {
                            bps.insert((entry_chunk.clone(), line as u32));
                        }
                    }
                }
                if let Some(vm) = session.as_mut() {
                    vm.set_breakpoints(bps.clone());
                }
                let verified: Vec<Value> = bps
                    .into_iter()
                    .map(|(_, line)| json!({ "verified": true, "line": line }))
                    .collect();
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({ "breakpoints": verified }),
                )?;
            }
            "configurationDone" => {
                send_response(&mut writer, &mut seq, req_seq, cmd, json!({}))?;
                send_event(
                    &mut writer,
                    &mut seq,
                    "stopped",
                    json!({ "reason": "entry", "threadId": 1 }),
                )?;
            }
            "continue" => {
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({ "allThreadsContinued": true }),
                )?;
                if let Some(vm) = session.as_mut() {
                    loop {
                        match vm.step()? {
                            StepOutcome::Continue => {}
                            StepOutcome::BreakpointHit { .. } => {
                                send_event(
                                    &mut writer,
                                    &mut seq,
                                    "stopped",
                                    json!({ "reason": "breakpoint", "threadId": 1 }),
                                )?;
                                break;
                            }
                            StepOutcome::Halted(_) => {
                                send_event(&mut writer, &mut seq, "terminated", json!({}))?;
                                break;
                            }
                        }
                    }
                }
            }
            "next" | "stepIn" => {
                send_response(&mut writer, &mut seq, req_seq, cmd, json!({}))?;
                if let Some(vm) = session.as_mut() {
                    match vm.step()? {
                        StepOutcome::Halted(_) => {
                            send_event(&mut writer, &mut seq, "terminated", json!({}))?;
                        }
                        _ => {
                            send_event(
                                &mut writer,
                                &mut seq,
                                "stopped",
                                json!({ "reason": "step", "threadId": 1 }),
                            )?;
                        }
                    }
                }
            }
            "stepBack" => {
                send_response(&mut writer, &mut seq, req_seq, cmd, json!({}))?;
                if let Some(vm) = session.as_mut() {
                    let _ = vm.step_back();
                    send_event(
                        &mut writer,
                        &mut seq,
                        "stopped",
                        json!({ "reason": "step", "threadId": 1 }),
                    )?;
                }
            }
            "threads" => {
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({ "threads": [{ "id": 1, "name": "ueas-vm-main" }] }),
                )?;
            }
            "stackTrace" => {
                let frames: Vec<Value> = session
                    .as_ref()
                    .map(|vm| {
                        vm.frames()
                            .iter()
                            .enumerate()
                            .rev()
                            .map(|(idx, f)| {
                                let chunk = &vm.program().chunks[f.chunk_idx];
                                let line = chunk
                                    .spans
                                    .get(f.pc)
                                    .and_then(|s| s.as_ref())
                                    .map_or(1, |s| s.start_line);
                                json!({
                                    "id": idx + 1,
                                    "name": chunk.name,
                                    "source": { "path": program_path },
                                    "line": line,
                                    "column": 1
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({ "stackFrames": frames, "totalFrames": frames.len() }),
                )?;
            }
            "scopes" => {
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({
                        "scopes": [{ "name": "Locals", "variablesReference": 1, "expensive": false }]
                    }),
                )?;
            }
            "variables" => {
                let vars: Vec<Value> = session
                    .as_ref()
                    .and_then(|vm| {
                        vm.current_frame().map(|f| {
                            let chunk = &vm.program().chunks[f.chunk_idx];
                            f.locals
                                .iter()
                                .enumerate()
                                .map(|(i, v)| {
                                    let name = chunk
                                        .local_names
                                        .get(i)
                                        .cloned()
                                        .unwrap_or_else(|| format!("local_{i}"));
                                    json!({
                                        "name": name,
                                        "value": v.to_string(),
                                        "variablesReference": 0
                                    })
                                })
                                .collect()
                        })
                    })
                    .unwrap_or_default();
                send_response(
                    &mut writer,
                    &mut seq,
                    req_seq,
                    cmd,
                    json!({ "variables": vars }),
                )?;
            }
            "disconnect" => {
                send_response(&mut writer, &mut seq, req_seq, cmd, json!({}))?;
                break;
            }
            _ => {
                send_response(&mut writer, &mut seq, req_seq, cmd, json!({}))?;
            }
        }
    }

    Ok(())
}

pub fn read_dap_message<R: BufRead + Read>(reader: &mut R) -> anyhow::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(val) = trimmed.strip_prefix("Content-Length:") {
            content_length = val.trim().parse::<usize>().ok();
        }
    }
    let len = content_length.ok_or_else(|| anyhow::anyhow!("missing Content-Length header"))?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(Some(serde_json::from_slice(&buf)?))
}

fn write_dap_frame<W: Write>(writer: &mut W, payload: &Value) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(payload)?;
    write!(writer, "Content-Length: {}\r\n\r\n", bytes.len())?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

fn send_response<W: Write>(
    writer: &mut W,
    seq: &mut u64,
    request_seq: u64,
    command: &str,
    body: Value,
) -> anyhow::Result<()> {
    let cur = *seq;
    *seq += 1;
    write_dap_frame(
        writer,
        &json!({
            "seq": cur,
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": command,
            "body": body
        }),
    )
}

fn send_event<W: Write>(
    writer: &mut W,
    seq: &mut u64,
    event: &str,
    body: Value,
) -> anyhow::Result<()> {
    let cur = *seq;
    *seq += 1;
    write_dap_frame(
        writer,
        &json!({
            "seq": cur,
            "type": "event",
            "event": event,
            "body": body
        }),
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_content_length_dap_frame() {
        let body = r#"{"seq":1,"type":"request","command":"initialize"}"#;
        let frame = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let mut cur = Cursor::new(frame.as_bytes());
        let msg = read_dap_message(&mut cur).unwrap().unwrap();
        assert_eq!(msg["command"], "initialize");
    }
}
