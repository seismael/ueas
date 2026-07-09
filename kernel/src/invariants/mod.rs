//! Invariant engine for the UEAS abstract interpreter.
//!
//! Invariants are boolean predicates enforced at runtime. The actual
//! enforcement is performed by `execute_invariant()` in the interpreter
//! module, which evaluates conditions against the execution context.
//!
//! Per SPEC.md Section 6.3:
//! - On first encounter, the kernel evaluates the invariant expression.
//! - If false, the kernel sets the trap register to `INVARIANT_VIOLATION`
//!   and halts immediately.
//! - The kernel re-evaluates the invariant at every loop iteration.

use crate::ast::AstValue;
use crate::traps::ExitCode;

/// Check a literal boolean invariant condition.
/// Complex expressions are evaluated by the interpreter directly.
pub fn check_literal_condition(value: &AstValue) -> Result<(), ExitCode> {
    match value {
        AstValue::Boolean(false) => Err(ExitCode::InvariantViolation),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstValue;

    #[test]
    fn check_true_condition() {
        assert!(check_literal_condition(&AstValue::Boolean(true)).is_ok());
    }

    #[test]
    fn check_false_condition() {
        assert_eq!(
            check_literal_condition(&AstValue::Boolean(false)).unwrap_err(),
            ExitCode::InvariantViolation
        );
    }

    #[test]
    fn check_integer_condition() {
        assert!(check_literal_condition(&AstValue::Integer(1)).is_ok());
    }
    #[test]
    fn check_real_condition() {
        assert!(check_literal_condition(&AstValue::Real(1.0)).is_ok());
    }
    #[test]
    fn check_string_condition() {
        assert!(check_literal_condition(&AstValue::String("x".to_string())).is_ok());
    }
    #[test]
    fn check_none_condition() {
        assert!(check_literal_condition(&AstValue::None).is_ok());
    }
}
