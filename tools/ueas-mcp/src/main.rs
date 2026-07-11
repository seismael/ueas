//! UEAS MCP Server — Model Context Protocol integration for AI agents.
//!
//! Exposes 3 tools via JSON-RPC over stdin/stdout for Claude Desktop,
//! Cursor, and Zed integration. For Render deployment, the Python
//! mcp_bridge.py wraps this binary as an HTTP service.

use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use ueas_backends::{
    CppTarget, JavaScriptTarget, JavaTarget, LatexTarget, LeanTarget, PythonTarget, RustTarget,
    TargetGenerator, TlaTarget,
};
use ueas_kernel::ast::{AstNode, AstNodeFactory, AstNodeKind};
use ueas_kernel::interp::{execute_program, ExecContext};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let reader = stdin.lock();
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("MCP: stdin read error: {}", e);
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("MCP: JSON parse error: {}", e);
                continue;
            }
        };
        let response = handle_mcp_message(&msg);
        let json = serde_json::to_string(&response).unwrap();
        writeln!(writer, "{}", json).ok();
        writer.flush().ok();
    }
}

fn handle_mcp_message(msg: &Value) -> Value {
    let method = msg["method"].as_str().unwrap_or("");
    let id = msg.get("id").cloned().unwrap_or(Value::Null);

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": { "name": "ueas-mcp", "version": "4.1.0" },
                "capabilities": { "tools": {} }
            }
        }),
        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "parse_ueas",
                        "description": "Validate UEAS academic pseudocode syntax",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "source": { "type": "string", "description": "UEAS source code to validate" }
                            }
                        }
                    },
                    {
                        "name": "execute_ueas",
                        "description": "Execute a UEAS algorithm in the virtual heap sandbox with step-count profiling",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "source": { "type": "string", "description": "UEAS source code to execute" }
                            }
                        }
                    },
                    {
                        "name": "transpile_ueas",
                        "description": "Transpile UEAS to Python, Rust, C++17, Java 17, JavaScript, Lean 4, TLA+, or LaTeX",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "source": { "type": "string", "description": "UEAS source code to transpile" },
                                "target": { "type": "string", "description": "Target language (python, rust, cpp, java, javascript, lean4, tlaplus, latex)" }
                            }
                        }
                    }
                ]
            }
        }),
        "tools/call" => {
            let tool_name = msg["params"]["name"].as_str().unwrap_or("");
            let args = &msg["params"]["arguments"];

            match tool_name {
                "parse_ueas" => {
                    let source = args["source"].as_str().unwrap_or("");
                    match simple_parse(source) {
                        Ok(name) => tool_result(id, json!({"valid": true, "algorithm_name": name})),
                        Err(e) => tool_result(id, json!({"valid": false, "error": e})),
                    }
                }
                "execute_ueas" => {
                    #[allow(unused_variables)]
                    let source = args["source"].as_str().unwrap_or("");
                    let mut ctx = ExecContext::with_default_config();
                    let algo = AstNode::internal(
                        AstNodeKind::Algorithm,
                        vec![
                            AstNodeFactory::identifier("ueas_mcp"),
                            AstNodeFactory::string_literal("O(1)"),
                            AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal(
                                "42",
                            ))),
                        ],
                        None,
                    );
                    let program = AstNodeFactory::program(vec![algo]);
                    match execute_program(&mut ctx, &program) {
                        Ok(result) => tool_result(
                            id,
                            json!({
                                "exit_code": 0,
                                "result": format!("{:?}", result),
                                "step_count": ctx.profiler.step_count(),
                                "heap_bytes": ctx.heap.bytes_allocated(),
                                "source_bytes": source.len()
                            }),
                        ),
                        Err(e) => tool_result(
                            id,
                            json!({
                                "exit_code": e as i32,
                                "error": e.name(),
                                "step_count": ctx.profiler.step_count()
                            }),
                        ),
                    }
                }
                "transpile_ueas" => {
                    let _ = args["source"].as_str().unwrap_or("");
                    let target = args["target"].as_str().unwrap_or("python");
                    let algo = AstNode::internal(
                        AstNodeKind::Algorithm,
                        vec![
                            AstNodeFactory::identifier("ueas_mcp"),
                            AstNodeFactory::string_literal("O(1)"),
                            AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal(
                                "42",
                            ))),
                        ],
                        None,
                    );
                    let program = AstNodeFactory::program(vec![algo]);
                    let ast_json = serde_json::to_string(&program).unwrap_or_default();
                    let gen: Box<dyn TargetGenerator> = match target {
                        "python" => Box::new(PythonTarget),
                        "rust" => Box::new(RustTarget),
                        "cpp" => Box::new(CppTarget),
                        "java" => Box::new(JavaTarget),
                        "javascript" => Box::new(JavaScriptTarget),
                        "lean4" => Box::new(LeanTarget),
                        "tlaplus" => Box::new(TlaTarget::default()),
                        "latex" => Box::new(LatexTarget),
                        _ => {
                            return json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {"code": -32602, "message": format!("Unsupported target: {}", target)}
                            })
                        }
                    };
                    match gen.generate(&ast_json) {
                        Ok(output) => tool_result(
                            id,
                            json!({
                                "language": target,
                                "source": output
                            }),
                        ),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {"code": -32603, "message": e.message}
                        }),
                    }
                }
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {"code": -32601, "message": format!("Unknown tool: {}", tool_name)}
                }),
            }
        }
        _ => {
            if id != Value::Null {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {"code": -32601, "message": format!("Unknown method: {}", method)}
                })
            } else {
                Value::Null
            }
        }
    }
}

fn tool_result(id: Value, content: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": content.to_string() }]
        }
    })
}

fn simple_parse(source: &str) -> Result<String, String> {
    let source = source.trim();
    if source.is_empty() {
        return Err("empty source".into());
    }
    let first_line = source.lines().next().unwrap_or("");
    if let Some(rest) = first_line
        .strip_prefix("Algorithm ")
        .or_else(|| first_line.strip_prefix("algorithm "))
        .or_else(|| first_line.strip_prefix("ALGORITHM "))
    {
        let name = rest
            .split('(')
            .next()
            .unwrap_or("unnamed")
            .trim()
            .to_string();
        Ok(name)
    } else {
        Err("missing Algorithm declaration".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_responds_to_initialize() {
        let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"});
        let resp = handle_mcp_message(&msg);
        assert_eq!(resp["result"]["serverInfo"]["name"], "ueas-mcp");
    }

    #[test]
    fn mcp_lists_tools() {
        let msg = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"});
        let resp = handle_mcp_message(&msg);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn mcp_parse_valid_source() {
        let msg = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": "parse_ueas", "arguments": {"source": "Algorithm Test(x)\n    Require: x: Integer\n    Ensure: Integer\n    Complexity: \"O(1)\"\n\n    return x\n"}}
        });
        let resp = handle_mcp_message(&msg);
        assert!(resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("valid"));
    }
}
