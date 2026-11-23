//! Tree-sitter TypeScript -> UEAS v2.0 extractor.

use crate::purity::{check_symbol_purity, PurityViolation};
use crate::{ExtractError, ExtractionReport, Extractor};
use tree_sitter::{Node, Parser};

pub struct TypeScriptExtractor;

impl Extractor for TypeScriptExtractor {
    fn language_id(&self) -> &'static str {
        "typescript"
    }

    fn extract(&self, source: &str) -> Result<ExtractionReport, ExtractError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| ExtractError::ParserInit(e.to_string()))?;
        let tree = parser.parse(source, None).ok_or_else(|| {
            ExtractError::ParseFailed("tree-sitter-typescript returned None".into())
        })?;

        let mut violations = Vec::new();
        let mut algos = Vec::new();
        let mut ueas = String::from("module extracted.typescript\n\n");

        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            let fn_node = if child.kind() == "export_statement" {
                child.named_child(0)
            } else {
                Some(child)
            };
            if let Some(n) = fn_node {
                if n.kind() == "function_declaration" {
                    let (name, block) = extract_ts_fn(n, source.as_bytes(), &mut violations);
                    algos.push(name);
                    ueas.push_str(&block);
                    ueas.push('\n');
                }
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

fn map_ts_type(raw: &str) -> &'static str {
    let t = raw.trim().trim_start_matches(':').trim();
    match t {
        "number" => "Int",
        "boolean" => "Bool",
        "string" => "String",
        "void" => "Void",
        s if s.ends_with("[]") => "List<Int>",
        _ => "Int",
    }
}

fn extract_ts_fn(
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
            if p.kind() == "required_parameter" || p.kind() == "optional_parameter" {
                let pname = p
                    .child_by_field_name("pattern")
                    .map(|n| node_text(n, src))
                    .unwrap_or("x");
                let ptype = p
                    .child_by_field_name("type")
                    .map(|n| map_ts_type(node_text(n, src)))
                    .unwrap_or("Int");
                params_out.push(format!("{pname}: {ptype}"));
            }
        }
    }

    let ret_ty = node
        .child_by_field_name("return_type")
        .map(|n| map_ts_type(node_text(n, src)))
        .unwrap_or("Int");

    let mut out = format!(
        "Algorithm {name}({}) -> {ret_ty}\n    Complexity:\n        Time: O(N),\n        Space: O(1)\n\n",
        params_out.join(", ")
    );

    if let Some(body) = node.child_by_field_name("body") {
        emit_ts_block(body, src, 1, &mut out, violations);
    }
    out.push_str("end Algorithm\n");
    (name, out)
}

fn emit_ts_block(
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
        scan_ts_purity(stmt, src, line, violations);
        match stmt.kind() {
            "return_statement" => {
                let expr = stmt
                    .named_child(0)
                    .map(|n| node_text(n, src).to_string())
                    .unwrap_or_else(|| "0".to_string());
                out.push_str(&format!("{pad}return {expr}\n"));
            }
            "lexical_declaration" | "variable_declaration" => {
                let mut dc = stmt.walk();
                for decl in stmt.named_children(&mut dc) {
                    if decl.kind() == "variable_declarator" {
                        let lhs = decl
                            .child_by_field_name("name")
                            .map(|n| node_text(n, src))
                            .unwrap_or("x");
                        let rhs = decl
                            .child_by_field_name("value")
                            .map(|n| node_text(n, src))
                            .unwrap_or("0");
                        out.push_str(&format!("{pad}let mut {lhs} <- {rhs}\n"));
                    }
                }
            }
            "if_statement" => {
                let cond = stmt
                    .child_by_field_name("condition")
                    .map(|n| node_text(n, src).trim_matches(|c| c == '(' || c == ')'))
                    .unwrap_or("true");
                out.push_str(&format!("{pad}if {cond} then\n"));
                if let Some(conseq) = stmt.child_by_field_name("consequence") {
                    emit_ts_block(conseq, src, depth + 1, out, violations);
                }
                if let Some(alt) = stmt.child_by_field_name("alternative") {
                    out.push_str(&format!("{pad}else\n"));
                    emit_ts_block(alt, src, depth + 1, out, violations);
                }
                out.push_str(&format!("{pad}end if\n"));
            }
            "while_statement" => {
                let cond = stmt
                    .child_by_field_name("condition")
                    .map(|n| node_text(n, src).trim_matches(|c| c == '(' || c == ')'))
                    .unwrap_or("true");
                out.push_str(&format!("{pad}while {cond} do\n"));
                if let Some(body) = stmt.child_by_field_name("body") {
                    emit_ts_block(body, src, depth + 1, out, violations);
                }
                out.push_str(&format!("{pad}end while\n"));
            }
            _ => {}
        }
    }
}

fn scan_ts_purity(node: Node<'_>, src: &[u8], line: usize, violations: &mut Vec<PurityViolation>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            check_symbol_purity(node_text(func, src), line, violations);
        }
    }
    let mut c = node.walk();
    for ch in node.named_children(&mut c) {
        scan_ts_purity(ch, src, line, violations);
    }
}
