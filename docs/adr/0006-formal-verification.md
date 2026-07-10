# ADR 0006: Formal Verification Target Generation

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

UEAS brands itself as an executable mathematical blueprint for algorithms. The standard includes explicit `invariant()` loops and `assert()` statements. While the abstract interpreter verifies these dynamically at runtime, true algorithmic confidence comes from **compile-time formal verification** (mathematical proof of correctness). Epoch 8 aims to introduce formal verification capabilities without rewriting the core AST.

## Decision

We will implement two new `TargetGenerator` backends that output formal logic rather than executable machine code:
1. **Lean 4 Target:** Translates UEAS Canonical AST into Lean 4 theorem files. UEAS `Require` preambles become hypotheses, `Ensure` blocks become goals, and algorithm bodies are mapped to Lean's functional syntax for theorem proving.
2. **TLA+ Target:** Specifically designed for Epoch 7's concurrent constructs. Translates UEAS state mutations and `spawn` events into PlusCal/TLA+ processes to allow push-button model checking for deadlocks and race conditions.

## Consequences

**Positive:**
- Dramatically elevates UEAS from a "pseudocode transpiler" to an "academic proof standard."
- Does not require any changes to the existing `grammar/` or `kernel/`. It purely extends the `backends/` domain.

**Negative:**
- Target Generators for formal logic are significantly harder to write than those for imperative languages (e.g., Python/Java) because they must map mutable state (UEAS) into purely functional/relational models.
- Users must install the Lean 4 toolchain or TLC model checker to actually run the generated proofs.

## Alternatives Considered

1. **Z3 / SMT Solver in the Kernel:** We considered embedding an SMT solver directly into the UEAS Rust kernel to automatically verify `invariant` statements during parsing. Rejected because it violates the "lightweight interpreter" constraint and complicates the kernel. Formal proofs belong in the backend target domain.
2. **Coq instead of Lean 4:** Rejected. Lean 4 has significant momentum in modern mathematics and functions simultaneously as a programming language and a theorem prover, making the AST translation map cleaner than Coq.
