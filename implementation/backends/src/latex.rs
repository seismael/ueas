//! LaTeX algorithm2e transpiler target for UEAS Epoch 10 (Academic Bridge).

use super::*;

/// LaTeX algorithm2e transpiler target.
///
/// Generates publishable academic pseudocode from UEAS algorithms using
/// the `algorithm2e` package. This is a non-executable cosmetic typesetting
/// target — the academic bridge described in ADR 0012.
pub struct LatexTarget;

impl TargetGenerator for LatexTarget {
    fn language(&self) -> &str {
        "latex"
    }

    fn version(&self) -> &str {
        "algorithm2e/v5.2"
    }

    fn generate(&self, ast_json: &str) -> Result<String, TranspilationError> {
        let root: serde_json::Value =
            serde_json::from_str(ast_json).map_err(|e| TranspilationError::new(e.to_string()))?;

        let kind = root["kind"].as_str().unwrap_or("");
        match kind {
            "Program" => {
                let mut output = String::new();
                if let Some(algorithms) = root["children"].as_array() {
                    for algo in algorithms {
                        self.generate_algo(algo, &mut output)?;
                    }
                }
                Ok(output)
            }
            "Algorithm" => {
                let mut output = String::new();
                self.generate_algo(&root, &mut output)?;
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
            ("Integer", "Integer"),
            ("Real", "Real"),
            ("Boolean", "Boolean"),
            ("String", "String"),
        ]
    }
}

impl LatexTarget {
    fn map_function_name<'a>(&self, name: &'a str) -> &'a str {
        match name {
            "sqrt" => "\\sqrt",
            "length" => "\\text{length}",
            _ => name,
        }
    }

    fn map_ueas_type_to_academic<'a>(&self, ueas_type: &'a str) -> &'a str {
        match ueas_type {
            "Integer" => "\\mathbb{Z}",
            "Real" => "\\mathbb{R}",
            "Boolean" => "\\mathbb{B}",
            "String" => "\\mathbb{S}",
            _ => ueas_type,
        }
    }

    fn generate_algo(
        &self,
        node: &serde_json::Value,
        output: &mut String,
    ) -> Result<(), TranspilationError> {
        let children = node["children"]
            .as_array()
            .ok_or_else(|| TranspilationError::new("Algorithm missing children"))?;
        if children.is_empty() {
            return Ok(());
        }

        let name = children[0]["value"].as_str().unwrap_or("unnamed");

        let mut params: Vec<String> = Vec::new();
        let mut param_types: Vec<String> = Vec::new();
        let mut return_type_ueas: Option<String> = None;
        let mut _complexity = "O(1)";

        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            if kind == "Parameter" {
                if let Some(pc) = child["children"].as_array() {
                    let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                    params.push(pname);
                    if pc.len() >= 2 {
                        let ptype = pc[1]["children"]
                            .as_array()
                            .and_then(|tc| tc.first())
                            .and_then(|t| t["value"].as_str())
                            .unwrap_or("Integer");
                        param_types.push(ptype.to_string());
                    } else {
                        param_types.push("Integer".to_string());
                    }
                }
            } else if kind == "Type" {
                return_type_ueas = child["children"]
                    .as_array()
                    .and_then(|tc| tc.first())
                    .and_then(|t| t["value"].as_str())
                    .map(|s| s.to_string());
            } else if kind == "StringLiteral" {
                _complexity = child["value"].as_str().unwrap_or("O(1)");
            }
        }

        let title = if name == "unnamed" {
            name.to_string()
        } else {
            let param_list = params.join(", ");
            if param_list.is_empty() {
                name.to_string()
            } else {
                format!("{}({})", name, param_list)
            }
        };

        output.push_str("\\begin{algorithm}[H]\n");
        output.push_str("\\SetAlgoLined\n");

        if !param_types.is_empty() || !params.is_empty() {
            output.push_str("\\KwIn{");
            let mut first = true;
            for pname in params.iter() {
                if !first {
                    output.push_str(", ");
                }
                first = false;
                output.push_str(&format!("${}$", pname));
            }
            if !param_types.is_empty() {
                let mut seen = std::collections::HashSet::new();
                let type_strs: Vec<String> = param_types
                    .iter()
                    .filter(|t| seen.insert(*t))
                    .map(|t| self.map_ueas_type_to_academic(t).to_string())
                    .collect();
                output.push_str(&format!(" \\in {}", type_strs.join(", ")));
            }
            output.push_str("}\n");
        }

        if let Some(ref rt) = return_type_ueas {
            let rt_tex = self.map_ueas_type_to_academic(rt);
            output.push_str(&format!("\\KwOut{{Result $r \\in {}$}}\n", rt_tex));
        }

        output.push_str(&format!("\\caption{{{}}}\n", title));

        if !params.is_empty() {
            output.push_str(&format!("\\SetKwFunction{{{}}}{{{}}}\n", name, name));
        }

        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "Parameter" | "Type" | "StringLiteral" | "VariableBinding" => {}
                _ => self.generate_statement(child, output)?,
            }
        }

        output.push_str("\\end{algorithm}\n");
        Ok(())
    }

    fn generate_statement(
        &self,
        node: &serde_json::Value,
        output: &mut String,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        let children = node["children"].as_array();

        match kind {
            "VariableDeclaration" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_");
                output.push_str(&format!("${} \\gets ", name));
                if c.len() > 2 {
                    self.generate_node(c.last().unwrap(), output)?;
                } else {
                    output.push('0');
                }
                output.push_str("$\\;\n");
                Ok(())
            }
            "Assignment" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let target = c[0]["value"].as_str().unwrap_or("_");
                output.push_str(&format!("${} \\gets ", target));
                self.generate_node(&c[1], output)?;
                output.push_str("$\\;\n");
                Ok(())
            }
            "Return" => {
                output.push_str("\\Return{$");
                if let Some(c) = children {
                    if let Some(val) = c.first() {
                        self.generate_node(val, output)?;
                    }
                }
                output.push_str("$}\n");
                Ok(())
            }
            "If" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node(&c[0], &mut cond)?;

                let has_else = c.len() > 2;

                if has_else {
                    output.push_str(&format!("\\eIf{{${}$}}{{", cond));
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output)?;
                        }
                    }
                    output.push_str("}{");
                    if let Some(body) = c[2]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output)?;
                        }
                    }
                    output.push_str("}\n");
                } else {
                    output.push_str(&format!("\\If{{${}$}}{{", cond));
                    if c.len() > 1 {
                        if let Some(body) = c[1]["children"].as_array() {
                            for stmt in body {
                                self.generate_statement(stmt, output)?;
                            }
                        }
                    }
                    output.push_str("}\n");
                }
                Ok(())
            }
            "WhileLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                if !c.is_empty() {
                    self.generate_node(&c[0], &mut cond)?;
                }
                output.push_str(&format!("\\While{{${}$}}{{", cond));
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            self.generate_statement(stmt, output)?;
                        }
                    }
                }
                output.push_str("}\n");
                Ok(())
            }
            "ForLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let iterator = c[0]["value"].as_str().unwrap_or("_");
                let mut coll = String::new();
                self.generate_node(&c[1], &mut coll)?;
                output.push_str(&format!("\\ForEach{{${} \\in {}$}}{{", iterator, coll));
                for child in &c[2..] {
                    self.generate_statement(child, output)?;
                }
                output.push_str("}\n");
                Ok(())
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node(&c[0], &mut cond)?;
                output.push_str(&format!("\\tcc{{Assert: {}}}\n", cond));
                Ok(())
            }
            "Invariant" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node(&c[0], &mut cond)?;
                output.push_str(&format!("\\tcc{{Invariant: {}}}\n", cond));
                Ok(())
            }
            _ => Ok(()),
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
                if let Some(val) = node["value"].as_f64() {
                    if val.fract() == 0.0 && val != f64::INFINITY && val != f64::NEG_INFINITY {
                        output.push_str(&format!("{:.0}", val));
                    } else {
                        output.push_str(&val.to_string());
                    }
                } else if let Some(val) = node["value"].as_str() {
                    output.push_str(val);
                } else {
                    output.push('0');
                }
            }
            "BooleanLiteral" => {
                if let Some(val) = node["value"].as_str() {
                    let tex = match val {
                        "true" => "\\text{true}",
                        "false" => "\\text{false}",
                        _ => val,
                    };
                    output.push_str(tex);
                } else if let Some(val) = node["value"].as_bool() {
                    output.push_str(if val { "\\text{true}" } else { "\\text{false}" });
                } else {
                    output.push_str("\\text{false}");
                }
            }
            "StringLiteral" => {
                let val = node["value"].as_str().unwrap_or("");
                output.push_str(&format!("\\text{{{}}}", val));
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
                let tex_op = match op {
                    "*" => "\\cdot",
                    "/" => "\\div",
                    "<=" => "\\leq",
                    ">=" => "\\geq",
                    "!=" => "\\neq",
                    "mod" => "\\bmod",
                    _ => op,
                };
                self.generate_node(&children[1], output)?;
                output.push(' ');
                output.push_str(tex_op);
                output.push(' ');
                self.generate_node(&children[2], output)?;
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
                let tex_op = match op {
                    "not" => "\\neg",
                    _ => op,
                };
                output.push_str(tex_op);
                output.push('(');
                self.generate_node(&children[1], output)?;
                output.push(')');
            }
            "FunctionCall" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("FunctionCall missing children"))?;
                let name = children[0]["value"].as_str().unwrap_or("unknown");
                let tex_name = self.map_function_name(name);
                let is_sqrt = name == "sqrt";
                output.push_str(tex_name);
                if is_sqrt && children.len() > 1 {
                    output.push('{');
                    for arg in children.iter().skip(1) {
                        self.generate_node(arg, output)?;
                    }
                    output.push('}');
                } else {
                    output.push('(');
                    for (i, arg) in children.iter().skip(1).enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        self.generate_node(arg, output)?;
                    }
                    output.push(')');
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

// ===== Function name helpers (not methods, called from generate_node) =====

impl LatexTarget {
    #[allow(dead_code)]
    fn format_function_call(&self, name: &str, args: &[String]) -> String {
        let tex_name = self.map_function_name(name);
        if name == "sqrt" {
            format!("{}{{{}}}", tex_name, args.join(""))
        } else {
            format!("{}({})", tex_name, args.join(", "))
        }
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use ueas_kernel::ast::*;

    #[test]
    fn latex_target_language_is_latex() {
        let target = LatexTarget;
        assert_eq!(target.language(), "latex");
    }

    #[test]
    fn latex_target_version_is_algo2e() {
        let target = LatexTarget;
        assert_eq!(target.version(), "algorithm2e/v5.2");
    }

    #[test]
    fn latex_generates_integer_literal() {
        let target = LatexTarget;
        let node = AstNodeFactory::integer_literal("42");
        let json = serde_json::to_string(&node).unwrap();
        let result = target.generate(&json).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn latex_generates_addition() {
        let target = LatexTarget;
        let left = AstNodeFactory::integer_literal("1");
        let right = AstNodeFactory::integer_literal("2");
        let expr = AstNodeFactory::binary_expression("+", left, right);
        let json = serde_json::to_string(&expr).unwrap();
        let result = target.generate(&json).unwrap();
        assert_eq!(result, "1 + 2");
    }

    #[test]
    fn latex_generates_algorithm_document() {
        let target = LatexTarget;
        let algo = AstNodeFactory::algorithm(
            "EuclideanDistance",
            vec![AstNodeFactory::parameter(
                "x1",
                AstNodeFactory::type_node("Real", vec![]),
            )],
            Some(AstNodeFactory::type_node("Real", vec![])),
            "O(1)",
            vec![],
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("42"),
            ))],
        );
        let json = serde_json::to_string(&algo).unwrap();
        let result = target.generate(&json).unwrap();
        assert!(result.contains("\\begin{algorithm}[H]"));
        assert!(result.contains("\\SetAlgoLined"));
        assert!(result.contains("\\caption{EuclideanDistance(x1)}"));
        assert!(result.contains("\\Return{$"));
        assert!(result.contains("\\end{algorithm}"));
    }

    #[test]
    fn latex_generates_for_each_loop() {
        let assign = AstNodeFactory::assignment(
            AstNodeFactory::identifier("x"),
            AstNodeFactory::identifier("item"),
        );
        let for_loop =
            AstNodeFactory::for_loop("item", AstNodeFactory::identifier("data"), vec![assign]);
        let algo = AstNodeFactory::algorithm("Loop", vec![], None, "O(N)", vec![], vec![for_loop]);
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let target = LatexTarget;
        let output = target.generate(&json).unwrap();
        assert!(output.contains("\\ForEach"));
    }

    #[test]
    fn latex_generates_if_else() {
        let then_body = AstNode::internal(
            AstNodeKind::If,
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("1"),
            ))],
            None,
        );
        let else_body = AstNode::internal(
            AstNodeKind::If,
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("0"),
            ))],
            None,
        );
        let if_stmt = AstNode::internal(
            AstNodeKind::If,
            vec![AstNodeFactory::boolean_literal(true), then_body, else_body],
            None,
        );
        let algo = AstNodeFactory::algorithm("IfElse", vec![], None, "O(1)", vec![], vec![if_stmt]);
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let target = LatexTarget;
        let output = target.generate(&json).unwrap();
        assert!(output.contains("\\eIf"));
    }
}
