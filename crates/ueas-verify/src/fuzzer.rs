//! Level 2 precondition-guided recursive property fuzzer with minimal counterexample shrinking.

use crate::diagnostic::{RepairDiagnostic, VerificationLevel};
use indexmap::{IndexMap, IndexSet};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use ueas_hir::hir::{AlgorithmDecl, RecordDecl};
use ueas_hir::types::{PrimitiveType, Type};
use ueas_vm::{
    CompiledProgram, DeterministicPrng, HeapObject, RuntimeValue, TrapCode, VmConfig, VmSession,
};

/// Recursive generated input specification (serializable & shrinkable).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FuzzArg {
    Int(i64),
    Bool(bool),
    Real(f64),
    Str(String),
    List(Vec<FuzzArg>),
    Set(Vec<FuzzArg>),
    Map(Vec<(FuzzArg, FuzzArg)>),
    Tuple(Vec<FuzzArg>),
    OptionVal(Option<Box<FuzzArg>>),
    Record {
        name: String,
        fields: Vec<(String, FuzzArg)>,
    },
    Unit,
}

impl FuzzArg {
    /// Convert to JSON for diagnostic counterexamples and external `ueas oracle` stdin/stdout.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Int(i) => serde_json::json!(i),
            Self::Bool(b) => serde_json::json!(b),
            Self::Real(r) => serde_json::json!(r),
            Self::Str(s) => serde_json::json!(s),
            Self::List(v) | Self::Set(v) | Self::Tuple(v) => {
                serde_json::Value::Array(v.iter().map(Self::to_json).collect())
            }
            Self::Map(entries) => serde_json::Value::Array(
                entries
                    .iter()
                    .map(|(k, v)| serde_json::json!([k.to_json(), v.to_json()]))
                    .collect(),
            ),
            Self::OptionVal(None) | Self::Unit => serde_json::Value::Null,
            Self::OptionVal(Some(inner)) => inner.to_json(),
            Self::Record { fields, .. } => {
                let mut m = serde_json::Map::new();
                for (k, v) in fields {
                    m.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(m)
            }
        }
    }

    fn alloc_in_vm(&self, session: &mut VmSession) -> Result<RuntimeValue, ueas_vm::TrapError> {
        match self {
            Self::Int(i) => Ok(RuntimeValue::Int(BigInt::from(*i))),
            Self::Bool(b) => Ok(RuntimeValue::Bool(*b)),
            Self::Real(r) => Ok(RuntimeValue::Real(ueas_vm::CanonicalF64::new(*r)?)),
            Self::Str(s) => Ok(RuntimeValue::Str(s.clone())),
            Self::Unit => Ok(RuntimeValue::Unit),
            Self::OptionVal(None) => Ok(RuntimeValue::OptionVal(None)),
            Self::OptionVal(Some(inner)) => Ok(RuntimeValue::OptionVal(Some(Box::new(
                inner.alloc_in_vm(session)?,
            )))),
            Self::Tuple(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    out.push(it.alloc_in_vm(session)?);
                }
                Ok(RuntimeValue::Tuple(out))
            }
            Self::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    out.push(it.alloc_in_vm(session)?);
                }
                let h = session.heap_mut().alloc(HeapObject::List(out))?;
                Ok(RuntimeValue::Handle(h))
            }
            Self::Set(items) => {
                let mut set = IndexSet::new();
                for it in items {
                    let rv = it.alloc_in_vm(session)?;
                    set.insert(rv.to_key()?);
                }
                let h = session.heap_mut().alloc(HeapObject::Set(set))?;
                Ok(RuntimeValue::Handle(h))
            }
            Self::Map(entries) => {
                let mut map = IndexMap::new();
                for (k, v) in entries {
                    let rk = k.alloc_in_vm(session)?.to_key()?;
                    let rv = v.alloc_in_vm(session)?;
                    map.insert(rk, rv);
                }
                let h = session.heap_mut().alloc(HeapObject::Map(map))?;
                Ok(RuntimeValue::Handle(h))
            }
            Self::Record { name, fields } => {
                let mut map = IndexMap::new();
                for (fname, fval) in fields {
                    map.insert(fname.clone(), fval.alloc_in_vm(session)?);
                }
                let h = session.heap_mut().alloc(HeapObject::Record {
                    type_name: name.clone(),
                    fields: map,
                })?;
                Ok(RuntimeValue::Handle(h))
            }
        }
    }
}

/// Summary of Level 2 property fuzzing for an algorithm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FuzzReport {
    pub algorithm: String,
    pub valid_runs: usize,
    pub skipped_precondition: usize,
    pub passed: bool,
    pub minimal_counterexample: Option<Vec<FuzzArg>>,
}

/// Run precondition-guided property fuzzing on `algo`.
pub fn fuzz_algorithm(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
    iterations: usize,
    seed: u64,
) -> (FuzzReport, Vec<RepairDiagnostic>) {
    let mut prng = DeterministicPrng::new(seed);
    let mut valid_runs = 0usize;
    let mut skipped_precondition = 0usize;
    let mut diags = Vec::new();
    let param_types: Vec<Type> = algo.parameters.iter().map(|p| p.ty.clone()).collect();

    for iter in 0..iterations {
        let strategy = iter % 4;
        let args = gen_args(&param_types, &compiled.records, strategy, &mut prng);
        match execute_with_fuzz_args(compiled, &algo.name, &args) {
            Ok(_) => valid_runs += 1,
            Err(trap) if trap.trap == TrapCode::PreconditionViolated => {
                skipped_precondition += 1;
            }
            Err(trap) => {
                let shrunk = shrink_counterexample(compiled, &algo.name, &args);
                let ce_json: Vec<serde_json::Value> = shrunk.iter().map(FuzzArg::to_json).collect();
                diags.push(RepairDiagnostic {
                    code: format!("L2_{}", trap.trap.code()),
                    level: VerificationLevel::L2,
                    message: format!(
                        "Property fuzzing failed on '{}' ({}): {}",
                        algo.name,
                        trap.trap.code(),
                        trap.message
                    ),
                    span: trap.span.or(Some(algo.span)),
                    counterexample: Some(serde_json::Value::Array(ce_json)),
                    suggested_fix: Some(
                        "Check Require preconditions, loop bounds, or Ensure postconditions against the minimal counterexample"
                            .to_string(),
                    ),
                });
                return (
                    FuzzReport {
                        algorithm: algo.name.clone(),
                        valid_runs,
                        skipped_precondition,
                        passed: false,
                        minimal_counterexample: Some(shrunk),
                    },
                    diags,
                );
            }
        }
    }

    (
        FuzzReport {
            algorithm: algo.name.clone(),
            valid_runs,
            skipped_precondition,
            passed: true,
            minimal_counterexample: None,
        },
        diags,
    )
}

/// Generate a batch of precondition-valid `FuzzArg` vectors for `algo` (reused by `ueas oracle`).
pub fn generate_valid_inputs(
    compiled: &CompiledProgram,
    algo: &AlgorithmDecl,
    count: usize,
    seed: u64,
) -> Vec<Vec<FuzzArg>> {
    let mut prng = DeterministicPrng::new(seed);
    let param_types: Vec<Type> = algo.parameters.iter().map(|p| p.ty.clone()).collect();
    let mut out = Vec::with_capacity(count);
    let max_attempts = count.saturating_mul(8).max(32);

    for attempt in 0..max_attempts {
        if out.len() >= count {
            break;
        }
        let args = gen_args(&param_types, &compiled.records, attempt % 4, &mut prng);
        if execute_with_fuzz_args(compiled, &algo.name, &args).is_ok() {
            out.push(args);
        }
    }
    out
}

fn gen_args(
    types: &[Type],
    records: &[RecordDecl],
    strategy: usize,
    prng: &mut DeterministicPrng,
) -> Vec<FuzzArg> {
    types
        .iter()
        .map(|ty| gen_arg(ty, records, strategy, prng, 0))
        .collect()
}

fn gen_arg(
    ty: &Type,
    records: &[RecordDecl],
    strategy: usize,
    prng: &mut DeterministicPrng,
    depth: usize,
) -> FuzzArg {
    if depth > 4 {
        return FuzzArg::Unit;
    }
    match ty {
        Type::Primitive {
            prim: PrimitiveType::Int | PrimitiveType::Int64,
        } => {
            let v = match strategy {
                0 => prng.gen_range_i64(0, 20),
                1 => prng.gen_range_i64(-50, 50),
                2 => 0,
                _ => prng.gen_range_i64(1, 100),
            };
            FuzzArg::Int(v)
        }
        Type::Primitive {
            prim: PrimitiveType::UInt64 | PrimitiveType::UInt32 | PrimitiveType::Byte,
        } => FuzzArg::Int(prng.gen_range_i64(0, 100)),
        Type::Primitive {
            prim: PrimitiveType::Bool,
        } => FuzzArg::Bool((prng.next_u64() & 1) == 0),
        Type::Primitive {
            prim: PrimitiveType::Real,
        } => FuzzArg::Real(prng.gen_unit_f64() * 100.0 - 50.0),
        Type::Primitive {
            prim: PrimitiveType::String,
        } => {
            let len = prng.gen_range_i64(0, 8) as usize;
            let s: String = (0..len)
                .map(|_| (b'a' + (prng.next_u64() % 26) as u8) as char)
                .collect();
            FuzzArg::Str(s)
        }
        Type::List { elem } => {
            let len = match strategy {
                2 => 0,
                3 => 1,
                _ => prng.gen_range_i64(1, 12) as usize,
            };
            let mut items: Vec<FuzzArg> = (0..len)
                .map(|_| gen_arg(elem, records, strategy, prng, depth + 1))
                .collect();
            if matches!(
                elem.as_ref(),
                Type::Primitive {
                    prim: PrimitiveType::Int | PrimitiveType::Int64
                }
            ) && strategy == 0
            {
                items.sort_by_key(|a| match a {
                    FuzzArg::Int(v) => *v,
                    _ => 0,
                });
            }
            FuzzArg::List(items)
        }
        Type::Set { elem } => {
            let len = prng.gen_range_i64(0, 6) as usize;
            let items = (0..len)
                .map(|_| gen_arg(elem, records, strategy, prng, depth + 1))
                .collect();
            FuzzArg::Set(items)
        }
        Type::Map { key, val } => {
            let len = prng.gen_range_i64(0, 6) as usize;
            let entries = (0..len)
                .map(|_| {
                    (
                        gen_arg(key, records, strategy, prng, depth + 1),
                        gen_arg(val, records, strategy, prng, depth + 1),
                    )
                })
                .collect();
            FuzzArg::Map(entries)
        }
        Type::Tuple { elems } => FuzzArg::Tuple(
            elems
                .iter()
                .map(|e| gen_arg(e, records, strategy, prng, depth + 1))
                .collect(),
        ),
        Type::Option { inner } => {
            if strategy == 2 {
                FuzzArg::OptionVal(None)
            } else {
                FuzzArg::OptionVal(Some(Box::new(gen_arg(
                    inner,
                    records,
                    strategy,
                    prng,
                    depth + 1,
                ))))
            }
        }
        Type::Secret { inner } => gen_arg(inner, records, strategy, prng, depth),
        Type::Record { name, .. } => {
            if let Some(rec) = records.iter().find(|r| &r.name == name) {
                let fields = rec
                    .fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone(),
                            gen_arg(&f.ty, records, strategy, prng, depth + 1),
                        )
                    })
                    .collect();
                FuzzArg::Record {
                    name: name.clone(),
                    fields,
                }
            } else {
                FuzzArg::Unit
            }
        }
        _ => FuzzArg::Unit,
    }
}

pub(crate) fn execute_with_fuzz_args(
    compiled: &CompiledProgram,
    algo: &str,
    args: &[FuzzArg],
) -> Result<RuntimeValue, ueas_vm::TrapError> {
    let placeholder_args = vec![RuntimeValue::Unit; args.len()];
    let mut session = VmSession::new(
        compiled.clone(),
        algo,
        placeholder_args,
        VmConfig::default(),
    )?;
    for (idx, a) in args.iter().enumerate() {
        let rv = a.alloc_in_vm(&mut session)?;
        session.set_arg(idx, rv);
    }
    session.run().map(|r| r.value)
}

fn shrink_counterexample(
    compiled: &CompiledProgram,
    algo: &str,
    initial: &[FuzzArg],
) -> Vec<FuzzArg> {
    let mut best = initial.to_vec();
    for idx in 0..best.len() {
        match best[idx].clone() {
            FuzzArg::List(mut list) => {
                while list.len() > 1 {
                    let half = list.len() / 2;
                    let candidate_vec = list[..half].to_vec();
                    let mut cand = best.clone();
                    cand[idx] = FuzzArg::List(candidate_vec.clone());
                    if matches!(execute_with_fuzz_args(compiled, algo, &cand), Err(t) if t.trap != TrapCode::PreconditionViolated)
                    {
                        list = candidate_vec;
                        best = cand;
                    } else {
                        break;
                    }
                }
            }
            FuzzArg::Int(mut val) => {
                while val.abs() > 1 {
                    let next = val / 2;
                    let mut cand = best.clone();
                    cand[idx] = FuzzArg::Int(next);
                    if matches!(execute_with_fuzz_args(compiled, algo, &cand), Err(t) if t.trap != TrapCode::PreconditionViolated)
                    {
                        val = next;
                        best = cand;
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    best
}
