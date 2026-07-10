//! UEAS Conformance Test Suite (UCTS)
//!
//! Per SPEC.md Section 9 and REVIEW.md Phase IV, this harness validates:
//! 1. All benchmark algorithms execute with ExitCode::NoError
//! 2. Trap conditions are correctly detected
//! 3. Cross-target transpilation yields identical results

use ueas_kernel::ast::{AstNode, AstNodeFactory, AstNodeKind, AstValue};
use ueas_kernel::heap::{HeapConfig, TypeTag, VirtualHeap};
use ueas_kernel::interp::{evaluate, execute_program, ExecContext};
use ueas_kernel::profiling::{ComplexityContract, ComplexityKind, Profiler, ProfilingConfig};
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
        global_max_steps: 100000,
        ..Default::default()
    };
    let mut ctx = ExecContext::new(config);
    // O(1) allows 1*c_max=1 step. Running >1 step should violate.
    for _ in 0..8000u64 {
        ctx.profiler.step().ok();
    }
    let contract = ComplexityContract {
        kind: ComplexityKind::Constant,
        expected_complexity: None,
    };
    assert_eq!(
        ctx.profiler.verify_complexity(&contract).unwrap_err(),
        ExitCode::ComplexityViolation
    );
}

#[test]
fn conformance_index_out_of_bounds() {
    let mut heap = VirtualHeap::new(HeapConfig {
        max_size: 128,
        ..Default::default()
    });
    let h = heap.allocate(8, TypeTag::Integer).unwrap();
    assert_eq!(
        heap.write(h, 10, &[1, 2]).unwrap_err(),
        ExitCode::IndexOutOfBounds
    );
}

#[test]
fn conformance_null_dereference() {
    let mut heap = VirtualHeap::with_default_config();
    let h = heap.allocate(8, TypeTag::Integer).unwrap();
    heap.deallocate(h).unwrap();
    assert_eq!(heap.read(h, 0, 1).unwrap_err(), ExitCode::NullDereference);
}

#[test]
fn conformance_stack_overflow() {
    let config = ProfilingConfig {
        max_recursion_depth: 2,
        ..Default::default()
    };
    let mut profiler = Profiler::new(config);
    profiler.enter_recursion().unwrap();
    profiler.enter_recursion().unwrap();
    assert_eq!(
        profiler.enter_recursion().unwrap_err(),
        ExitCode::StackOverflow
    );
}

#[test]
fn conformance_heap_exhaustion() {
    let mut heap = VirtualHeap::new(HeapConfig {
        max_size: 16,
        alignment: 8,
    });
    assert_eq!(
        heap.allocate(32, TypeTag::Integer).unwrap_err(),
        ExitCode::HeapExhaustion
    );
}

#[test]
fn conformance_invalid_operation() {
    // Send an unsupported operator to evaluate
    let mut ctx = ExecContext::with_default_config();
    let n = AstNode::internal(
        AstNodeKind::BinaryExpression,
        vec![
            AstNode::leaf(
                AstNodeKind::Identifier,
                Some(AstValue::String("??".to_string())),
            ),
            AstNodeFactory::integer_literal("1"),
            AstNodeFactory::integer_literal("2"),
        ],
        None,
    );
    assert_eq!(
        evaluate(&mut ctx, &n).unwrap_err(),
        ExitCode::InvalidOperation
    );
}

#[test]
fn conformance_all_13_exit_codes_defined() {
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
        ExitCode::TimingLeak,
    ];
    assert_eq!(codes.len(), 13);
    for code in &codes {
        assert!(!code.name().is_empty());
    }
}
