//! Lean 4 formal verification transpiler target for UEAS Epoch 8.

use super::*;

/// Lean 4 theorem-prover transpiler target.
///
/// Translates UEAS algorithms into Lean 4 proof structures.
/// Emits `def` declarations with proof obligation placeholders
/// for `Require`/`Ensure` contracts, `assert` conditions, and
/// `invariant` statements.
pub struct LeanTarget;

impl TargetGenerator for LeanTarget {
    fn language(&self) -> &str {
        "lean4"
    }

    fn version(&self) -> &str {
        "4.0"
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
            "BooleanLiteral",
            "StringLiteral",
            "Identifier",
            "FunctionCall",
            "VariableDeclaration",
            "ConstDeclaration",
            "Assignment",
            "Return",
            "If",
            "WhileLoop",
            "ForLoop",
            "Assert",
            "Invariant",
        ]
    }

    fn type_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("Integer", "ℕ"),
            ("Real", "ℝ"),
            ("Boolean", "Bool"),
            ("String", "String"),
        ]
    }
}

impl LeanTarget {
    fn prelude_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("sqrt", "Real.sqrt"),
            ("length", "List.length"),
            ("cardinality", "Finset.card"),
        ]
    }

    fn map_function_name(&self, name: &str) -> String {
        for (ueas_name, lean_name) in &self.prelude_map() {
            if *ueas_name == name {
                return lean_name.to_string();
            }
        }
        name.to_string()
    }

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
        let mut return_type_str = "ℕ".to_string();

        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            if kind == "Parameter" {
                if let Some(pc) = child["children"].as_array() {
                    if pc.len() >= 2 {
                        let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                        let ptype = pc[1]["children"]
                            .as_array()
                            .and_then(|tc| tc.first())
                            .and_then(|t| t["value"].as_str())
                            .unwrap_or("ℕ");
                        let lean_type = self.ueas_type_to_lean(ptype);
                        declared.insert(pname.clone());
                        params.push((pname, lean_type.to_string()));
                    } else if !pc.is_empty() {
                        let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                        declared.insert(pname.clone());
                        params.push((pname, "ℕ".to_string()));
                    }
                }
            } else if kind == "Type" {
                let return_type = child["children"]
                    .as_array()
                    .and_then(|tc| tc.first())
                    .and_then(|t| t["value"].as_str())
                    .unwrap_or("Integer");
                return_type_str = self.ueas_type_to_lean(return_type).to_string();
            }
        }

        // complexity annotation
        let complexity = children
            .iter()
            .find(|c| c["kind"] == "StringLiteral")
            .and_then(|c| c["value"].as_str())
            .unwrap_or("O(1)");

        output.push_str(&format!(
            "/-\nAlgorithm: {}\nComplexity: {}\n-/\n\n",
            name, complexity
        ));
        output.push_str(&format!("def {} ", name));

        for (pname, ptype) in &params {
            output.push_str(&format!("({} : {}) ", pname, ptype));
        }

        output.push_str(&format!(": {} :=\n", return_type_str));

        // body statements
        let mut body_lines = 0;
        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "Parameter" | "Type" | "StringLiteral" | "VariableBinding" => {}
                _ => {
                    self.generate_statement(child, output, 1, declared)?;
                    body_lines += 1;
                }
            }
        }

        if body_lines == 0 {
            output.push_str("  0\n");
        }

        output.push('\n');
        Ok(())
    }

    fn ueas_type_to_lean(&self, ueas_type: &str) -> &str {
        match ueas_type {
            "Integer" => "ℕ",
            "Real" => "ℝ",
            "Boolean" => "Bool",
            "String" => "String",
            _ => "ℕ",
        }
    }

    fn generate_statement(
        &self,
        node: &serde_json::Value,
        output: &mut String,
        indent: usize,
        declared: &mut HashSet<String>,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        let prefix = "  ".repeat(indent);
        let children = node["children"].as_array();

        match kind {
            "VariableDeclaration" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_").to_string();
                declared.insert(name.clone());
                let type_name = if c.len() >= 2 && c[1]["kind"] == "Type" {
                    c[1]["children"]
                        .as_array()
                        .and_then(|tc| tc.first())
                        .and_then(|t| t["value"].as_str())
                        .unwrap_or("Integer")
                } else {
                    "Integer"
                };
                let lean_type = self.ueas_type_to_lean(type_name);
                output.push_str(&format!("{}let {} : {} := ", prefix, name, lean_type));
                if c.len() > 2 && c.last().map(|v| v["kind"].as_str().unwrap_or("")) != Some("Type")
                {
                    self.generate_node(c.last().unwrap(), output)?;
                } else {
                    output.push('0');
                }
                output.push('\n');
                Ok(())
            }
            "Assignment" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let target = c[0]["value"].as_str().unwrap_or("_").to_string();
                if declared.contains(&target) {
                    output.push_str(&format!("{}{} := ", prefix, target));
                } else {
                    declared.insert(target.clone());
                    output.push_str(&format!("{}let {} := ", prefix, target));
                }
                self.generate_node(&c[1], output)?;
                output.push('\n');
                Ok(())
            }
            "Return" => {
                output.push_str(&prefix);
                if let Some(c) = children {
                    if let Some(val) = c.first() {
                        self.generate_node(val, output)?;
                    }
                }
                output.push('\n');
                Ok(())
            }
            "If" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}if ", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(" then\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                if c.len() > 2 {
                    output.push_str(&format!("{}else\n", prefix));
                    if let Some(body) = c[2]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                Ok(())
            }
            "WhileLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}-- LOOP: while ", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(" do\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                output.push_str(&format!("{}-- END LOOP\n", prefix));
                Ok(())
            }
            "ForLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let iterator = c[0]["value"].as_str().unwrap_or("_");
                output.push_str(&format!("{}-- FOR {} in ", prefix, iterator));
                self.generate_node(&c[1], output)?;
                output.push_str(" :\n");
                for child in &c[2..] {
                    self.generate_statement(child, output, indent + 1, declared)?;
                }
                output.push_str(&format!("{}-- END FOR\n", prefix));
                Ok(())
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}-- ASSERT: ", prefix));
                self.generate_node(&c[0], output)?;
                output.push('\n');
                Ok(())
            }
            "Invariant" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}have h : ", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(" := by\n");
                output.push_str(&format!("{}  sorry\n", prefix));
                Ok(())
            }
            _ => {
                output.push_str(&prefix);
                self.generate_node(node, output)?;
                output.push('\n');
                Ok(())
            }
        }
    }

    fn generate_node(
        &self,
        node: &serde_json::Value,
        output: &mut String,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");

        match kind {
            "IntegerLiteral" => {
                if let Some(val) = node["value"].as_str() {
                    output.push_str(val);
                } else if let Some(val) = node["value"].as_i64() {
                    output.push_str(&val.to_string());
                } else {
                    output.push('0');
                }
            }
            "RealLiteral" => {
                let val = node["value"].as_f64().unwrap_or(0.0);
                output.push_str(&format!("({} : ℝ)", val));
            }
            "BooleanLiteral" => {
                let val = node["value"].as_bool().unwrap_or(false);
                output.push_str(if val { "true" } else { "false" });
            }
            "StringLiteral" => {
                let val = node["value"].as_str().unwrap_or("");
                output.push_str(&format!("\"{}\"", val));
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
                let lean_op = match op {
                    "+" => "+",
                    "-" => "-",
                    "*" => "*",
                    "/" => "/",
                    "mod" => "%",
                    "==" => " == ",
                    "!=" => " ≠ ",
                    "<" => " < ",
                    "<=" => " ≤ ",
                    ">" => " > ",
                    ">=" => " ≥ ",
                    _ => op,
                };
                output.push('(');
                self.generate_node(&children[1], output)?;
                output.push_str(lean_op);
                self.generate_node(&children[2], output)?;
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
                self.generate_node(&children[1], output)?;
                output.push(')');
            }
            "FunctionCall" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("FunctionCall missing children"))?;
                let name = children[0]["value"].as_str().unwrap_or("unknown");
                let lean_name = self.map_function_name(name);
                output.push_str(&lean_name);
                output.push(' ');
                for (i, arg) in children.iter().skip(1).enumerate() {
                    if i > 0 {
                        output.push(' ');
                    }
                    self.generate_node(arg, output)?;
                }
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
    use ueas_kernel::ast::AstNodeFactory;

    #[test]
    fn lean_target_language_is_lean4() {
        let target = LeanTarget;
        assert_eq!(target.language(), "lean4");
    }

    #[test]
    fn lean_target_version_is_4_0() {
        let target = LeanTarget;
        assert_eq!(target.version(), "4.0");
    }

    #[test]
    fn lean_generates_integer_literal() {
        let target = LeanTarget;
        let node = AstNodeFactory::integer_literal("42");
        let json = serde_json::to_string(&node).unwrap();
        let result = target.generate(&json).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn lean_generates_addition() {
        let target = LeanTarget;
        let left = AstNodeFactory::integer_literal("1");
        let right = AstNodeFactory::integer_literal("2");
        let expr = AstNodeFactory::binary_expression("+", left, right);
        let json = serde_json::to_string(&expr).unwrap();
        let result = target.generate(&json).unwrap();
        assert_eq!(result, "(1+2)");
    }

    #[test]
    fn lean_generates_function_definition() {
        let target = LeanTarget;
        let algo = AstNodeFactory::algorithm(
            "add",
            vec![],
            Some(AstNodeFactory::type_node("Integer", vec![])),
            "O(1)",
            vec![],
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("42"),
            ))],
        );
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let result = target.generate(&json).unwrap();
        assert!(result.contains("def add"));
        assert!(result.contains("Algorithm: add"));
        assert!(result.contains("Complexity: O(1)"));
        assert!(result.contains("42"));
    }
}
