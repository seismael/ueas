//! Language-agnostic L4 Differential Oracle Harness (`ueas oracle`).
//!
//! Generates precondition-valid inputs via `ueas-verify`'s fuzzer, executes the UEAS VM
//! as the reference oracle, and compares JSON outputs against any external executable.

use crate::fuzzer::{execute_with_fuzz_args, generate_valid_inputs, FuzzArg};
use serde::{Deserialize, Serialize};
use ueas_hir::check::linker::ModuleResolver;
use ueas_hir::compile_to_hir;
use ueas_vm::{compile_program, HeapObject, RuntimeValue, VirtualHeap};

/// Single test vector exported to an external executable in `ueas oracle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleVector {
    pub args: Vec<serde_json::Value>,
    pub expected: serde_json::Value,
}

/// Summary report returned by `OracleHarness`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleReport {
    pub algorithm: String,
    pub total_vectors: usize,
    pub passed: bool,
    pub mismatch: Option<OracleMismatch>,
}

/// First mismatching input/output discovered by `OracleHarness`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleMismatch {
    pub vector_index: usize,
    pub args: Vec<serde_json::Value>,
    pub expected_ueas: serde_json::Value,
    pub actual_external: serde_json::Value,
}

/// Build reference input/output test vectors from a UEAS algorithm using the VM oracle.
pub fn build_oracle_vectors(
    source: &str,
    entry: &str,
    samples: usize,
    seed: u64,
    resolver: &dyn ModuleResolver,
) -> Result<Vec<OracleVector>, String> {
    let prog = compile_to_hir(source, resolver).map_err(|e| e.join("\n"))?;
    let algo = prog
        .algorithms
        .iter()
        .find(|a| a.name == entry)
        .ok_or_else(|| format!("algorithm '{entry}' not found"))?;
    let compiled = compile_program(&prog)?;
    let valid_inputs = generate_valid_inputs(&compiled, algo, samples, seed);
    let mut vectors = Vec::with_capacity(valid_inputs.len());

    for args in valid_inputs {
        let mut session = ueas_vm::VmSession::new(
            compiled.clone(),
            &algo.name,
            vec![RuntimeValue::Unit; algo.parameters.len()],
            ueas_vm::VmConfig::default(),
        )
        .map_err(|e| e.to_string())?;
        for (idx, a) in args.iter().enumerate() {
            let rv = alloc_fuzz_arg(a, session.heap_mut()).map_err(|e| e.to_string())?;
            session.set_arg(idx, rv);
        }
        let res = session.run().map_err(|e| e.to_string())?;
        let expected_json = runtime_to_json(&res.value, session.heap());
        vectors.push(OracleVector {
            args: args.iter().map(FuzzArg::to_json).collect(),
            expected: expected_json,
        });
    }
    let _ = execute_with_fuzz_args;
    Ok(vectors)
}

fn alloc_fuzz_arg(
    arg: &FuzzArg,
    heap: &mut VirtualHeap,
) -> Result<RuntimeValue, ueas_vm::TrapError> {
    match arg {
        FuzzArg::Int(i) => Ok(RuntimeValue::Int(num_bigint::BigInt::from(*i))),
        FuzzArg::Bool(b) => Ok(RuntimeValue::Bool(*b)),
        FuzzArg::Real(r) => Ok(RuntimeValue::Real(ueas_vm::CanonicalF64::new(*r)?)),
        FuzzArg::Str(s) => Ok(RuntimeValue::Str(s.clone())),
        FuzzArg::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.push(alloc_fuzz_arg(it, heap)?);
            }
            Ok(RuntimeValue::Handle(heap.alloc(HeapObject::List(out))?))
        }
        FuzzArg::Tuple(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                out.push(alloc_fuzz_arg(it, heap)?);
            }
            Ok(RuntimeValue::Tuple(out))
        }
        FuzzArg::OptionVal(None) => Ok(RuntimeValue::OptionVal(None)),
        FuzzArg::OptionVal(Some(inner)) => Ok(RuntimeValue::OptionVal(Some(Box::new(
            alloc_fuzz_arg(inner, heap)?,
        )))),
        _ => Ok(RuntimeValue::Unit),
    }
}

/// Convert any `RuntimeValue` (including `VirtualHeap` objects) into canonical JSON for `ueas oracle`.
pub fn runtime_to_json(val: &RuntimeValue, heap: &VirtualHeap) -> serde_json::Value {
    match val {
        RuntimeValue::Unit => serde_json::Value::Null,
        RuntimeValue::Bool(b) => serde_json::json!(b),
        RuntimeValue::Int(i) => i.to_string().parse::<i64>().map_or_else(
            |_| serde_json::json!(i.to_string()),
            |n| serde_json::json!(n),
        ),
        RuntimeValue::Int64(i) => serde_json::json!(i),
        RuntimeValue::UInt64(u) => serde_json::json!(u),
        RuntimeValue::Real(r) => serde_json::json!(r.get()),
        RuntimeValue::Char(c) => serde_json::json!(c.to_string()),
        RuntimeValue::Str(s) => serde_json::json!(s),
        RuntimeValue::OptionVal(None) => serde_json::Value::Null,
        RuntimeValue::OptionVal(Some(inner)) => runtime_to_json(inner, heap),
        RuntimeValue::ResultVal(Ok(v)) => serde_json::json!({ "ok": runtime_to_json(v, heap) }),
        RuntimeValue::ResultVal(Err(e)) => serde_json::json!({ "err": runtime_to_json(e, heap) }),
        RuntimeValue::Tuple(items) => {
            serde_json::Value::Array(items.iter().map(|x| runtime_to_json(x, heap)).collect())
        }
        RuntimeValue::Handle(h) => match heap.get(*h) {
            Ok((HeapObject::List(items), _)) => {
                serde_json::Value::Array(items.iter().map(|x| runtime_to_json(x, heap)).collect())
            }
            Ok((HeapObject::Record { fields, .. }, _)) => {
                let mut m = serde_json::Map::new();
                for (k, v) in fields {
                    m.insert(k.clone(), runtime_to_json(v, heap));
                }
                serde_json::Value::Object(m)
            }
            _ => serde_json::Value::Null,
        },
        RuntimeValue::FnRef(i) => serde_json::json!(i),
    }
}
