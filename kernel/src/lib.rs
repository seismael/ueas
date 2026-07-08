//! UEAS Abstract Interpreter — kernel for AST evaluation, invariant
//! enforcement, and deterministic complexity profiling.
//!
//! The kernel operates on the canonical JSON AST defined in SPEC.md
//! Section 5. It maintains a zero-I/O virtual heap, step counter, and
//! trap register. All error conditions route through the trap system.

pub mod ast;
