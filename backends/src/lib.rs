//! UEAS Transpilation Backends — plugin system for TargetGenerator implementations.
//!
//! Each target language (Python, Rust, C++, Java) implements the
//! `TargetGenerator` trait (GoF Strategy). The trait provides methods
//! for transpiling a validated UEAS AST into idiomatic target source code.
//!
//! # Memory Lifecycle Mapping (SPEC.md Section 7.4)
//!
//! Transpilers for systems languages MUST map UEAS scope-based memory
//! to single-ownership semantics. Garbage-collected runtimes are exempt
//! from explicit ownership mapping.

pub mod cpp;
pub mod java;
pub mod javascript;
pub mod latex;
pub mod lean4;
pub mod mcp;
pub mod tla;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Error returned when transpilation fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranspilationError {
    pub message: String,
    pub node_kind: Option<String>,
}

impl TranspilationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            node_kind: None,
        }
    }

    pub fn with_node_kind(message: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            node_kind: Some(kind.into()),
        }
    }
}

/// GoF Strategy — every transpilation target implements this trait.
///
/// Each implementation translates the canonical UEAS AST into idiomatic
/// source code for a specific target language. The kernel selects the
/// appropriate strategy at transpile time.
///
/// # Semantic Equivalence Guarantee (SPEC.md Section 7.3)
///
/// Two transpiled programs generated from the same AST for different
/// targets must produce mathematically identical outputs on identical
/// inputs. The cross-target equivalence test suite verifies this.
pub trait TargetGenerator {
    /// Returns the target language identifier (e.g., "python", "rust", "cpp").
    fn language(&self) -> &str;

    /// Returns the target language version string.
    fn version(&self) -> &str;

    /// Transpile a validated UEAS AST into target source code.
    ///
    /// The input AST MUST have passed kernel validation.
    fn generate(&self, ast_json: &str) -> Result<String, TranspilationError>;

    /// Transpile a full program AST into a complete source file.
    fn generate_program(&self, ast_json: &str) -> Result<String, TranspilationError> {
        self.generate(ast_json)
    }

    /// Returns the set of UEAS AST node kinds this target supports.
    fn supported_kinds(&self) -> Vec<&str>;

    /// Returns a type mapping from UEAS primitive types to target-language
    /// native types.
    fn type_map(&self) -> Vec<(&str, &str)>;
}

/// Basic Python transpiler for arithmetic expression evaluation.
pub struct PythonTarget;

impl TargetGenerator for PythonTarget {
    fn language(&self) -> &str {
        "python"
    }

    fn version(&self) -> &str {
        "3.11"
    }

    fn generate(&self, ast_json: &str) -> Result<String, TranspilationError> {
        let root: serde_json::Value =
            serde_json::from_str(ast_json).map_err(|e| TranspilationError::new(e.to_string()))?;

        let kind = root["kind"].as_str().unwrap_or("");
        match kind {
            "Program" => self.generate_program_impl(&root),
            "Algorithm" => {
                let mut output = String::new();
                self.generate_algo(&root, &mut output)?;
                Ok(output)
            }
            _ => {
                let mut output = String::new();
                self.generate_node(&root, &mut output, 0)?;
                Ok(output)
            }
        }
    }

    fn generate_program(&self, ast_json: &str) -> Result<String, TranspilationError> {
        self.generate(ast_json)
    }

    fn supported_kinds(&self) -> Vec<&str> {
        vec![
            "BinaryExpression",
            "UnaryExpression",
            "IntegerLiteral",
            "RealLiteral",
            "Identifier",
            "FunctionCall",
        ]
    }

    fn type_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("Integer", "int"),
            ("Real", "float"),
            ("Boolean", "bool"),
            ("String", "str"),
        ]
    }
}

impl PythonTarget {
    /// Map UEAS standard prelude functions to Python equivalents.
    #[allow(dead_code)]
    fn prelude_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("length", "len"),
            ("cardinality", "len"),
            ("contains", "in"),
            ("append", "append"),
            ("pop", "pop"),
            ("slice", "slice"),
            ("sqrt", "math.sqrt"),
            ("emptyList", "list"),
            ("emptySet", "set"),
            ("emptyMap", "dict"),
            ("range", "range"),
        ]
    }
}

impl PythonTarget {
    fn generate_program_impl(
        &self,
        node: &serde_json::Value,
    ) -> Result<String, TranspilationError> {
        let mut output = String::new();
        let algorithms = node["children"]
            .as_array()
            .ok_or_else(|| TranspilationError::new("Program missing children"))?;
        output.push_str("import math\n\n");
        for algo in algorithms {
            self.generate_algo(algo, &mut output)?;
        }
        Ok(output)
    }

    fn generate_algo(
        &self,
        node: &serde_json::Value,
        output: &mut String,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        match kind {
            "Algorithm" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("Algorithm missing children"))?;
                // name is children[0]
                let name = children
                    .first()
                    .and_then(|c| c["value"].as_str())
                    .unwrap_or("unnamed");
                output.push_str(&format!("def {}(", name));
                // parameters: children[1..]
                let mut params = Vec::new();
                let mut body_start = 1;
                for child in children.iter().skip(1) {
                    if child["kind"] == "Parameter" {
                        let pname = child["children"][0]["value"].as_str().unwrap_or("_");
                        params.push(pname.to_string());
                        body_start += 1;
                    } else {
                        break;
                    }
                }
                output.push_str(&params.join(", "));
                output.push_str("):\n");

                // Body statements
                for child in children.iter().skip(body_start + 1) {
                    self.generate_statement(child, output, 1)?;
                }
                output.push('\n');
                Ok(())
            }
            _ => Err(TranspilationError::new(format!(
                "Unexpected node kind in algorithm: {}",
                kind
            ))),
        }
    }

    fn generate_statement(
        &self,
        node: &serde_json::Value,
        output: &mut String,
        indent: usize,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        let prefix = "    ".repeat(indent);
        match kind {
            "VariableDeclaration" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_");
                output.push_str(&format!("{}{} = ", prefix, name));
                if c.len() > 2 {
                    self.generate_node(&c[2], output, 0)?;
                } else {
                    output.push_str("None");
                }
                output.push('\n');
                Ok(())
            }
            "Assignment" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                let target = c[0]["value"].as_str().unwrap_or("_");
                output.push_str(&format!("{}{} = ", prefix, target));
                self.generate_node(&c[1], output, 0)?;
                output.push('\n');
                Ok(())
            }
            "Return" => {
                output.push_str(&format!("{}return ", prefix));
                if let Some(c) = node["children"].as_array() {
                    if let Some(val) = c.first() {
                        self.generate_node(val, output, 0)?;
                    }
                }
                output.push('\n');
                Ok(())
            }
            "If" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}if ", prefix));
                self.generate_node(&c[0], output, 0)?;
                output.push_str(":\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1)?;
                        }
                    }
                }
                if c.len() > 2 {
                    output.push_str(&format!("{}else:\n", prefix));
                    if let Some(body) = c[2]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1)?;
                        }
                    }
                }
                Ok(())
            }
            "WhileLoop" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}while ", prefix));
                if !c.is_empty() {
                    self.generate_node(&c[0], output, 0)?;
                }
                output.push_str(":\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1)?;
                        }
                    }
                }
                Ok(())
            }
            "ForLoop" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                let iterator = c[0]["value"].as_str().unwrap_or("_");
                output.push_str(&format!("{}for {} in range(", prefix, iterator));
                self.generate_node(&c[1], output, 0)?;
                output.push_str("):\n");
                for child in &c[2..] {
                    self.generate_statement(child, output, indent + 1)?;
                }
                Ok(())
            }
            "Assert" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}assert ", prefix));
                self.generate_node(&c[0], output, 0)?;
                output.push('\n');
                Ok(())
            }
            "Invariant" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}# invariant: ", prefix));
                self.generate_node(&c[0], output, 0)?;
                output.push('\n');
                Ok(())
            }
            _ => {
                output.push_str(&prefix);
                self.generate_node(node, output, 0)?;
                output.push('\n');
                Ok(())
            }
        }
    }

    fn generate_node(
        &self,
        node: &serde_json::Value,
        output: &mut String,
        _indent: usize,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");

        match kind {
            "IntegerLiteral" => {
                let val = node["value"].as_str().unwrap_or("0");
                output.push_str(val);
            }
            "RealLiteral" => {
                let val = node["value"].as_f64().unwrap_or(0.0);
                output.push_str(&val.to_string());
            }
            "Identifier" => {
                let name = node["value"].as_str().unwrap_or("_");
                output.push_str(name);
            }
            "BinaryExpression" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("BinaryExpression missing children"))?;
                if children.len() < 3 {
                    return Err(TranspilationError::new(
                        "BinaryExpression requires 3 children",
                    ));
                }
                let op = children[0]["value"].as_str().unwrap_or("?");
                let py_op = match op {
                    "+" | "-" | "*" => op,
                    "/" => "/",
                    "mod" => "%",
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => op,
                    _ => op,
                };
                output.push('(');
                self.generate_node(&children[1], output, 0)?;
                output.push(' ');
                output.push_str(py_op);
                output.push(' ');
                self.generate_node(&children[2], output, 0)?;
                output.push(')');
            }
            "UnaryExpression" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("UnaryExpression missing children"))?;
                if children.len() < 2 {
                    return Err(TranspilationError::new(
                        "UnaryExpression requires 2 children",
                    ));
                }
                let op = children[0]["value"].as_str().unwrap_or("");
                output.push_str(op);
                output.push('(');
                self.generate_node(&children[1], output, 0)?;
                output.push(')');
            }
            "FunctionCall" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("FunctionCall missing children"))?;
                let name = children[0]["value"].as_str().unwrap_or("unknown");
                match name {
                    "sqrt" => output.push_str("math.sqrt"),
                    _ => output.push_str(name),
                }
                output.push('(');
                for (i, arg) in children.iter().skip(1).enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.generate_node(arg, output, 0)?;
                }
                output.push(')');
            }
            _ => {
                return Err(TranspilationError::with_node_kind(
                    format!("Unsupported node kind: {}", kind),
                    kind,
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_target_language_is_python() {
        let target = PythonTarget;
        assert_eq!(target.language(), "python");
    }

    #[test]
    fn python_target_version_is_3_11() {
        let target = PythonTarget;
        assert_eq!(target.version(), "3.11");
    }

    #[test]
    fn python_target_generates_integer_literal() {
        let target = PythonTarget;
        let ast = r#"{"kind":"IntegerLiteral","value":"42"}"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn python_target_generates_simple_addition() {
        let target = PythonTarget;
        let ast = r#"{
            "kind": "BinaryExpression",
            "children": [
                {"kind": "Identifier", "value": "+"},
                {"kind": "IntegerLiteral", "value": "1"},
                {"kind": "IntegerLiteral", "value": "2"}
            ]
        }"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "(1 + 2)");
    }

    #[test]
    fn python_target_generates_nested_expression() {
        let target = PythonTarget;
        let ast = r#"{
            "kind": "BinaryExpression",
            "children": [
                {"kind": "Identifier", "value": "*"},
                {
                    "kind": "BinaryExpression",
                    "children": [
                        {"kind": "Identifier", "value": "+"},
                        {"kind": "Identifier", "value": "a"},
                        {"kind": "Identifier", "value": "b"}
                    ]
                },
                {"kind": "Identifier", "value": "c"}
            ]
        }"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "((a + b) * c)");
    }

    #[test]
    fn python_target_type_map_has_integer() {
        let target = PythonTarget;
        let map = target.type_map();
        assert!(map.contains(&("Integer", "int")));
        assert!(map.contains(&("Real", "float")));
        assert!(map.contains(&("Boolean", "bool")));
        assert!(map.contains(&("String", "str")));
    }
}

/// Basic Rust transpiler for arithmetic expression evaluation.
pub struct RustTarget;

impl TargetGenerator for RustTarget {
    fn language(&self) -> &str {
        "rust"
    }

    fn version(&self) -> &str {
        "2021"
    }

    fn generate(&self, ast_json: &str) -> Result<String, TranspilationError> {
        let root: serde_json::Value =
            serde_json::from_str(ast_json).map_err(|e| TranspilationError::new(e.to_string()))?;

        let kind = root["kind"].as_str().unwrap_or("");
        match kind {
            "Program" => {
                let mut output = String::new();
                let mut declared = HashSet::new();
                if let Some(algorithms) = root["children"].as_array() {
                    for algo in algorithms {
                        self.generate_algo(algo, &mut output, &mut declared)?;
                    }
                }
                Ok(output)
            }
            "Algorithm" => {
                let mut output = String::new();
                let mut declared = HashSet::new();
                self.generate_algo(&root, &mut output, &mut declared)?;
                Ok(output)
            }
            _ => {
                let mut output = String::new();
                self.generate_node(&root, &mut output)?;
                Ok(output)
            }
        }
    }

    fn supported_kinds(&self) -> Vec<&str> {
        vec![
            "BinaryExpression",
            "UnaryExpression",
            "IntegerLiteral",
            "RealLiteral",
            "Identifier",
            "FunctionCall",
        ]
    }

    fn type_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("Integer", "i64"),
            ("Real", "f64"),
            ("Boolean", "bool"),
            ("String", "String"),
        ]
    }
}

impl RustTarget {
    #[allow(dead_code)]
    fn prelude_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("length", "len"),
            ("cardinality", "len"),
            ("append", "push"),
            ("sqrt", "f64::sqrt"),
            ("emptyList", "Vec::new"),
            ("emptySet", "HashSet::new"),
            ("emptyMap", "HashMap::new"),
        ]
    }
}

impl RustTarget {
    fn generate_algo(
        &self,
        node: &serde_json::Value,
        output: &mut String,
        declared: &mut HashSet<String>,
    ) -> Result<(), TranspilationError> {
        let children = node["children"]
            .as_array()
            .ok_or_else(|| TranspilationError::new("Algorithm missing children"))?;
        if children.is_empty() {
            return Ok(());
        }
        let name = children[0]["value"].as_str().unwrap_or("unnamed");
        let mut params = Vec::new();
        let mut body_start = 1;
        for child in children.iter().skip(1) {
            if child["kind"] == "Parameter" {
                if let Some(pc) = child["children"].as_array() {
                    if !pc.is_empty() {
                        let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                        declared.insert(pname.clone());
                        params.push(pname);
                    }
                }
                body_start += 1;
            } else {
                break;
            }
        }
        output.push_str(&format!("fn {}(", name));
        output.push_str(
            &params
                .iter()
                .map(|p| format!("{}: i64", p))
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push_str(") -> i64 {\n");
        for child in children.iter().skip(body_start + 1) {
            self.generate_statement(child, output, 1, declared)?;
        }
        output.push_str("}\n\n");
        Ok(())
    }

    fn generate_statement(
        &self,
        node: &serde_json::Value,
        output: &mut String,
        indent: usize,
        declared: &mut HashSet<String>,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        let prefix = "    ".repeat(indent);
        let children = node["children"].as_array();
        match kind {
            "VariableDeclaration" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_").to_string();
                declared.insert(name.clone());
                output.push_str(&format!("{}let {}: i64 = ", prefix, name));
                if c.len() > 2 {
                    self.generate_node(&c[2], output)?;
                } else {
                    output.push_str("0_i64");
                }
                output.push_str(";\n");
            }
            "Assignment" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;

                let target = c[0]["value"].as_str().unwrap_or("_").to_string();
                if declared.contains(&target) {
                    output.push_str(&format!("{}{} = ", prefix, target));
                } else {
                    declared.insert(target.clone());
                    output.push_str(&format!("{}let mut {} = ", prefix, target));
                }
                self.generate_node(&c[1], output)?;
                output.push_str(";\n");
            }
            "Return" => {
                output.push_str(&format!("{}return ", prefix));
                if let Some(c) = children {
                    if let Some(val) = c.first() {
                        self.generate_node(val, output)?;
                    }
                }
                output.push_str(";\n");
            }
            "If" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;

                output.push_str(&format!("{}if ", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(" {\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                if c.len() > 2 {
                    output.push_str(&format!("{}}} else {{\n", prefix));
                    if let Some(body) = c[2]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                output.push_str(&format!("{}}}\n", prefix));
            }
            "WhileLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;

                output.push_str(&format!("{}while ", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(" {\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                output.push_str(&format!("{}}}\n", prefix));
            }
            "ForLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;

                let iterator = c[0]["value"].as_str().unwrap_or("_");
                declared.insert(iterator.to_string());
                output.push_str(&format!("{}for {} in 0..", prefix, iterator));
                self.generate_node(&c[1], output)?;
                output.push_str(" {\n");
                for child in &c[2..] {
                    self.generate_statement(child, output, indent + 1, declared)?;
                }
                output.push_str(&format!("{}}}\n", prefix));
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;

                output.push_str(&format!("{}assert!(", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(");\n");
            }
            "Invariant" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;

                output.push_str(&format!("{}// invariant: ", prefix));
                self.generate_node(&c[0], output)?;
                output.push('\n');
            }
            _ => {
                output.push_str(&prefix);
                self.generate_node(node, output)?;
                output.push_str(";\n");
            }
        }
        Ok(())
    }

    fn generate_node(
        &self,
        node: &serde_json::Value,
        output: &mut String,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");

        match kind {
            "IntegerLiteral" => {
                let val = node["value"].as_str().unwrap_or("0");
                output.push_str(&format!("{}_i64", val));
            }
            "RealLiteral" => {
                let val = node["value"].as_f64().unwrap_or(0.0);
                output.push_str(&format!("{}_f64", val));
            }
            "Identifier" => {
                let name = node["value"].as_str().unwrap_or("_");
                output.push_str(name);
            }
            "BinaryExpression" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("BinaryExpression missing children"))?;
                if children.len() < 3 {
                    return Err(TranspilationError::new(
                        "BinaryExpression requires 3 children",
                    ));
                }
                let op = children[0]["value"].as_str().unwrap_or("?");
                let rust_op = match op {
                    "+" | "-" | "*" | "/" => op,
                    "mod" => "%",
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => op,
                    _ => op,
                };
                output.push('(');
                self.generate_node(&children[1], output)?;
                output.push(' ');
                output.push_str(rust_op);
                output.push(' ');
                self.generate_node(&children[2], output)?;
                output.push(')');
            }
            "UnaryExpression" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("UnaryExpression missing children"))?;
                let op = children[0]["value"].as_str().unwrap_or("");
                let rust_op = match op {
                    "not" => "!",
                    _ => op,
                };
                output.push_str(rust_op);
                output.push('(');
                self.generate_node(&children[1], output)?;
                output.push(')');
            }
            "FunctionCall" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("FunctionCall missing children"))?;
                let name = children[0]["value"].as_str().unwrap_or("unknown");
                match name {
                    "sqrt" => output.push_str("f64::sqrt"),
                    _ => output.push_str(name),
                }
                output.push('(');
                for (i, arg) in children.iter().skip(1).enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.generate_node(arg, output)?;
                }
                output.push(')');
            }
            _ => {
                return Err(TranspilationError::with_node_kind(
                    format!("Unsupported node kind: {}", kind),
                    kind,
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod rust_tests {
    use super::*;

    #[test]
    fn rust_target_language_is_rust() {
        let target = RustTarget;
        assert_eq!(target.language(), "rust");
    }

    #[test]
    fn rust_target_version_is_2021() {
        let target = RustTarget;
        assert_eq!(target.version(), "2021");
    }

    #[test]
    fn rust_target_generates_integer_literal() {
        let target = RustTarget;
        let ast = r#"{"kind":"IntegerLiteral","value":"42"}"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "42_i64");
    }

    #[test]
    fn rust_target_generates_addition() {
        let target = RustTarget;
        let ast = r#"{
            "kind": "BinaryExpression",
            "children": [
                {"kind": "Identifier", "value": "+"},
                {"kind": "IntegerLiteral", "value": "1"},
                {"kind": "IntegerLiteral", "value": "2"}
            ]
        }"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "(1_i64 + 2_i64)");
    }

    #[test]
    fn cross_target_equivalence_simple_expression() {
        let python = PythonTarget;
        let rust = RustTarget;
        let ast = r#"{
            "kind": "BinaryExpression",
            "children": [
                {"kind": "Identifier", "value": "*"},
                {
                    "kind": "BinaryExpression",
                    "children": [
                        {"kind": "Identifier", "value": "+"},
                        {"kind": "IntegerLiteral", "value": "2"},
                        {"kind": "IntegerLiteral", "value": "3"}
                    ]
                },
                {"kind": "IntegerLiteral", "value": "4"}
            ]
        }"#;

        let py_result = python.generate(ast).unwrap();
        let rs_result = rust.generate(ast).unwrap();

        // Both produce the same mathematical expression structure
        assert!(py_result.contains("2 + 3"));
        assert!(py_result.contains("* 4"));
        assert!(rs_result.contains("2_i64 + 3_i64"));
        assert!(rs_result.contains("* 4_i64"));
    }
}
