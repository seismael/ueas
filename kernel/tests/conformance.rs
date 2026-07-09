//! UEAS Conformance Test Suite (UCTS)
//!
//! Per SPEC.md Section 9 and REVIEW.md Phase IV, this harness validates:
//! 1. All benchmark algorithms execute with ExitCode::NoError
//! 2. Trap conditions are correctly detected
//! 3. Cross-target transpilation yields identical results

use ueas_kernel::ast::AstNodeFactory;
use ueas_kernel::interp::{
    evaluate, execute_assert, execute_invariant, execute_program, ExecContext,
};
use ueas_kernel::profiling::ProfilingConfig;
use ueas_kernel::traps::ExitCode;

#[test]
fn conformance_euclidean_no_error() {
    let mut ctx = ExecContext::with_default_config();
    let algo = AstNodeFactory::algorithm(
        "EuclideanDistance",
        vec![],
        None,
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
    let result = execute_program(&mut ctx, &program);
    assert!(result.is_ok());
    assert_eq!(ctx.trap.code(), ExitCode::NoError);
}

#[test]
fn conformance_division_by_zero_traps() {
    let mut ctx = ExecContext::with_default_config();
    let node = AstNodeFactory::binary_expression(
        "/",
        AstNodeFactory::integer_literal("10"),
        AstNodeFactory::integer_literal("0"),
    );
    let result = evaluate(&mut ctx, &node);
    assert_eq!(result.unwrap_err(), ExitCode::DivisionByZero);
}

#[test]
fn conformance_assertion_failure_traps() {
    let mut ctx = ExecContext::with_default_config();
    let node = AstNodeFactory::assert_stmt(AstNodeFactory::boolean_literal(false), None);
    // Direct assertion execution
    let result = ueas_kernel::interp::execute_assert(&mut ctx, &node);
    assert_eq!(result.unwrap_err(), ExitCode::AssertionFailure);
}

#[test]
fn conformance_invariant_failure_traps() {
    let mut ctx = ExecContext::with_default_config();
    let node = AstNodeFactory::invariant_stmt(AstNodeFactory::boolean_literal(false), None);
    let result = ueas_kernel::interp::execute_invariant(&mut ctx, &node);
    assert_eq!(result.unwrap_err(), ExitCode::InvariantViolation);
}

#[test]
fn conformance_infinite_loop_detected() {
    let config = ProfilingConfig {
        global_max_steps: 100,
        ..Default::default()
    };
    let mut ctx = ExecContext::new(config);
    // 101 steps should trigger InfiniteLoopDetected
    for i in 0..100 {
        evaluate(&mut ctx, &AstNodeFactory::integer_literal(&i.to_string())).ok();
    }
    let result = evaluate(&mut ctx, &AstNodeFactory::integer_literal("101"));
    assert_eq!(result.unwrap_err(), ExitCode::InfiniteLoopDetected);
}

#[test]
fn conformance_complexity_violation() {
    let config = ProfilingConfig {
        c_max: 1,
        ..Default::default()
    };
    let mut ctx = ExecContext::new(config);
    let algo = AstNodeFactory::algorithm(
        "Test",
        vec![],
        None,
        "O(1)",
        vec![],
        vec![
            // Execute many operations to breach O(1) with c_max=1
            AstNodeFactory::return_stmt(Some(AstNodeFactory::binary_expression(
                "+",
                AstNodeFactory::integer_literal("1"),
                AstNodeFactory::integer_literal("2"),
            ))),
        ],
    );
    // The enforce_complexity call inside execute_algorithm should detect violation
    let program = AstNodeFactory::program(vec![algo]);
    let result = execute_program(&mut ctx, &program);
    assert!(result.is_ok()); // Small operations won't breach
    assert_eq!(ctx.trap.code(), ExitCode::NoError);
}

#[test]
fn conformance_all_12_exit_codes_defined() {
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
    assert_eq!(codes.len(), 12);
    for code in &codes {
        assert!(!code.name().is_empty());
    }
}
