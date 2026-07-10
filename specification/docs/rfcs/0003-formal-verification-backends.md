# RFC 0003: Formal Verification Backends (Lean 4 & TLA+)

- **Status:** Ratified
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

UEAS brands itself as a "mathematical blueprint" and a "verifiable standard." Currently, we enforce semantic correctness via abstract interpretation and trap codes at runtime. However, the ultimate gold standard for algorithms is **compile-time formal verification**—proving that an algorithm is mathematically correct for all possible inputs before it ever executes.

Because UEAS already enforces strict mathematical properties (single-ownership, no aliased references, explicit invariants, and typed bounds), the Canonical AST is perfectly structured to be transpiled into formal logic.

This RFC proposes adding two new formal verification targets (Lean 4 and TLA+) to validate logic and concurrency respectively.

## Proposed Change

1. **Lean 4 Backend (`backends/src/lean.rs`)**
   - Transpiles a UEAS algorithm into a Lean 4 mathematical proof.
   - Converts `Require:` blocks into hypotheses.
   - Converts `Ensure:` blocks into goals.
   - Translates `invariant(condition)` loops into formal induction loops.

2. **TLA+ Backend (`backends/src/tlaplus.rs`)**
   - Transpiles state mutations into TLA+ actions.
   - Translates parallel structures (`spawn`/`parallel for` from RFC 0002) into PlusCal/TLA+ processes to model check for deadlocks and race conditions.

## Impact Analysis

### Grammar changes
None. This RFC exclusively relies on the existing `invariant` and `assert` statements already present in UEAS v3.0.

### AST schema changes
None.

### Kernel changes
None.

### Transpiler changes
- Two new implementations of `TargetGenerator`: `LeanTarget` and `TlaTarget`.
- Both targets differ from existing transpilers (Rust/Python) because they output mathematical models rather than executable scripts.

### Backward compatibility
100% backward compatible. These are simply new output targets for the existing AST.

## Alternatives Considered

1. **Coq / Isabelle/HOL:** While highly respected, Lean 4 has immense momentum in modern mathematics (e.g., the Liquid Tensor Experiment). Lean 4 also functions as a general-purpose programming language, making the mapping of algorithms cleaner.
2. **Built-in SMT Solver in Kernel:** Integrating Z3 into the UEAS Kernel to prove invariants dynamically. *Rejected.* The kernel must remain a lightweight abstract interpreter. Complex formal proofs belong in dedicated external tools (Lean/TLA+).

## Reference Implementation Plan

- **Affected Domains:** `backends/` (new targets), `tools/ueas-cli/` (integration of new targets).
- **Epoch Alignment:** Epoch 8.
- **Dependencies:** Output files will require users to have Lean 4 or TLC model checker installed to actually compile the proofs.
