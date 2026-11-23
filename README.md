# UEAS v2.0 — Unified Executable Algorithmic Specification

UEAS is a **pure, deterministic, cost-metered algorithmic specification and dual-track synthesis language**. Write an algorithm once with formal contracts (`Require` / `Ensure` / `invariant` / `decreases`), asymptotic complexity bounds (`Complexity`), and `@Test` suites — verify it end-to-end across 5 progressive levels (`L0`–`L4`), and synthesize idiomatic production code (`Rust`, `Python`, `TypeScript`, `C++20`, `Go`), formal proofs (`Dafny 4`, `Lean 4`, `Verus`, `Quint`, `TLA+`), or academic pseudocode (`LaTeX`, `Typst`).

## Architecture

- **`spec/`**: Authoritative language specification ([SPEC.md](spec/SPEC.md)), JSON Schemas (`spec/schemas/`), Tree-sitter grammar (`spec/tree-sitter-ueas/`), ADRs, and RFCs.
- **`crates/`**: Pure, `#![forbid(unsafe_code)]`, `wasm32`-compatible Rust engine:
  - `ueas-syntax`: Lossless Rowan CST + Logos lexer + 13-level Pratt parser + formatter
  - `ueas-hir`: Typed canonical HIR, module linker, bidirectional type checker, Mutable Value Semantics (MVS) checker, Constant-Time (`Public ⊑ Secret`) lattice
  - `ueas-vm`: Cost-metered bytecode VM, generational `VirtualHeap`, $K$-way LRU L1/L2/L3 cache simulator, fork-join Work/Span DAG profiler, snapshot `stepBack`
  - `ueas-verify`: Progressive `L0`–`L3` verifier, precondition-guided property fuzzer + shrinker, adversarial multi-scale BIC Big-O curve fitter, SHA-256 `.ueas.cert.json` generator
  - `ueas-codegen`: Dual-track Track A/B/C code generators
  - `ueas-extract`: Tree-sitter Python/TypeScript/Rust extractors + Axiom 1 purity auditor
  - `ueas-wasm`: `wasm-bindgen` bindings for Browser Web Worker & Edge MCP
- **`tools/`**: `ueas-cli`, `ueas-lsp`, `ueas-dap`, `ueas-vscode`, `ueas-mcp-edge`, `ueas-playground`, `ueas-cloud-verify`, `ueas-jupyter`.
- **`stdlib/` & `examples/`**: 84 verified `.ueas` modules ([stdlib/INDEX.md](stdlib/INDEX.md)).
- **`tests/`**: Universal Conformance Test Suite (`tests/ucts/`) and L4 compiled differential equivalence suite (`tests/differential/`).

## Quickstart

```bash
cargo test --workspace
cargo run --bin ueas -- verify stdlib/searching/exponential_search.ueas
cargo run --bin ueas -- transpile examples/core/binary_search.ueas --target rust
```
