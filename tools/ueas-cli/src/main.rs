//! Unified UEAS v2.1 CLI (`run`, `check`, `verify`, `verify-cert`, `transpile`, `oracle`, `fmt`, `extract`, `lsp`, `dap`, `mcp`, `ci`).
#![allow(missing_docs)]

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use ueas_codegen::{generate_target, lower_hir_to_lir, TargetKind};
use ueas_extract::extract_source;
use ueas_hir::check::linker::{ChainedModuleResolver, ModuleResolver};
use ueas_hir::{compile_to_hir, compile_to_hir_detailed, EmbeddedStdlibResolver};
use ueas_syntax::format_source;
use ueas_verify::{
    build_oracle_vectors, verify_source, VerificationCertificate, VerificationLevel, VerifyOptions,
};
use ueas_vm::{compile_program, run_hir, VmConfig, VmSession};

/// Resolves dotted module paths relative to a workspace root directory.
struct FsModuleResolver {
    root: PathBuf,
}

impl ModuleResolver for FsModuleResolver {
    fn resolve(&self, dotted_path: &str) -> Result<String, String> {
        let rel = dotted_path.replace('.', "/");
        for base in [self.root.clone(), self.root.join("stdlib")] {
            let cand = base.join(format!("{rel}.ueas"));
            if cand.exists() {
                return std::fs::read_to_string(&cand)
                    .map_err(|e| format!("failed to read {}: {e}", cand.display()));
            }
        }
        Err(format!("module '{dotted_path}' not found on filesystem"))
    }
}

fn default_resolver(
    file: &Path,
) -> ChainedModuleResolver<FsModuleResolver, EmbeddedStdlibResolver> {
    let root = file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    ChainedModuleResolver {
        primary: FsModuleResolver { root },
        fallback: EmbeddedStdlibResolver,
    }
}

#[derive(Parser)]
#[command(
    name = "ueas",
    version,
    about = "UEAS v2.1 Algorithmic Specification, Verification & Synthesis Engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Static L0 check (parse, HIR, bidirectional types, MVS, CT lattice)
    Check { file: PathBuf },
    /// Execute @Test blocks or an Algorithm in the deterministic VM
    Run {
        file: PathBuf,
        #[arg(long)]
        entry: Option<String>,
        #[arg(long, default_value_t = 2_000_000)]
        steps: u64,
    },
    /// Progressive verification (L0..L3) + SHA-256 certificate emission
    Verify {
        file: PathBuf,
        #[arg(long, default_value = "L3")]
        level: String,
        #[arg(long)]
        cert_out: Option<PathBuf>,
    },
    /// Cryptographically verify a `.ueas.cert.json` certificate against source
    VerifyCert { file: PathBuf, cert: PathBuf },
    /// Dual-track transpilation or external plugin synthesis (`--plugin <cmd>`)
    Transpile {
        file: PathBuf,
        #[arg(long, default_value = "rust")]
        target: String,
        #[arg(long)]
        plugin: Option<String>,
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Language-agnostic L4 differential oracle against any external executable
    Oracle {
        file: PathBuf,
        #[arg(long)]
        entry: String,
        #[arg(long)]
        exec: String,
        #[arg(long, default_value_t = 32)]
        samples: usize,
    },
    /// Lossless CST formatter
    Fmt {
        file: PathBuf,
        #[arg(long)]
        check: bool,
    },
    /// Extract pure UEAS from Python, TypeScript, or Rust (with Axiom 1 purity audit)
    Extract {
        file: PathBuf,
        #[arg(long)]
        lang: String,
    },
    /// Start UEAS Language Server (LSP) over stdio
    Lsp,
    /// Start UEAS Debug Adapter (DAP) over stdio
    Dap,
    /// Start UEAS MCP JSON-RPC 2.0 server over stdio
    Mcp,
    /// Verify all .ueas files recursively in a directory
    Ci { dir: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { file } => {
            let src = std::fs::read_to_string(&file)?;
            let res = default_resolver(&file);
            match compile_to_hir_detailed(&src, &res) {
                Ok(prog) => {
                    println!(
                        "OK: {} algorithms, {} tests",
                        prog.algorithms.len(),
                        prog.tests.len()
                    );
                }
                Err(errs) => {
                    for d in errs {
                        match d.span {
                            Some(sp) => eprintln!(
                                "[{}] line {}:{}: {}",
                                d.code, sp.start_line, sp.start_col, d.message
                            ),
                            None => eprintln!("[{}] {}", d.code, d.message),
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Run { file, entry, steps } => {
            let src = std::fs::read_to_string(&file)?;
            let res = default_resolver(&file);
            let prog =
                compile_to_hir(&src, &res).map_err(|e| anyhow::anyhow!("{}", e.join("\n")))?;
            let cfg = VmConfig {
                step_budget: steps,
                ..VmConfig::default()
            };
            if let Some(name) = entry {
                let out =
                    run_hir(&prog, &name, Vec::new(), cfg).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                let compiled = compile_program(&prog).map_err(|e| anyhow::anyhow!("{e}"))?;
                for (name, _) in &compiled.tests {
                    let mut sess = VmSession::new(
                        compiled.clone(),
                        &format!("test::{name}"),
                        Vec::new(),
                        cfg.clone(),
                    )?;
                    let out = sess.run()?;
                    println!(
                        "PASS @Test \"{name}\" ({} steps, peak {} bytes)",
                        out.steps, out.heap.peak_live_bytes
                    );
                }
            }
        }
        Commands::Verify {
            file,
            level,
            cert_out,
        } => {
            let src = std::fs::read_to_string(&file)?;
            let res = default_resolver(&file);
            let lvl = match level.to_ascii_uppercase().as_str() {
                "L0" => VerificationLevel::L0,
                "L1" => VerificationLevel::L1,
                "L2" => VerificationLevel::L2,
                _ => VerificationLevel::L3,
            };
            let cert = verify_source(
                &src,
                &res,
                &VerifyOptions {
                    target_level: lvl,
                    ..VerifyOptions::default()
                },
            );
            let json = serde_json::to_string_pretty(&cert)?;
            if let Some(out_path) = cert_out {
                std::fs::write(&out_path, &json)?;
            } else {
                println!("{json}");
            }
            if !cert.passed {
                std::process::exit(1);
            }
        }
        Commands::VerifyCert { file, cert } => {
            let src = std::fs::read_to_string(&file)?;
            let cert_raw = std::fs::read_to_string(&cert)?;
            let parsed_cert: VerificationCertificate = serde_json::from_str(&cert_raw)?;
            let res = default_resolver(&file);
            match parsed_cert.verify_against(&src, &res) {
                Ok(()) => println!("VALID CERTIFICATE: {}", parsed_cert.certificate_sha256),
                Err(errs) => {
                    for e in errs {
                        eprintln!("INVALID CERTIFICATE: {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Transpile {
            file,
            target,
            plugin,
            out,
        } => {
            let src = std::fs::read_to_string(&file)?;
            let res = default_resolver(&file);
            let prog =
                compile_to_hir(&src, &res).map_err(|e| anyhow::anyhow!("{}", e.join("\n")))?;
            let emitted = if let Some(cmd_str) = plugin {
                let lir = lower_hir_to_lir(&prog);
                let payload = serde_json::to_vec(&json!({ "hir": prog, "lir": lir }))?;
                let mut child = if cfg!(windows) {
                    Command::new("cmd")
                        .args(["/C", &cmd_str])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()?
                } else {
                    Command::new("sh")
                        .args(["-c", &cmd_str])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()?
                };
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(&payload)?;
                }
                let output = child.wait_with_output()?;
                if !output.status.success() {
                    anyhow::bail!("plugin '{cmd_str}' exited with status {}", output.status);
                }
                String::from_utf8(output.stdout)?
            } else {
                let tk = TargetKind::parse(&target)
                    .ok_or_else(|| anyhow::anyhow!("unknown target '{target}'"))?;
                generate_target(&prog, tk)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .code
            };
            if let Some(out_path) = out {
                std::fs::write(&out_path, emitted)?;
            } else {
                println!("{emitted}");
            }
        }
        Commands::Oracle {
            file,
            entry,
            exec,
            samples,
        } => {
            let src = std::fs::read_to_string(&file)?;
            let res = default_resolver(&file);
            let vectors = build_oracle_vectors(&src, &entry, samples, 0x5545_4153, &res)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut passed = 0usize;
            for (idx, vec) in vectors.iter().enumerate() {
                let line_in = serde_json::to_string(&vec.args)?;
                let mut child = if cfg!(windows) {
                    Command::new("cmd")
                        .args(["/C", &exec])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()?
                } else {
                    Command::new("sh")
                        .args(["-c", &exec])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()?
                };
                if let Some(mut stdin) = child.stdin.take() {
                    writeln!(stdin, "{line_in}")?;
                }
                let out = child.wait_with_output()?;
                let actual: Value = serde_json::from_slice(&out.stdout)
                    .unwrap_or_else(|_| json!(String::from_utf8_lossy(&out.stdout).trim()));
                if actual != vec.expected {
                    eprintln!(
                        "ORACLE MISMATCH at vector #{idx}: args={} expected={} actual={}",
                        line_in, vec.expected, actual
                    );
                    std::process::exit(1);
                }
                passed += 1;
            }
            println!(
                "ORACLE PASS: {passed}/{} vectors matched UEAS VM",
                vectors.len()
            );
        }
        Commands::Fmt { file, check } => {
            let src = std::fs::read_to_string(&file)?;
            let formatted = format_source(&src)
                .map_err(|d| anyhow::anyhow!("syntax error: {}", d[0].message))?;
            if check {
                if src != formatted {
                    eprintln!("{} is not formatted", file.display());
                    std::process::exit(1);
                }
            } else {
                std::fs::write(&file, formatted)?;
            }
        }
        Commands::Extract { file, lang } => {
            let src = std::fs::read_to_string(&file)?;
            let rep = extract_source(&lang, &src)?;
            println!("{}", serde_json::to_string_pretty(&rep)?);
            if !rep.axiom1_violations.is_empty() {
                std::process::exit(1);
            }
        }
        Commands::Lsp => ueas_lsp::run_lsp_stdio().await,
        Commands::Dap => ueas_dap::run_dap_stdio()?,
        Commands::Mcp => run_mcp_stdio()?,
        Commands::Ci { dir } => {
            let mut files = Vec::new();
            collect_ueas(&dir, &mut files);
            let opts = VerifyOptions {
                target_level: VerificationLevel::L1,
                ..VerifyOptions::default()
            };
            let mut failed = 0usize;
            for f in &files {
                let src = std::fs::read_to_string(f)?;
                let res = default_resolver(f);
                let cert = verify_source(&src, &res, &opts);
                if cert.passed {
                    println!("PASS {}", f.display());
                } else {
                    eprintln!("FAIL {}: {:?}", f.display(), cert.diagnostics);
                    failed += 1;
                }
            }
            if failed > 0 {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn collect_ueas(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_ueas(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("ueas") {
                out.push(p);
            }
        }
    }
}

fn mcp_tool_catalog() -> Value {
    let source_prop = json!({ "type": "string", "description": "UEAS v2.1 source code" });
    json!([
        {
            "name": "ueas_check",
            "description": "Static L0 check (parse, HIR, types, MVS, CT lattice)",
            "inputSchema": { "type": "object", "properties": { "source": source_prop }, "required": ["source"] }
        },
        {
            "name": "ueas_run",
            "description": "Execute @Test blocks or an Algorithm in the deterministic VM",
            "inputSchema": { "type": "object", "properties": { "source": source_prop, "entry": { "type": "string" } }, "required": ["source"] }
        },
        {
            "name": "ueas_verify",
            "description": "Progressive L0-L3 verification & SHA-256 certificate",
            "inputSchema": { "type": "object", "properties": { "source": source_prop, "level": { "type": "string", "enum": ["L0", "L1", "L2", "L3"] } }, "required": ["source"] }
        },
        {
            "name": "ueas_transpile",
            "description": "Dual-track synthesis across 15 targets (rust, python, ts, cpp, go, wasm, wit, smt2, dafny, lean4, verus, quint, tla, latex, typst)",
            "inputSchema": { "type": "object", "properties": { "source": source_prop, "target": { "type": "string" } }, "required": ["source", "target"] }
        },
        {
            "name": "ueas_format",
            "description": "Lossless CST formatter for UEAS source",
            "inputSchema": { "type": "object", "properties": { "source": source_prop }, "required": ["source"] }
        },
        {
            "name": "ueas_extract",
            "description": "Extract UEAS from Python/TypeScript/Rust with Axiom 1 purity audit",
            "inputSchema": { "type": "object", "properties": { "source": { "type": "string" }, "lang": { "type": "string", "enum": ["python", "typescript", "rust"] } }, "required": ["source", "lang"] }
        },
        {
            "name": "ueas_oracle_diff",
            "description": "Generate precondition-valid L4 reference input/output vectors from UEAS VM",
            "inputSchema": { "type": "object", "properties": { "source": source_prop, "entry": { "type": "string" }, "samples": { "type": "integer" } }, "required": ["source", "entry"] }
        }
    ])
}

fn run_mcp_stdio() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = serde_json::from_str(&line)?;
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => json!({
                "protocolVersion": "2025-03-26",
                "serverInfo": { "name": "ueas-mcp", "version": env!("CARGO_PKG_VERSION") },
                "capabilities": { "tools": {} }
            }),
            "tools/list" => json!({ "tools": mcp_tool_catalog() }),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
                let tool = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let src = args.get("source").and_then(Value::as_str).unwrap_or("");
                let (is_err, text) = match tool {
                    "ueas_check" => match compile_to_hir_detailed(src, &EmbeddedStdlibResolver) {
                        Ok(p) => (
                            false,
                            json!({ "ok": true, "algorithms": p.algorithms.len() }).to_string(),
                        ),
                        Err(e) => (true, json!({ "ok": false, "diagnostics": e }).to_string()),
                    },
                    "ueas_run" => {
                        let entry = args.get("entry").and_then(Value::as_str).unwrap_or("");
                        match compile_to_hir(src, &EmbeddedStdlibResolver) {
                            Ok(p) if !entry.is_empty() => {
                                match run_hir(&p, entry, Vec::new(), VmConfig::default()) {
                                    Ok(r) => (false, serde_json::to_string(&r).unwrap_or_default()),
                                    Err(e) => (true, e),
                                }
                            }
                            Ok(p) => match compile_program(&p) {
                                Ok(c) => (
                                    false,
                                    json!({ "ok": true, "tests": c.tests.len() }).to_string(),
                                ),
                                Err(e) => (true, e),
                            },
                            Err(e) => (true, e.join("\n")),
                        }
                    }
                    "ueas_verify" => {
                        let cert =
                            verify_source(src, &EmbeddedStdlibResolver, &VerifyOptions::default());
                        (
                            !cert.passed,
                            serde_json::to_string(&cert).unwrap_or_default(),
                        )
                    }
                    "ueas_transpile" => {
                        let t = args.get("target").and_then(Value::as_str).unwrap_or("rust");
                        match TargetKind::parse(t)
                            .zip(compile_to_hir(src, &EmbeddedStdlibResolver).ok())
                        {
                            Some((tk, prog)) => match generate_target(&prog, tk) {
                                Ok(a) => (false, a.code),
                                Err(e) => (true, e),
                            },
                            None => (true, "invalid target or source".to_string()),
                        }
                    }
                    "ueas_format" => match format_source(src) {
                        Ok(f) => (false, f),
                        Err(d) => (true, format!("syntax error: {}", d[0].message)),
                    },
                    "ueas_extract" => {
                        let lang = args.get("lang").and_then(Value::as_str).unwrap_or("python");
                        match extract_source(lang, src) {
                            Ok(r) => (false, serde_json::to_string(&r).unwrap_or_default()),
                            Err(e) => (true, e.to_string()),
                        }
                    }
                    "ueas_oracle_diff" => {
                        let entry = args.get("entry").and_then(Value::as_str).unwrap_or("");
                        let samples =
                            args.get("samples").and_then(Value::as_u64).unwrap_or(16) as usize;
                        match build_oracle_vectors(
                            src,
                            entry,
                            samples,
                            0x5545_4153,
                            &EmbeddedStdlibResolver,
                        ) {
                            Ok(v) => (false, serde_json::to_string(&v).unwrap_or_default()),
                            Err(e) => (true, e),
                        }
                    }
                    _ => (true, format!("unknown tool '{tool}'")),
                };
                json!({ "isError": is_err, "content": [{ "type": "text", "text": text }] })
            }
            _ => json!({}),
        };
        let resp = json!({ "jsonrpc": "2.0", "id": id, "result": result });
        writeln!(writer, "{resp}")?;
        writer.flush()?;
    }
    Ok(())
}
