//! UEAS v2.0 Type System (`Int`, `Int64`, `UInt64`, `Real`, `Bool`, `Option`, `List`, `Record`, `Secret`, etc.).

use serde::{Deserialize, Serialize};

/// Security sensitivity level for `@ConstantTime` information-flow lattice (`Public ⊑ Secret`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum SecurityLevel {
    #[default]
    Public,
    Secret,
}

/// Primitive UEAS types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    Int,
    Int64,
    UInt64,
    UInt32,
    Byte,
    Real,
    Bool,
    String,
    Void,
}

/// Full UEAS Type representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Type {
    Primitive {
        prim: PrimitiveType,
    },
    List {
        elem: Box<Type>,
    },
    Set {
        elem: Box<Type>,
    },
    Map {
        key: Box<Type>,
        val: Box<Type>,
    },
    Graph {
        node: Box<Type>,
        weight: Box<Type>,
        directed: bool,
    },
    Matrix {
        elem: Box<Type>,
    },
    Option {
        inner: Box<Type>,
    },
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    Tuple {
        elems: Vec<Type>,
    },
    Secret {
        inner: Box<Type>,
    },
    Record {
        name: String,
        type_args: Vec<Type>,
    },
    TypeVar {
        name: String,
    },
    /// Internal unknown type resolved during bidirectional inference.
    Infer,
}

impl Type {
    pub fn int() -> Self {
        Self::Primitive {
            prim: PrimitiveType::Int,
        }
    }
    pub fn int64() -> Self {
        Self::Primitive {
            prim: PrimitiveType::Int64,
        }
    }
    pub fn real() -> Self {
        Self::Primitive {
            prim: PrimitiveType::Real,
        }
    }
    pub fn bool() -> Self {
        Self::Primitive {
            prim: PrimitiveType::Bool,
        }
    }
    pub fn string() -> Self {
        Self::Primitive {
            prim: PrimitiveType::String,
        }
    }
    pub fn void() -> Self {
        Self::Primitive {
            prim: PrimitiveType::Void,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self.peel_secret(),
            Self::Primitive {
                prim: PrimitiveType::Int
                    | PrimitiveType::Int64
                    | PrimitiveType::UInt64
                    | PrimitiveType::UInt32
                    | PrimitiveType::Byte
            }
        )
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer()
            || matches!(
                self.peel_secret(),
                Self::Primitive {
                    prim: PrimitiveType::Real
                }
            )
    }

    pub fn is_secret(&self) -> bool {
        matches!(self, Self::Secret { .. })
    }

    pub fn peel_secret(&self) -> &Self {
        match self {
            Self::Secret { inner } => inner.peel_secret(),
            other => other,
        }
    }

    /// Check whether casting from `self` to `target` is permitted per `SPEC.md`.
    pub fn can_cast_to(&self, target: &Self) -> bool {
        let s = self.peel_secret();
        let t = target.peel_secret();
        if s == t {
            return true;
        }
        matches!(
            (s, t),
            (
                Self::Primitive {
                    prim: PrimitiveType::Int
                        | PrimitiveType::Int64
                        | PrimitiveType::UInt64
                        | PrimitiveType::UInt32
                        | PrimitiveType::Byte
                },
                Self::Primitive {
                    prim: PrimitiveType::Int
                        | PrimitiveType::Int64
                        | PrimitiveType::UInt64
                        | PrimitiveType::UInt32
                        | PrimitiveType::Byte
                        | PrimitiveType::Real
                }
            ) | (
                Self::Primitive {
                    prim: PrimitiveType::Real
                },
                Self::Primitive {
                    prim: PrimitiveType::Int | PrimitiveType::Int64
                }
            )
        )
    }
}
