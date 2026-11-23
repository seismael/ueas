//! Tree-sitter Rust -> UEAS v2.0 extractor.

use crate::purity::{check_symbol_purity, PurityViolation};
use crate::{ExtractError, ExtractionReport, Extractor};
use tree_sitter::{Node, Parser};

pub struct RustExtractor;

impl Extractor for RustExtractor {
    fn language_id(&self) -> &'static str {
        "rust"
    }

    fn extract(&self, source: &str) -> Result<ExtractionReport, ExtractError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| ExtractError::ParserInit(e.to_string()))?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| ExtractError::ParseFailed("tree-sitter-rust returned None".into()))?;

        let mut violations = Vec::new();
        let mut algos = Vec::new();
        let mut ueas = String::from("module extracted.rust\n\n");

        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_item" {
                let (name, block) = extract_rs_fn(child, source.as_bytes(), &mut violations);
                algos.push(name);
                ueas.push_str(&block);
                ueas.push('\n');
            }
        }

        Ok(ExtractionReport {
            ueas_source: ueas,
            extracted_algorithms: algos,
            axiom1_violations: violations,
        })
    }
}

fn node_text<'a>(node: Node<'_>, src: &'a [u8]) -> &'a str {
    node.utf8_text(src).unwrap_or("")
}

fn map_rs_type(raw: &str) -> &'static str {
    let t = raw.trim();
    match t {
        "i64" | "i32" | "isize" => "Int",
        "u64" | "u32" | "usize" => "Int",
        "f64" | "f32" => "Real",
        "bool" => "Bool",
        "String" | "&str" => "String",
        s if s.starts_with("Vec<") || s.starts_with("&[") => "List<Int>",
        _ => "Int",
    }
}

fn extract_rs_fn(
    node: Node<'_>,
    src: &[u8],
    violations: &mut Vec<PurityViolation>,
) -> (String, String) {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, src).to_string())
        .unwrap_or_else(|| "ExtractedFn".to_string());

    let mut params_out = Vec::new();
    if let Some(params) = node.child_by_field_name("parameters") {
        let mut c = params.walk();
        for p in params.named_children(&mut c) {
            if p.kind() == "parameter" {
                let pname = p
                    .child_by_field_name("pattern")
                    .map(|n| node_text(n, src))
                    .unwrap_or("x");
                let ptype = p
                    .child_by_field_name("type")
                    .map(|n| map_rs_type(node_text(n, src)))
                    .unwrap_or("Int");
                params_out.push(format!("{pname}: {ptype}"));
            }
        }
    }

    let ret_ty = node
        .child_by_field_name("return_type")
        .map(|n| map_rs_type(node_text(n, src)))
        .unwrap_or("Int");

    let mut out = format!(
        "Algorithm {name}({}) -> {ret_ty}\n    Complexity:\n        Time: O(N),\n        Space: O(1)\n\n",
        params_out.join(", ")
    );

    if let Some(body) = node.child_by_field_name("body") {
        emit_rs_block(body, src, 1, &mut out, violations);
    }
    out.push_str("end Algorithm\n");
    (name, out)
}

fn emit_rs_block(
    block: Node<'_>,
    src: &[u8],
    depth: usize,
    out: &mut String,
    violations: &mut Vec<PurityViolation>,
) {
    let pad = "    ".repeat(depth);
    let mut cursor = block.walk();
    for stmt in block.named_children(&mut cursor) {
        let line = stmt.start_position().row + 1;
        scan_rs_purity(stmt, src, line, violations);
        match stmt.kind() {
            "return_expression" => {
                let expr = stmt
                    .named_child(0)
                    .map(|n| node_text(n, src).to_string())
                    .unwrap_or_else(|| "0".to_string());
                out.push_str(&format!("{pad}return {expr}\n"));
            }
            "let_declaration" => {
                let lhs = stmt
                    .child_by_field_name("pattern")
                    .map(|n| node_text(n, src).trim_start_matches("mut ").trim())
                    .unwrap_or("x");
                let rhs = stmt
                    .child_by_field_name("value")
                    .map(|n| node_text(n, src))
                    .unwrap_or("0");
                out.push_str(&format!("{pad}let mut {lhs} <- {rhs}\n"));
            }
            "if_expression" => {
                let cond = stmt
                    .child_by_field_name("condition")
                    .map(|n| node_text(n, src))
                    .unwrap_or("true");
                out.push_str(&format!("{pad}if {cond} then\n"));
                if let Some(conseq) = stmt.child_by_field_name("consequence") {
                    emit_rs_block(conseq, src, depth + 1, out, violations);
                }
                if let Some(alt) = stmt.child_by_field_name("alternative") {
                    out.push_str(&format!("{pad}else\n"));
                    emit_rs_block(alt, src, depth + 1, out, violations);
                }
                out.push_str(&format!("{pad}end if\n"));
            }
            "while_expression" => {
                let cond = stmt
                    .child_by_field_name("condition")
                    .map(|n| node_text(n, src))
                    .unwrap_or("true");
                out.push_str(&format!("{pad}while {cond} do\n"));
                if let Some(body) = stmt.child_by_field_name("body") {
                    emit_rs_block(body, src, depth + 1, out, violations);
                }
                out.push_str(&format!("{pad}end while\n"));
            }
            _ => {}
        }
    }
}

fn scan_rs_purity(node: Node<'_>, src: &[u8], line: usize, violations: &mut Vec<PurityViolation>) {
    if node.kind() == "unsafe_block" {
        violations.push(PurityViolation {
            line,
            symbol: "unsafe".to_string(),
            reason: "Unsafe Rust block violates UEAS Axiom 1".to_string(),
        });
    } else if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            check_symbol_purity(node_text(func, src), line, violations);
        }
    }
    let mut c = node.walk();
    for ch in node.named_children(&mut c) {
        scan_rs_purity(ch, src, line, violations);
    }
}
