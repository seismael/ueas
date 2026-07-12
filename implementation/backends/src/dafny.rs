//! Dafny formal verification + transpilation target for UEAS Final Architecture.
//!
//! Replaces the 5 deprecated imperative transpilers (Python, Rust, C++, Java, JS)
//! with a single Dafny pipeline. The DafnyTarget generates verifiable .dfy source
//! code that can be fed to the Z3 SMT solver for mathematical proof, then compiled
//! to C++, Python, Java, Go, C#, and JavaScript via `dafny build`.

use super::*;

/// Dafny transpiler target — single source of truth for all imperative code generation.
pub struct DafnyTarget;

impl TargetGenerator for DafnyTarget {
    fn language(&self) -> &str {
        "dafny"
    }

    fn version(&self) -> &str {
        "4.6.0"
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
            _ => self
                .generate_node(&root, &mut String::new())
                .map(|_| String::new()),
        }
    }

    fn supported_kinds(&self) -> Vec<&str> {
        vec![
            "Algorithm",
            "VariableDeclaration",
            "Assignment",
            "Return",
            "If",
            "WhileLoop",
            "ForLoop",
            "Assert",
            "Invariant",
            "ConstDeclaration",
            "Break",
            "Continue",
            "IntegerLiteral",
            "RealLiteral",
            "BooleanLiteral",
            "StringLiteral",
            "Identifier",
            "BinaryExpression",
            "UnaryExpression",
            "FunctionCall",
            "ListLiteral",
            "SetLiteral",
            "Type",
            "Parameter",
        ]
    }

    fn type_map(&self) -> Vec<(&str, &str)> {
        vec![
            ("Integer", "int"),
            ("Real", "real"),
            ("Boolean", "bool"),
            ("String", "string"),
        ]
    }
}

impl DafnyTarget {
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

        // Collect parameters, return type, requires/ensures clauses
        let mut params = Vec::new();
        let mut return_type = "int".to_string();

        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "Parameter" | "VariableDeclaration" => {
                    if let Some(pc) = child["children"].as_array() {
                        if !pc.is_empty() {
                            let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                            declared.insert(pname.clone());
                            params.push(format!("{}: int", pname));
                        }
                    }
                }
                "Type" => {
                    return_type = type_str(child);
                }
                _ => {}
            }
        }

        // Dafny method header
        output.push_str(&format!("method {}(", name));
        output.push_str(&params.join(", "));
        if return_type != "void" {
            output.push_str(&format!(") returns (result: {})\n", return_type));
        } else {
            output.push_str(")\n");
        }

        // requires/ensures from Require/Ensure blocks (come as body children)
        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "Parameter"
                | "Type"
                | "VariableDeclaration"
                | "StringLiteral"
                | "VariableBinding" => {
                    // skip metadata
                }
                "Invariant" => {
                    // preamble-level invariant → requires clause
                    if let Some(ic) = child["children"].as_array() {
                        if !ic.is_empty() {
                            let mut cond = String::new();
                            self.generate_node(&ic[0], &mut cond)?;
                            output.push_str(&format!("  requires {}\n", cond));
                        }
                    }
                }
                _ => {}
            }
        }

        output.push_str("{\n");

        // Body statements
        for child in children.iter().skip(1) {
            let kind = child["kind"].as_str().unwrap_or("");
            match kind {
                "Parameter" | "Type" | "StringLiteral" | "VariableBinding" | "Invariant" => {
                    continue; // skip metadata
                }
                "VariableDeclaration" => {
                    if let Some(vc) = child["children"].as_array() {
                        if vc.len() >= 2 {
                            let vname = vc[0]["value"].as_str().unwrap_or("_");
                            let mut val = String::new();
                            if vc.len() > 2 {
                                self.generate_node(&vc[2], &mut val)?;
                            } else {
                                val.push('0');
                            }
                            output.push_str(&format!("  var {} := {};\n", vname, val));
                        }
                    }
                }
                _ => {
                    self.generate_statement(child, output, 1, declared)?;
                }
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
        let prefix = "  ".repeat(indent);
        let children = node["children"].as_array();

        match kind {
            "Assignment" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let target = c[0]["value"].as_str().unwrap_or("_");
                let mut val = String::new();
                self.generate_node(&c[1], &mut val)?;
                if !declared.contains(target) {
                    declared.insert(target.to_string());
                    output.push_str(&format!("{}var {} := {};\n", prefix, target, val));
                } else {
                    output.push_str(&format!("{}{} := {};\n", prefix, target, val));
                }
            }
            "Return" => {
                if let Some(expr) = children.and_then(|c| c.first()) {
                    let mut val = String::new();
                    self.generate_node(expr, &mut val)?;
                    output.push_str(&format!("{}return {};\n", prefix, val));
                }
            }
            "If" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node(&c[0], &mut cond)?;
                output.push_str(&format!("{}if {} {{\n", prefix, cond));
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
                let mut cond = String::new();
                self.generate_node(&c[0], &mut cond)?;
                output.push_str(&format!("{}while {}\n", prefix, cond));

                // invariants inside the while body
                let body_node = if c.len() > 1 { &c[1] } else { node };
                let body_children = body_node["children"].as_array();

                if let Some(bc) = body_children {
                    let (invariants, rest): (Vec<_>, Vec<_>) = bc
                        .iter()
                        .partition(|s| s["kind"].as_str().unwrap_or("") == "Invariant");

                    for inv in &invariants {
                        if let Some(ic) = inv["children"].as_array() {
                            if !ic.is_empty() {
                                let mut inv_cond = String::new();
                                self.generate_node(&ic[0], &mut inv_cond)?;
                                output.push_str(&format!("{}  invariant {}\n", prefix, inv_cond));
                            }
                        }
                    }

                    output.push_str(&format!("{}{{\n", prefix));
                    for stmt in &rest {
                        self.generate_statement(stmt, output, indent + 1, declared)?;
                    }
                } else {
                    output.push_str(&format!("{}{{\n", prefix));
                }
                output.push_str(&format!("{}}}\n", prefix));
            }
            "ForLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let iter = c[0]["value"].as_str().unwrap_or("i");
                let mut coll = String::new();
                self.generate_node(&c[1], &mut coll)?;
                output.push_str(&format!("{}for {} := 0 to |{}| - 1\n", prefix, iter, coll));

                let body_node = if c.len() > 2 { &c[2] } else { node };
                let body_children = body_node["children"].as_array();

                if let Some(bc) = body_children {
                    let (invariants, rest): (Vec<_>, Vec<_>) = bc
                        .iter()
                        .partition(|s| s["kind"].as_str().unwrap_or("") == "Invariant");

                    for inv in &invariants {
                        if let Some(ic) = inv["children"].as_array() {
                            if !ic.is_empty() {
                                let mut inv_cond = String::new();
                                self.generate_node(&ic[0], &mut inv_cond)?;
                                output.push_str(&format!("{}  invariant {}\n", prefix, inv_cond));
                            }
                        }
                    }

                    output.push_str(&format!("{}{{\n", prefix));
                    for stmt in &rest {
                        self.generate_statement(stmt, output, indent + 1, declared)?;
                    }
                } else {
                    output.push_str(&format!("{}{{\n", prefix));
                }
                output.push_str(&format!("{}}}\n", prefix));
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node(&c[0], &mut cond)?;
                output.push_str(&format!("{}assert {};\n", prefix, cond));
            }
            "Invariant" => {
                // Skipped — handled inline by WhileLoop/ForLoop
            }
            "Break" => {
                output.push_str(&format!("{}break;\n", prefix));
            }
            "Continue" => {}
            "ConstDeclaration" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_");
                let mut val = String::new();
                if c.len() > 2 {
                    self.generate_node(&c[2], &mut val)?;
                } else {
                    val.push('0');
                }
                output.push_str(&format!("{}const {} := {};\n", prefix, name, val));
            }
            _ => {
                // Fallback: try expression node
                let mut expr = String::new();
                self.generate_node(node, &mut expr)?;
                output.push_str(&format!("{}{};\n", prefix, expr));
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
                let val = node["value"].as_i64().unwrap_or(0);
                output.push_str(&val.to_string());
            }
            "RealLiteral" => {
                let val = node["value"].as_f64().unwrap_or(0.0);
                output.push_str(&format!("{}.0", val as i64));
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
                let mapped = self.map_fn(name);
                output.push_str(&mapped);
            }
            "BinaryExpression" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("BinaryExpression missing children"))?;
                if c.len() < 3 {
                    return Err(TranspilationError::new("BinaryExpression needs 3 children"));
                }
                let op = c[0]["value"].as_str().unwrap_or("+");
                let op_map = match op {
                    "+" => "+",
                    "-" => "-",
                    "*" => "*",
                    "/" => "/",
                    "%" | "mod" => "%",
                    "==" => "==",
                    "!=" => "!=",
                    "<" => "<",
                    "<=" => "<=",
                    ">" => ">",
                    ">=" => ">=",
                    "and" | "&&" => "&&",
                    "or" | "||" => "||",
                    _ => op,
                };
                output.push('(');
                self.generate_node(&c[1], output)?;
                output.push_str(&format!(" {} ", op_map));
                self.generate_node(&c[2], output)?;
                output.push(')');
            }
            "UnaryExpression" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("UnaryExpression missing children"))?;
                if c.len() < 2 {
                    return Err(TranspilationError::new("UnaryExpression needs 2 children"));
                }
                let op = c[0]["value"].as_str().unwrap_or("-");
                output.push_str(&format!("({}", op));
                self.generate_node(&c[1], output)?;
                output.push(')');
            }
            "FunctionCall" => {
                let c = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("FunctionCall missing children"))?;
                let name = c[0]["value"].as_str().unwrap_or("fn");
                output.push_str(&self.map_fn(name));
                output.push('(');
                for (i, arg) in c.iter().skip(1).enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.generate_node(arg, output)?;
                }
                output.push(')');
            }
            "ListLiteral" | "SetLiteral" => {
                let children = node["children"].as_array();
                output.push('[');
                if let Some(elems) = children {
                    for (i, elem) in elems.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        self.generate_node(elem, output)?;
                    }
                }
                output.push(']');
            }
            _ => {
                return Err(TranspilationError::new(format!(
                    "Unsupported node kind: {}",
                    kind
                )));
            }
        }
        Ok(())
    }

    fn map_fn(&self, name: &str) -> String {
        match name {
            "sqrt" => "Dafny.Sqrt".to_string(),
            "length" => "".to_string(), // Dafny uses |seq| syntax
            "cardinality" => "".to_string(),
            "append" => "".to_string(),
            _ => name.to_string(),
        }
    }
}

fn type_str(node: &serde_json::Value) -> String {
    let kind = node["kind"].as_str().unwrap_or("");
    match kind {
        "Type" => {
            let name = node["children"]
                .as_array()
                .and_then(|c| c.first())
                .and_then(|c| c["value"].as_str())
                .unwrap_or("int");
            match name {
                "Integer" => "int".to_string(),
                "Real" => "real".to_string(),
                "Boolean" => "bool".to_string(),
                "String" => "string".to_string(),
                "List" => "seq<int>".to_string(),
                _ => "int".to_string(),
            }
        }
        _ => "int".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ueas_kernel::ast::*;

    #[test]
    fn dafny_target_language_is_dafny() {
        let t = DafnyTarget;
        assert_eq!(t.language(), "dafny");
    }

    #[test]
    fn dafny_target_version_is_4_6_0() {
        let t = DafnyTarget;
        assert_eq!(t.version(), "4.6.0");
    }

    #[test]
    fn dafny_generates_integer_literal() {
        let t = DafnyTarget;
        let mut out = String::new();
        t.generate_node(
            &serde_json::json!({"kind": "IntegerLiteral", "value": 42}),
            &mut out,
        )
        .unwrap();
        assert_eq!(out, "42");
    }

    #[test]
    fn dafny_generates_addition() {
        let t = DafnyTarget;
        let mut out = String::new();
        t.generate_node(
            &serde_json::json!({
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "StringLiteral", "value": "+"},
                    {"kind": "IntegerLiteral", "value": 1},
                    {"kind": "IntegerLiteral", "value": 2}
                ]
            }),
            &mut out,
        )
        .unwrap();
        assert_eq!(out, "(1 + 2)");
    }

    #[test]
    fn dafny_generates_method_definition() {
        let t = DafnyTarget;
        let algo = AstNodeFactory::algorithm(
            "Test",
            vec![],
            None,
            "O(1)",
            vec![],
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("42"),
            ))],
        );
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let output = t.generate(&json).unwrap();
        assert!(output.contains("method Test("));
        assert!(output.contains("return 42"));
    }

    #[test]
    fn dafny_generates_requires_clause() {
        let t = DafnyTarget;
        let invariant = AstNodeFactory::invariant_stmt(
            AstNodeFactory::binary_expression(
                ">",
                AstNodeFactory::identifier("n"),
                AstNodeFactory::integer_literal("0"),
            ),
            None,
        );
        let algo = AstNodeFactory::algorithm(
            "Test",
            vec![],
            None,
            "O(1)",
            vec![],
            vec![
                invariant,
                AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("0"))),
            ],
        );
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string(&program).unwrap();
        let output = t.generate(&json).unwrap();
        assert!(output.contains("requires"));
    }
}
