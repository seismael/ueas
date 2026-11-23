//! Deterministic bytecode instruction set for the UEAS VM.

use serde::{Deserialize, Serialize};

/// Bytecode operation executed by the UEAS stack VM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    /// Push constant at `constants[idx]`.
    LoadConst(u16),
    /// Push local variable `locals[slot]`.
    LoadLocal(u16),
    /// Pop top of stack and store into `locals[slot]` (deep-cloned for MVS if handle).
    StoreLocal(u16),
    /// Move top of stack into `locals[slot]` without extra copy.
    InitLocal(u16),
    /// Push `old(locals[slot])` snapshot captured at algorithm entry.
    LoadOldLocal(u16),
    /// Pop and discard top of stack.
    Pop,
    /// Duplicate top of stack.
    Dup,

    // Arithmetic & Bitwise
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Neg,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,

    // Comparisons
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Control Flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),

    // Calls & Builtins
    CallNamed {
        name_idx: u16,
        arg_count: u8,
        writeback_id: Option<u16>,
    },
    CallMethod {
        method_idx: u16,
        arg_count: u8,
    },
    Return,

    // Heap & Collections
    MakeList(u16),
    MakeSet(u16),
    MakeMap(u16),
    MakeTuple(u16),
    MakeRecord {
        type_idx: u16,
        field_count: u16,
    },
    IndexGet,
    IndexSet,
    FieldGet(u16),
    FieldSet(u16),
    DropLocal(u16),

    // Algebraic wrappers
    MakeSome,
    MakeNone,
    MakeOk,
    MakeErr,
    IsSome,
    IsNone,
    IsOk,
    IsErr,
    Unwrap,
    TupleGet(u16),

    // Contracts & Verification
    CheckRequire(u16),
    CheckEnsure(u16),
    CheckInvariant(u16),
    CheckDecreases(u16),
    Assert(u16),

    // Constant-time branchless select(cond, a, b)
    SelectBranchless,

    // Parallelism (Work/Span DAG)
    ForkScope,
    BeginBranch,
    EndBranch,
    SyncScope,

    /// No-op placeholder.
    Nop,
}
