//! UEAS Abstract Syntax Tree — canonical representation of all UEAS programs.
//!
//! This module defines every AST node kind specified in SPEC.md Section 5.
//! All nodes carry a `kind` discriminant, optional source location, and
//! metadata for complexity profiling. The module provides a factory for
//! validated node construction and a Visitor trait for traversal.
//!
//! # Architecture
//!
//! - `AstNodeKind` — discriminant enum (20+ variants per SPEC.md Section 5.1)
//! - `AstNode` — structural node with kind, children, and typed value
//! - `AstNodeFactory` — GoF Factory ensuring valid invariants on construction
//! - `AstVisitor` — GoF Visitor decoupling operations from node structure
//! - Type system types — `PrimitiveType`, `CompositeType`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Discriminant for all AST node kinds defined in SPEC.md Section 5.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AstNodeKind {
    Program,
    Algorithm,
    Parameter,
    VariableBinding,
    VariableDeclaration,
    Assignment,
    Return,
    If,
    ForLoop,
    WhileLoop,
    Assert,
    Invariant,
    IntegerLiteral,
    RealLiteral,
    StringLiteral,
    BooleanLiteral,
    NoneLiteral,
    Identifier,
    BinaryExpression,
    UnaryExpression,
    FunctionCall,
    CastExpression,
    SetLiteral,
    ListLiteral,
    MapLiteral,
    GraphLiteral,
    MatrixLiteral,
    EdgeLiteral,
    Type,
}

/// Source location in the UEAS source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

/// Value types carried by AST nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstValue {
    Integer(String),
    Real(f64),
    Boolean(bool),
    String(String),
    None,
}

/// The core AST node structure. Every node has a kind discriminant and
/// zero or more children. The children list represents syntactic containment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub value: Option<AstValue>,
    pub children: Vec<AstNode>,
    pub location: Option<SourceLocation>,
    pub metadata: HashMap<String, String>,
}

impl AstNode {
    /// Create a leaf node (no children, no location).
    pub fn leaf(kind: AstNodeKind, value: Option<AstValue>) -> Self {
        Self {
            kind,
            value,
            children: vec![],
            location: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an internal node with children.
    pub fn internal(
        kind: AstNodeKind,
        children: Vec<AstNode>,
        location: Option<SourceLocation>,
    ) -> Self {
        Self {
            kind,
            value: None,
            children,
            location,
            metadata: HashMap::new(),
        }
    }
}

/// GoF Factory — centralized AST node construction with invariant validation.
///
/// Direct `AstNode::leaf()` / `AstNode::internal()` calls are permitted
/// only within this module. External code MUST use the factory to ensure
/// every node carries valid invariants per the SPEC.md schema.
pub struct AstNodeFactory;

impl AstNodeFactory {
    pub fn program(algorithms: Vec<AstNode>) -> AstNode {
        for algo in &algorithms {
            debug_assert_eq!(algo.kind, AstNodeKind::Algorithm);
        }
        AstNode::internal(AstNodeKind::Program, algorithms, None)
    }

    pub fn algorithm(
        name: &str,
        parameters: Vec<AstNode>,
        return_type: Option<AstNode>,
        complexity: &str,
        bindings: Vec<AstNode>,
        body: Vec<AstNode>,
    ) -> AstNode {
        let mut children = Vec::new();
        children.push(AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(name.to_string())),
        ));
        children.extend(parameters);
        if let Some(ret) = return_type {
            children.push(ret);
        }
        children.push(AstNode::leaf(
            AstNodeKind::StringLiteral,
            Some(AstValue::String(complexity.to_string())),
        ));
        children.extend(bindings);
        children.extend(body);
        AstNode::internal(AstNodeKind::Algorithm, children, None)
    }

    pub fn parameter(name: &str, typ: AstNode) -> AstNode {
        let mut children = vec![AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(name.to_string())),
        )];
        children.push(typ);
        AstNode::internal(AstNodeKind::Parameter, children, None)
    }

    pub fn variable_binding(variable: &str, expression: AstNode) -> AstNode {
        let mut children = vec![AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(variable.to_string())),
        )];
        children.push(expression);
        AstNode::internal(AstNodeKind::VariableBinding, children, None)
    }

    pub fn variable_declaration(name: &str, typ: AstNode, initializer: Option<AstNode>) -> AstNode {
        let mut children = vec![AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(name.to_string())),
        )];
        children.push(typ);
        if let Some(init) = initializer {
            children.push(init);
        }
        AstNode::internal(AstNodeKind::VariableDeclaration, children, None)
    }

    pub fn assignment(target: AstNode, value: AstNode) -> AstNode {
        AstNode::internal(AstNodeKind::Assignment, vec![target, value], None)
    }

    pub fn return_stmt(value: Option<AstNode>) -> AstNode {
        let children = value.map(|v| vec![v]).unwrap_or_default();
        AstNode::internal(AstNodeKind::Return, children, None)
    }

    pub fn if_stmt(
        condition: AstNode,
        consequent: Vec<AstNode>,
        alternate: Option<Vec<AstNode>>,
    ) -> AstNode {
        let mut children = vec![condition];
        children.push(AstNode::internal(AstNodeKind::If, consequent, None));
        if let Some(alt) = alternate {
            children.push(AstNode::internal(AstNodeKind::If, alt, None));
        }
        AstNode::internal(AstNodeKind::If, children, None)
    }

    pub fn for_loop(iterator: &str, collection: AstNode, body: Vec<AstNode>) -> AstNode {
        let mut children = vec![
            AstNode::leaf(
                AstNodeKind::Identifier,
                Some(AstValue::String(iterator.to_string())),
            ),
            collection,
        ];
        children.extend(body);
        AstNode::internal(AstNodeKind::ForLoop, children, None)
    }

    pub fn while_loop(condition: AstNode, body: Vec<AstNode>) -> AstNode {
        let mut children = vec![condition];
        children.extend(body);
        AstNode::internal(AstNodeKind::WhileLoop, children, None)
    }

    pub fn assert_stmt(condition: AstNode, message: Option<&str>) -> AstNode {
        let mut children = vec![condition];
        if let Some(msg) = message {
            children.push(AstNode::leaf(
                AstNodeKind::StringLiteral,
                Some(AstValue::String(msg.to_string())),
            ));
        }
        AstNode::internal(AstNodeKind::Assert, children, None)
    }

    pub fn invariant_stmt(condition: AstNode, message: Option<&str>) -> AstNode {
        let mut children = vec![condition];
        if let Some(msg) = message {
            children.push(AstNode::leaf(
                AstNodeKind::StringLiteral,
                Some(AstValue::String(msg.to_string())),
            ));
        }
        AstNode::internal(AstNodeKind::Invariant, children, None)
    }

    pub fn integer_literal(value: &str) -> AstNode {
        AstNode::leaf(
            AstNodeKind::IntegerLiteral,
            Some(AstValue::Integer(value.to_string())),
        )
    }

    pub fn real_literal(value: f64) -> AstNode {
        AstNode::leaf(AstNodeKind::RealLiteral, Some(AstValue::Real(value)))
    }

    pub fn string_literal(value: &str) -> AstNode {
        AstNode::leaf(
            AstNodeKind::StringLiteral,
            Some(AstValue::String(value.to_string())),
        )
    }

    pub fn boolean_literal(value: bool) -> AstNode {
        AstNode::leaf(AstNodeKind::BooleanLiteral, Some(AstValue::Boolean(value)))
    }

    pub fn none_literal() -> AstNode {
        AstNode::leaf(AstNodeKind::NoneLiteral, Some(AstValue::None))
    }

    pub fn identifier(name: &str) -> AstNode {
        AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(name.to_string())),
        )
    }

    pub fn binary_expression(operator: &str, left: AstNode, right: AstNode) -> AstNode {
        AstNode::internal(
            AstNodeKind::BinaryExpression,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String(operator.to_string())),
                ),
                left,
                right,
            ],
            None,
        )
    }

    pub fn unary_expression(operator: &str, operand: AstNode) -> AstNode {
        AstNode::internal(
            AstNodeKind::UnaryExpression,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String(operator.to_string())),
                ),
                operand,
            ],
            None,
        )
    }

    pub fn function_call(name: &str, arguments: Vec<AstNode>) -> AstNode {
        let mut children = vec![AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(name.to_string())),
        )];
        children.extend(arguments);
        AstNode::internal(AstNodeKind::FunctionCall, children, None)
    }

    pub fn cast_expression(expression: AstNode, target_type: AstNode) -> AstNode {
        AstNode::internal(
            AstNodeKind::CastExpression,
            vec![expression, target_type],
            None,
        )
    }

    pub fn set_literal(elements: Vec<AstNode>) -> AstNode {
        AstNode::internal(AstNodeKind::SetLiteral, elements, None)
    }

    pub fn list_literal(elements: Vec<AstNode>) -> AstNode {
        AstNode::internal(AstNodeKind::ListLiteral, elements, None)
    }

    pub fn map_literal(entries: Vec<(AstNode, AstNode)>) -> AstNode {
        let mut children = Vec::new();
        for (k, v) in entries {
            children.push(k);
            children.push(v);
        }
        AstNode::internal(AstNodeKind::MapLiteral, children, None)
    }

    pub fn graph_literal(
        node_type: AstNode,
        edge_type: AstNode,
        nodes: Vec<AstNode>,
        edges: Vec<AstNode>,
    ) -> AstNode {
        let mut children = vec![node_type, edge_type];
        children.push(AstNode::internal(AstNodeKind::SetLiteral, nodes, None));
        for edge in edges {
            children.push(edge);
        }
        AstNode::internal(AstNodeKind::GraphLiteral, children, None)
    }

    pub fn matrix_literal(rows: u64, cols: u64, typ: AstNode, elements: Vec<AstNode>) -> AstNode {
        let mut children = vec![
            AstNode::leaf(
                AstNodeKind::IntegerLiteral,
                Some(AstValue::Integer(rows.to_string())),
            ),
            AstNode::leaf(
                AstNodeKind::IntegerLiteral,
                Some(AstValue::Integer(cols.to_string())),
            ),
            typ,
        ];
        children.extend(elements);
        AstNode::internal(AstNodeKind::MatrixLiteral, children, None)
    }

    pub fn edge_literal(from: AstNode, to: AstNode, weight: Option<AstNode>) -> AstNode {
        let mut children = vec![from, to];
        if let Some(w) = weight {
            children.push(w);
        }
        AstNode::internal(AstNodeKind::EdgeLiteral, children, None)
    }

    pub fn type_node(name: &str, type_arguments: Vec<AstNode>) -> AstNode {
        let mut children = vec![AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String(name.to_string())),
        )];
        children.extend(type_arguments);
        AstNode::internal(AstNodeKind::Type, children, None)
    }
}

// ===== Type System =====

/// UEAS primitive types per SPEC.md Section 3.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    Integer,
    Real,
    Boolean,
    String,
    Void,
}

impl PrimitiveType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Integer => "Integer",
            Self::Real => "Real",
            Self::Boolean => "Boolean",
            Self::String => "String",
            Self::Void => "Void",
        }
    }
}

/// UEAS composite types per SPEC.md Section 3.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompositeType {
    Set(Box<Type>),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Graph(Box<Type>, Box<Type>),
    Matrix(u64, u64, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
}

/// The unified Type representation for UEAS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Primitive(PrimitiveType),
    Composite(CompositeType),
}

impl Type {
    pub fn integer() -> Self {
        Self::Primitive(PrimitiveType::Integer)
    }
    pub fn real() -> Self {
        Self::Primitive(PrimitiveType::Real)
    }
    pub fn boolean() -> Self {
        Self::Primitive(PrimitiveType::Boolean)
    }
    pub fn string() -> Self {
        Self::Primitive(PrimitiveType::String)
    }
    pub fn void() -> Self {
        Self::Primitive(PrimitiveType::Void)
    }
}

// ===== Visitor Pattern (GoF) =====

/// The visitor trait for AST traversal. One method per node kind.
/// Visitors implement only the callbacks they need (Interface Segregation).
///
/// The default implementations do nothing — the visitor traverses children
/// without side effects. Override specific methods to add behavior.
pub trait AstVisitor {
    fn visit_program(&mut self, _node: &AstNode) {}
    fn visit_algorithm(&mut self, _node: &AstNode) {}
    fn visit_parameter(&mut self, _node: &AstNode) {}
    fn visit_variable_binding(&mut self, _node: &AstNode) {}
    fn visit_variable_declaration(&mut self, _node: &AstNode) {}
    fn visit_assignment(&mut self, _node: &AstNode) {}
    fn visit_return(&mut self, _node: &AstNode) {}
    fn visit_if(&mut self, _node: &AstNode) {}
    fn visit_for_loop(&mut self, _node: &AstNode) {}
    fn visit_while_loop(&mut self, _node: &AstNode) {}
    fn visit_assert(&mut self, _node: &AstNode) {}
    fn visit_invariant(&mut self, _node: &AstNode) {}
    fn visit_integer_literal(&mut self, _node: &AstNode) {}
    fn visit_real_literal(&mut self, _node: &AstNode) {}
    fn visit_string_literal(&mut self, _node: &AstNode) {}
    fn visit_boolean_literal(&mut self, _node: &AstNode) {}
    fn visit_none_literal(&mut self, _node: &AstNode) {}
    fn visit_identifier(&mut self, _node: &AstNode) {}
    fn visit_binary_expression(&mut self, _node: &AstNode) {}
    fn visit_unary_expression(&mut self, _node: &AstNode) {}
    fn visit_function_call(&mut self, _node: &AstNode) {}
    fn visit_cast_expression(&mut self, _node: &AstNode) {}
    fn visit_set_literal(&mut self, _node: &AstNode) {}
    fn visit_list_literal(&mut self, _node: &AstNode) {}
    fn visit_map_literal(&mut self, _node: &AstNode) {}
    fn visit_graph_literal(&mut self, _node: &AstNode) {}
    fn visit_matrix_literal(&mut self, _node: &AstNode) {}
    fn visit_edge_literal(&mut self, _node: &AstNode) {}
    fn visit_type(&mut self, _node: &AstNode) {}

    /// Traverse an AST node by dispatching to the appropriate visit method,
    /// then recursively visiting all children.
    fn traverse(&mut self, node: &AstNode) {
        match node.kind {
            AstNodeKind::Program => self.visit_program(node),
            AstNodeKind::Algorithm => self.visit_algorithm(node),
            AstNodeKind::Parameter => self.visit_parameter(node),
            AstNodeKind::VariableBinding => self.visit_variable_binding(node),
            AstNodeKind::VariableDeclaration => self.visit_variable_declaration(node),
            AstNodeKind::Assignment => self.visit_assignment(node),
            AstNodeKind::Return => self.visit_return(node),
            AstNodeKind::If => self.visit_if(node),
            AstNodeKind::ForLoop => self.visit_for_loop(node),
            AstNodeKind::WhileLoop => self.visit_while_loop(node),
            AstNodeKind::Assert => self.visit_assert(node),
            AstNodeKind::Invariant => self.visit_invariant(node),
            AstNodeKind::IntegerLiteral => self.visit_integer_literal(node),
            AstNodeKind::RealLiteral => self.visit_real_literal(node),
            AstNodeKind::StringLiteral => self.visit_string_literal(node),
            AstNodeKind::BooleanLiteral => self.visit_boolean_literal(node),
            AstNodeKind::NoneLiteral => self.visit_none_literal(node),
            AstNodeKind::Identifier => self.visit_identifier(node),
            AstNodeKind::BinaryExpression => self.visit_binary_expression(node),
            AstNodeKind::UnaryExpression => self.visit_unary_expression(node),
            AstNodeKind::FunctionCall => self.visit_function_call(node),
            AstNodeKind::CastExpression => self.visit_cast_expression(node),
            AstNodeKind::SetLiteral => self.visit_set_literal(node),
            AstNodeKind::ListLiteral => self.visit_list_literal(node),
            AstNodeKind::MapLiteral => self.visit_map_literal(node),
            AstNodeKind::GraphLiteral => self.visit_graph_literal(node),
            AstNodeKind::MatrixLiteral => self.visit_matrix_literal(node),
            AstNodeKind::EdgeLiteral => self.visit_edge_literal(node),
            AstNodeKind::Type => self.visit_type(node),
        }
        for child in &node.children {
            self.traverse(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== AstNodeKind Tests =====

    #[test]
    fn ast_node_kind_program_exists() {
        assert_eq!(
            serde_json::to_string(&AstNodeKind::Program).unwrap(),
            r#""Program""#
        );
    }

    #[test]
    fn ast_node_kind_algorithm_exists() {
        assert_eq!(
            serde_json::to_string(&AstNodeKind::Algorithm).unwrap(),
            r#""Algorithm""#
        );
    }

    #[test]
    fn all_node_kinds_can_serialize() {
        let kinds = vec![
            AstNodeKind::Program,
            AstNodeKind::Algorithm,
            AstNodeKind::Parameter,
            AstNodeKind::VariableBinding,
            AstNodeKind::VariableDeclaration,
            AstNodeKind::Assignment,
            AstNodeKind::Return,
            AstNodeKind::If,
            AstNodeKind::ForLoop,
            AstNodeKind::WhileLoop,
            AstNodeKind::Assert,
            AstNodeKind::Invariant,
            AstNodeKind::IntegerLiteral,
            AstNodeKind::RealLiteral,
            AstNodeKind::StringLiteral,
            AstNodeKind::BooleanLiteral,
            AstNodeKind::NoneLiteral,
            AstNodeKind::Identifier,
            AstNodeKind::BinaryExpression,
            AstNodeKind::UnaryExpression,
            AstNodeKind::FunctionCall,
            AstNodeKind::CastExpression,
            AstNodeKind::SetLiteral,
            AstNodeKind::ListLiteral,
            AstNodeKind::MapLiteral,
            AstNodeKind::GraphLiteral,
            AstNodeKind::MatrixLiteral,
            AstNodeKind::EdgeLiteral,
            AstNodeKind::Type,
        ];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let round_tripped: AstNodeKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, round_tripped);
        }
    }

    // ===== Factory Tests =====

    #[test]
    fn factory_integer_literal_creates_correct_node() {
        let node = AstNodeFactory::integer_literal("42");
        assert_eq!(node.kind, AstNodeKind::IntegerLiteral);
        assert_eq!(node.value, Some(AstValue::Integer("42".to_string())));
        assert!(node.children.is_empty());
    }

    #[test]
    fn factory_real_literal_creates_correct_node() {
        let node = AstNodeFactory::real_literal(3.14);
        assert_eq!(node.kind, AstNodeKind::RealLiteral);
        assert_eq!(node.value, Some(AstValue::Real(3.14)));
    }

    #[test]
    fn factory_string_literal_creates_correct_node() {
        let node = AstNodeFactory::string_literal("hello");
        assert_eq!(node.kind, AstNodeKind::StringLiteral);
        assert_eq!(node.value, Some(AstValue::String("hello".to_string())));
    }

    #[test]
    fn factory_boolean_literal_true() {
        let node = AstNodeFactory::boolean_literal(true);
        assert_eq!(node.value, Some(AstValue::Boolean(true)));
    }

    #[test]
    fn factory_boolean_literal_false() {
        let node = AstNodeFactory::boolean_literal(false);
        assert_eq!(node.value, Some(AstValue::Boolean(false)));
    }

    #[test]
    fn factory_none_literal() {
        let node = AstNodeFactory::none_literal();
        assert_eq!(node.kind, AstNodeKind::NoneLiteral);
        assert_eq!(node.value, Some(AstValue::None));
    }

    #[test]
    fn factory_identifier() {
        let node = AstNodeFactory::identifier("myVar");
        assert_eq!(node.kind, AstNodeKind::Identifier);
        assert_eq!(node.value, Some(AstValue::String("myVar".to_string())));
    }

    #[test]
    fn factory_binary_expression() {
        let left = AstNodeFactory::identifier("a");
        let right = AstNodeFactory::integer_literal("1");
        let expr = AstNodeFactory::binary_expression("+", left, right);
        assert_eq!(expr.kind, AstNodeKind::BinaryExpression);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn factory_unary_expression() {
        let operand = AstNodeFactory::identifier("x");
        let expr = AstNodeFactory::unary_expression("-", operand);
        assert_eq!(expr.kind, AstNodeKind::UnaryExpression);
        assert_eq!(expr.children.len(), 2);
    }

    #[test]
    fn factory_function_call_no_args() {
        let call = AstNodeFactory::function_call("sqrt", vec![]);
        assert_eq!(call.kind, AstNodeKind::FunctionCall);
        assert_eq!(call.children.len(), 1);
    }

    #[test]
    fn factory_function_call_with_args() {
        let arg = AstNodeFactory::integer_literal("16");
        let call = AstNodeFactory::function_call("sqrt", vec![arg]);
        assert_eq!(call.kind, AstNodeKind::FunctionCall);
        assert_eq!(call.children.len(), 2);
    }

    #[test]
    fn factory_variable_declaration_without_initializer() {
        let typ = AstNodeFactory::type_node("Integer", vec![]);
        let decl = AstNodeFactory::variable_declaration("count", typ, None);
        assert_eq!(decl.kind, AstNodeKind::VariableDeclaration);
        assert_eq!(decl.children.len(), 2);
    }

    #[test]
    fn factory_variable_declaration_with_initializer() {
        let typ = AstNodeFactory::type_node("Integer", vec![]);
        let init = AstNodeFactory::integer_literal("0");
        let decl = AstNodeFactory::variable_declaration("count", typ, Some(init));
        assert_eq!(decl.kind, AstNodeKind::VariableDeclaration);
        assert_eq!(decl.children.len(), 3);
    }

    #[test]
    fn factory_return_with_value() {
        let val = AstNodeFactory::integer_literal("42");
        let ret = AstNodeFactory::return_stmt(Some(val));
        assert_eq!(ret.kind, AstNodeKind::Return);
        assert_eq!(ret.children.len(), 1);
    }

    #[test]
    fn factory_return_void() {
        let ret = AstNodeFactory::return_stmt(None);
        assert_eq!(ret.kind, AstNodeKind::Return);
        assert!(ret.children.is_empty());
    }

    #[test]
    fn factory_program_with_algorithms() {
        let algo = AstNodeFactory::algorithm("Test", vec![], None, "O(1)", vec![], vec![]);
        let program = AstNodeFactory::program(vec![algo.clone()]);
        assert_eq!(program.kind, AstNodeKind::Program);
        assert_eq!(program.children.len(), 1);
        assert_eq!(program.children[0].kind, AstNodeKind::Algorithm);
    }

    #[test]
    fn factory_set_literal_elements() {
        let elem1 = AstNodeFactory::integer_literal("1");
        let elem2 = AstNodeFactory::integer_literal("2");
        let set = AstNodeFactory::set_literal(vec![elem1, elem2]);
        assert_eq!(set.kind, AstNodeKind::SetLiteral);
        assert_eq!(set.children.len(), 2);
    }

    #[test]
    fn factory_list_literal_elements() {
        let elem = AstNodeFactory::string_literal("a");
        let list = AstNodeFactory::list_literal(vec![elem]);
        assert_eq!(list.kind, AstNodeKind::ListLiteral);
        assert_eq!(list.children.len(), 1);
    }

    // ===== Type Tests =====

    #[test]
    fn type_integer() {
        assert_eq!(Type::integer(), Type::Primitive(PrimitiveType::Integer));
    }

    #[test]
    fn type_real() {
        assert_eq!(Type::real(), Type::Primitive(PrimitiveType::Real));
    }

    #[test]
    fn type_serialization_round_trip() {
        let typ = Type::integer();
        let json = serde_json::to_string(&typ).unwrap();
        let round_tripped: Type = serde_json::from_str(&json).unwrap();
        assert_eq!(typ, round_tripped);
    }

    #[test]
    fn composite_type_set_serialization() {
        let typ = Type::Composite(CompositeType::Set(Box::new(Type::integer())));
        let json = serde_json::to_string(&typ).unwrap();
        let round_tripped: Type = serde_json::from_str(&json).unwrap();
        assert_eq!(typ, round_tripped);
    }

    // ===== Visitor Tests =====

    struct CountingVisitor {
        count: usize,
    }

    impl AstVisitor for CountingVisitor {
        fn visit_integer_literal(&mut self, _node: &AstNode) {
            self.count += 1;
        }
    }

    #[test]
    fn visitor_counts_integer_literals() {
        let node = AstNodeFactory::integer_literal("42");
        let mut visitor = CountingVisitor { count: 0 };
        visitor.traverse(&node);
        assert_eq!(visitor.count, 1);
    }

    #[test]
    fn visitor_does_not_count_non_integers() {
        let node = AstNodeFactory::string_literal("hello");
        let mut visitor = CountingVisitor { count: 0 };
        visitor.traverse(&node);
        assert_eq!(visitor.count, 0);
    }

    #[test]
    fn visitor_traverses_children() {
        let left = AstNodeFactory::integer_literal("1");
        let right = AstNodeFactory::integer_literal("2");
        let expr = AstNodeFactory::binary_expression("+", left, right);
        let mut visitor = CountingVisitor { count: 0 };
        visitor.traverse(&expr);
        assert_eq!(visitor.count, 2);
    }

    // ===== Round-Trip Serde Tests =====

    #[test]
    fn round_trip_integer_literal_via_serde() {
        let original = AstNodeFactory::integer_literal("999");
        let json = serde_json::to_string_pretty(&original).unwrap();
        let restored: AstNode = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn round_trip_complex_program_via_serde() {
        let algo = AstNodeFactory::algorithm(
            "EuclideanDistance",
            vec![
                AstNodeFactory::parameter("x1", AstNodeFactory::type_node("Real", vec![])),
                AstNodeFactory::parameter("y1", AstNodeFactory::type_node("Real", vec![])),
            ],
            Some(AstNodeFactory::type_node("Real", vec![])),
            "O(1)",
            vec![],
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::binary_expression(
                    "+",
                    AstNodeFactory::integer_literal("0"),
                    AstNodeFactory::integer_literal("0"),
                ),
            ))],
        );
        let program = AstNodeFactory::program(vec![algo]);
        let json = serde_json::to_string_pretty(&program).unwrap();
        let restored: AstNode = serde_json::from_str(&json).unwrap();
        assert_eq!(program, restored);
    }

    #[test]
    fn round_trip_type_node() {
        let typ =
            AstNodeFactory::type_node("Set", vec![AstNodeFactory::type_node("Integer", vec![])]);
        let json = serde_json::to_string(&typ).unwrap();
        let restored: AstNode = serde_json::from_str(&json).unwrap();
        assert_eq!(typ, restored);
    }
}
