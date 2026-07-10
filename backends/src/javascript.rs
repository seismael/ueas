//! JavaScript (ES2020) transpiler target for UEAS Epoch 6.

use super::*;

pub struct JavaScriptTarget;

impl TargetGenerator for JavaScriptTarget {
    fn language(&self) -> &str {
        "javascript"
    }

    fn version(&self) -> &str {
        "ES2020"
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
                self.generate_node(&root, &mut output, 0)?;
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
            ("Integer", "Number"),
            ("Real", "Number"),
            ("Boolean", "boolean"),
            ("String", "string"),
        ]
    }
}

impl JavaScriptTarget {
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
        output.push_str(&format!("function {}(", name));
        output.push_str(&params.join(", "));
        output.push_str(") {\n");
        for child in children.iter().skip(body_start) {
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
        let prefix = "  ".repeat(indent);
        let children = node["children"].as_array();
        match kind {
            "VariableDeclaration" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_").to_string();
                declared.insert(name.clone());
                output.push_str(&format!("{}let {} = ", prefix, name));
                if c.len() > 2 {
                    self.generate_node(&c[2], output, 0)?;
                } else {
                    output.push_str("null");
                }
                output.push_str(";\n");
                Ok(())
            }
            "Assignment" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let target = c[0]["value"].as_str().unwrap_or("_").to_string();
                if declared.contains(&target) {
                    output.push_str(&format!("{}{} = ", prefix, target));
                } else {
                    declared.insert(target.clone());
                    output.push_str(&format!("{}let {} = ", prefix, target));
                }
                self.generate_node(&c[1], output, 0)?;
                output.push_str(";\n");
                Ok(())
            }
            "Return" => {
                output.push_str(&format!("{}return ", prefix));
                if let Some(c) = children {
                    if let Some(val) = c.first() {
                        self.generate_node(val, output, 0)?;
                    }
                }
                output.push_str(";\n");
                Ok(())
            }
            "If" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}if (", prefix));
                self.generate_node(&c[0], output, 0)?;
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
                Ok(())
            }
            "WhileLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}while (", prefix));
                if !c.is_empty() {
                    self.generate_node(&c[0], output, 0)?;
                }
                output.push_str(") {\n");
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output, indent + 1, declared)?;
                        }
                    }
                }
                output.push_str(&format!("{}}}\n", prefix));
                Ok(())
            }
            "ForLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let iterator = c[0]["value"].as_str().unwrap_or("_");
                declared.insert(iterator.to_string());
                output.push_str(&format!("{}for (let {} of ", prefix, iterator));
                self.generate_node(&c[1], output, 0)?;
                output.push_str(") {\n");
                for child in &c[2..] {
                    self.generate_statement(child, output, indent + 1, declared)?;
                }
                output.push_str(&format!("{}}}\n", prefix));
                Ok(())
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}console.assert(", prefix));
                self.generate_node(&c[0], output, 0)?;
                output.push_str(", \"Assertion failed\");\n");
                Ok(())
            }
            "Invariant" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                output.push_str(&format!("{}// invariant: ", prefix));
                self.generate_node(&c[0], output, 0)?;
                output.push('\n');
                Ok(())
            }
            _ => {
                output.push_str(&prefix);
                self.generate_node(node, output, 0)?;
                output.push_str(";\n");
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
                let js_op = match op {
                    "+" | "-" | "*" => op,
                    "/" => "/",
                    "mod" => "%",
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => op,
                    _ => op,
                };
                output.push('(');
                self.generate_node(&children[1], output, 0)?;
                output.push(' ');
                output.push_str(js_op);
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
                let js_op = match op {
                    "not" => "!",
                    _ => op,
                };
                output.push_str(js_op);
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
                    "sqrt" => output.push_str("Math.sqrt"),
                    "cardinality" => output.push_str("length"),
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
    fn js_target_language_is_javascript() {
        let target = JavaScriptTarget;
        assert_eq!(target.language(), "javascript");
    }

    #[test]
    fn js_target_version_is_es2020() {
        let target = JavaScriptTarget;
        assert_eq!(target.version(), "ES2020");
    }

    #[test]
    fn js_generates_integer_literal() {
        let target = JavaScriptTarget;
        let ast = r#"{"kind":"IntegerLiteral","value":"42"}"#;
        let result = target.generate(ast).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn js_generates_addition() {
        let target = JavaScriptTarget;
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
    fn js_generates_function_definition() {
        let target = JavaScriptTarget;
        let ast = r#"{
            "kind": "Algorithm",
            "children": [
                {"kind": "Identifier", "value": "add"},
                {"kind": "Parameter", "children": [{"kind": "Identifier", "value": "a"}]},
                {"kind": "Parameter", "children": [{"kind": "Identifier", "value": "b"}]},
                {"kind": "Return", "children": [
                    {"kind": "BinaryExpression", "children": [
                        {"kind": "Identifier", "value": "+"},
                        {"kind": "Identifier", "value": "a"},
                        {"kind": "Identifier", "value": "b"}
                    ]}
                ]}
            ]
        }"#;
        let result = target.generate(ast).unwrap();
        assert!(result.contains("function add(a, b)"));
        assert!(result.contains("return (a + b);"));
    }
}
