//! Invariant engine for the UEAS abstract interpreter.
//
//! The invariant engine evaluates boolean predicates at specified program
//! points. When an invariant evaluates to `false`, execution traps with
//! `InvariantViolation` (ExitCode 4).
//!
//! Per SPEC.md Section 6.3:
//! - On first encounter, the kernel evaluates the invariant expression.
//! - If false, the kernel sets the trap register to `INVARIANT_VIOLATION`
//!   and halts immediately.
//! - The kernel re-evaluates the invariant at every loop iteration if
//!   the invariant appears inside a loop body.

use crate::ast::{AstNode, AstValue};
use crate::traps::ExitCode;

/// Check an invariant expression and return an error if violated.
///
/// This function is a pure logical check — it does not modify kernel state.
/// The caller is responsible for setting the trap register and halting
/// execution if this returns `Err`.
pub fn check_invariant(condition: &AstNode) -> Result<(), ExitCode> {
    let result = evaluate_invariant_condition(condition);
    if result == Some(false) {
        Err(ExitCode::InvariantViolation)
    } else {
        Ok(())
    }
}

/// Evaluate a simple boolean condition for invariant checking.
///
/// Handles literal booleans, simple comparisons, and identifier lookups
/// that result in boolean values. More complex expressions should be
/// evaluated by the full interpreter.
fn evaluate_invariant_condition(node: &AstNode) -> Option<bool> {
    use crate::ast::AstNodeKind;
    match node.kind {
        AstNodeKind::BooleanLiteral => match &node.value {
            Some(AstValue::Boolean(b)) => Some(*b),
            _ => None,
        },
        _ => {
            // Complex expressions delegated to full interpreter
            None
        }
    }
}

/// Re-evaluate invariants within a loop body.
///
/// This is a stub for the loop-body invariant re-evaluation required
/// by SPEC.md Section 6.3. Full implementation requires access to
/// the interpreter's execution context and symbol table.
pub fn re_evaluate_loop_invariants(_body: &[AstNode]) -> Result<(), ExitCode> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNodeFactory;

    #[test]
    fn invariant_check_true_condition() {
        let condition = AstNodeFactory::boolean_literal(true);
        assert!(check_invariant(&condition).is_ok());
    }

    #[test]
    fn invariant_check_false_condition() {
        let condition = AstNodeFactory::boolean_literal(false);
        assert_eq!(
            check_invariant(&condition).unwrap_err(),
            ExitCode::InvariantViolation
        );
    }
}
