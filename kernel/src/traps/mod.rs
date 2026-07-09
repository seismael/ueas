//! Exit status codes and trap register for the UEAS abstract interpreter.
//!
//! The kernel produces exit status codes as defined in SPEC.md Section 6.6.
//! Code 0 indicates successful completion. Non-zero codes are trap codes
//! indicating controlled error halts.

use serde::{Deserialize, Serialize};

/// Exit status / trap code per SPEC.md Section 6.6.
///
/// Code `0` (NO_ERROR) is a normal exit — not a trap.
/// Codes `1` through `11` are trap codes indicating controlled error halts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExitCode {
    /// Normal termination (exit, not a trap).
    NoError = 0,
    /// Division or modulo by zero.
    DivisionByZero = 1,
    /// List, Tuple, or Matrix access beyond declared bounds.
    IndexOutOfBounds = 2,
    /// Access on an `Option` of `None`.
    NullDereference = 3,
    /// An `invariant` expression evaluated to `false`.
    InvariantViolation = 4,
    /// Step count breached the declared complexity contract.
    ComplexityViolation = 5,
    /// Recursion depth exceeded configurable limit (default 10^4).
    StackOverflow = 6,
    /// Virtual heap allocation failed (configured size exceeded).
    HeapExhaustion = 7,
    /// An `assert` expression evaluated to `false`.
    AssertionFailure = 8,
    /// Step counter exceeded configurable global maximum (default 10^12).
    InfiniteLoopDetected = 9,
    /// The complexity string references a variable not bound by a variableBinding.
    InvalidComplexityBinding = 10,
    /// Unsupported operation, type mismatch, or unimplemented built-in function.
    InvalidOperation = 11,
}

impl ExitCode {
    /// Returns `true` if this is a trap (non-zero) code.
    pub fn is_trap(self) -> bool {
        !matches!(self, ExitCode::NoError)
    }

    /// Returns `true` if this is a normal exit (code 0).
    pub fn is_ok(self) -> bool {
        matches!(self, ExitCode::NoError)
    }

    /// Human-readable name of the exit code.
    pub fn name(self) -> &'static str {
        match self {
            ExitCode::NoError => "NO_ERROR",
            ExitCode::DivisionByZero => "DIVISION_BY_ZERO",
            ExitCode::IndexOutOfBounds => "INDEX_OUT_OF_BOUNDS",
            ExitCode::NullDereference => "NULL_DEREFERENCE",
            ExitCode::InvariantViolation => "INVARIANT_VIOLATION",
            ExitCode::ComplexityViolation => "COMPLEXITY_VIOLATION",
            ExitCode::StackOverflow => "STACK_OVERFLOW",
            ExitCode::HeapExhaustion => "HEAP_EXHAUSTION",
            ExitCode::AssertionFailure => "ASSERTION_FAILURE",
            ExitCode::InfiniteLoopDetected => "INFINITE_LOOP_DETECTED",
            ExitCode::InvalidComplexityBinding => "INVALID_COMPLEXITY_BINDING",
            ExitCode::InvalidOperation => "INVALID_OPERATION",
        }
    }

    /// Description of what caused this exit code.
    pub fn description(self) -> &'static str {
        match self {
            ExitCode::NoError => "Normal termination.",
            ExitCode::DivisionByZero => "Division or modulo by zero.",
            ExitCode::IndexOutOfBounds => "List, Tuple, or Matrix access beyond declared bounds.",
            ExitCode::NullDereference => "Access on an Option of None.",
            ExitCode::InvariantViolation => "An invariant expression evaluated to false.",
            ExitCode::ComplexityViolation => {
                "Step count breached the declared complexity contract."
            }
            ExitCode::StackOverflow => "Recursion depth exceeded configurable limit.",
            ExitCode::HeapExhaustion => {
                "Virtual heap allocation failed (configured size exceeded)."
            }
            ExitCode::AssertionFailure => "An assert expression evaluated to false.",
            ExitCode::InfiniteLoopDetected => "Step counter exceeded configurable global maximum.",
            ExitCode::InvalidComplexityBinding => {
                "The complexity string references a variable not bound by a variableBinding."
            }
            ExitCode::InvalidOperation => {
                "Unsupported operation, type mismatch, or unimplemented built-in."
            }
        }
    }
}

/// The trap register holds the current execution state.
///
/// When execution ends normally, the register holds `ExitCode::NoError`.
/// When a trap condition is detected, the register is set and execution halts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapRegister {
    code: ExitCode,
}

impl Default for TrapRegister {
    fn default() -> Self {
        Self {
            code: ExitCode::NoError,
        }
    }
}

impl TrapRegister {
    /// Create a new trap register initialized to NO_ERROR.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the trap code. Overwrites any previously set code.
    pub fn set(&mut self, code: ExitCode) {
        self.code = code;
    }

    /// Returns the current exit code.
    pub fn code(&self) -> ExitCode {
        self.code
    }

    /// Returns `true` if a trap has been set (non-zero).
    pub fn is_trapped(&self) -> bool {
        self.code != ExitCode::NoError
    }

    /// Returns `true` if execution has ended normally.
    pub fn is_ok(&self) -> bool {
        self.code == ExitCode::NoError
    }

    /// Clear the trap register, resetting to NO_ERROR.
    pub fn clear(&mut self) {
        self.code = ExitCode::NoError;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_error_is_not_a_trap() {
        assert!(ExitCode::NoError.is_ok());
        assert!(!ExitCode::NoError.is_trap());
    }

    #[test]
    fn division_by_zero_is_a_trap() {
        assert!(!ExitCode::DivisionByZero.is_ok());
    }

    #[test]
    fn complexity_violation_is_a_trap() {
        assert_eq!(ExitCode::ComplexityViolation as u8, 5);
        assert!(!ExitCode::ComplexityViolation.is_ok());
    }

    #[test]
    fn exit_code_serialization_round_trip() {
        let code = ExitCode::HeapExhaustion;
        let json = serde_json::to_string(&code).unwrap();
        let restored: ExitCode = serde_json::from_str(&json).unwrap();
        assert_eq!(code, restored);
    }

    #[test]
    fn exit_code_name_returns_correct_string() {
        assert_eq!(ExitCode::NoError.name(), "NO_ERROR");
        assert_eq!(ExitCode::DivisionByZero.name(), "DIVISION_BY_ZERO");
        assert_eq!(ExitCode::IndexOutOfBounds.name(), "INDEX_OUT_OF_BOUNDS");
        assert_eq!(ExitCode::NullDereference.name(), "NULL_DEREFERENCE");
        assert_eq!(ExitCode::InvariantViolation.name(), "INVARIANT_VIOLATION");
        assert_eq!(ExitCode::ComplexityViolation.name(), "COMPLEXITY_VIOLATION");
        assert_eq!(ExitCode::StackOverflow.name(), "STACK_OVERFLOW");
        assert_eq!(ExitCode::HeapExhaustion.name(), "HEAP_EXHAUSTION");
        assert_eq!(ExitCode::AssertionFailure.name(), "ASSERTION_FAILURE");
        assert_eq!(
            ExitCode::InfiniteLoopDetected.name(),
            "INFINITE_LOOP_DETECTED"
        );
        assert_eq!(
            ExitCode::InvalidComplexityBinding.name(),
            "INVALID_COMPLEXITY_BINDING"
        );
        assert_eq!(ExitCode::InvalidOperation.name(), "INVALID_OPERATION");
    }

    #[test]
    fn trap_register_starts_ok() {
        let tr = TrapRegister::new();
        assert!(tr.is_ok());
        assert!(!tr.is_trapped());
    }

    #[test]
    fn trap_register_set_and_query() {
        let mut tr = TrapRegister::new();
        tr.set(ExitCode::HeapExhaustion);
        assert_eq!(tr.code(), ExitCode::HeapExhaustion);
        assert!(tr.is_trapped());
    }

    #[test]
    fn trap_register_clear() {
        let mut tr = TrapRegister::new();
        tr.set(ExitCode::InvariantViolation);
        tr.clear();
        assert!(tr.is_ok());
    }

    #[test]
    fn trap_register_set_overwrites() {
        let mut tr = TrapRegister::new();
        tr.set(ExitCode::StackOverflow);
        tr.set(ExitCode::DivisionByZero);
        assert_eq!(tr.code(), ExitCode::DivisionByZero);
    }

    #[test]
    fn all_trap_codes_have_names() {
        let codes = [
            ExitCode::NoError,
            ExitCode::DivisionByZero,
            ExitCode::IndexOutOfBounds,
            ExitCode::NullDereference,
            ExitCode::InvariantViolation,
            ExitCode::ComplexityViolation,
            ExitCode::StackOverflow,
            ExitCode::HeapExhaustion,
            ExitCode::AssertionFailure,
            ExitCode::InfiniteLoopDetected,
            ExitCode::InvalidComplexityBinding,
            ExitCode::InvalidOperation,
        ];
        for code in &codes {
            assert!(!code.name().is_empty());
            assert!(!code.description().is_empty());
        }
    }
}
