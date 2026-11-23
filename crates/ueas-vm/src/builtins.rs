//! Deterministic standard built-in functions and collection methods for `ueas-vm`.

use crate::cache::CacheHierarchy;
use crate::heap::{HeapObject, VirtualHeap};
use crate::prng::DeterministicPrng;
use crate::trap::{TrapCode, TrapError};
use crate::value::{CanonicalF64, RuntimeKey, RuntimeValue};
use indexmap::{IndexMap, IndexSet};
use num_bigint::BigInt;

fn trap(code: TrapCode, msg: impl Into<String>) -> TrapError {
    TrapError::new(code, msg, 0, 0, None)
}

/// Dispatch a global built-in function call (`len`, `abs`, `min`, `max`, `Some`, `Ok`, `Err`, `rand_int`, etc.).
///
/// Returns `Ok(Some(val))` if handled as a builtin, or `Ok(None)` if `name` is a user algorithm.
pub fn call_builtin(
    name: &str,
    args: &[RuntimeValue],
    heap: &mut VirtualHeap,
    prng: &mut DeterministicPrng,
) -> Result<Option<RuntimeValue>, TrapError> {
    let val = match name {
        "Some" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "Some expects 1 argument"));
            }
            RuntimeValue::OptionVal(Some(Box::new(args[0].clone())))
        }
        "Ok" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "Ok expects 1 argument"));
            }
            RuntimeValue::ResultVal(Ok(Box::new(args[0].clone())))
        }
        "Err" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "Err expects 1 argument"));
            }
            RuntimeValue::ResultVal(Err(Box::new(args[0].clone())))
        }
        "len" | "length" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "len expects 1 argument"));
            }
            match &args[0] {
                RuntimeValue::Str(s) => RuntimeValue::Int(BigInt::from(s.len())),
                RuntimeValue::Tuple(t) => RuntimeValue::Int(BigInt::from(t.len())),
                RuntimeValue::Handle(h) => {
                    let (obj, _) = heap.get(*h)?;
                    let n = match obj {
                        HeapObject::List(v) => v.len(),
                        HeapObject::Set(s) => s.len(),
                        HeapObject::Map(m) => m.len(),
                        HeapObject::Graph { adj, .. } => adj.len(),
                        HeapObject::Matrix { rows, .. } => *rows,
                        HeapObject::Record { fields, .. } => fields.len(),
                    };
                    RuntimeValue::Int(BigInt::from(n))
                }
                _ => return Err(trap(TrapCode::TypeMismatch, "len() unsupported on value")),
            }
        }
        "abs" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "abs expects 1 argument"));
            }
            match &args[0] {
                RuntimeValue::Int(b) => {
                    if b < &BigInt::from(0) {
                        RuntimeValue::Int(-b)
                    } else {
                        RuntimeValue::Int(b.clone())
                    }
                }
                RuntimeValue::Int64(i) => RuntimeValue::Int64(
                    i.checked_abs()
                        .ok_or_else(|| trap(TrapCode::IntegerOverflow, "i64 abs overflow"))?,
                ),
                RuntimeValue::Real(r) => RuntimeValue::Real(CanonicalF64::new(r.get().abs())?),
                _ => return Err(trap(TrapCode::TypeMismatch, "abs() expects numeric value")),
            }
        }
        "min" => {
            if args.len() != 2 {
                return Err(trap(TrapCode::ArityMismatch, "min expects 2 arguments"));
            }
            if cmp_values(&args[0], &args[1])? == std::cmp::Ordering::Less {
                args[0].clone()
            } else {
                args[1].clone()
            }
        }
        "max" => {
            if args.len() != 2 {
                return Err(trap(TrapCode::ArityMismatch, "max expects 2 arguments"));
            }
            if cmp_values(&args[0], &args[1])? == std::cmp::Ordering::Greater {
                args[0].clone()
            } else {
                args[1].clone()
            }
        }
        "sqrt" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "sqrt expects 1 argument"));
            }
            let x = args[0]
                .as_f64()
                .ok_or_else(|| trap(TrapCode::TypeMismatch, "sqrt expects numeric"))?;
            if x < 0.0 {
                return Err(trap(TrapCode::FloatInvalid, "sqrt of negative number"));
            }
            RuntimeValue::Real(CanonicalF64::new(x.sqrt())?)
        }
        "floor" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "floor expects 1 argument"));
            }
            let x = args[0]
                .as_f64()
                .ok_or_else(|| trap(TrapCode::TypeMismatch, "floor expects numeric"))?;
            RuntimeValue::Int(BigInt::from(x.floor() as i64))
        }
        "ceil" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "ceil expects 1 argument"));
            }
            let x = args[0]
                .as_f64()
                .ok_or_else(|| trap(TrapCode::TypeMismatch, "ceil expects numeric"))?;
            RuntimeValue::Int(BigInt::from(x.ceil() as i64))
        }
        "log2" => {
            if args.len() != 1 {
                return Err(trap(TrapCode::ArityMismatch, "log2 expects 1 argument"));
            }
            let x = args[0]
                .as_f64()
                .ok_or_else(|| trap(TrapCode::TypeMismatch, "log2 expects numeric"))?;
            if x <= 0.0 {
                return Err(trap(TrapCode::FloatInvalid, "log2 domain error (x <= 0)"));
            }
            RuntimeValue::Real(CanonicalF64::new(x.log2())?)
        }
        "pow" => {
            if args.len() != 2 {
                return Err(trap(TrapCode::ArityMismatch, "pow expects 2 arguments"));
            }
            match (&args[0], &args[1]) {
                (RuntimeValue::Int(a), RuntimeValue::Int(b)) => {
                    let exp = b.try_into().map_err(|_| {
                        trap(TrapCode::IntegerOverflow, "negative or huge pow exponent")
                    })?;
                    RuntimeValue::Int(a.pow(exp))
                }
                _ => {
                    let a = args[0]
                        .as_f64()
                        .ok_or_else(|| trap(TrapCode::TypeMismatch, "pow expects numeric"))?;
                    let b = args[1]
                        .as_f64()
                        .ok_or_else(|| trap(TrapCode::TypeMismatch, "pow expects numeric"))?;
                    RuntimeValue::Real(CanonicalF64::new(a.powf(b))?)
                }
            }
        }
        "rand_int" => {
            if args.len() != 2 {
                return Err(trap(
                    TrapCode::ArityMismatch,
                    "rand_int expects 2 arguments",
                ));
            }
            let low = args[0]
                .as_i64()
                .ok_or_else(|| trap(TrapCode::TypeMismatch, "rand_int low must be Int"))?;
            let high = args[1]
                .as_i64()
                .ok_or_else(|| trap(TrapCode::TypeMismatch, "rand_int high must be Int"))?;
            RuntimeValue::Int(BigInt::from(prng.gen_range_i64(low, high)))
        }
        "rand_real" => {
            if !args.is_empty() {
                return Err(trap(
                    TrapCode::ArityMismatch,
                    "rand_real expects 0 arguments",
                ));
            }
            RuntimeValue::Real(CanonicalF64::new(prng.gen_unit_f64())?)
        }
        "Matrix" | "new_matrix" => {
            if args.len() != 3 {
                return Err(trap(
                    TrapCode::ArityMismatch,
                    "Matrix(rows, cols, fill) expects 3 arguments",
                ));
            }
            let rows = args[0].as_usize().ok_or_else(|| {
                trap(
                    TrapCode::TypeMismatch,
                    "Matrix rows must be non-negative Int",
                )
            })?;
            let cols = args[1].as_usize().ok_or_else(|| {
                trap(
                    TrapCode::TypeMismatch,
                    "Matrix cols must be non-negative Int",
                )
            })?;
            let data = vec![args[2].clone(); rows.saturating_mul(cols)];
            let h = heap.alloc(HeapObject::Matrix { rows, cols, data })?;
            RuntimeValue::Handle(h)
        }
        "Graph" | "new_graph" => {
            let directed = match args.first() {
                Some(RuntimeValue::Bool(b)) => *b,
                None => true,
                Some(_) => {
                    return Err(trap(TrapCode::TypeMismatch, "Graph(directed) expects Bool"))
                }
            };
            let h = heap.alloc(HeapObject::Graph {
                directed,
                adj: IndexMap::new(),
            })?;
            RuntimeValue::Handle(h)
        }
        "Set" | "new_set" => {
            let h = heap.alloc(HeapObject::Set(IndexSet::new()))?;
            RuntimeValue::Handle(h)
        }
        "Map" | "new_map" => {
            let h = heap.alloc(HeapObject::Map(IndexMap::new()))?;
            RuntimeValue::Handle(h)
        }
        "List" | "new_list" => {
            let h = heap.alloc(HeapObject::List(Vec::new()))?;
            RuntimeValue::Handle(h)
        }
        "to_string" | "str" => {
            if args.len() != 1 {
                return Err(trap(
                    TrapCode::ArityMismatch,
                    "to_string expects 1 argument",
                ));
            }
            RuntimeValue::Str(args[0].to_string())
        }
        "declassify" => {
            if args.len() != 1 {
                return Err(trap(
                    TrapCode::ArityMismatch,
                    "declassify expects 1 argument",
                ));
            }
            args[0].clone()
        }
        _ => return Ok(None),
    };
    Ok(Some(val))
}

/// Dispatch a method call (`receiver.method(args...)`).
pub fn call_method(
    receiver: &RuntimeValue,
    method: &str,
    args: &[RuntimeValue],
    heap: &mut VirtualHeap,
    cache: &mut CacheHierarchy,
) -> Result<(RuntimeValue, Option<u64>), TrapError> {
    match receiver {
        RuntimeValue::Str(s) => match method {
            "length" | "len" => Ok((RuntimeValue::Int(BigInt::from(s.len())), None)),
            "is_empty" => Ok((RuntimeValue::Bool(s.is_empty()), None)),
            "contains" => {
                let sub = match args.first() {
                    Some(RuntimeValue::Str(x)) => x.as_str(),
                    _ => return Err(trap(TrapCode::TypeMismatch, "str.contains expects Str")),
                };
                Ok((RuntimeValue::Bool(s.contains(sub)), None))
            }
            _ => Err(trap(
                TrapCode::UndefinedSymbol,
                format!("unknown String method '{method}'"),
            )),
        },
        RuntimeValue::OptionVal(opt) => match method {
            "is_some" => Ok((RuntimeValue::Bool(opt.is_some()), None)),
            "is_none" => Ok((RuntimeValue::Bool(opt.is_none()), None)),
            "unwrap" => opt
                .as_ref()
                .map(|v| ((**v).clone(), None))
                .ok_or_else(|| trap(TrapCode::AssertionFailed, "unwrap() called on None")),
            "unwrap_or" => {
                let default = args
                    .first()
                    .ok_or_else(|| trap(TrapCode::ArityMismatch, "unwrap_or expects 1 arg"))?;
                Ok((
                    opt.as_ref()
                        .map_or_else(|| default.clone(), |v| (**v).clone()),
                    None,
                ))
            }
            _ => Err(trap(
                TrapCode::UndefinedSymbol,
                format!("unknown Option method '{method}'"),
            )),
        },
        RuntimeValue::ResultVal(res) => match method {
            "is_ok" => Ok((RuntimeValue::Bool(res.is_ok()), None)),
            "is_err" => Ok((RuntimeValue::Bool(res.is_err()), None)),
            "unwrap" => match res {
                Ok(v) => Ok(((**v).clone(), None)),
                Err(e) => Err(trap(
                    TrapCode::AssertionFailed,
                    format!("unwrap() on Err({e})"),
                )),
            },
            _ => Err(trap(
                TrapCode::UndefinedSymbol,
                format!("unknown Result method '{method}'"),
            )),
        },
        RuntimeValue::Handle(h) => {
            let handle = *h;
            match method {
                "length" | "len" => {
                    let (obj, base) = heap.get(handle)?;
                    let line = cache.access(base, 8);
                    let n = match obj {
                        HeapObject::List(v) => v.len(),
                        HeapObject::Set(s) => s.len(),
                        HeapObject::Map(m) => m.len(),
                        HeapObject::Graph { adj, .. } => adj.len(),
                        HeapObject::Matrix { rows, .. } => *rows,
                        HeapObject::Record { fields, .. } => fields.len(),
                    };
                    Ok((RuntimeValue::Int(BigInt::from(n)), Some(line)))
                }
                "is_empty" => {
                    let (obj, base) = heap.get(handle)?;
                    let line = cache.access(base, 8);
                    let empty = match obj {
                        HeapObject::List(v) => v.is_empty(),
                        HeapObject::Set(s) => s.is_empty(),
                        HeapObject::Map(m) => m.is_empty(),
                        HeapObject::Graph { adj, .. } => adj.is_empty(),
                        HeapObject::Matrix { data, .. } => data.is_empty(),
                        HeapObject::Record { fields, .. } => fields.is_empty(),
                    };
                    Ok((RuntimeValue::Bool(empty), Some(line)))
                }
                "push" | "append" => {
                    if args.len() != 1 {
                        return Err(trap(TrapCode::ArityMismatch, "push expects 1 argument"));
                    }
                    let val = heap.deep_clone_value(&args[0])?;
                    let line = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::List(v) => {
                            let idx = v.len() as u64;
                            v.push(val);
                            Ok(cache.access(base + idx * 8, 8))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "push() only valid on List")),
                    })?;
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "pop" => {
                    let (popped, line) = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::List(v) => {
                            let val = v.pop();
                            let idx = v.len() as u64;
                            Ok((
                                RuntimeValue::OptionVal(val.map(Box::new)),
                                cache.access(base + idx * 8, 8),
                            ))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "pop() only valid on List")),
                    })?;
                    Ok((popped, Some(line)))
                }
                "swap" => {
                    if args.len() != 2 {
                        return Err(trap(TrapCode::ArityMismatch, "swap(i, j) expects 2 args"));
                    }
                    let i = args[0]
                        .as_usize()
                        .ok_or_else(|| trap(TrapCode::TypeMismatch, "swap index must be Int"))?;
                    let j = args[1]
                        .as_usize()
                        .ok_or_else(|| trap(TrapCode::TypeMismatch, "swap index must be Int"))?;
                    let line = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::List(v) => {
                            if i >= v.len() || j >= v.len() {
                                return Err(trap(
                                    TrapCode::IndexOutOfBounds,
                                    format!("swap({i}, {j}) out of bounds for len {}", v.len()),
                                ));
                            }
                            v.swap(i, j);
                            let _ = cache.access(base + (i as u64) * 8, 8);
                            Ok(cache.access(base + (j as u64) * 8, 8))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "swap() only valid on List")),
                    })?;
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "reverse" => {
                    let line = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::List(v) => {
                            v.reverse();
                            Ok(cache.access(base, v.len().max(1) * 8))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "reverse() only valid on List")),
                    })?;
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "contains" => {
                    if args.len() != 1 {
                        return Err(trap(TrapCode::ArityMismatch, "contains expects 1 argument"));
                    }
                    let (obj, base) = heap.get(handle)?;
                    let line = cache.access(base, 8);
                    let found = match obj {
                        HeapObject::List(v) => v.contains(&args[0]),
                        HeapObject::Set(s) => s.contains(&args[0].to_key()?),
                        HeapObject::Map(m) => m.contains_key(&args[0].to_key()?),
                        HeapObject::Graph { adj, .. } => adj.contains_key(&args[0].to_key()?),
                        _ => return Err(trap(TrapCode::TypeMismatch, "contains() unsupported")),
                    };
                    Ok((RuntimeValue::Bool(found), Some(line)))
                }
                "insert" => {
                    let line = match args.len() {
                        1 => {
                            let key = args[0].to_key()?;
                            heap.with_mut(handle, |obj, base| match obj {
                                HeapObject::Set(s) => {
                                    s.insert(key);
                                    Ok(cache.access(base, 8))
                                }
                                _ => {
                                    Err(trap(TrapCode::TypeMismatch, "1-arg insert() requires Set"))
                                }
                            })?
                        }
                        2 => {
                            let key = args[0].to_key()?;
                            let val = heap.deep_clone_value(&args[1])?;
                            heap.with_mut(handle, |obj, base| match obj {
                                HeapObject::Map(m) => {
                                    m.insert(key, val);
                                    Ok(cache.access(base, 8))
                                }
                                _ => {
                                    Err(trap(TrapCode::TypeMismatch, "2-arg insert() requires Map"))
                                }
                            })?
                        }
                        _ => {
                            return Err(trap(TrapCode::ArityMismatch, "insert expects 1 or 2 args"))
                        }
                    };
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "remove" => {
                    if args.len() != 1 {
                        return Err(trap(TrapCode::ArityMismatch, "remove expects 1 argument"));
                    }
                    let key_or_idx = &args[0];
                    let line = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::Set(s) => {
                            s.shift_remove(&key_or_idx.to_key()?);
                            Ok(cache.access(base, 8))
                        }
                        HeapObject::Map(m) => {
                            m.shift_remove(&key_or_idx.to_key()?);
                            Ok(cache.access(base, 8))
                        }
                        HeapObject::List(v) => {
                            let idx = key_or_idx.as_usize().ok_or_else(|| {
                                trap(TrapCode::TypeMismatch, "List.remove index must be Int")
                            })?;
                            if idx >= v.len() {
                                return Err(trap(TrapCode::IndexOutOfBounds, "List.remove OOB"));
                            }
                            v.remove(idx);
                            Ok(cache.access(base + (idx as u64) * 8, 8))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "remove() unsupported")),
                    })?;
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "get" => {
                    if args.len() != 1 {
                        return Err(trap(TrapCode::ArityMismatch, "get expects 1 argument"));
                    }
                    let (obj, base) = heap.get(handle)?;
                    let line = cache.access(base, 8);
                    match obj {
                        HeapObject::Map(m) => {
                            let k = args[0].to_key()?;
                            let out = m.get(&k).cloned().map(Box::new);
                            Ok((RuntimeValue::OptionVal(out), Some(line)))
                        }
                        HeapObject::List(v) => {
                            let idx = args[0].as_usize().ok_or_else(|| {
                                trap(TrapCode::TypeMismatch, "List.get expects Int")
                            })?;
                            let out = v.get(idx).cloned().map(Box::new);
                            Ok((RuntimeValue::OptionVal(out), Some(line)))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "get() unsupported")),
                    }
                }
                "keys" => {
                    let keys: Vec<RuntimeValue> = {
                        let (obj, _) = heap.get(handle)?;
                        match obj {
                            HeapObject::Map(m) => m.keys().map(RuntimeKey::to_value).collect(),
                            HeapObject::Graph { adj, .. } => {
                                adj.keys().map(RuntimeKey::to_value).collect()
                            }
                            _ => {
                                return Err(trap(
                                    TrapCode::TypeMismatch,
                                    "keys() requires Map/Graph",
                                ))
                            }
                        }
                    };
                    let out = heap.alloc(HeapObject::List(keys))?;
                    Ok((RuntimeValue::Handle(out), None))
                }
                "add_vertex" => {
                    if args.len() != 1 {
                        return Err(trap(TrapCode::ArityMismatch, "add_vertex expects 1 arg"));
                    }
                    let v = args[0].to_key()?;
                    let line = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::Graph { adj, .. } => {
                            adj.entry(v).or_default();
                            Ok(cache.access(base, 8))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "add_vertex requires Graph")),
                    })?;
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "add_edge" => {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(trap(
                            TrapCode::ArityMismatch,
                            "add_edge(u, v [, weight]) expects 2 or 3 args",
                        ));
                    }
                    let u = args[0].to_key()?;
                    let v = args[1].to_key()?;
                    let w = args
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| RuntimeValue::Int(BigInt::from(1)));
                    let line = heap.with_mut(handle, |obj, base| match obj {
                        HeapObject::Graph { directed, adj } => {
                            adj.entry(u.clone())
                                .or_default()
                                .push((v.clone(), w.clone()));
                            adj.entry(v.clone()).or_default();
                            if !*directed {
                                adj.entry(v).or_default().push((u, w));
                            }
                            Ok(cache.access(base, 8))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "add_edge requires Graph")),
                    })?;
                    Ok((RuntimeValue::Unit, Some(line)))
                }
                "neighbors" => {
                    if args.len() != 1 {
                        return Err(trap(TrapCode::ArityMismatch, "neighbors expects 1 arg"));
                    }
                    let u = args[0].to_key()?;
                    let list: Vec<RuntimeValue> = {
                        let (obj, _) = heap.get(handle)?;
                        match obj {
                            HeapObject::Graph { adj, .. } => adj
                                .get(&u)
                                .map(|edges| {
                                    edges
                                        .iter()
                                        .map(|(v, w)| {
                                            RuntimeValue::Tuple(vec![v.to_value(), w.clone()])
                                        })
                                        .collect()
                                })
                                .unwrap_or_default(),
                            _ => {
                                return Err(trap(
                                    TrapCode::TypeMismatch,
                                    "neighbors requires Graph",
                                ))
                            }
                        }
                    };
                    let out = heap.alloc(HeapObject::List(list))?;
                    Ok((RuntimeValue::Handle(out), None))
                }
                "rows" => {
                    let (obj, _) = heap.get(handle)?;
                    match obj {
                        HeapObject::Matrix { rows, .. } => {
                            Ok((RuntimeValue::Int(BigInt::from(*rows)), None))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "rows() requires Matrix")),
                    }
                }
                "cols" => {
                    let (obj, _) = heap.get(handle)?;
                    match obj {
                        HeapObject::Matrix { cols, .. } => {
                            Ok((RuntimeValue::Int(BigInt::from(*cols)), None))
                        }
                        _ => Err(trap(TrapCode::TypeMismatch, "cols() requires Matrix")),
                    }
                }
                "clone" => {
                    let cloned = heap.deep_clone_value(receiver)?;
                    Ok((cloned, None))
                }
                _ => Err(trap(
                    TrapCode::UndefinedSymbol,
                    format!("unknown method '{method}'"),
                )),
            }
        }
        _ => Err(trap(
            TrapCode::UndefinedSymbol,
            format!("method '{method}' unsupported on primitive value"),
        )),
    }
}

/// Compare two `RuntimeValue`s for `<`, `<=`, `>`, `>=`.
pub fn cmp_values(a: &RuntimeValue, b: &RuntimeValue) -> Result<std::cmp::Ordering, TrapError> {
    match (a, b) {
        (RuntimeValue::Int(x), RuntimeValue::Int(y)) => Ok(x.cmp(y)),
        (RuntimeValue::Int64(x), RuntimeValue::Int64(y)) => Ok(x.cmp(y)),
        (RuntimeValue::UInt64(x), RuntimeValue::UInt64(y)) => Ok(x.cmp(y)),
        (RuntimeValue::Real(x), RuntimeValue::Real(y)) => Ok(x.cmp(y)),
        (RuntimeValue::Char(x), RuntimeValue::Char(y)) => Ok(x.cmp(y)),
        (RuntimeValue::Str(x), RuntimeValue::Str(y)) => Ok(x.cmp(y)),
        (RuntimeValue::Bool(x), RuntimeValue::Bool(y)) => Ok(x.cmp(y)),
        _ => {
            if let (Some(bx), Some(by)) = (a.as_bigint(), b.as_bigint()) {
                return Ok(bx.cmp(&by));
            }
            if let (Some(fx), Some(fy)) = (a.as_f64(), b.as_f64()) {
                let cx = CanonicalF64::new(fx)?;
                let cy = CanonicalF64::new(fy)?;
                return Ok(cx.cmp(&cy));
            }
            Err(trap(
                TrapCode::TypeMismatch,
                format!("cannot compare values {a} and {b}"),
            ))
        }
    }
}
