//! Deterministic trap codes and structured runtime error values.

use serde::{Deserialize, Serialize};
use ueas_syntax::SourceSpan;

/// Deterministic VM trap codes (no host panics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrapCode {
    DivisionByZero,
    IndexOutOfBounds,
    IntegerOverflow,
    UseAfterFree,
    PreconditionViolated,
    PostconditionViolated,
    InvariantViolated,
    TerminationMeasureViolated,
    StepBudgetExceeded,
    HeapBudgetExceeded,
    TypeMismatch,
    AssertionFailed,
    UndefinedSymbol,
    ArityMismatch,
    FloatInvalid,
    StackUnderflow,
    InvalidBytecode,
}

impl TrapCode {
    /// Stable canonical trap code identifier.
    pub fn code(self) -> &'static str {
        match self {
            Self::DivisionByZero => "T_DIV_ZERO",
            Self::IndexOutOfBounds => "T_OOB",
            Self::IntegerOverflow => "T_OVERFLOW",
            Self::UseAfterFree => "T_USE_AFTER_FREE",
            Self::PreconditionViolated => "T_PRECONDITION",
            Self::PostconditionViolated => "T_POSTCONDITION",
            Self::InvariantViolated => "T_INVARIANT",
            Self::TerminationMeasureViolated => "T_TERMINATION",
            Self::StepBudgetExceeded => "T_STEP_BUDGET",
            Self::HeapBudgetExceeded => "T_HEAP_BUDGET",
            Self::TypeMismatch => "T_TYPE_MISMATCH",
            Self::AssertionFailed => "T_ASSERT",
            Self::UndefinedSymbol => "T_UNDEFINED_SYMBOL",
            Self::ArityMismatch => "T_ARITY",
            Self::FloatInvalid => "T_FLOAT_INVALID",
            Self::StackUnderflow => "T_STACK_UNDERFLOW",
            Self::InvalidBytecode => "T_INVALID_BYTECODE",
        }
    }
}

/// Structured trap error returned by VM execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrapError {
    pub trap: TrapCode,
    pub message: String,
    pub pc: usize,
    pub step: u64,
    pub span: Option<SourceSpan>,
}

impl TrapError {
    /// Create a new structured trap error.
    pub fn new(
        trap: TrapCode,
        message: impl Into<String>,
        pc: usize,
        step: u64,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            trap,
            message: message.into(),
            pc,
            step,
            span,
        }
    }
}

impl std::fmt::Display for TrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} (step {}, pc {})",
            self.trap.code(),
            self.message,
            self.step,
            self.pc
        )
    }
}

impl std::error::Error for TrapError {}
