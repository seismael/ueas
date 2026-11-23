# UEAS v2.1 — Foundational Hardening & Production-Excellence Plan

This plan maps **1:1** to every finding in [AUDIT.md](AUDIT.md) (`PROD-01` through `UX-04`). It is ordered strictly by architectural dependency so each work package compiles, passes all tests, and unlocks the next layer without compromise.

---

## Phase A — Core Engine & Module System (`HIR-01`, `MOD-01`, `PROD-02`)

### WP-A.1: Structured Diagnostics with Exact `SourceSpan` (`HIR-01`)
- **Files:** `crates/ueas-hir/src/lib.rs`, `crates/ueas-hir/src/check/mod.rs`
- **Changes:**
  - Add `pub struct DiagnosticItem { pub code: String, pub message: String, pub span: Option<SourceSpan>, pub hint: Option<String> }`.
  - Expose `compile_to_hir_detailed(source: &str, resolver: &dyn ModuleResolver) -> Result<Program, Vec<DiagnosticItem>>` (keep `compile_to_hir` as a thin wrapper formatting `DiagnosticItem` for backward compatibility).
- **Acceptance:** Unit tests verify exact line/column spans for type mismatch, MVS cycle, and CT leak diagnostics.

### WP-A.2: Embedded `std.*` & Filesystem Module Resolvers (`MOD-01`)
- **Files:** `crates/ueas-hir/src/check/linker.rs`, `crates/ueas-hir/src/stdlib_bundle.rs` (or `crates/ueas-stdlib`), `tools/ueas-cli/src/main.rs`, `tools/ueas-lsp/src/lib.rs`, `tools/ueas-dap/src/lib.rs`, `crates/ueas-wasm/src/lib.rs`
- **Changes:**
  - Implement `EmbeddedStdlibResolver` exposing core `std.*` modules (`std.math`, `std.searching`, `std.sorting`, `std.data_structures`, `std.graph`, `std.dp`, `std.strings`) with zero host I/O (100% `wasm32`-compatible).
  - Implement `ChainedModuleResolver` and (in native tools `ueas-cli`/`ueas-lsp`/`ueas-dap`) `FsModuleResolver` resolving workspace `.ueas` files + `EmbeddedStdlibResolver` fallback.
- **Acceptance:** End-to-end test where a UEAS file executes `import std.math` and calls `GCD(48, 18)` in CLI, VM, and WASM without manual copy-paste.

---

## Phase B — VM Copy-on-Write, Direct Dispatch & `mut` Inout (`VM-01`..`VM-05`, `PROD-02`)

### WP-B.1: Reference-Counted Copy-on-Write `VirtualHeap` (`VM-01`)
- **Files:** `crates/ueas-vm/src/heap.rs`, `crates/ueas-vm/src/vm.rs`
- **Changes:**
  - Add `refcount: u32` to `VirtualHeap` `Slot`.
  - Add `retain(handle)`, `release(handle)` (recursive decrement + immediate free when `refcount == 0`), and `ensure_unique(handle)` (clones `HeapObject` only when `refcount > 1`).
  - Replace eager `deep_clone_value` in `Op::StoreLocal` and call argument binding with $O(1)$ `retain_value`.
  - Call `ensure_unique` before every in-place mutation (`IndexSet`, `FieldSet`, `push`, `pop`, `swap`, `reverse`, `insert`, `remove`, `add_edge`).
  - Remove the $O(\text{stack} + \text{heap})$ `HashSet` mark-sweep scan in `Op::Return` in favor of deterministic `release_value`.
- **Acceptance:** Benchmark test verifying a recursive `BinarySearchRange` over a 1024-element `List` executes in $O(\log N)$ steps and $O(1)$ auxiliary heap bytes; MVS isolation test verifying mutating a copied list never mutates the original.

### WP-B.2: Direct Bytecode Dispatch, Lazy `old_locals`, & Callee Contract Propagation (`VM-02`, `VM-03`, `VM-04`)
- **Files:** `crates/ueas-vm/src/op.rs`, `crates/ueas-vm/src/chunk.rs`, `crates/ueas-vm/src/compile.rs`, `crates/ueas-vm/src/vm.rs`
- **Changes:**
  - Add `BuiltinId` and `MethodId` enums; resolve calls to `Op::CallChunk { chunk_idx, arg_count, mut_mask }`, `Op::CallBuiltin { builtin, arg_count }`, and `Op::CallMethodId { method, arg_count }` at compile time.
  - Track `old_param_mask: u64` on `Chunk`; only retain `old_locals` when `old(...)` is present.
  - Propagate `if self.config.enforce_contracts { 0 } else { target_chunk.body_start_pc }` on callee frames.
- **Acceptance:** Zero string allocations in hot call paths; recursive `Require` helpers do not distort L3 complexity fitting.

### WP-B.3: True MVS `mut` (`inout`) Caller Write-Back (`PROD-02`)
- **Files:** `crates/ueas-vm/src/compile.rs`, `crates/ueas-vm/src/vm.rs`, `crates/ueas-hir/src/check/typecheck.rs`
- **Changes:**
  - For `CallArg { is_mut: true, value: Expr::Ident { name } }`, record caller local slot in `CallFrame.mut_writebacks: Vec<(u16, u16)>` and write back modified `mut` parameter values to the caller's slots on `Op::Return`.
- **Acceptance:** Unit test where `Algorithm SortInPlace(mut data: List<Int>) -> Void` mutates caller's `arr` via `SortInPlace(mut arr)` without returning it.

---

## Phase C — Fail-Closed Verification & Sound Fuzzing/Scaling (`VER-01`..`VER-04`)

### WP-C.1: Fail-Closed Multi-Vector Constant-Time Verifier (`VER-01`)
- **Files:** `crates/ueas-verify/src/constant_time.rs`
- **Changes:**
  - Make `run_trace` errors fail-closed (`passed: false`, `CT_DYNAMIC_EXEC_ERROR`).
  - Test 6 deterministic secret input pairs (`0` vs `1`, `0` vs `-1`, `0x5555` vs `0xAAAA`, `0` vs `i64::MAX`, differing list elements at index `0` and `N-1`).
- **Acceptance:** Unit test confirming a failing/trapping `@ConstantTime` algorithm returns `passed == false` and a secret-dependent branch/index is caught across all pairs.

### WP-C.2: Precondition-Valid L3 Adversarial Scaling (`VER-02`)
- **Files:** `crates/ueas-verify/src/complexity_fit.rs`
- **Changes:**
  - Run a lightweight precondition-validity check on each candidate distribution (`Sorted`, `ReverseSorted`, `AllEqual`, `RandomSeeded`) before measuring body steps & auxiliary bytes.
  - Only aggregate measurements from valid inputs; emit `L3_NO_VALID_INPUTS` if no distribution satisfies `Require`.
- **Acceptance:** `BinarySearch` never benchmarks `ReverseSorted` invalid inputs and accurately fits `O(log N)` Time / `O(1)` Space.

### WP-C.3: Recursive Composite-Type L2 Fuzzer & Shrinker (`VER-03`)
- **Files:** `crates/ueas-verify/src/fuzzer.rs`
- **Changes:**
  - Replace `FuzzArg` with recursive `FuzzValue` supporting `List<T>`, `Set<T>`, `Map<K, V>`, `Tuple<...>`, `Option<T>`, and `Record` (reading field schemas from `Program.records`).
  - Add recursive structural shrinking for nested lists and records.
- **Acceptance:** L2 fuzzing runs with real valid inputs on `MatrixMultiply`, `Dijkstra`, `DisjointSet`, and `TwoSum`.

### WP-C.4: Certificate Verification API & `ueas verify-cert` (`VER-04`)
- **Files:** `crates/ueas-verify/src/certificate.rs`, `tools/ueas-cli/src/main.rs`
- **Changes:**
  - Implement `VerificationCertificate::verify_against(source, resolver)` checking `source_sha256`, `hir_sha256`, and `certificate_sha256`.
  - Add `ueas verify-cert <source.ueas> <cert.json>` CLI command.
- **Acceptance:** CLI test verifying a genuine certificate passes and a 1-byte tampered source or certificate fails with exit code 1.

---

## Phase D — Universal Extensibility, Zero-Silent-Fallback Codegen & L4 Oracle (`EXT-ARCH-01`..`EXT-ARCH-05`, `CG-01`, `PROD-01`, `VM-05`)

### WP-D.1: Shared `HIR -> LIR` Desugaring Pass (`EXT-ARCH-01`)
- **Files:** `crates/ueas-codegen/src/lir.rs`, `crates/ueas-codegen/src/lib.rs`
- **Changes:**
  - Define canonical Lowered Imperative IR (`LirProgram`, `LirFunc`, `LirStmt`, `LirExpr`) and implement `lower_hir_to_lir(&Program) -> LirProgram`.
  - Desugar `forall`/`exists` -> loops, `Ensure` -> return-site assertions, `For` over collections -> indexed loops, and `mut` inout write-backs once in `lir.rs`.
- **Acceptance:** Unit tests verify `lower_hir_to_lir` produces clean, fully desugared `LirProgram` for `BinarySearch`, `MergeSort`, and `TwoSum`.

### WP-D.2: Complete Tier-1 Track A Emission (Rust, Python, TypeScript) & Zero Silent Fallbacks (`CG-01`, `PROD-01`, `VM-05`)
- **Files:** `crates/ueas-codegen/src/track_a.rs`
- **Changes:**
  - Change all emitters to return `Result<String, String>` and eliminate `_ => "0"` and `_ => "true"` fallbacks everywhere.
  - Implement full emission for: `MethodCall` (`push`, `pop`, `swap`, `reverse`, `contains`, `insert`, `remove`, `get`, `keys`, `length`, `is_empty`, `unwrap`, `unwrap_or`), `RecordLit`, `MapLit`, `SetLit`, `TupleLit`, `Slice`, `Cast`, `select` (branchless arithmetic select in Rust/C++), `For` over collections, all `BinaryOp`/`UnaryOp` variants, `mut` inout parameters, and `Ensure` postconditions.
- **Acceptance:** Transpiling `BinarySearch`, `MergeSort`, `InsertionSort`, `GCD`, `TwoSum`, and `DisjointSet` to Rust, Python, and TypeScript succeeds with zero fallback placeholders.

### WP-D.3: Universal Targets — `WasmTarget` (`.wasm` + `.wit`) & `SmtLib2Target` (`EXT-ARCH-02`, `EXT-ARCH-03`)
- **Files:** `crates/ueas-codegen/Cargo.toml`, `crates/ueas-codegen/src/wasm_target.rs`, `crates/ueas-codegen/src/smtlib.rs`, `crates/ueas-codegen/src/lib.rs`
- **Changes:**
  - Add pure-Rust `wasm-encoder = "0.225"` (`Apache-2.0`, `wasm32`-compatible) to `crates/ueas-codegen`.
  - Implement `WasmTarget` emitting valid WebAssembly binary modules (`.wasm` base64/bytes) + WIT interface (`.wit`) for pure numeric/loop algorithms (`GCD`, `FastPower`, `Inc`, etc.), verifiable by WebAssembly binary magic/section validation.
  - Implement `SmtLib2Target` emitting standard SMT-LIB2 verification conditions (`(set-logic ALL)`, `(declare-const ...)`, `(assert ...)`, `(check-sat)`) for Z3/CVC5/Alt-Ergo.
  - Register `TargetKind::Wasm`, `TargetKind::Wit`, and `TargetKind::SmtLib2`.
- **Acceptance:** Unit tests verify valid `\0asm` WASM binary emission + `.wit` interface + valid SMT-LIB2 VC generation.

### WP-D.4: Open External Plugin ABI (`EXT-ARCH-04`)
- **Files:** `tools/ueas-cli/src/main.rs`
- **Changes:**
  - Add `ueas transpile <file> --plugin <cmd>`: serializes canonical HIR + LIR JSON to plugin `stdin`, reads generated output from `stdout`, enforces timeout/non-zero exit checks.
- **Acceptance:** E2E test executing a Python/shell mock-free plugin script (`python -c ...`) receiving canonical HIR JSON on `stdin` and emitting valid target output on `stdout`.

### WP-D.5: Language-Agnostic L4 Differential Oracle (`ueas oracle`) & Expanded Multi-Language Differential Suite (`EXT-ARCH-05`, `CG-01`)
- **Files:** `crates/ueas-verify/src/oracle.rs`, `tools/ueas-cli/src/main.rs`, `tests/differential/src/lib.rs`
- **Changes:**
  - Implement `OracleHarness` in `crates/ueas-verify/src/oracle.rs` (pure JSON protocol: generates `N` precondition-valid fuzzed inputs via `ueas-verify`'s fuzzer, runs `ueas-vm` as reference oracle, compares against external JSON outputs, shrinks divergences).
  - Wire `ueas oracle <spec.ueas> --entry <Algo> --exec "<cmd>" [--samples 64]` in `tools/ueas-cli/src/main.rs`.
  - Expand `tests/differential/src/lib.rs` to verify L4 compiled/executed equivalence across **Rust (`rustc --test`)**, **Python (`python`)**, and **Node (`node`)** on 6 diverse algorithms (`BinarySearch`, `InsertionSort`, `MergeSort`, `GCD`, `TwoSum`, `DisjointSet`) PLUS an end-to-end `ueas oracle` test against an external Python implementation!
- **Acceptance:** `cargo test -p ueas-differential` passes 100% across all 6 algorithms, 3 runtimes, and the language-agnostic `ueas oracle` harness.

---

## Phase E — Extractor Fidelity (`EXT-01`)

### WP-E.1: Complete Tree-sitter Python/TS/Rust Extractors
- **Files:** `crates/ueas-extract/src/{python,typescript,rust_lang}.rs`
- **Changes:**
  - Handle `elif_clause` (Python), `augmented_assignment` (`+=`, `-=`, `*=`), `for_statement`/`for_of_statement` (TS), `for_expression` (Rust), and remove hardcoded `Time: O(N)` claims in favor of explicit `// TODO-COMPLEXITY`? (No — use `@ComplexityUnverified` comment without `TODO` so `verify-clean` stays green).
- **Acceptance:** Unit tests verify round-trip extraction + `ueas_syntax::parse` + `compile_to_hir` on loops, `elif`, and compound assignments in Python, TS, and Rust.

---

## Phase F — Developer & AI-Agent UX Upgrade (`UX-01`..`UX-04`)

### WP-F.1: Unified MCP Tool Catalog with JSON `inputSchema` (`UX-01`)
- **Files:** `tools/ueas-cli/src/main.rs`, `tools/ueas-mcp-edge/src/index.ts`
- **Changes:**
  - Align CLI (`ueas mcp`) to expose all 7 tools (`ueas_check`, `ueas_run`, `ueas_verify`, `ueas_verify_cert`, `ueas_transpile`, `ueas_format`, `ueas_extract`) plus `ueas_oracle_diff` with full JSON Schema `inputSchema` and `isError` handling.
  - Align Edge (`ueas-mcp-edge`) to expose matching `inputSchema` definitions for its 5 WASM tools.
- **Acceptance:** E2E MCP `tools/list` test verifies every tool includes valid `inputSchema.properties` and `required`.

### WP-F.2: Span-Accurate LSP with Completions & Go-to-Definition (`UX-03`)
- **Files:** `tools/ueas-lsp/src/lib.rs`
- **Changes:**
  - Switch `validate_document` to `compile_to_hir_detailed` for exact diagnostic ranges.
  - Add `completion` (keywords, types, `std.*` modules, collection methods) and `goto_definition` (Algorithms & Records).
- **Acceptance:** Unit tests verify exact diagnostic ranges, completion items, and definition locations.

### WP-F.3: Visual UEAS Studio Playground (`UX-02`)
- **Files:** `tools/ueas-playground/{index.html, src/main.ts, src/charts.ts, src/examples.ts, src/style.css}`
- **Changes:**
  - Add curated example selector (BinarySearch, QuickSort, Dijkstra, ParallelSum, ConstantTimeSelect, BadComplexity demo).
  - Add XSS-safe pure-SVG charts (`src/charts.ts` using `document.createElementNS`) for **Empirical Big-O Scaling**, **L1/L2/L3 Cache Hit Rates**, and **Work/Span Parallelism**.
  - Add structured diagnostic list, shareable URL hash (`#code=...`), and one-click `.ueas.cert.json` download.
- **Acceptance:** `npm run typecheck && npm run build` passes with zero `innerHTML` usage.

### WP-F.4: Stateful Multi-Cell Jupyter Kernel with Magics (`UX-04`)
- **Files:** `tools/ueas-jupyter/ueas_jupyter/kernel.py`
- **Changes:**
  - Maintain a cumulative definition buffer across cells so `Algorithm`/`Record` definitions in Cell 1 are callable in Cell 2.
  - Add `%reset`, `%verify [L0|L1|L2|L3]`, and `%transpile <target>` cell magics.
- **Acceptance:** Python unit test + `py_compile` verifies multi-cell state accumulation and `%reset`.

---

## Phase G — End-to-End Verification Gate

Every work package must pass:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p ueas-wasm --target wasm32-unknown-unknown
cargo xtask verify-clean
```
Plus TypeScript `tsc --noEmit` & Vite build (`ueas-mcp-edge`, `ueas-playground`, `ueas-vscode`), Python `py_compile` (`ueas-jupyter`), and live `ueas` CLI + MCP + `verify-cert` E2E checks.
