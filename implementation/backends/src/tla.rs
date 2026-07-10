//! TLA+ formal verification transpiler target for UEAS Epoch 8.

use super::*;

/// TLA+ formal verification transpiler target.
///
/// Translates UEAS algorithm state mutations into TLA+ specifications
/// suitable for verification with the TLC model checker. Each algorithm
/// becomes a module with `Init`, `Next`, invariants, and a temporal
/// specification formula.
pub struct TlaTarget {
    init_conds: Vec<String>,
    next_disjuncts: Vec<String>,
    invariants: Vec<String>,
    var_names: Vec<String>,
}

impl TlaTarget {
    pub fn new() -> Self {
        Self {
            init_conds: Vec::new(),
            next_disjuncts: Vec::new(),
            invariants: Vec::new(),
            var_names: Vec::new(),
        }
    }
}

impl Default for TlaTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl TlaTarget {
    fn gather_body_stats(&mut self, body: &[serde_json::Value]) -> Result<(), TranspilationError> {
        for stmt in body {
            self.gather_statement_vars(stmt)?;
        }
        Ok(())
    }

    fn gather_statement_vars(
        &mut self,
        node: &serde_json::Value,
    ) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        match kind {
            "VariableDeclaration" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                if !children.is_empty() {
                    let name = children[0]["value"].as_str().unwrap_or("_").to_string();
                    if !self.var_names.contains(&name) {
                        self.var_names.push(name);
                    }
                }
            }
            "If" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                for child in children.iter().skip(1) {
                    if let Some(body) = child["children"].as_array() {
                        self.gather_body_stats(body)?;
                    }
                }
            }
            "WhileLoop" | "ForLoop" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("No children"))?;
                for child in children.iter().skip(1) {
                    if let Some(body) = child["children"].as_array() {
                        self.gather_body_stats(body)?;
                    } else {
                        self.gather_statement_vars(child)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn build_init(
        &mut self,
        params: &[String],
        _body: &[serde_json::Value],
    ) -> Result<(), TranspilationError> {
        for name in params {
            let init_val = format!("{}_init", name);
            self.init_conds
                .push(format!("    /\\ {} = {}", name, init_val));
        }
        for name in &self.var_names.clone() {
            self.init_conds.push(format!("    /\\ {} = 0", name));
        }
        self.init_conds.push("    /\\ result = 0".to_string());
        Ok(())
    }

    fn build_body(&mut self, body: &[serde_json::Value]) -> Result<(), TranspilationError> {
        for stmt in body {
            self.translate_statement(stmt)?;
        }
        Ok(())
    }

    fn translate_statement(&mut self, node: &serde_json::Value) -> Result<(), TranspilationError> {
        let kind = node["kind"].as_str().unwrap_or("");
        let children = node["children"].as_array();
        match kind {
            "VariableDeclaration" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let name = c[0]["value"].as_str().unwrap_or("_");
                let mut init_val = String::new();
                if c.len() > 2 {
                    self.generate_node_inline(&c[2], &mut init_val)?;
                } else {
                    init_val.push('0');
                }
                self.init_conds
                    .push(format!("    /\\ {} = {}", name, init_val));
            }
            "Assignment" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let target = c[0]["value"].as_str().unwrap_or("_");
                let mut expr = String::new();
                self.generate_node_inline(&c[1], &mut expr)?;
                self.next_disjuncts.push(format!(
                    "    \\/ /\\ TRUE\n       /\\ {}' = {}",
                    target, expr
                ));
            }
            "Return" => {
                let mut expr = String::new();
                if let Some(c) = children {
                    if let Some(val) = c.first() {
                        self.generate_node_inline(val, &mut expr)?;
                    }
                }
                self.next_disjuncts
                    .push(format!("    \\/ /\\ TRUE\n       /\\ result' = {}", expr));
            }
            "If" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node_inline(&c[0], &mut cond)?;

                let mut then_body_str = String::new();
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            match stmt["kind"].as_str().unwrap_or("") {
                                "Assignment" => {
                                    if let Some(sc) = stmt["children"].as_array() {
                                        if sc.len() >= 2 {
                                            let t = sc[0]["value"].as_str().unwrap_or("_");
                                            let mut e = String::new();
                                            self.generate_node_inline(&sc[1], &mut e)?;
                                            then_body_str.push_str(&format!(
                                                "            /\\ {}' = {}\n",
                                                t, e
                                            ));
                                        }
                                    }
                                }
                                "Return" => {
                                    let mut ret_val = String::new();
                                    if let Some(val) =
                                        stmt["children"].as_array().and_then(|a| a.first())
                                    {
                                        self.generate_node_inline(val, &mut ret_val)?;
                                    }
                                    then_body_str.push_str(&format!(
                                        "            /\\ result' = {}\n",
                                        ret_val
                                    ));
                                }
                                _ => {
                                    if let Some(body_inner) = stmt["children"].as_array() {
                                        for b in body_inner {
                                            self.translate_statement(b)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let mut else_body_str = String::new();
                if c.len() > 2 {
                    if let Some(body) = c[2]["children"].as_array() {
                        for stmt in body {
                            match stmt["kind"].as_str().unwrap_or("") {
                                "Assignment" => {
                                    if let Some(sc) = stmt["children"].as_array() {
                                        if sc.len() >= 2 {
                                            let t = sc[0]["value"].as_str().unwrap_or("_");
                                            let mut e = String::new();
                                            self.generate_node_inline(&sc[1], &mut e)?;
                                            else_body_str.push_str(&format!(
                                                "            /\\ {}' = {}\n",
                                                t, e
                                            ));
                                        }
                                    }
                                }
                                "Return" => {
                                    let mut ret_val = String::new();
                                    if let Some(val) =
                                        stmt["children"].as_array().and_then(|a| a.first())
                                    {
                                        self.generate_node_inline(val, &mut ret_val)?;
                                    }
                                    else_body_str.push_str(&format!(
                                        "            /\\ result' = {}\n",
                                        ret_val
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let mut disjunct = format!("    \\/ /\\ {}\n       /\\ ", cond);
                if !then_body_str.is_empty() {
                    disjunct.push_str(then_body_str.trim_end());
                } else {
                    disjunct.push_str("TRUE");
                }
                if !else_body_str.is_empty() {
                    disjunct.push_str(&format!(
                        "\n    \\/ /\\ ~({})\n       /\\ {}",
                        cond,
                        else_body_str.trim_end()
                    ));
                }
                self.next_disjuncts.push(disjunct);
            }
            "WhileLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node_inline(&c[0], &mut cond)?;

                let mut body_assigns = String::new();
                if c.len() > 1 {
                    if let Some(body) = c[1]["children"].as_array() {
                        for stmt in body {
                            if stmt["kind"].as_str().unwrap_or("") == "Assignment" {
                                if let Some(sc) = stmt["children"].as_array() {
                                    if sc.len() >= 2 {
                                        let t = sc[0]["value"].as_str().unwrap_or("_");
                                        let mut e = String::new();
                                        self.generate_node_inline(&sc[1], &mut e)?;
                                        body_assigns
                                            .push_str(&format!("            /\\ {}' = {}\n", t, e));
                                    }
                                }
                            }
                        }
                    }
                }

                self.next_disjuncts.push(format!(
                    "    \\/ /\\ {}\n       /\\ {}       /\\ UNCHANGED <<result>>",
                    cond, body_assigns
                ));
            }
            "ForLoop" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let iterator = c[0]["value"].as_str().unwrap_or("_");
                let mut range_expr = String::new();
                self.generate_node_inline(&c[1], &mut range_expr)?;

                let mut body_assigns = String::new();
                for child in &c[2..] {
                    if child["kind"].as_str().unwrap_or("") == "Assignment" {
                        if let Some(sc) = child["children"].as_array() {
                            if sc.len() >= 2 {
                                let t = sc[0]["value"].as_str().unwrap_or("_");
                                let mut e = String::new();
                                self.generate_node_inline(&sc[1], &mut e)?;
                                body_assigns.push_str(&format!("            /\\ {}' = {}\n", t, e));
                            }
                        }
                    }
                }

                self.next_disjuncts.push(format!(
                    "    \\/ /\\ {} \\in 0..({} - 1)\n       /\\ {}\n       /\\ {}' = {}\n       /\\ UNCHANGED <<result>>",
                    iterator, range_expr, body_assigns.trim_end(), iterator, iterator
                ));
            }
            "Assert" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node_inline(&c[0], &mut cond)?;
                self.invariants
                    .push(format!("Assert({}, \"assertion failed\")", cond));
            }
            "Invariant" => {
                let c = children.ok_or_else(|| TranspilationError::new("No children"))?;
                let mut cond = String::new();
                self.generate_node_inline(&c[0], &mut cond)?;
                self.invariants.push(cond);
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_node_inline(
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
            "BooleanLiteral" => {
                if let Some(val) = node["value"].as_str() {
                    output.push_str(val);
                } else if let Some(val) = node["value"].as_bool() {
                    output.push_str(if val { "TRUE" } else { "FALSE" });
                } else {
                    output.push_str("FALSE");
                }
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
                let tla_op = match op {
                    "+" => "+",
                    "-" => "-",
                    "*" => "*",
                    "/" => "\\div",
                    "mod" => "%",
                    "==" => "=",
                    "!=" => "#",
                    "<" => "<",
                    "<=" => "=<",
                    ">" => ">",
                    ">=" => ">=",
                    "and" => "/\\",
                    "or" => "\\/",
                    _ => op,
                };
                output.push('(');
                self.generate_node_inline(&children[1], output)?;
                output.push(' ');
                output.push_str(tla_op);
                output.push(' ');
                self.generate_node_inline(&children[2], output)?;
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
                let tla_op = match op {
                    "not" => "~",
                    "-" => "-",
                    _ => op,
                };
                output.push_str(tla_op);
                output.push('(');
                self.generate_node_inline(&children[1], output)?;
                output.push(')');
            }
            "FunctionCall" => {
                let children = node["children"]
                    .as_array()
                    .ok_or_else(|| TranspilationError::new("FunctionCall missing children"))?;
                let name = children[0]["value"].as_str().unwrap_or("unknown");
                match name {
                    "sqrt" => output.push_str("Sqrt"),
                    "length" => output.push_str("Len"),
                    "cardinality" => output.push_str("Cardinality"),
                    _ => output.push_str(name),
                }
                output.push('(');
                for (i, arg) in children.iter().skip(1).enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.generate_node_inline(arg, output)?;
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

    fn generate_node_expr(
        &self,
        node: &serde_json::Value,
        output: &mut String,
        _indent: usize,
    ) -> Result<(), TranspilationError> {
        self.generate_node_inline(node, output)
    }
}

impl TargetGenerator for TlaTarget {
    fn language(&self) -> &str {
        "tlaplus"
    }

    fn version(&self) -> &str {
        "2.18"
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
                        output.push('\n');
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
                self.generate_node_expr(&root, &mut output, 0)?;
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
            "Identifier",
            "FunctionCall",
            "Algorithm",
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
            ("Integer", "Int"),
            ("Real", "Real"),
            ("Boolean", "Bool"),
            ("String", "String"),
        ]
    }
}

impl TlaTarget {
    fn generate_algo(
        &self,
        node: &serde_json::Value,
        output: &mut String,
    ) -> Result<(), TranspilationError> {
        let mut state = Self::new();

        let children = node["children"]
            .as_array()
            .ok_or_else(|| TranspilationError::new("Algorithm missing children"))?;
        if children.is_empty() {
            return Ok(());
        }

        let name = children[0]["value"].as_str().unwrap_or("unnamed");
        let complexity = children
            .iter()
            .find(|c| c["kind"] == "StringLiteral")
            .and_then(|c| c["value"].as_str())
            .unwrap_or("O(1)");

        let mut params: Vec<String> = Vec::new();
        for child in children.iter().skip(1) {
            if child["kind"] == "Parameter" {
                if let Some(pc) = child["children"].as_array() {
                    if !pc.is_empty() {
                        let pname = pc[0]["value"].as_str().unwrap_or("_").to_string();
                        if !state.var_names.contains(&pname) {
                            state.var_names.push(pname.clone());
                        }
                        params.push(pname);
                    }
                }
            }
        }

        let mut body: Vec<&serde_json::Value> = Vec::new();
        for child in children.iter().skip(1) {
            let child_kind = child["kind"].as_str().unwrap_or("");
            match child_kind {
                "Parameter" | "Type" | "StringLiteral" | "VariableBinding" => {
                    // skip
                }
                _ => {
                    body.push(child);
                }
            }
        }
        let body_clone: Vec<serde_json::Value> = body.iter().map(|v| (*v).clone()).collect();

        state.gather_body_stats(&body_clone)?;
        state.build_init(&params, &body_clone)?;
        state.build_body(&body_clone)?;

        let all_vars: Vec<String> = state.var_names.clone();
        let all_vars_str = all_vars.join(", ");

        output.push_str(&format!("---- MODULE {} ----\n", name));
        output.push_str("EXTENDS Naturals, Sequences, TLC\n\n");
        output.push_str(&format!("\\* Algorithm: {}\n", name));
        output.push_str(&format!("\\* Complexity: {}\n\n", complexity));

        output.push_str("VARIABLES ");
        output.push_str(&all_vars_str);
        if !all_vars.is_empty() {
            output.push_str(", ");
        }
        output.push_str("result\n\n");

        output.push_str("Init ==\n");
        if state.init_conds.is_empty() {
            output.push_str("    TRUE\n");
        } else {
            output.push_str(&state.init_conds.join("\n"));
            output.push('\n');
        }

        output.push_str("\nTypeInvariant ==\n");
        for var_name in &all_vars {
            output.push_str(&format!("    /\\ {} \\in Int\n", var_name));
        }
        output.push_str("    /\\ result \\in Int");

        if !state.invariants.is_empty() {
            output.push_str("\n\nInvariant ==\n");
            output.push_str("    /\\ TypeInvariant\n");
            for inv in &state.invariants {
                output.push_str(&format!("    /\\ {}\n", inv));
            }
        }

        output.push_str("\n\nNext ==\n");
        if state.next_disjuncts.is_empty() {
            output.push_str("    UNCHANGED <<");
            output.push_str(&all_vars_str);
            output.push_str(", result>>\n");
        } else {
            for disjunct in &state.next_disjuncts {
                output.push_str(disjunct);
                output.push('\n');
            }
        }

        output.push_str(&format!(
            "\nSpec == Init /\\ [][Next]_<<{}>>\n",
            if all_vars.is_empty() {
                "result".to_string()
            } else {
                format!("{}, result", all_vars_str)
            }
        ));

        output.push_str("====\n");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ueas_kernel::ast::AstNodeFactory;

    #[test]
    fn tla_target_language_is_tlaplus() {
        let target = TlaTarget::new();
        assert_eq!(target.language(), "tlaplus");
    }

    #[test]
    fn tla_target_version_is_2_18() {
        let target = TlaTarget::new();
        assert_eq!(target.version(), "2.18");
    }

    #[test]
    fn tla_generates_integer_literal() {
        let target = TlaTarget::new();
        let node = AstNodeFactory::integer_literal("42");
        let json = serde_json::to_string(&node).unwrap();
        let result = target.generate(&json).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn tla_generates_addition() {
        let target = TlaTarget::new();
        let left = AstNodeFactory::integer_literal("1");
        let right = AstNodeFactory::integer_literal("2");
        let expr = AstNodeFactory::binary_expression("+", left, right);
        let json = serde_json::to_string(&expr).unwrap();
        let result = target.generate(&json).unwrap();
        assert_eq!(result, "(1 + 2)");
    }

    #[test]
    fn tla_generates_spec_with_init() {
        let target = TlaTarget::new();
        let algo = AstNodeFactory::algorithm(
            "Test",
            vec![AstNodeFactory::parameter(
                "x",
                AstNodeFactory::type_node("Integer", vec![]),
            )],
            Some(AstNodeFactory::type_node("Integer", vec![])),
            "O(1)",
            vec![],
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::binary_expression(
                    "+",
                    AstNodeFactory::identifier("x"),
                    AstNodeFactory::integer_literal("1"),
                ),
            ))],
        );
        let json = serde_json::to_string(&algo).unwrap();
        let result = target.generate(&json).unwrap();
        assert!(result.contains("---- MODULE Test ----"));
        assert!(result.contains("EXTENDS Naturals, Sequences, TLC"));
        assert!(result.contains("VARIABLES "));
        assert!(result.contains("x"));
        assert!(result.contains("Init =="));
        assert!(result.contains("Spec =="));
        assert!(result.contains("====\n"));
    }
}
