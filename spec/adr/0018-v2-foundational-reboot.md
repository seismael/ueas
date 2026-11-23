# ADR 0018: UEAS v2.0 Foundational Architectural Reboot

- **Status:** Accepted
- **Date:** 2026-11-23
- **Deciders:** UEAS Core Maintainers
- **Supersedes:** ADR 0001 (partial implicit-var hacks), ADR 0004/Phase 8 (Dafny-only codegen), ADR 0009, ADR 0010, ADR 0011, ADR 0015 (subprocess Jupyter hack), ADR 0017 (multi-lockfile split), and `AGENTS.md` Section "No-Browser-WASM"
- **Superseded By:** None

## Context

A comprehensive end-to-end audit ([AUDIT.md](../../AUDIT.md)) revealed that UEAS v1/v5 had accumulated severe structural contradictions between its specification and implementation:

1. Four divergent, broken hand-rolled parsers replaced an uncompilable `UEAS.g4` grammar.
2. The tree-walking interpreter lacked real collections, user function calls, `return` unwinding, and accurate Big-O verification.
3. Deleting direct idiomatic transpilers in favor of `dafny build` introduced a heavy `.NET 8 + Z3` dependency and emitted alien, runtime-wrapped code.
4. Multiple independent `Cargo.lock` files fragmented the repository and blinded CI.
5. Forbidding client-side browser WASM forced the static playground onto a 10ms-CPU-capped edge worker, which resorted to JavaScript regex heuristics.
6. Java-era GoF Singleton mandates conflicted with Rust's ownership model, parallel testing, and isolated VM sessions.

## Decision

Execute a clean-slate, zero-compromise architectural reboot to **UEAS v2.0.0** ([PLAN.md](../../PLAN.md)):

1. **Unified Root Cargo Workspace:** One `Cargo.toml` and one `Cargo.lock` for all `crates/*`, `tools/*`, and `tests/*`, while maintaining strict physical separation between `spec/` (zero Rust code), `crates/` (pure zero-I/O Rust engine), and `tools/` (applications).
2. **Modern Compiler Frontend (`ueas-syntax` + `ueas-hir` + `tree-sitter-ueas`):** Lossless Rowan CST + 13-level Pratt parser + bidirectional Hindley-Milner type inference + Mutable Value Semantics (`mut`) + Arena `Record` types + Static `@ConstantTime` security lattice.
3. **Deterministic Cost-Metered Bytecode VM (`ueas-vm`):** Flat bytecode VM with generational `VirtualHeap`, real collections, $K$-way set-associative LRU cache simulator, and true fork-join Work/Span DAG.
4. **5-Level Progressive Verification (`ueas-verify`):** Static soundness (L0), property fuzzing with shrinking (L1), adversarial multi-scale BIC-penalized Big-O Time/Space curve fitting & observational constant-time trace equality (L2), formal Dafny/Lean4/Verus/Quint proofs (L3), and compiled L4 differential equivalence (`tests/differential/`).
5. **Dual-Track + Academic Codegen (`ueas-codegen`):** Direct idiomatic zero-dependency Rust, Python, TypeScript, C++20, and Go (Track A); valid Dafny 4.6, Lean 4, Verus, and Quint/TLA+ (Track B); LaTeX, Typst, and ACM Artifact Evaluation bundles (Track C).
6. **Client-Side Browser Web-Worker WASM + Real WASM Edge MCP:** Both powered by the single `crates/ueas-wasm` build with zero regex heuristics.
7. **Idiomatic Rust & Curated Dependencies:** `#![forbid(unsafe_code)]`, zero global Singletons, zero `unwrap()`/`panic!()`, and `cargo-deny` license auditing.

## Consequences

- All legacy `implementation/` crates, duplicate parsers, theatrical stubs (Quantum/Distributed/Stream/Tensor no-ops), and `CLA.md` (replaced by DCO) are removed with zero deprecated shims.
- Every claim in `README.md` and `spec/SPEC.md` is continuously machine-verified in CI.
