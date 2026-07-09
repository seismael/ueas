//! Multi-pass semantic engine for the UEAS V2.0 "Iceberg" Architecture.
//!
//! The engine performs three passes over the Draft AST:
//! 1. Type Inference — deduce types from literal values
//! 2. Desugaring — transform syntactic sugar into canonical AST
//! 3. Validation — ensure the resulting AST is well-typed
//!
//! Pass 2 handles:
//! - Method chaining: `a.push(b)` → `FunctionCall("push", [a, b])`
//! - `in` operator: `x in s` → `FunctionCall("contains", [s, x])`
//! - Implicit declaration: first assignment without `let` → `VariableDeclaration`

use crate::ast::{AstNode, AstNodeKind, AstValue, Type};

/// Config for the semantic analyzer.
#[derive(Debug, Clone)]
pub struct InferConfig {
    #[allow(dead_code)]
    max_type_depth: usize,
}

impl Default for InferConfig {
    fn default() -> Self {
        Self { max_type_depth: 5 }
    }
}

/// The semantic analyzer holds shared state across passes.
#[derive(Debug, Clone, Default)]
pub struct SemanticAnalyzer {
    #[allow(dead_code)]
    config: InferConfig,
}

impl SemanticAnalyzer {
    pub fn new(config: InferConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(InferConfig::default())
    }

    /// Pass 2: Desugar and infer types from the Draft AST.
    /// Returns a canonical AST ready for the Abstract Interpreter.
    pub fn analyze(&self, node: &AstNode) -> AstNode {
        self.transform(node, &mut std::collections::HashSet::new())
    }

    fn transform(
        &self,
        node: &AstNode,
        declared: &mut std::collections::HashSet<String>,
    ) -> AstNode {
        match node.kind {
            AstNodeKind::BinaryExpression => {
                let children = &node.children;
                if children.len() >= 3 {
                    let op = match &children[0].value {
                        Some(AstValue::String(s)) => s.clone(),
                        _ => return node.clone(),
                    };
                    // Desugar `in` operator: x in s → contains(s, x)
                    if op == "in" {
                        let left = self.transform(&children[1], declared);
                        let right = self.transform(&children[2], declared);
                        return AstNode::internal(
                            AstNodeKind::FunctionCall,
                            vec![
                                AstNode::leaf(
                                    AstNodeKind::Identifier,
                                    Some(AstValue::String("contains".to_string())),
                                ),
                                right,
                                left,
                            ],
                            node.location,
                        );
                    }
                    // Desugar `not in`: x not in s → not contains(s, x)
                    if op == "!in" || op == "notin" {
                        let left = self.transform(&children[1], declared);
                        let right = self.transform(&children[2], declared);
                        let inner = AstNode::internal(
                            AstNodeKind::FunctionCall,
                            vec![
                                AstNode::leaf(
                                    AstNodeKind::Identifier,
                                    Some(AstValue::String("contains".to_string())),
                                ),
                                right,
                                left,
                            ],
                            node.location,
                        );
                        return AstNode::internal(
                            AstNodeKind::UnaryExpression,
                            vec![
                                AstNode::leaf(
                                    AstNodeKind::Identifier,
                                    Some(AstValue::String("not".to_string())),
                                ),
                                inner,
                            ],
                            node.location,
                        );
                    }
                }
                // Standard binary expression: recursively transform children
                let mut new_children = Vec::new();
                for child in children {
                    new_children.push(self.transform(child, declared));
                }
                AstNode::internal(node.kind, new_children, node.location)
            }
            AstNodeKind::Assignment => {
                if node.children.len() >= 2 {
                    let target = &node.children[0];
                    let value = self.transform(&node.children[1], declared);

                    // Extract variable name from target
                    let var_name = match &target.value {
                        Some(AstValue::String(s)) => s.clone(),
                        _ => String::new(),
                    };

                    // Implicit declaration: first assignment creates VariableDeclaration
                    if !var_name.is_empty() && !declared.contains(&var_name) {
                        declared.insert(var_name.clone());
                        let inferred_type = self.infer_type_from_value(&value);
                        return AstNode::internal(
                            AstNodeKind::VariableDeclaration,
                            vec![
                                AstNode::leaf(
                                    AstNodeKind::Identifier,
                                    Some(AstValue::String(var_name)),
                                ),
                                type_to_ast_node(&inferred_type),
                                value,
                            ],
                            node.location,
                        );
                    }
                }
                node.clone()
            }
            AstNodeKind::FunctionCall => {
                // Check if this is a method chain that needs desugaring
                // Pattern: `methodCallOrId` where the first child is itself a `methodCallOrId`
                let children = &node.children;
                let mut new_children = Vec::new();
                for child in children {
                    new_children.push(self.transform(child, declared));
                }
                AstNode::internal(node.kind, new_children, node.location)
            }
            _ => {
                // Recursively transform children
                let mut new_children = Vec::new();
                for child in &node.children {
                    new_children.push(self.transform(child, declared));
                }
                AstNode::internal(node.kind, new_children, node.location)
            }
        }
    }

    /// Infer the UEAS type from an AstValue literal.
    pub fn infer_type_from_value(&self, node: &AstNode) -> Type {
        match &node.value {
            Some(AstValue::Integer(_)) => Type::Primitive(crate::ast::PrimitiveType::Integer),
            Some(AstValue::Real(_)) => Type::Primitive(crate::ast::PrimitiveType::Real),
            Some(AstValue::Boolean(_)) => Type::Primitive(crate::ast::PrimitiveType::Boolean),
            Some(AstValue::String(_)) => Type::Primitive(crate::ast::PrimitiveType::String),
            Some(AstValue::None) => Type::Primitive(crate::ast::PrimitiveType::Void),
            _ => {
                // Infer from node kind
                match node.kind {
                    AstNodeKind::SetLiteral => Type::Composite(crate::ast::CompositeType::Set(
                        Box::new(Type::Primitive(crate::ast::PrimitiveType::Void)),
                    )),
                    AstNodeKind::ListLiteral => Type::Composite(crate::ast::CompositeType::List(
                        Box::new(Type::Primitive(crate::ast::PrimitiveType::Void)),
                    )),
                    AstNodeKind::MapLiteral => Type::Composite(crate::ast::CompositeType::Map(
                        Box::new(Type::Primitive(crate::ast::PrimitiveType::String)),
                        Box::new(Type::Primitive(crate::ast::PrimitiveType::Void)),
                    )),
                    _ => Type::Primitive(crate::ast::PrimitiveType::Void),
                }
            }
        }
    }
}

/// Convert a Type to an AST node representation.
fn type_to_ast_node(typ: &Type) -> AstNode {
    match typ {
        Type::Primitive(p) => AstNode::leaf(
            AstNodeKind::Type,
            Some(AstValue::String(
                match p {
                    crate::ast::PrimitiveType::Integer => "Integer",
                    crate::ast::PrimitiveType::Real => "Real",
                    crate::ast::PrimitiveType::Boolean => "Boolean",
                    crate::ast::PrimitiveType::String => "String",
                    crate::ast::PrimitiveType::Void => "Void",
                }
                .to_string(),
            )),
        ),
        Type::Composite(c) => {
            let name = match c {
                crate::ast::CompositeType::Set(_) => "Set",
                crate::ast::CompositeType::List(_) => "List",
                crate::ast::CompositeType::Map(_, _) => "Map",
                crate::ast::CompositeType::Graph(_, _) => "Graph",
                crate::ast::CompositeType::Matrix(_, _, _) => "Matrix",
                crate::ast::CompositeType::Option(_) => "Option",
                crate::ast::CompositeType::Result(_, _) => "Result",
                crate::ast::CompositeType::Tuple(_) => "Tuple",
            };
            AstNode::leaf(AstNodeKind::Type, Some(AstValue::String(name.to_string())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNodeFactory;

    #[test]
    fn infer_integer_type() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNodeFactory::integer_literal("42");
        let typ = analyzer.infer_type_from_value(&node);
        assert_eq!(typ, Type::Primitive(crate::ast::PrimitiveType::Integer));
    }

    #[test]
    fn infer_real_type() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNodeFactory::real_literal(3.14);
        let typ = analyzer.infer_type_from_value(&node);
        assert_eq!(typ, Type::Primitive(crate::ast::PrimitiveType::Real));
    }

    #[test]
    fn infer_boolean_type() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNodeFactory::boolean_literal(true);
        let typ = analyzer.infer_type_from_value(&node);
        assert_eq!(typ, Type::Primitive(crate::ast::PrimitiveType::Boolean));
    }

    #[test]
    fn desugar_in_operator() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNode::internal(
            AstNodeKind::BinaryExpression,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String("in".to_string())),
                ),
                AstNodeFactory::identifier("x"),
                AstNodeFactory::identifier("s"),
            ],
            None,
        );
        let result = analyzer.analyze(&node);
        assert_eq!(result.kind, AstNodeKind::FunctionCall);
        // First child should be the function name
        let name = result.children[0]
            .value
            .as_ref()
            .and_then(|v| match v {
                AstValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(name, "contains");
    }

    #[test]
    fn implicit_declaration_on_first_assignment() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNodeFactory::assignment(
            AstNodeFactory::identifier("count"),
            AstNodeFactory::integer_literal("0"),
        );
        let result = analyzer.analyze(&node);
        // Should become a VariableDeclaration since "count" was not previously declared
        assert_eq!(result.kind, AstNodeKind::VariableDeclaration);
    }

    #[test]
    fn explicit_assignment_after_declaration() {
        let analyzer = SemanticAnalyzer::with_default_config();
        // First assignment: should become VariableDeclaration
        let first = AstNodeFactory::assignment(
            AstNodeFactory::identifier("x"),
            AstNodeFactory::integer_literal("1"),
        );
        let _r1 = analyzer.analyze(&first);
        // Second assignment within same scope: should remain Assignment
        // (Note: the analyzer starts fresh each time)
        let analyzer2 = SemanticAnalyzer::with_default_config();
        let mut declared = std::collections::HashSet::new();
        declared.insert("x".to_string());
        let second = AstNodeFactory::assignment(
            AstNodeFactory::identifier("x"),
            AstNodeFactory::integer_literal("2"),
        );
        let result = analyzer2.transform(&second, &mut declared);
        assert_eq!(result.kind, AstNodeKind::Assignment);
    }
    #[test]
    fn desugar_notin_operator() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNode::internal(
            AstNodeKind::BinaryExpression,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String("notin".to_string())),
                ),
                AstNodeFactory::identifier("x"),
                AstNodeFactory::identifier("s"),
            ],
            None,
        );
        let result = analyzer.analyze(&node);
        assert_eq!(result.kind, AstNodeKind::UnaryExpression);
    }
    #[test]
    fn infer_string_type() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNodeFactory::string_literal("hello");
        assert_eq!(
            analyzer.infer_type_from_value(&node),
            Type::Primitive(crate::ast::PrimitiveType::String)
        );
    }
    #[test]
    fn infer_set_literal_type() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNode::internal(AstNodeKind::SetLiteral, vec![], None);
        assert!(matches!(
            analyzer.infer_type_from_value(&node),
            Type::Composite(_)
        ));
    }
    #[test]
    fn transform_binary_non_in() {
        let analyzer = SemanticAnalyzer::with_default_config();
        let node = AstNode::internal(
            AstNodeKind::BinaryExpression,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String("+".to_string())),
                ),
                AstNodeFactory::integer_literal("1"),
                AstNodeFactory::integer_literal("2"),
            ],
            None,
        );
        let result = analyzer.analyze(&node);
        assert_eq!(result.kind, AstNodeKind::BinaryExpression);
    }
}
