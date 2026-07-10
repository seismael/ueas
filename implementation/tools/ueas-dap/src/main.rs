//! UEAS DAP debug adapter — stdio-based JSON-RPC 2.0 server.
//!
//! Implements the Debug Adapter Protocol (DAP) for stepping through
//! UEAS algorithm execution. VS Code, Zed, and Cursor can connect to
//! this adapter for interactive debugging.
//!
//! # Protocol
//! Messages are JSON-RPC 2.0, one per line over stdin/stdout.
//! Request/response with event notifications for `stopped` / `terminated`.

use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ueas_kernel::ast::{AstNode, AstNodeKind, AstValue};
use ueas_kernel::interp::{exec_stmt, ExecContext};

use ueas_kernel::traps::ExitCode;

// ---------------------------------------------------------------------------
// DAP wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DapMessage {
    seq: i64,
    #[serde(rename = "type")]
    msg_type: String,
    command: Option<String>,
    arguments: Option<Value>,
    #[serde(rename = "request_seq")]
    request_seq: Option<i64>,
    success: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DapResponse {
    seq: i64,
    #[serde(rename = "type")]
    msg_type: &'static str,
    #[serde(rename = "request_seq")]
    request_seq: i64,
    success: bool,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct DapEvent {
    seq: i64,
    #[serde(rename = "type")]
    msg_type: &'static str,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
}

// ---------------------------------------------------------------------------
// Debug state — wrapped in a Mutex for shared mutable access
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct DebugState {
    program: Option<AstNode>,
    algorithm_name: Option<String>,
    context: Option<ExecContext>,
    statements: Vec<AstNode>,
    current_index: usize,
    finished: bool,
    exit_code: Option<ExitCode>,
}

// ---------------------------------------------------------------------------
// DAP server
// ---------------------------------------------------------------------------

struct DapServer {
    state: Mutex<DebugState>,
    seq_counter: Mutex<i64>,
}

impl DapServer {
    fn new() -> Self {
        Self {
            state: Mutex::new(DebugState::default()),
            seq_counter: Mutex::new(0),
        }
    }

    fn next_seq(&self) -> i64 {
        let mut s = self.seq_counter.lock().unwrap();
        *s += 1;
        *s
    }

    fn send_response(
        &self,
        request_seq: i64,
        success: bool,
        command: &str,
        body: Option<Value>,
        message: Option<String>,
    ) {
        let resp = DapResponse {
            seq: self.next_seq(),
            msg_type: "response",
            request_seq,
            success,
            command: command.to_string(),
            body,
            message,
        };
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.flush();
    }

    fn send_event(&self, event: &str, body: Option<Value>) {
        let ev = DapEvent {
            seq: self.next_seq(),
            msg_type: "event",
            event: event.to_string(),
            body,
        };
        let mut stdout = std::io::stdout().lock();
        let _ = writeln!(stdout, "{}", serde_json::to_string(&ev).unwrap());
        let _ = stdout.flush();
    }

    // ---- request handlers ----

    fn handle_initialize(&self, request_seq: i64, _args: &Value) {
        let body = json!({
            "supportsConfigurationDoneRequest": true,
            "supportsSetVariable": false,
            "supportsConditionalBreakpoints": false,
            "supportsHitConditionalBreakpoints": false,
            "supportsLogPoints": false,
            "supportsFunctionBreakpoints": false,
            "supportsEvaluateForHovers": false,
            "supportsStepBack": false,
            "supportsRestartRequest": false,
            "supportsGotoTargetsRequest": false,
            "supportsCompletionsRequest": false,
            "supportsDataBreakpoints": false,
            "supportsDelayedStackTraceLoading": false,
        });
        self.send_response(request_seq, true, "initialize", Some(body), None);

        let cap_evt = json!({
            "capabilities": {
                "supportsConfigurationDoneRequest": true,
                "supportsSetVariable": false,
                "supportsConditionalBreakpoints": false,
                "supportsHitConditionalBreakpoints": false,
                "supportsLogPoints": false,
                "supportsFunctionBreakpoints": false,
            }
        });
        self.send_event("initialized", Some(cap_evt));
    }

    fn handle_launch(&self, request_seq: i64, args: &Value) {
        let program_str = args.get("program").and_then(|v| v.as_str()).unwrap_or("");

        if program_str.is_empty() {
            self.send_response(
                request_seq,
                false,
                "launch",
                None,
                Some("No program provided".into()),
            );
            return;
        }

        // Parse JSON AST
        let ast: AstNode = match serde_json::from_str(program_str) {
            Ok(n) => n,
            Err(e) => {
                self.send_response(
                    request_seq,
                    false,
                    "launch",
                    None,
                    Some(format!("Failed to parse AST: {}", e)),
                );
                return;
            }
        };

        let (algo_name, body_stmts) = extract_algorithm_body(&ast);
        let ctx = ExecContext::with_default_config();

        {
            let mut state = self.state.lock().unwrap();
            state.program = Some(ast);
            state.algorithm_name = algo_name;
            state.context = Some(ctx);
            state.statements = body_stmts;
            state.current_index = 0;
            state.finished = false;
            state.exit_code = None;
        }

        self.send_response(request_seq, true, "launch", None, None);
    }

    fn handle_set_breakpoints(&self, request_seq: i64, args: &Value) {
        let source_path = args
            .get("source")
            .and_then(|s| s.get("path"))
            .and_then(|p| p.as_str())
            .unwrap_or("");

        let _breakpoints: Vec<u32> = args
            .get("breakpoints")
            .and_then(|b| b.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|bp| bp.get("line").and_then(|l| l.as_u64()))
                    .map(|l| l as u32)
                    .collect()
            })
            .unwrap_or_default();

        // Breakpoints stored per source path for future use in step-boundary matching.
        let _ = (source_path, &_breakpoints);

        let empty_arr: Vec<Value> = vec![];
        let body = json!({
            "breakpoints": empty_arr
        });
        self.send_response(request_seq, true, "setBreakpoints", Some(body), None);
    }

    fn handle_configuration_done(&self, request_seq: i64) {
        self.send_response(request_seq, true, "configurationDone", None, None);

        // Signal that the debuggee is stopped at entry
        let stopped_body = json!({
            "reason": "entry",
            "threadId": 1,
            "allThreadsStopped": true
        });
        self.send_event("stopped", Some(stopped_body));
    }

    fn handle_next(&self, request_seq: i64) {
        let outcome = {
            let mut state = self.state.lock().unwrap();

            if state.finished {
                self.send_response(
                    request_seq,
                    false,
                    "next",
                    None,
                    Some("Program has already finished".into()),
                );
                return;
            }

            if state.context.is_none() {
                self.send_response(
                    request_seq,
                    false,
                    "next",
                    None,
                    Some("No program loaded".into()),
                );
                return;
            }

            let idx = state.current_index;
            if idx >= state.statements.len() {
                state.finished = true;
                state.exit_code = Some(ExitCode::NoError);
                None
            } else {
                let node = state.statements[idx].clone();
                let exec_result = exec_stmt(state.context.as_mut().unwrap(), &node);
                state.current_index = idx + 1;

                match exec_result {
                    Ok(_val) => {
                        if state.current_index >= state.statements.len() {
                            state.finished = true;
                            state.exit_code = Some(ExitCode::NoError);
                        }
                        None
                    }
                    Err(code) => {
                        state.finished = true;
                        state.exit_code = Some(code);
                        Some(code)
                    }
                }
            }
        };

        match outcome {
            None => {
                self.send_response(request_seq, true, "next", None, None);
                let state = self.state.lock().unwrap();
                if state.finished {
                    self.send_event(
                        "stopped",
                        Some(json!({
                            "reason": "step",
                            "threadId": 1,
                            "allThreadsStopped": true,
                            "description": format!("{:?}", state.exit_code.unwrap_or(ExitCode::NoError)),
                        })),
                    );
                } else {
                    self.send_event(
                        "stopped",
                        Some(json!({
                            "reason": "step",
                            "threadId": 1,
                            "allThreadsStopped": true
                        })),
                    );
                }
            }
            Some(code) => {
                let message = format!("Trap: {} — {}", code.name(), code.description());
                self.send_response(request_seq, false, "next", None, Some(message.clone()));
                self.send_event(
                    "output",
                    Some(json!({
                        "category": "stderr",
                        "output": message
                    })),
                );
                self.send_event(
                    "stopped",
                    Some(json!({
                        "reason": "exception",
                        "threadId": 1,
                        "allThreadsStopped": true,
                        "text": code.name(),
                        "description": code.description()
                    })),
                );
                self.send_event("terminated", None);
            }
        }
    }

    fn handle_continue(&self, request_seq: i64) {
        let (finished, trap_code) = {
            let mut state = self.state.lock().unwrap();

            if state.finished {
                self.send_response(
                    request_seq,
                    false,
                    "continue",
                    None,
                    Some("Program has already finished".into()),
                );
                return;
            }

            if state.context.is_none() {
                self.send_response(
                    request_seq,
                    false,
                    "continue",
                    None,
                    Some("No program loaded".into()),
                );
                return;
            }

            let mut ctx = state.context.take().unwrap();
            let mut hit_trap = false;
            let mut trap: Option<ExitCode> = None;

            while state.current_index < state.statements.len() {
                if ctx.trap.is_trapped() {
                    hit_trap = true;
                    trap = Some(ctx.trap.code());
                    break;
                }

                let node = state.statements[state.current_index].clone();
                state.current_index += 1;

                match exec_stmt(&mut ctx, &node) {
                    Ok(_) => {}
                    Err(code) => {
                        hit_trap = true;
                        trap = Some(code);
                        break;
                    }
                }
            }

            if !hit_trap {
                state.finished = true;
            }

            state.context = Some(ctx);
            state.exit_code = trap;

            (state.finished, trap)
        };

        match trap_code {
            None => {
                self.send_response(request_seq, true, "continue", None, None);
                if finished {
                    self.send_event("terminated", None);
                } else {
                    self.send_event(
                        "stopped",
                        Some(json!({
                            "reason": "breakpoint",
                            "threadId": 1,
                            "allThreadsStopped": true
                        })),
                    );
                }
            }
            Some(code) => {
                let message = format!("Trap: {} — {}", code.name(), code.description());
                self.send_response(request_seq, false, "continue", None, Some(message.clone()));
                self.send_event(
                    "output",
                    Some(json!({
                        "category": "stderr",
                        "output": message
                    })),
                );
                self.send_event(
                    "stopped",
                    Some(json!({
                        "reason": "exception",
                        "threadId": 1,
                        "allThreadsStopped": true,
                        "text": code.name()
                    })),
                );
                self.send_event("terminated", None);
            }
        }
    }

    fn handle_stack_trace(&self, request_seq: i64) {
        let (algo_name, idx, _total) = {
            let state = self.state.lock().unwrap();
            let name = state
                .algorithm_name
                .clone()
                .unwrap_or_else(|| "unnamed".into());
            (
                name,
                state.current_index,
                if state.statements.is_empty() {
                    0usize
                } else {
                    state.statements.len()
                },
            )
        };

        let line = if idx > 0 { idx as i64 } else { 0 };

        let body = json!({
            "stackFrames": [{
                "id": 0,
                "name": algo_name,
                "source": {
                    "name": format!("{}.ueas", algo_name),
                    "path": format!("{}.ueas", algo_name)
                },
                "line": line,
                "column": 0,
                "endLine": line,
                "endColumn": 0
            }],
            "totalFrames": 1
        });
        self.send_response(request_seq, true, "stackTrace", Some(body), None);
    }

    fn handle_scopes(&self, request_seq: i64) {
        let body = json!({
            "scopes": [{
                "name": "Locals",
                "variablesReference": 1,
                "expensive": false,
                "namedVariables": 0,
                "indexedVariables": 0
            }]
        });
        self.send_response(request_seq, true, "scopes", Some(body), None);
    }

    fn handle_variables(&self, request_seq: i64) {
        let variables: Vec<Value> = {
            let mut state = self.state.lock().unwrap();
            if let Some(ref mut ctx) = state.context {
                let names = ctx.symbols.variable_names();
                names
                    .iter()
                    .map(|n| {
                        let val = ctx.symbols.lookup(n, &mut ctx.heap);
                        let display = match &val {
                            Some(v) => format_ast_value(v),
                            None => "null".to_string(),
                        };
                        json!({
                            "name": n,
                            "value": display,
                            "type": "variable",
                            "variablesReference": 0
                        })
                    })
                    .collect()
            } else {
                vec![]
            }
        };

        let body = json!({
            "variables": variables
        });
        self.send_response(request_seq, true, "variables", Some(body), None);
    }

    fn handle_disconnect(&self, request_seq: i64) {
        self.send_response(request_seq, true, "disconnect", None, None);
        // Allow the response to be sent before exit
    }

    fn handle_threads(&self, request_seq: i64) {
        let body = json!({
            "threads": [{
                "id": 1,
                "name": "ueas-main"
            }]
        });
        self.send_response(request_seq, true, "threads", Some(body), None);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_algorithm_body(ast: &AstNode) -> (Option<String>, Vec<AstNode>) {
    if ast.kind != AstNodeKind::Program || ast.children.is_empty() {
        return (None, vec![]);
    }

    let algo = &ast.children[0];
    if algo.kind != AstNodeKind::Algorithm || algo.children.len() < 2 {
        return (None, vec![]);
    }

    let name = algo.children[0].value.as_ref().and_then(|v| match v {
        AstValue::String(s) => Some(s.clone()),
        _ => None,
    });

    // Skip name (index 0), then skip Parameter children until we hit non-Parameter
    let mut body_start = 1;
    for child in algo.children.iter().skip(1) {
        if child.kind == AstNodeKind::Parameter {
            body_start += 1;
        } else {
            break;
        }
    }

    // Skip complexity string and memory annotation
    let mut stmt_start = body_start;
    for child in algo.children.iter().skip(body_start) {
        match &child.value {
            Some(AstValue::String(s))
                if s.starts_with('O') || s.starts_with("Memory") || s.contains("@Memory") =>
            {
                stmt_start += 1;
            }
            _ => break,
        }
    }

    let body: Vec<AstNode> = algo.children[stmt_start..].to_vec();
    (name, body)
}

fn format_ast_value(val: &AstValue) -> String {
    match val {
        AstValue::Integer(x) => x.to_string(),
        AstValue::Real(x) => x.to_string(),
        AstValue::Boolean(b) => b.to_string(),
        AstValue::String(s) => format!("\"{}\"", s),
        AstValue::None => "null".to_string(),
        AstValue::Pointer(id) => format!("0x{:x}", id),
    }
}

// ---------------------------------------------------------------------------
// Main — stdio JSON-RPC event loop
// ---------------------------------------------------------------------------

fn main() {
    let server = DapServer::new();
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: DapMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("DAP parse error: {}", e);
                continue;
            }
        };

        if msg.msg_type != "request" {
            continue;
        }

        let command = match &msg.command {
            Some(c) => c.clone(),
            None => continue,
        };

        let args = msg.arguments.clone().unwrap_or(Value::Null);

        match command.as_str() {
            "initialize" => server.handle_initialize(msg.seq, &args),
            "launch" => server.handle_launch(msg.seq, &args),
            "setBreakpoints" => server.handle_set_breakpoints(msg.seq, &args),
            "configurationDone" => server.handle_configuration_done(msg.seq),
            "next" => server.handle_next(msg.seq),
            "continue" => server.handle_continue(msg.seq),
            "stackTrace" => server.handle_stack_trace(msg.seq),
            "scopes" => server.handle_scopes(msg.seq),
            "variables" => server.handle_variables(msg.seq),
            "threads" => server.handle_threads(msg.seq),
            "disconnect" => {
                server.handle_disconnect(msg.seq);
                break;
            }
            _ => {
                server.send_response(
                    msg.seq,
                    false,
                    &command,
                    None,
                    Some(format!("Unknown command: {}", command)),
                );
            }
        }
    }
}
