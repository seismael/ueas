//! UEAS Language Frontends — reverse transpilation from target languages
//! into the canonical UEAS JSON AST.
//!
//! This domain implements "Reverse Transpilation" (ADR 0008). Each module
//! parses source code from a target language and maps it *inward* to the
//! UEAS Canonical JSON AST, enabling complexity profiling and formal
//! verification for existing codebases.
//!
//! # Axiom Enforcement
//! All frontends MUST reject code that violates UEAS Axiom 1 (Zero System
//! I/O) — calls to print, open, file access, or network operations return
//! a hard error. This ensures extracted algorithms are pure, deterministic
//! state mutations suitable for the abstract interpreter.

pub mod python;
