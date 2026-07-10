//! UEAS MCP Server — Model Context Protocol integration for AI agents.
//!
//! Exposes 3 tools via JSON-RPC over stdin/stdout:
//! - `parse_ueas`   — validate .ueas syntax
//! - `execute_ueas` — run in virtual heap, return step count + exit code
//! - `transpile_ueas` — transpile to target language

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use ueas_backends::{PythonTarget, RustTarget, TargetGenerator};
use ueas_kernel::ast::{AstNode, AstNodeFactory, AstNodeKind};
use ueas_kernel::interp::{execute_program, ExecContext};
use ueas_kernel::traps::ExitCode;

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
        // Notifications have no id — skip them
        if msg["id"].is_null() {
            continue;
        }
        let response = handle_message(&msg);
        let response_str = match serde_json::to_string(&response) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("MCP: JSON serialization error: {}", e);
                continue;
            }
        };
        if writeln!(writer, "{}", response_str).is_err() {
            break;
        }
        if writer.flush().is_err() {
            break;
        }
    }
}

fn handle_message(msg: &Value) -> Value {
    let method = msg["method"].as_str().unwrap_or("");
    let id = &msg["id"];

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "ueas",
                    "version": "0.1.0"
                }
            }
        }),
        "tools/list" => tools_list_response(id),
        "tools/call" => tools_call_response(id, msg),
        _ => error_response(id, -32601, &format!("Method not found: {}", method)),
    }
}

fn tools_list_response(id: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "parse_ueas",
                    "description": "Validate .ueas syntax and return algorithm info",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "source": {
                                "type": "string",
                                "description": "UEAS source code to parse and validate"
                            }
                        },
                        "required": ["source"]
                    }
                },
                {
                    "name": "execute_ueas",
                    "description": "Execute a UEAS algorithm in the virtual heap, returning step count, exit code, and heap usage",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "source": {
                                "type": "string",
                                "description": "UEAS source code to execute"
                            }
                        },
                        "required": ["source"]
                    }
                },
                {
                    "name": "transpile_ueas",
                    "description": "Transpile a UEAS algorithm to a target language",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "source": {
                                "type": "string",
                                "description": "UEAS source code to transpile"
                            },
                            "target": {
                                "type": "string",
                                "description": "Target language: python or rust",
                                "enum": ["python", "rust"]
                            }
                        },
                        "required": ["source", "target"]
                    }
                }
            ]
        }
    })
}

fn tools_call_response(id: &Value, msg: &Value) -> Value {
    let params = &msg["params"];
    let tool_name = params["name"].as_str().unwrap_or("");
    let arguments = &params["arguments"];

    match tool_name {
        "parse_ueas" => {
            let source = arguments["source"].as_str().unwrap_or("");
            let result = tool_parse_ueas(source);
            tool_result_response(id, &result)
        }
        "execute_ueas" => {
            let source = arguments["source"].as_str().unwrap_or("");
            let result = tool_execute_ueas(source);
            tool_result_response(id, &result)
        }
        "transpile_ueas" => {
            let source = arguments["source"].as_str().unwrap_or("");
            let target = arguments["target"].as_str().unwrap_or("python");
            let result = tool_transpile_ueas(source, target);
            tool_result_response(id, &result)
        }
        _ => error_response(id, -32602, &format!("Unknown tool: {}", tool_name)),
    }
}

fn tool_result_response(id: &Value, result: &serde_json::Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": result.to_string()
                }
            ]
        }
    })
}

fn error_response(id: &Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

// ===== Tool Implementations =====

fn tool_parse_ueas(source: &str) -> Value {
    match parse_algorithm(source) {
        Ok((name, _)) => json!({
            "valid": true,
            "algorithm_name": name,
            "error": null
        }),
        Err(e) => json!({
            "valid": false,
            "algorithm_name": null,
            "error": format!("{:#}", e)
        }),
    }
}

fn tool_execute_ueas(source: &str) -> Value {
    match parse_algorithm(source) {
        Ok((_name, algo)) => {
            let program = AstNodeFactory::program(vec![algo]);
            let mut ctx = ExecContext::with_default_config();
            match execute_program(&mut ctx, &program) {
                Ok(_result) => json!({
                    "exit_code": 0_i32,
                    "exit_name": ExitCode::NoError.name(),
                    "step_count": ctx.profiler.step_count(),
                    "heap_bytes": ctx.heap.bytes_allocated(),
                    "trap": null
                }),
                Err(code) => json!({
                    "exit_code": code as i32,
                    "exit_name": code.name(),
                    "step_count": ctx.profiler.step_count(),
                    "heap_bytes": ctx.heap.bytes_allocated(),
                    "trap": code.description()
                }),
            }
        }
        Err(e) => json!({
            "exit_code": -1_i32,
            "exit_name": "PARSE_ERROR",
            "step_count": 0_u64,
            "heap_bytes": 0_u64,
            "trap": format!("{:#}", e)
        }),
    }
}

fn tool_transpile_ueas(source: &str, target: &str) -> Value {
    match parse_algorithm(source) {
        Ok((_name, algo)) => {
            let program = AstNodeFactory::program(vec![algo]);
            let ast_json = match serde_json::to_string_pretty(&program) {
                Ok(j) => j,
                Err(e) => {
                    return json!({
                        "language": target,
                        "version": null,
                        "source": null,
                        "error": format!("JSON serialization failed: {}", e)
                    })
                }
            };

            match target.to_lowercase().as_str() {
                "python" => {
                    let gen = PythonTarget;
                    let version = gen.version();
                    match gen.generate(&ast_json) {
                        Ok(output) => json!({
                            "language": "python",
                            "version": version,
                            "source": output,
                            "error": null
                        }),
                        Err(e) => json!({
                            "language": "python",
                            "version": version,
                            "source": null,
                            "error": e.message
                        }),
                    }
                }
                "rust" => {
                    let gen = RustTarget;
                    let version = gen.version();
                    match gen.generate(&ast_json) {
                        Ok(output) => json!({
                            "language": "rust",
                            "version": version,
                            "source": output,
                            "error": null
                        }),
                        Err(e) => json!({
                            "language": "rust",
                            "version": version,
                            "source": null,
                            "error": e.message
                        }),
                    }
                }
                _ => json!({
                    "language": target,
                    "version": null,
                    "source": null,
                    "error": format!("Unsupported transpilation target: {}. Available: python, rust", target)
                }),
            }
        }
        Err(e) => json!({
            "language": target,
            "version": null,
            "source": null,
            "error": format!("{:#}", e)
        }),
    }
}

// ===== UEAS Parser (adapted from tools/ueas-cli/src/main.rs) =====

fn parse_algorithm(source: &str) -> Result<(String, AstNode)> {
    let source = source.trim();
    if source.is_empty() {
        anyhow::bail!("empty source file");
    }
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        anyhow::bail!("empty source file");
    }

    let first = lines[0].trim();
    let header_rest = first
        .strip_prefix("Algorithm ")
        .or_else(|| first.strip_prefix("algorithm "))
        .or_else(|| first.strip_prefix("ALGORITHM "))
        .with_context(|| "line 1: expected 'Algorithm Name(params)'".to_string())?;

    let (name, params) = if let Some(paren_idx) = header_rest.find('(') {
        let name = header_rest[..paren_idx].trim().to_string();
        let close = header_rest[paren_idx..]
            .find(')')
            .context("line 1: missing ')'")?;
        let params_raw = &header_rest[paren_idx + 1..paren_idx + close];
        let v: Vec<String> = if params_raw.trim().is_empty() {
            vec![]
        } else {
            params_raw
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };
        (name, v)
    } else {
        (header_rest.trim().to_string(), vec![])
    };

    let mut complexity = String::from("O(1)");
    let mut bindings: Vec<AstNode> = vec![];
    let mut return_type: Option<AstNode> = None;
    let mut stmt_lines: Vec<&str> = Vec::new();

    for raw_line in &lines[1..] {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("Require:") || line.starts_with("Require") {
            continue;
        }
        if line.starts_with("Ensure:") || line.starts_with("Ensure") {
            let ts = line.split_once(':').map(|x| x.1).unwrap_or("").trim();
            return_type = Some(AstNodeFactory::type_node(ts, vec![]));
            continue;
        }
        if line.starts_with("Complexity:") || line.starts_with("Complexity") {
            let rest = line.split_once(':').map(|x| x.1).unwrap_or("").trim();
            let parts: Vec<&str> = rest.split(',').collect();
            complexity = parts[0].trim().trim_matches('"').to_string();
            for part in &parts[1..] {
                if let Some(eq) = part.find('=') {
                    bindings.push(AstNodeFactory::variable_binding(
                        part[..eq].trim(),
                        parse_expr(part[eq + 1..].trim())?,
                    ));
                }
            }
            continue;
        }
        if line.starts_with("Memory:") || line.starts_with("Memory") {
            continue;
        }
        if line.starts_with("end Algorithm")
            || line.starts_with("End Algorithm")
            || line.starts_with("END ALGORITHM")
        {
            continue;
        }
        stmt_lines.push(raw_line);
    }

    let int_ty = AstNodeFactory::type_node("Integer", vec![]);
    let mut param_decls: Vec<AstNode> = vec![];
    for p in &params {
        param_decls.push(AstNode::internal(
            AstNodeKind::VariableDeclaration,
            vec![
                AstNodeFactory::identifier(p),
                int_ty.clone(),
                AstNodeFactory::integer_literal("0"),
            ],
            None,
        ));
    }

    let mut body = param_decls;
    body.extend(parse_body(&stmt_lines)?);

    let mut children = vec![
        AstNodeFactory::identifier(&name),
        AstNodeFactory::string_literal(&complexity),
    ];
    if let Some(rt) = return_type {
        children.push(rt);
    }
    children.extend(bindings);
    children.extend(body);
    let algo = AstNode::internal(AstNodeKind::Algorithm, children, None);

    Ok((name, algo))
}

fn parse_body(lines: &[&str]) -> Result<Vec<AstNode>> {
    let mut stmts = vec![];
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }
        let (stmt, consumed) = parse_stmt(lines, i)?;
        stmts.push(stmt);
        i += consumed;
    }
    Ok(stmts)
}

fn parse_stmt(lines: &[&str], idx: usize) -> Result<(AstNode, usize)> {
    let line = lines[idx].trim();

    if let Some(rest) = line
        .strip_prefix("return ")
        .or_else(|| line.strip_prefix("Return "))
    {
        if rest.trim().is_empty() {
            return Ok((AstNodeFactory::return_stmt(None), 1));
        }
        return Ok((AstNodeFactory::return_stmt(Some(parse_expr(rest)?)), 1));
    }
    if line == "return" || line == "Return" {
        return Ok((AstNodeFactory::return_stmt(None), 1));
    }

    if let Some(rest) = line.strip_prefix("if ") {
        return parse_if(lines, idx, rest);
    }

    if let Some(rest) = line.strip_prefix("while ") {
        return parse_while(lines, idx, rest);
    }

    if let Some(rest) = line.strip_prefix("for ") {
        return parse_for(lines, idx, rest);
    }

    if line.starts_with("assert(") || line.starts_with("Assert(") {
        let inner = trim_fn_call(line);
        let (c, m) = split_cond_msg(inner);
        return Ok((AstNodeFactory::assert_stmt(parse_expr(c)?, m.as_deref()), 1));
    }

    if line.starts_with("invariant(") || line.starts_with("Invariant(") {
        let inner = trim_fn_call(line);
        let (c, m) = split_cond_msg(inner);
        return Ok((
            AstNodeFactory::invariant_stmt(parse_expr(c)?, m.as_deref()),
            1,
        ));
    }

    if let Some(pos) = find_assign_op(line) {
        let target = line[..pos].trim();
        return Ok((
            AstNodeFactory::assignment(
                AstNodeFactory::identifier(target),
                parse_expr(&line[pos + 2..])?,
            ),
            1,
        ));
    }

    if let Ok(expr) = parse_expr(line) {
        return Ok((expr, 1));
    }

    anyhow::bail!("line {}: unrecognized statement: '{}'", idx + 1, line)
}

fn trim_fn_call(line: &str) -> &str {
    let start = line.find('(').unwrap_or(0) + 1;
    let end = line.rfind(')').unwrap_or(line.len());
    &line[start..end]
}

fn split_cond_msg(inner: &str) -> (&str, Option<String>) {
    if let Some(ci) = inner.rfind(',') {
        (
            inner[..ci].trim(),
            Some(inner[ci + 1..].trim().trim_matches('"').to_string()),
        )
    } else {
        (inner, None)
    }
}

fn find_assign_op(line: &str) -> Option<usize> {
    line.find("<-").or_else(|| line.find(":="))
}

fn parse_if(lines: &[&str], idx: usize, first_cond: &str) -> Result<(AstNode, usize)> {
    let cond = parse_expr(first_cond.trim_end_matches(" then"))?;
    let (then_body, j) = collect_until_depth(lines, idx + 1)?;
    let then_stmts = parse_body(&then_body)?;

    Ok((
        AstNode::internal(
            AstNodeKind::If,
            vec![cond, wrap(AstNodeKind::If, then_stmts)],
            None,
        ),
        j - idx,
    ))
}

fn parse_while(lines: &[&str], idx: usize, rest: &str) -> Result<(AstNode, usize)> {
    let cond = parse_expr(rest.trim_end_matches(" do"))?;
    let (body, j) = collect_until_depth(lines, idx + 1)?;
    let stmts = parse_body(&body)?;
    Ok((
        AstNode::internal(
            AstNodeKind::WhileLoop,
            vec![cond, wrap(AstNodeKind::WhileLoop, stmts)],
            None,
        ),
        j - idx,
    ))
}

fn parse_for(lines: &[&str], idx: usize, rest: &str) -> Result<(AstNode, usize)> {
    let rest = rest.trim_start_matches("each ");
    let parts: Vec<&str> = rest.splitn(3, ' ').collect();
    if parts.len() < 3 || parts[1] != "in" {
        anyhow::bail!("line {}: expected 'for var in collection do'", idx + 1);
    }
    let iter = AstNodeFactory::identifier(parts[0].trim());
    let coll = AstNodeFactory::identifier(parts[2].trim().trim_end_matches(" do"));
    let (body, j) = collect_until_depth(lines, idx + 1)?;
    let stmts = parse_body(&body)?;
    let mut children = vec![iter, coll];
    children.extend(stmts);
    Ok((
        AstNode::internal(AstNodeKind::ForLoop, children, None),
        j - idx,
    ))
}

fn collect_until_depth<'a>(lines: &[&'a str], start: usize) -> Result<(Vec<&'a str>, usize)> {
    let mut body = vec![];
    let mut depth = 1u32;
    let mut i = start;

    loop {
        if i >= lines.len() {
            anyhow::bail!("line {}: expected matching 'end' closure not found", start);
        }
        let l = lines[i].trim().to_lowercase();
        if l.starts_with("if ") || l.starts_with("for ") || l.starts_with("while ") {
            depth += 1;
        }
        if l == "end if" || l == "end for" || l == "end while" || l == "end algorithm" {
            depth -= 1;
        }
        if depth == 0 {
            i += 1;
            break;
        }
        body.push(lines[i]);
        i += 1;
    }
    Ok((body, i))
}

fn wrap(kind: AstNodeKind, stmts: Vec<AstNode>) -> AstNode {
    AstNode::internal(kind, stmts, None)
}

fn parse_expr(s: &str) -> Result<AstNode> {
    let expr = s.trim();
    if expr.is_empty() {
        anyhow::bail!("empty expression");
    }

    if let Ok(n) = expr.parse::<i64>() {
        return Ok(AstNodeFactory::integer_literal(&n.to_string()));
    }
    if let Ok(n) = expr.parse::<f64>() {
        return Ok(AstNodeFactory::real_literal(n));
    }
    if expr == "true" || expr == "True" || expr == "TRUE" {
        return Ok(AstNodeFactory::boolean_literal(true));
    }
    if expr == "false" || expr == "False" || expr == "FALSE" {
        return Ok(AstNodeFactory::boolean_literal(false));
    }
    if expr.starts_with('"') && expr.ends_with('"') {
        return Ok(AstNodeFactory::string_literal(&expr[1..expr.len() - 1]));
    }
    if expr == "[]" || expr == "{}" {
        return Ok(AstNodeFactory::set_literal(vec![]));
    }
    if expr.starts_with('[') && expr.ends_with(']') {
        let inner = &expr[1..expr.len() - 1];
        let elems: Result<Vec<AstNode>> = inner
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| parse_expr(s.trim()))
            .collect();
        return Ok(AstNodeFactory::list_literal(elems?));
    }

    if let Some(result) = try_binary(expr) {
        return Ok(result);
    }

    if let Some(p) = expr.find('(') {
        if expr.ends_with(')') {
            let func = expr[..p].trim();
            let a_str = &expr[p + 1..expr.len() - 1];
            let args = if a_str.trim().is_empty() {
                vec![]
            } else {
                let mut depth = 0u32;
                let mut cur = String::new();
                let mut res = vec![];
                for ch in a_str.chars() {
                    match ch {
                        '(' => {
                            depth += 1;
                            cur.push(ch);
                        }
                        ')' => {
                            depth -= 1;
                            cur.push(ch);
                        }
                        ',' if depth == 0 => {
                            res.push(parse_expr(&cur)?);
                            cur.clear();
                        }
                        _ => cur.push(ch),
                    }
                }
                if !cur.trim().is_empty() {
                    res.push(parse_expr(&cur)?);
                }
                res
            };
            return Ok(AstNodeFactory::function_call(func, args));
        }
    }

    if expr.contains('.') {
        let dot = expr.rfind('.').unwrap();
        let obj = &expr[..dot];
        let meth = &expr[dot + 1..];
        if let Some(stripped) = meth.strip_suffix("()") {
            return Ok(AstNodeFactory::function_call(
                stripped,
                vec![parse_expr(obj)?],
            ));
        }
        if let Some(p) = meth.find('(') {
            if meth.ends_with(')') {
                let m = &meth[..p];
                let a_str = &meth[p + 1..meth.len() - 1];
                let mut args = vec![parse_expr(obj)?];
                if !a_str.trim().is_empty() {
                    args.push(parse_expr(a_str)?);
                }
                return Ok(AstNodeFactory::function_call(m, args));
            }
        }
        return Ok(AstNodeFactory::identifier(expr));
    }

    Ok(AstNodeFactory::identifier(expr))
}

fn try_binary(expr: &str) -> Option<AstNode> {
    let ops = ["==", "!=", "<=", ">=", "+", "-", "*", "/", "<", ">"];
    for op in &ops {
        let skip = if *op == "-" { 1 } else { 0 };
        if let Some(pos) = expr[skip..].find(op) {
            let pos = pos + skip;
            if pos > 0 && !expr[..pos].ends_with('<') {
                let left = parse_expr(&expr[..pos]).ok()?;
                let right = parse_expr(&expr[pos + op.len()..]).ok()?;
                return Some(AstNodeFactory::binary_expression(op, left, right));
            }
        }
    }
    None
}
