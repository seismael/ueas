//! Strongly-typed Canonical HIR (High-level Intermediate Representation) for UEAS v2.0.

use crate::complexity::ComplexitySpec;
use crate::types::Type;
use serde::{Deserialize, Serialize};
use ueas_syntax::SourceSpan;

/// Top-level UEAS v2.0 program conforming to `spec/schemas/ueas-hir.v2.schema.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub spec_version: String,
    pub engine_version: String,
    pub module: Option<String>,
    pub imports: Vec<ImportDecl>,
    pub records: Vec<RecordDecl>,
    pub algorithms: Vec<AlgorithmDecl>,
    pub tests: Vec<TestDecl>,
    pub benchmarks: Vec<BenchmarkDecl>,
}

/// An `import a.b.c` declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportDecl {
    pub path: String,
    pub span: SourceSpan,
}

/// Value-semantic `Record` declaration (RFC 0001).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<FieldDecl>,
    pub span: SourceSpan,
}

/// A field in a `Record`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDecl {
    pub name: String,
    pub ty: Type,
    pub span: SourceSpan,
}

/// Generic type parameter with optional trait bounds (`<T: Ord + Eq>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

/// Trait bounds supported by UEAS generics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraitBound {
    Eq,
    Ord,
    Hash,
    Numeric,
}

/// Hardware cache profile configuration (`@HardwareProfile`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub l1_bytes: u64,
    pub l2_bytes: u64,
    pub l3_bytes: u64,
    pub line_bytes: u64,
    pub associativity: u64,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            l1_bytes: 64 * 1024,
            l2_bytes: 256 * 1024,
            l3_bytes: 8 * 1024 * 1024,
            line_bytes: 64,
            associativity: 8,
        }
    }
}

/// Top-level `Algorithm` declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlgorithmDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub parameters: Vec<Param>,
    pub return_type: Type,
    pub preconditions: Vec<Expr>,
    pub postcondition_binding: Option<String>,
    pub postconditions: Vec<Expr>,
    pub complexity: ComplexitySpec,
    pub hardware_profile: Option<HardwareProfile>,
    pub constant_time: bool,
    pub declassifications: Vec<SourceSpan>,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// Algorithm parameter with optional `mut` modifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_mut: bool,
    pub span: SourceSpan,
}

/// `@Test "name" ... end Test` declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestDecl {
    pub name: String,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// `@Benchmark "name" ... end Benchmark` declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkDecl {
    pub name: String,
    pub scales: Vec<u64>,
    pub generator_param: String,
    pub generator_body: Option<Expr>,
    pub span: SourceSpan,
}

/// Typed HIR Statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Stmt {
    Let {
        name: String,
        is_mut: bool,
        ty: Option<Type>,
        init: Expr,
        span: SourceSpan,
    },
    Const {
        name: String,
        ty: Option<Type>,
        init: Expr,
        span: SourceSpan,
    },
    Assign {
        target: LValue,
        value: Expr,
        span: SourceSpan,
    },
    Expr {
        expr: Expr,
        span: SourceSpan,
    },
    Return {
        value: Option<Expr>,
        span: SourceSpan,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_if_branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        span: SourceSpan,
    },
    While {
        condition: Expr,
        invariants: Vec<Expr>,
        decreases: Option<Expr>,
        body: Vec<Stmt>,
        span: SourceSpan,
    },
    For {
        var: String,
        iterable: Expr,
        is_parallel: bool,
        invariants: Vec<Expr>,
        body: Vec<Stmt>,
        span: SourceSpan,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
        span: SourceSpan,
    },
    Assert {
        condition: Expr,
        message: Option<String>,
        span: SourceSpan,
    },
    Invariant {
        condition: Expr,
        message: Option<String>,
        span: SourceSpan,
    },
    Decreases {
        rank: Expr,
        span: SourceSpan,
    },
    Break {
        span: SourceSpan,
    },
    Continue {
        span: SourceSpan,
    },
    Sync {
        span: SourceSpan,
    },
}

/// Match arm in `match` statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// Pattern for `match` arms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Pattern {
    Wildcard,
    Ident { name: String },
    Some { inner: Box<Pattern> },
    None,
    Ok { inner: Box<Pattern> },
    Err { inner: Box<Pattern> },
    IntLit { value: String },
    BoolLit { value: bool },
    StringLit { value: String },
}

/// Assignable LValue target (`x`, `arr[i]`, `mat[i][j]`, `rec.field`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LValue {
    Var {
        name: String,
        span: SourceSpan,
    },
    Index {
        target: Box<LValue>,
        index: Expr,
        span: SourceSpan,
    },
    Field {
        target: Box<LValue>,
        field: String,
        span: SourceSpan,
    },
}

/// Typed HIR Expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Expr {
    IntLit {
        value: String,
        span: SourceSpan,
    },
    RealLit {
        value: f64,
        span: SourceSpan,
    },
    BoolLit {
        value: bool,
        span: SourceSpan,
    },
    StringLit {
        value: String,
        span: SourceSpan,
    },
    NoneLit {
        span: SourceSpan,
    },
    InfinityLit {
        span: SourceSpan,
    },
    NanLit {
        span: SourceSpan,
    },
    Ident {
        name: String,
        span: SourceSpan,
    },
    ListLit {
        elements: Vec<Expr>,
        span: SourceSpan,
    },
    SetLit {
        elements: Vec<Expr>,
        span: SourceSpan,
    },
    MapLit {
        entries: Vec<(Expr, Expr)>,
        span: SourceSpan,
    },
    TupleLit {
        elements: Vec<Expr>,
        span: SourceSpan,
    },
    RecordLit {
        name: String,
        fields: Vec<(String, Expr)>,
        span: SourceSpan,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: SourceSpan,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: SourceSpan,
    },
    Cast {
        expr: Box<Expr>,
        target_type: Type,
        span: SourceSpan,
    },
    Call {
        callee: String,
        args: Vec<CallArg>,
        span: SourceSpan,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<CallArg>,
        span: SourceSpan,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: SourceSpan,
    },
    Slice {
        target: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        span: SourceSpan,
    },
    Field {
        target: Box<Expr>,
        field: String,
        span: SourceSpan,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        span: SourceSpan,
    },
    Quantifier {
        is_forall: bool,
        var: String,
        iterable: Box<Expr>,
        body: Box<Expr>,
        span: SourceSpan,
    },
    Spawn {
        expr: Box<Expr>,
        span: SourceSpan,
    },
}

impl Expr {
    /// Retrieve the source span of any expression node.
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::IntLit { span, .. }
            | Self::RealLit { span, .. }
            | Self::BoolLit { span, .. }
            | Self::StringLit { span, .. }
            | Self::NoneLit { span }
            | Self::InfinityLit { span }
            | Self::NanLit { span }
            | Self::Ident { span, .. }
            | Self::ListLit { span, .. }
            | Self::SetLit { span, .. }
            | Self::MapLit { span, .. }
            | Self::TupleLit { span, .. }
            | Self::RecordLit { span, .. }
            | Self::Binary { span, .. }
            | Self::Unary { span, .. }
            | Self::Cast { span, .. }
            | Self::Call { span, .. }
            | Self::MethodCall { span, .. }
            | Self::Index { span, .. }
            | Self::Slice { span, .. }
            | Self::Field { span, .. }
            | Self::Range { span, .. }
            | Self::Quantifier { span, .. }
            | Self::Spawn { span, .. } => *span,
        }
    }
}

/// Argument to a function or method call (tracking explicit `mut` pass-through).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallArg {
    pub is_mut: bool,
    pub value: Expr,
}

/// Binary operators in UEAS v2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Implies,
    Iff,
    In,
    NotIn,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operators in UEAS v2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}
