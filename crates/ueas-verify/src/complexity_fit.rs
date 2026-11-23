//! Level 3 Adversarial Multi-Scale BIC-penalized Big-O Time & Space Curve Fitter.

use crate::diagnostic::{RepairDiagnostic, VerificationLevel};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use ueas_hir::complexity::AsymptoticClass;
use ueas_hir::hir::AlgorithmDecl;
use ueas_hir::types::{PrimitiveType, Type};
use ueas_vm::{CompiledProgram, HeapObject, RuntimeValue, TrapCode, VmConfig, VmSession};

/// Empirical measurement at a single input scale `n`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaleMeasurement {
    pub n: usize,
    pub worst_steps: u64,
    pub worst_peak_bytes: usize,
}

/// Fitted complexity verdict for an algorithm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityFitReport {
    pub algorithm: String,
    pub declared_time: Option<AsymptoticClass>,
    pub fitted_time: AsymptoticClass,
    pub declared_space: Option<AsymptoticClass>,
    pub fitted_space: AsymptoticClass,
    pub measurements: Vec<ScaleMeasurement>,
    pub passed: bool,
}

/// Run adversarial multi-scale complexity verification against declared `Complexity:` bounds.
pub fn verify_complexity(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
    scales: &[usize],
) -> (ComplexityFitReport, Vec<RepairDiagnostic>) {
    let default_scales = [8usize, 16, 32, 64, 128, 256];
    let scales = if scales.is_empty() {
        &default_scales[..]
    } else {
        scales
    };

    let mut measurements = Vec::with_capacity(scales.len());
    let mut diags = Vec::new();
    for &n in scales {
        let mut worst_steps = 0u64;
        let mut worst_bytes = 0usize;
        let mut valid_count = 0usize;
        for dist in [
            AdversarialDist::Sorted,
            AdversarialDist::ReverseSorted,
            AdversarialDist::AllEqual,
        ] {
            if let Ok((steps, bytes)) = run_at_scale_if_valid(compiled, algo, n, dist) {
                valid_count += 1;
                worst_steps = worst_steps.max(steps);
                worst_bytes = worst_bytes.max(bytes);
            }
        }
        if valid_count == 0 {
            diags.push(RepairDiagnostic {
                code: "L3_NO_VALID_INPUTS".to_string(),
                level: VerificationLevel::L3,
                message: format!(
                    "Algorithm '{}' had 0 valid inputs satisfying Require preconditions at scale N={n}",
                    algo.name
                ),
                span: Some(algo.span),
                counterexample: None,
                suggested_fix: Some("Check Require preconditions or provide a @Benchmark generator".to_string()),
            });
            return (
                ComplexityFitReport {
                    algorithm: algo.name.clone(),
                    declared_time: algo.complexity.time.as_ref().map(|b| b.class),
                    fitted_time: AsymptoticClass::Constant,
                    declared_space: algo.complexity.space.as_ref().map(|b| b.class),
                    fitted_space: AsymptoticClass::Constant,
                    measurements,
                    passed: false,
                },
                diags,
            );
        }
        measurements.push(ScaleMeasurement {
            n,
            worst_steps: worst_steps.max(1),
            worst_peak_bytes: worst_bytes.max(1),
        });
    }

    let time_points: Vec<(f64, f64)> = measurements
        .iter()
        .map(|m| (m.n as f64, m.worst_steps as f64))
        .collect();
    let space_points: Vec<(f64, f64)> = measurements
        .iter()
        .map(|m| (m.n as f64, m.worst_peak_bytes as f64))
        .collect();

    let fitted_time = fit_best_class(&time_points);
    let fitted_space = fit_best_class(&space_points);
    let declared_time = algo.complexity.time.as_ref().map(|b| b.class);
    let declared_space = algo.complexity.space.as_ref().map(|b| b.class);

    let mut passed = true;

    if let Some(decl) = declared_time {
        if fitted_time > decl {
            passed = false;
            diags.push(RepairDiagnostic {
                code: "L3_COMPLEXITY_TIME_MISMATCH".to_string(),
                level: VerificationLevel::L3,
                message: format!(
                    "Algorithm '{}' declared Time: {}, but adversarial multi-scale BIC fit measured {}",
                    algo.name,
                    decl.label(),
                    fitted_time.label()
                ),
                span: Some(algo.span),
                counterexample: serde_json::to_value(&measurements).ok(),
                suggested_fix: Some(format!(
                    "Optimize algorithm or update Complexity Time bound to {}",
                    fitted_time.label()
                )),
            });
        }
    }

    if let Some(decl) = declared_space {
        if fitted_space > decl {
            passed = false;
            diags.push(RepairDiagnostic {
                code: "L3_COMPLEXITY_SPACE_MISMATCH".to_string(),
                level: VerificationLevel::L3,
                message: format!(
                    "Algorithm '{}' declared Space: {}, but adversarial multi-scale BIC fit measured {}",
                    algo.name,
                    decl.label(),
                    fitted_space.label()
                ),
                span: Some(algo.span),
                counterexample: serde_json::to_value(&measurements).ok(),
                suggested_fix: Some(format!(
                    "Reduce auxiliary heap allocations or update Complexity Space bound to {}",
                    fitted_space.label()
                )),
            });
        }
    }

    (
        ComplexityFitReport {
            algorithm: algo.name.clone(),
            declared_time,
            fitted_time,
            declared_space,
            fitted_space,
            measurements,
            passed,
        },
        diags,
    )
}

#[derive(Debug, Clone, Copy)]
enum AdversarialDist {
    Sorted,
    ReverseSorted,
    AllEqual,
}

fn run_at_scale_if_valid(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
    n: usize,
    dist: AdversarialDist,
) -> Result<(u64, usize), TrapCode> {
    if !algo.preconditions.is_empty() {
        run_at_scale(compiled, algo, n, dist, true)?;
    }
    run_at_scale(compiled, algo, n, dist, false)
}

fn run_at_scale(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
    n: usize,
    dist: AdversarialDist,
    enforce_contracts: bool,
) -> Result<(u64, usize), TrapCode> {
    let mut session = VmSession::new(
        compiled.clone(),
        &algo.name,
        vec![RuntimeValue::Unit; algo.parameters.len()],
        VmConfig {
            step_budget: 5_000_000,
            enforce_contracts,
            max_snapshots: 0,
            ..VmConfig::default()
        },
    )
    .map_err(|e| e.trap)?;

    for (idx, param) in algo.parameters.iter().enumerate() {
        let rv = match &param.ty {
            Type::List { .. } => {
                let data: Vec<RuntimeValue> = (0..n)
                    .map(|i| {
                        let v = match dist {
                            AdversarialDist::Sorted => i as i64 * 2,
                            AdversarialDist::ReverseSorted => (n - i) as i64 * 2,
                            AdversarialDist::AllEqual => 7,
                        };
                        RuntimeValue::Int(BigInt::from(v))
                    })
                    .collect();
                let h = session
                    .heap_mut()
                    .alloc(HeapObject::List(data))
                    .map_err(|e| e.trap)?;
                RuntimeValue::Handle(h)
            }
            Type::Primitive {
                prim: PrimitiveType::Int | PrimitiveType::Int64 | PrimitiveType::UInt64,
            } => {
                // Target or size parameter: choose worst-case non-matching target for search
                if param.name.to_ascii_lowercase().contains("target")
                    || param.name.to_ascii_lowercase().contains("key")
                {
                    RuntimeValue::Int(BigInt::from(-1))
                } else {
                    RuntimeValue::Int(BigInt::from(n as i64))
                }
            }
            Type::Primitive {
                prim: PrimitiveType::Bool,
            } => RuntimeValue::Bool(true),
            _ => RuntimeValue::Unit,
        };
        session.set_arg(idx, rv);
    }

    let input_bytes = session.heap().stats().live_bytes;
    let res = session.run().map_err(|e| e.trap)?;
    let aux_bytes = res.heap.peak_live_bytes.saturating_sub(input_bytes);
    Ok((res.steps, aux_bytes))
}

/// Fit the best `AsymptoticClass` using BIC-penalized normalized least-squares regression.
pub fn fit_best_class(points: &[(f64, f64)]) -> AsymptoticClass {
    let classes = [
        AsymptoticClass::Constant,
        AsymptoticClass::Logarithmic,
        AsymptoticClass::Sqrt,
        AsymptoticClass::Linear,
        AsymptoticClass::Linearithmic,
        AsymptoticClass::NLog2N,
        AsymptoticClass::Quadratic,
        AsymptoticClass::Cubic,
    ];

    let max_y = points
        .iter()
        .map(|(_, y)| *y)
        .fold(1.0_f64, f64::max)
        .max(1.0);
    let min_y = points.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);

    // If absolute growth from smallest to largest scale is <= 15%, it is O(1) constant.
    if (max_y - min_y) / max_y <= 0.15 {
        return AsymptoticClass::Constant;
    }

    let m = points.len().max(1) as f64;
    let mut best_class = AsymptoticClass::Linear;
    let mut best_bic = f64::INFINITY;

    for (rank, class) in classes.iter().enumerate() {
        let mut sum_f2 = 0.0_f64;
        let mut sum_fy = 0.0_f64;
        for &(n, y) in points {
            let f = class.eval_basis(n);
            let yn = y / max_y;
            sum_f2 += f * f;
            sum_fy += f * yn;
        }
        if sum_f2 <= 1e-12 {
            continue;
        }
        let c = (sum_fy / sum_f2).max(0.0);
        let mut rss = 0.0_f64;
        for &(n, y) in points {
            let pred = c * class.eval_basis(n);
            let yn = y / max_y;
            let diff = yn - pred;
            rss += diff * diff;
        }
        let k = if *class == AsymptoticClass::Constant {
            1.0
        } else {
            2.0 + (rank as f64) * 0.05
        };
        let bic = m * (rss / m).max(1e-12).ln() + k * m.ln();
        if bic < best_bic {
            best_bic = bic;
            best_class = *class;
        }
    }

    best_class
}
