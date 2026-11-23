//! Runtime values and canonical numeric types for the UEAS deterministic VM.

use crate::trap::{TrapCode, TrapError};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Generational handle to a `HeapObject` in `VirtualHeap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeapHandle {
    pub index: u32,
    pub generation: u32,
}

/// Canonical IEEE-754 finite float (`-0.0` normalized to `+0.0`, `NaN` disallowed).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CanonicalF64(f64);

impl CanonicalF64 {
    /// Construct a canonical float, rejecting `NaN` with `TrapCode::FloatInvalid`.
    pub fn new(v: f64) -> Result<Self, TrapError> {
        if v.is_nan() {
            return Err(TrapError::new(
                TrapCode::FloatInvalid,
                "NaN float value is forbidden in deterministic UEAS",
                0,
                0,
                None,
            ));
        }
        let canon = if v == 0.0 { 0.0 } else { v };
        Ok(Self(canon))
    }

    /// Extract underlying `f64`.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl PartialEq for CanonicalF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for CanonicalF64 {}

impl Hash for CanonicalF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialOrd for CanonicalF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CanonicalF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// Runtime value stored on the operand stack, in locals, and inside heap objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeValue {
    Unit,
    Bool(bool),
    Int(BigInt),
    Int64(i64),
    UInt64(u64),
    Real(CanonicalF64),
    Char(char),
    Str(String),
    OptionVal(Option<Box<RuntimeValue>>),
    ResultVal(Result<Box<RuntimeValue>, Box<RuntimeValue>>),
    Tuple(Vec<RuntimeValue>),
    Handle(HeapHandle),
    FnRef(usize),
}

impl RuntimeValue {
    /// Approximate shallow byte footprint for heap accounting.
    pub fn shallow_bytes(&self) -> usize {
        match self {
            Self::Unit | Self::Bool(_) | Self::Char(_) => 8,
            Self::Int64(_) | Self::UInt64(_) | Self::Real(_) | Self::Handle(_) | Self::FnRef(_) => {
                8
            }
            Self::Int(b) => 24 + (b.bits() as usize).div_ceil(8),
            Self::Str(s) => 24 + s.len(),
            Self::OptionVal(inner) => 16 + inner.as_ref().map_or(0, |v| v.shallow_bytes()),
            Self::ResultVal(res) => match res {
                Ok(v) | Err(v) => 16 + v.shallow_bytes(),
            },
            Self::Tuple(elems) => 24 + elems.iter().map(Self::shallow_bytes).sum::<usize>(),
        }
    }

    /// Convert integer variants (`Int`, `Int64`, `UInt64`) to `BigInt`.
    pub fn as_bigint(&self) -> Option<BigInt> {
        match self {
            Self::Int(b) => Some(b.clone()),
            Self::Int64(i) => Some(BigInt::from(*i)),
            Self::UInt64(u) => Some(BigInt::from(*u)),
            _ => None,
        }
    }

    /// Convert numeric variants to `i64` if in range.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(b) => b.to_i64(),
            Self::Int64(i) => Some(*i),
            Self::UInt64(u) => i64::try_from(*u).ok(),
            _ => None,
        }
    }

    /// Convert numeric variants to `usize` if non-negative and in range.
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Self::Int(b) => b.to_usize(),
            Self::Int64(i) => usize::try_from(*i).ok(),
            Self::UInt64(u) => usize::try_from(*u).ok(),
            _ => None,
        }
    }

    /// Convert numeric variants to `f64`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Real(r) => Some(r.get()),
            Self::Int(b) => b.to_f64(),
            Self::Int64(i) => Some(*i as f64),
            Self::UInt64(u) => Some(*u as f64),
            _ => None,
        }
    }

    /// Convert a value to a hashable `RuntimeKey` for `Set`/`Map`/`Graph`.
    pub fn to_key(&self) -> Result<RuntimeKey, TrapError> {
        match self {
            Self::Unit => Ok(RuntimeKey::Unit),
            Self::Bool(b) => Ok(RuntimeKey::Bool(*b)),
            Self::Int(b) => Ok(RuntimeKey::Int(b.clone())),
            Self::Int64(i) => Ok(RuntimeKey::Int(BigInt::from(*i))),
            Self::UInt64(u) => Ok(RuntimeKey::Int(BigInt::from(*u))),
            Self::Real(r) => Ok(RuntimeKey::Real(*r)),
            Self::Char(c) => Ok(RuntimeKey::Char(*c)),
            Self::Str(s) => Ok(RuntimeKey::Str(s.clone())),
            Self::Tuple(items) => {
                let mut keys = Vec::with_capacity(items.len());
                for item in items {
                    keys.push(item.to_key()?);
                }
                Ok(RuntimeKey::Tuple(keys))
            }
            _ => Err(TrapError::new(
                TrapCode::TypeMismatch,
                "value is not hashable as a Map/Set/Graph key",
                0,
                0,
                None,
            )),
        }
    }
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Int64(i) => write!(f, "{i}"),
            Self::UInt64(u) => write!(f, "{u}"),
            Self::Real(r) => write!(f, "{}", r.get()),
            Self::Char(c) => write!(f, "'{c}'"),
            Self::Str(s) => write!(f, "{s}"),
            Self::OptionVal(None) => write!(f, "None"),
            Self::OptionVal(Some(v)) => write!(f, "Some({v})"),
            Self::ResultVal(Ok(v)) => write!(f, "Ok({v})"),
            Self::ResultVal(Err(e)) => write!(f, "Err({e})"),
            Self::Tuple(items) => {
                let parts: Vec<String> = items.iter().map(ToString::to_string).collect();
                write!(f, "({})", parts.join(", "))
            }
            Self::Handle(h) => write!(f, "<handle:{}:{}>", h.index, h.generation),
            Self::FnRef(idx) => write!(f, "<fn:{idx}>"),
        }
    }
}

/// Hashable key representation for `Set`, `Map`, and `Graph` vertices.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RuntimeKey {
    Unit,
    Bool(bool),
    Int(BigInt),
    Real(CanonicalF64),
    Char(char),
    Str(String),
    Tuple(Vec<RuntimeKey>),
}

impl RuntimeKey {
    /// Convert back to a `RuntimeValue`.
    pub fn to_value(&self) -> RuntimeValue {
        match self {
            Self::Unit => RuntimeValue::Unit,
            Self::Bool(b) => RuntimeValue::Bool(*b),
            Self::Int(i) => RuntimeValue::Int(i.clone()),
            Self::Real(r) => RuntimeValue::Real(*r),
            Self::Char(c) => RuntimeValue::Char(*c),
            Self::Str(s) => RuntimeValue::Str(s.clone()),
            Self::Tuple(items) => RuntimeValue::Tuple(items.iter().map(Self::to_value).collect()),
        }
    }
}
