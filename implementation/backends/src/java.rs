//! Java 17 transpiler target for UEAS Epoch 6.

use super::*;

pub struct JavaTarget;

impl TargetGenerator for JavaTarget {
    fn language(&self) -> &str {
        "java"
    }

    fn version(&self) -> &str {
        "17"
    }

    fn generate(&self, ast_json: &str) -> Result<String, TranspilationError> {
        let root: serde_json::Value =
            serde_json::from_str(ast_json).map_err(|e| TranspilationError::new(e.to_string()))?;

        let kind = root["kind"].as_str().unwrap_or("");
        match kind {
            "Program" => {
                let mut output = String::new();
                let mut declared = HashSet::new();
                output.push_str("import java.util.*;\n\n");
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
            ("Integer", "long"),
            ("Real", "double"),
            ("Boolean", "boolean"),
            ("String", "String"),
        ]
    }
}

impl JavaTarget {
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
        for child in children.iter().skip(1) {
            if child["kind"] == "Parameter" || child["kind"] == "VariableDeclaration" {
                if let Some(pc) = child["children"].as_array() {
                    if !pc.is_empty() {
                        let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                        declared.insert(pname.clone());
                        params.push(pname);
                    }
                }
            }
        }
        output.push_str(&format!("public static long {}(", name));
        output.push_str(
            &params
                .iter()
                .map(|p| format!("long {}", p))
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push_str(") {\n");
        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "Parameter" | "Type" | "StringLiteral" | "VariableBinding" => {}
                _ => self.generate_statement(child, output, 1, declared)?,
            }
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
                output.push_str(&format!("{}long {} = ", prefix, name));
                if c.len() > 2 {
                    self.generate_node(&c[2], output)?;
                } else {
                    output.push('0');
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
                    output.push_str(&format!("{}var {} = ", prefix, target));
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
                output.push_str(&format!("{}if (", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(") {\n");
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
                output.push_str(&format!("{}while (", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(") {\n");
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
                output.push_str(&format!("{}for (var {} : ", prefix, iterator));
                self.generate_node(&c[1], output)?;
                output.push_str(") {\n");
                for child in &c[2..] {
                    self.generate_statement(child, output, indent + 1, declared)?;
                }
                output.push_str(&format!("{}}}\n", prefix));
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}assert ", prefix));
                self.generate_node(&c[0], output)?;
                output.push_str(" : \"assertion failed\";\n");
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
                output.push_str(val);
            }
            "RealLiteral" => {
                let val = node["value"].as_f64().unwrap_or(0.0);
                output.push_str(&val.to_string());
            }
            "BooleanLiteral" => {
                let val = node["value"].as_bool().unwrap_or(false);
                output.push_str(if val { "true" } else { "false" });
            }
            "SetLiteral" | "ListLiteral" => {
                output.push_str("java.util.List.of(");
                if let Some(elems) = node["children"].as_array() {
                    for (i, elem) in elems.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        self.generate_node(elem, output)?;
                    }
                }
                output.push_str(")");
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
                let java_op = match op {
                    "+" | "-" | "*" => op,
                    "/" => "/",
                    "mod" => "%",
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => op,
                    _ => op,
                };
                output.push('(');
                self.generate_node(&children[1], output)?;
                output.push(' ');
                output.push_str(java_op);
                output.push(' ');
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
                match name {
                    "sqrt" => output.push_str("Math.sqrt"),
                    "length" => output.push_str("size"),
                    "cardinality" => output.push_str("size"),
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
mod tests {
    use super::*;
    use ueas_kernel::ast::*;

    #[test]
    fn java_target_language_is_java() {
        let target = JavaTarget;
        assert_eq!(target.language(), "java");
    }

    #[test]
    fn java_target_version_is_17() {
        let target = JavaTarget;
        assert_eq!(target.version(), "17");
    }

    #[test]
    fn java_generates_integer_literal() {
        let target = JavaTarget;
        let ast = r#"{"kind":"IntegerLiteral","value":"42"}"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn java_generates_addition() {
        let target = JavaTarget;
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
    fn java_generates_function_definition() {
        let target = JavaTarget;
        let ast = r#"{
            "kind": "Algorithm",
            "children": [
                {"kind": "Identifier", "value": "testFunc"},
                {"kind": "Type", "children": [{"kind": "Identifier", "value": "Integer"}]},
                {"kind": "Return", "children": [{"kind": "IntegerLiteral", "value": "42"}]}
            ]
        }"#;
        let result = target.generate(ast).unwrap();
        assert!(result.contains("public static long testFunc()"));
        assert!(result.contains("return 42;"));
    }

    #[test]
    fn java_generates_empty_algorithm() {
        let algo = AstNodeFactory::algorithm("Empty", vec![], None, "O(1)", vec![], vec![]);
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let target = JavaTarget;
        let output = target.generate(&json).unwrap();
        assert!(output.contains("public static long Empty("));
    }

    #[test]
    fn java_generates_while_loop() {
        let cond = AstNodeFactory::binary_expression(
            "==",
            AstNodeFactory::integer_literal("1"),
            AstNodeFactory::integer_literal("1"),
        );
        let body_node = AstNode::internal(
            AstNodeKind::WhileLoop,
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("0"),
            ))],
            None,
        );
        let while_loop = AstNodeFactory::while_loop(cond, vec![body_node]);
        let algo =
            AstNodeFactory::algorithm("While", vec![], None, "O(1)", vec![], vec![while_loop]);
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let target = JavaTarget;
        let output = target.generate(&json).unwrap();
        assert!(output.contains("while ("));
    }

    #[test]
    fn java_type_map_has_all_primitives() {
        let target = JavaTarget;
        let map = target.type_map();
        assert!(map.contains(&("Integer", "long")));
        assert!(map.contains(&("Boolean", "boolean")));
    }
}
