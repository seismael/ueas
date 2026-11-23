//! Tree-sitter Python -> UEAS v2.0 extractor.

use crate::purity::{check_symbol_purity, PurityViolation};
use crate::{ExtractError, ExtractionReport, Extractor};
use tree_sitter::{Node, Parser};

pub struct PythonExtractor;

impl Extractor for PythonExtractor {
    fn language_id(&self) -> &'static str {
        "python"
    }

    fn extract(&self, source: &str) -> Result<ExtractionReport, ExtractError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| ExtractError::ParserInit(e.to_string()))?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| ExtractError::ParseFailed("tree-sitter-python returned None".into()))?;

        let mut violations = Vec::new();
        let mut algos = Vec::new();
        let mut ueas = String::from("module extracted.python\n\n");

        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                let (name, block) = extract_py_fn(child, source.as_bytes(), &mut violations);
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

fn map_py_type(raw: &str) -> &'static str {
    match raw.trim() {
        "int" => "Int",
        "float" => "Real",
        "bool" => "Bool",
        "str" => "String",
        s if s.starts_with("list") || s.starts_with("List") => "List<Int>",
        _ => "Int",
    }
}

fn extract_py_fn(
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
        for p in params.children(&mut c) {
            match p.kind() {
                "identifier" => params_out.push(format!("{}: Int", node_text(p, src))),
                "typed_parameter" => {
                    let pname = p.child(0).map(|n| node_text(n, src)).unwrap_or("x");
                    let ptype = p
                        .child_by_field_name("type")
                        .map(|n| map_py_type(node_text(n, src)))
                        .unwrap_or("Int");
                    params_out.push(format!("{pname}: {ptype}"));
                }
                _ => {}
            }
        }
    }

    let ret_ty = node
        .child_by_field_name("return_type")
        .map(|n| map_py_type(node_text(n, src)))
        .unwrap_or("Int");

    let mut out = format!(
        "# @ComplexityUnverified\nAlgorithm {name}({}) -> {ret_ty}\n\n",
        params_out.join(", ")
    );

    if let Some(body) = node.child_by_field_name("body") {
        emit_py_block(body, src, 1, &mut out, violations);
    }
    out.push_str("end Algorithm\n");
    (name, out)
}

fn emit_py_block(
    block: Node<'_>,
    src: &[u8],
    depth: usize,
    out: &mut String,
    violations: &mut Vec<PurityViolation>,
) {
    let pad = "    ".repeat(depth);
    let mut cursor = block.walk();
    for stmt in block.children(&mut cursor) {
        let line = stmt.start_position().row + 1;
        match stmt.kind() {
            "return_statement" => {
                let expr = stmt
                    .child(1)
                    .map(|n| translate_py_expr(n, src, line, violations))
                    .unwrap_or_else(|| "0".to_string());
                out.push_str(&format!("{pad}return {expr}\n"));
            }
            "expression_statement" => {
                if let Some(ch) = stmt.child(0) {
                    if ch.kind() == "assignment" {
                        let lhs = ch
                            .child_by_field_name("left")
                            .map(|n| node_text(n, src))
                            .unwrap_or("x");
                        let rhs = ch
                            .child_by_field_name("right")
                            .map(|n| translate_py_expr(n, src, line, violations))
                            .unwrap_or_else(|| "0".to_string());
                        out.push_str(&format!("{pad}let mut {lhs} <- {rhs}\n"));
                    } else if ch.kind() == "augmented_assignment" {
                        let lhs = ch
                            .child_by_field_name("left")
                            .map(|n| node_text(n, src))
                            .unwrap_or("x");
                        let op_raw = ch
                            .child_by_field_name("operator")
                            .map(|n| node_text(n, src))
                            .unwrap_or("+=");
                        let op = op_raw.trim_end_matches('=');
                        let rhs = ch
                            .child_by_field_name("right")
                            .map(|n| translate_py_expr(n, src, line, violations))
                            .unwrap_or_else(|| "0".to_string());
                        out.push_str(&format!("{pad}{lhs} <- {lhs} {op} ({rhs})\n"));
                    } else {
                        let e = translate_py_expr(ch, src, line, violations);
                        out.push_str(&format!("{pad}{e}\n"));
                    }
                }
            }
            "if_statement" => {
                let cond = stmt
                    .child_by_field_name("condition")
                    .map(|n| translate_py_expr(n, src, line, violations))
                    .unwrap_or_else(|| "true".to_string());
                out.push_str(&format!("{pad}if {cond} then\n"));
                if let Some(conseq) = stmt.child_by_field_name("consequence") {
                    emit_py_block(conseq, src, depth + 1, out, violations);
                }
                let mut ac = stmt.walk();
                for alt in stmt.children_by_field_name("alternative", &mut ac) {
                    if alt.kind() == "elif_clause" {
                        let econd = alt
                            .child_by_field_name("condition")
                            .map(|n| translate_py_expr(n, src, line, violations))
                            .unwrap_or_else(|| "true".to_string());
                        out.push_str(&format!("{pad}else if {econd} then\n"));
                        if let Some(eb) = alt.child_by_field_name("consequence") {
                            emit_py_block(eb, src, depth + 1, out, violations);
                        }
                    } else if alt.kind() == "else_clause" {
                        out.push_str(&format!("{pad}else\n"));
                        if let Some(eb) = alt.child_by_field_name("body") {
                            emit_py_block(eb, src, depth + 1, out, violations);
                        }
                    }
                }
                out.push_str(&format!("{pad}end if\n"));
            }
            "while_statement" => {
                let cond = stmt
                    .child_by_field_name("condition")
                    .map(|n| translate_py_expr(n, src, line, violations))
                    .unwrap_or_else(|| "true".to_string());
                out.push_str(&format!("{pad}while {cond} do\n"));
                if let Some(body) = stmt.child_by_field_name("body") {
                    emit_py_block(body, src, depth + 1, out, violations);
                }
                out.push_str(&format!("{pad}end while\n"));
            }
            "for_statement" => {
                let var = stmt
                    .child_by_field_name("left")
                    .map(|n| node_text(n, src))
                    .unwrap_or("i");
                let right = stmt
                    .child_by_field_name("right")
                    .map(|n| translate_py_expr(n, src, line, violations))
                    .unwrap_or_else(|| "0..10".to_string());
                out.push_str(&format!("{pad}for {var} in {right} do\n"));
                if let Some(body) = stmt.child_by_field_name("body") {
                    emit_py_block(body, src, depth + 1, out, violations);
                }
                out.push_str(&format!("{pad}end for\n"));
            }
            _ => {}
        }
    }
}

fn translate_py_expr(
    node: Node<'_>,
    src: &[u8],
    line: usize,
    violations: &mut Vec<PurityViolation>,
) -> String {
    if node.kind() == "call" {
        if let Some(func) = node.child_by_field_name("function") {
            let fname = node_text(func, src);
            check_symbol_purity(fname, line, violations);
            if fname == "range" {
                if let Some(args) = node.child_by_field_name("arguments") {
                    let mut ac = args.walk();
                    let items: Vec<String> = args
                        .named_children(&mut ac)
                        .map(|n| translate_py_expr(n, src, line, violations))
                        .collect();
                    if items.len() == 1 {
                        return format!("0..{}", items[0]);
                    } else if items.len() >= 2 {
                        return format!("{}..{}", items[0], items[1]);
                    }
                }
            }
        }
    }
    let raw = node_text(node, src);
    raw.replace("//", "/")
        .replace("True", "true")
        .replace("False", "false")
}
