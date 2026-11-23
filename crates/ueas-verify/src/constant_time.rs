//! Fail-closed multi-vector dynamic observational constant-time verifier.

use crate::diagnostic::{RepairDiagnostic, VerificationLevel};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use ueas_hir::hir::AlgorithmDecl;
use ueas_hir::types::{PrimitiveType, Type};
use ueas_vm::{CompiledProgram, HeapObject, RuntimeValue, TraceEvent, VmConfig, VmSession};

/// Result of dynamic trace equivalence checking across secret input pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantTimeReport {
    pub algorithm: String,
    pub passed: bool,
    pub trace_len: usize,
    pub diverging_step: Option<usize>,
}

const SECRET_PAIRS: &[(i64, i64)] = &[
    (0, 1),
    (0, -1),
    (0x5555, 0xAAAA),
    (0x00AA_5511, 0x7FFF_0001),
    (1, i64::MAX),
    (42, -42),
];

/// Verify that `@ConstantTime` algorithm produces identical `(pc, cache_line, branch_taken)` traces
/// across 6 distinct `Secret` input pairs (fail-closed on any execution error).
pub fn verify_constant_time(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
) -> (ConstantTimeReport, Vec<RepairDiagnostic>) {
    if !algo.constant_time {
        return (
            ConstantTimeReport {
                algorithm: algo.name.clone(),
                passed: true,
                trace_len: 0,
                diverging_step: None,
            },
            Vec::new(),
        );
    }

    let mut max_len = 0usize;
    for &(s1, s2) in SECRET_PAIRS {
        let trace1 = match run_trace(compiled, algo, s1) {
            Ok(t) => t,
            Err(err) => {
                return fail_exec_error(algo, &err);
            }
        };
        let trace2 = match run_trace(compiled, algo, s2) {
            Ok(t) => t,
            Err(err) => {
                return fail_exec_error(algo, &err);
            }
        };
        max_len = max_len.max(trace1.len());

        let min_len = trace1.len().min(trace2.len());
        for i in 0..min_len {
            if trace1[i] != trace2[i] {
                let diag = RepairDiagnostic {
                    code: "CT_DYNAMIC_LEAK".to_string(),
                    level: VerificationLevel::L2,
                    message: format!(
                        "Observational constant-time violation in '{}' at trace step {} (pc {} vs {}) for secret pair ({s1}, {s2})",
                        algo.name, i, trace1[i].pc, trace2[i].pc
                    ),
                    span: Some(algo.span),
                    counterexample: Some(serde_json::json!({
                        "step": i,
                        "secret_a": s1,
                        "secret_b": s2,
                        "trace_a": trace1[i],
                        "trace_b": trace2[i],
                    })),
                    suggested_fix: Some(
                        "Replace secret-dependent branch or memory index with branchless `select(cond, a, b)`"
                            .to_string(),
                    ),
                };
                return (
                    ConstantTimeReport {
                        algorithm: algo.name.clone(),
                        passed: false,
                        trace_len: min_len,
                        diverging_step: Some(i),
                    },
                    vec![diag],
                );
            }
        }

        if trace1.len() != trace2.len() {
            let diag = RepairDiagnostic {
                code: "CT_DYNAMIC_LENGTH_LEAK".to_string(),
                level: VerificationLevel::L2,
                message: format!(
                    "Observational constant-time trace length leak in '{}': {} vs {} steps",
                    algo.name,
                    trace1.len(),
                    trace2.len()
                ),
                span: Some(algo.span),
                counterexample: None,
                suggested_fix: Some("Ensure loop bounds depend only on Public values".to_string()),
            };
            return (
                ConstantTimeReport {
                    algorithm: algo.name.clone(),
                    passed: false,
                    trace_len: min_len,
                    diverging_step: Some(min_len),
                },
                vec![diag],
            );
        }
    }

    (
        ConstantTimeReport {
            algorithm: algo.name.clone(),
            passed: true,
            trace_len: max_len,
            diverging_step: None,
        },
        Vec::new(),
    )
}

fn fail_exec_error(algo: &AlgorithmDecl, err: &str) -> (ConstantTimeReport, Vec<RepairDiagnostic>) {
    let diag = RepairDiagnostic {
        code: "CT_DYNAMIC_EXEC_ERROR".to_string(),
        level: VerificationLevel::L2,
        message: format!(
            "Dynamic constant-time verification failed to execute '{}': {err}",
            algo.name
        ),
        span: Some(algo.span),
        counterexample: None,
        suggested_fix: Some(
            "Ensure @ConstantTime algorithm executes cleanly without runtime traps".to_string(),
        ),
    };
    (
        ConstantTimeReport {
            algorithm: algo.name.clone(),
            passed: false,
            trace_len: 0,
            diverging_step: None,
        },
        vec![diag],
    )
}

fn run_trace(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
    secret_val: i64,
) -> Result<Vec<TraceEvent>, String> {
    let mut session = VmSession::new(
        compiled.clone(),
        &algo.name,
        vec![RuntimeValue::Unit; algo.parameters.len()],
        VmConfig {
            record_trace: true,
            enforce_contracts: false,
            ..VmConfig::default()
        },
    )
    .map_err(|e| e.to_string())?;

    for (idx, param) in algo.parameters.iter().enumerate() {
        let is_secret = matches!(param.ty, Type::Secret { .. });
        let inner = match &param.ty {
            Type::Secret { inner } => inner.as_ref(),
            other => other,
        };
        let rv = match inner {
            Type::Primitive {
                prim: PrimitiveType::Int | PrimitiveType::Int64 | PrimitiveType::UInt64,
            } => RuntimeValue::Int(BigInt::from(if is_secret { secret_val } else { 8 })),
            Type::Primitive {
                prim: PrimitiveType::Bool,
            } => RuntimeValue::Bool(if is_secret {
                (secret_val & 1) == 1
            } else {
                true
            }),
            Type::List { .. } => {
                let items: Vec<RuntimeValue> = (0..8)
                    .map(|i| {
                        RuntimeValue::Int(BigInt::from(if is_secret { secret_val + i } else { i }))
                    })
                    .collect();
                let h = session
                    .heap_mut()
                    .alloc(HeapObject::List(items))
                    .map_err(|e| e.to_string())?;
                RuntimeValue::Handle(h)
            }
            _ => RuntimeValue::Unit,
        };
        session.set_arg(idx, rv);
    }

    let res = session.run().map_err(|e| e.to_string())?;
    Ok(res.trace)
}
