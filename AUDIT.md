# UEAS v2.0 -> v2.1 Specialist Architecture, Product & Engineering Audit

**Audit Date:** 2026-11-23  
**Scope:** End-to-end specialist critique of the UEAS v2.0 concept, architecture, VM performance, verification soundness, codegen fidelity, module system, developer/AI-agent UX, and reliability.  
**Standard:** Zero compromise — every finding is backed by direct codebase evidence and paired with a superior architectural solution.

---

## Executive Verdict

UEAS v2.0 successfully established a clean, `#![forbid(unsafe_code)]`, unified Rust workspace with a lossless CST parser, typed HIR, cost-metered VM, progressive verifier, and 84-module corpus. However, **a ruthless specialist audit reveals 18 major architectural, semantic, performance, security, and UX flaws** that currently prevent UEAS v2.0 from being adopted in serious production, research, or AI-agent engineering workflows:

1. **Silent Codegen Miscompilation:** Track A (`Rust`, `Python`, `TypeScript`, `C++`, `Go`) silently emits `"0"` or `"true"` for unsupported HIR nodes (`MethodCall`, `RecordLit`, `MapLit`, `SetLit`, `TupleLit`, `Slice`, `Select`, non-range `For`, and non-Rust/Python quantifiers) and omits `Ensure` postconditions.
2. **Fail-Open Security Verifier:** Dynamic `@ConstantTime` verification silently returns `passed: true` whenever trace execution errors.
3. **O(N) Deep-Clone Blowup & Broken `mut` Semantics in `ueas-vm`:** Every function call and assignment deep-clones heap collections (destroying `O(log N)` bounds on recursive helpers), `mut` parameters do not write back to callers, and `Op::CallNamed` performs string lookups and re-enables contracts on recursive calls during L3 benchmarking.
4. **Disconnected Module System:** `NullModuleResolver` is hardcoded across CLI, LSP, DAP, WASM, Verify, and UCTS — `import std.*` and multi-file projects do not actually resolve.
5. **Unsound L3 Scaling & Shallow L2 Fuzzing:** L3 complexity fitting runs adversarial unsorted inputs on `Require`-guarded algorithms by jumping past `Require`, and L2 fuzzing cannot generate `Record`, `List<List<T>>`, `Map`, `Set`, or `Tuple` inputs.
6. **Fragmented Developer & AI-Agent UX:** LSP maps all HIR errors to line `0:0` (losing `SourceSpan`), MCP tools lack mandatory JSON `inputSchema` and drift between CLI and Edge, the Playground is a bare `<textarea>` with no visual complexity/cache charts or `stdlib` browser, and the Jupyter kernel is stateless across cells.

---

## Domain 1: Concept, Product Positioning & Adoption Friction

### [PROD-01] Risk of "Broad-but-Shallow" Polyglot Transpilation Undermining Trust
- **Evidence:** `crates/ueas-codegen/src/track_a.rs` targets 5 production languages (`Rust`, `Python`, `TypeScript`, `C++`, `Go`) and `track_b.rs` targets 5 formal provers, yet `track_a.rs:780` falls back to `_ => "0".to_string()`.
- **Critique:** Engineers and AI agents will immediately abandon a verification/synthesis tool if it ever silently emits wrong code (`0`) instead of either generating exact semantics or failing loudly with a typed diagnostic. Breadth without fidelity destroys credibility.
- **Better Solution:**
  1. Enforce a **Zero-Silent-Fallback Invariant** across all codegen backends: every `CodeGenerator` returns `Result<GeneratedArtifact, CodegenError>` and NEVER silently emits `"0"` or `"true"`.
  2. Establish **Tier-1 Certified Targets** (`Rust`, `Python`, `TypeScript`) with 100% HIR feature parity (Records, Lists, Maps, Sets, Tuples, Slices, Methods, `Option`/`Result`, `select`, `mut` inout, `Require` + `Ensure`) verified by an expanded L4 Differential Equivalence Suite across 10+ diverse algorithms.
  3. Keep `C++20`, `Go`, and Track B (`Dafny 4`, `Lean 4`, `Verus`, `Quint`, `TLA+`) honest: emit full supported subsets and return explicit `CodegenError::UnsupportedFeature { target, feature, span }` for any unmapped construct.

### [PROD-02] Ergonomic Friction of Manual Return-Tuple `mut` Workarounds
- **Evidence:** In `stdlib/sorting/*.ueas` and `stdlib/data_structures/binary_search_tree.ueas`, algorithms declared `mut data: List<Int>` must awkwardly return `List<Int>` or `Tuple<List<BstNode>, Int>` because `mut` parameters do not propagate mutations back to the caller.
- **Critique:** UEAS advertises Mutable Value Semantics (MVS, like Swift `inout` / Val / Hylo), where `Sort(mut arr)` mutates `arr` in place with exclusive ownership and zero aliasing. Without caller write-back, UEAS forces boilerplate tuple plumbing that hurts readability for both humans and LLMs.
- **Better Solution:** Implement true **MVS `inout` (`mut`) call-by-value-result** semantics end-to-end across HIR (`CallArg.is_mut` validation against `Param.is_mut`), VM (write-back of `mut` slots to caller LValues on `Op::Return`), and Track A codegen (`&mut` in Rust, return-tuple/in-place mutation in Python/TS).

---

## Domain 2: VM Architecture, Semantics & Performance (`crates/ueas-vm`)

### [VM-01] Eager Deep-Clone on Every Call & Store Causes $O(N)$ Asymptotic Distortion
- **Evidence:** `crates/ueas-vm/src/vm.rs:576` (`self.heap.deep_clone_value(&arg)`) clones every argument on every `Op::CallNamed`, and `vm.rs:413` (`Op::StoreLocal`) deep-clones on every variable store.
- **Critique:** Calling a recursive helper `BinarySearchRange(data, target, low, high)` copies the entire `List` of size $N$ at every recursion level — turning an $O(\log N)$ time / $O(1)$ space algorithm into $O(N \log N)$ time and $O(N \log N)$ space in the VM! It also floods the L1/L2/L3 cache simulator with artificial copy traffic.
- **Better Solution:** Upgrade `VirtualHeap` to **Reference-Counted Copy-on-Write (CoW)**:
  - Each `Slot` tracks `refcount: u32`.
  - Passing a `Handle` by value or binding `let y <- x` increments `refcount` in $O(1)$ time and $0$ extra heap bytes.
  - `heap.ensure_unique(handle)` clones the `HeapObject` **only on mutation** when `refcount > 1` (true MVS CoW).
  - Dropping a local decrements `refcount` in $O(1)$ and frees immediately when `refcount == 0`, eliminating the $O(\text{stack} + \text{heap})$ `HashSet` mark-sweep scan on every `Op::Return` (`vm.rs:627-646`).

### [VM-02] Unnecessary `old_locals` Deep-Cloning on Every Call
- **Evidence:** `crates/ueas-vm/src/vm.rs:135-141` and `vm.rs:574-579` unconditionally deep-clone all arguments into `old_locals` on every function call, even when the algorithm has zero `old(...)` expressions.
- **Critique:** Doubles memory pressure and call overhead for every algorithm in the language.
- **Better Solution:** Have `compile.rs` compute `uses_old: bool` (or a bitmask `old_param_mask: u64`) per `Chunk`. Only retain CoW handles in `old_locals` for slots actually referenced by `Op::LoadOldLocal`.

### [VM-03] Dynamic String Lookup & Allocation on Every Function/Method Call
- **Evidence:** `crates/ueas-vm/src/vm.rs:514-560` clones `String` from `chunk.strings` and performs `HashMap<String, usize>` lookup on every `Op::CallNamed` and string `match` on every `Op::CallMethod`.
- **Critique:** In tight recursive or inner-loop algorithms, string cloning + hashing dominates VM step wall-clock time (especially in `wasm32` browser/edge execution).
- **Better Solution:** Resolve call targets at compile time in `compile.rs` into dedicated zero-allocation opcodes:
  - `Op::CallChunk { chunk_idx: u16, arg_count: u8, mut_mask: u16 }`
  - `Op::CallBuiltin { builtin: BuiltinId, arg_count: u8 }`
  - `Op::CallMethod { method: MethodId, arg_count: u8 }` (typed enum dispatch, zero heap allocations per call).

### [VM-04] `Op::CallNamed` Ignores `enforce_contracts: false` on Callee Frames
- **Evidence:** `crates/ueas-vm/src/vm.rs:582` hardcodes `pc: 0` when pushing a callee `CallFrame`, whereas `VmSession::new` (`vm.rs:145`) sets `pc: if config.enforce_contracts { 0 } else { chunk.body_start_pc }`.
- **Critique:** During L3 complexity fitting (`enforce_contracts: false`), any algorithm that calls a helper with `Require: forall ...` still pays the $O(N)$ precondition cost on every recursive call.
- **Better Solution:** Use `pc: if self.config.enforce_contracts { 0 } else { target_chunk.body_start_pc }` uniformly in `Op::CallChunk` as well.

### [VM-05] `Op::SelectBranchless` Uses Conditional Branch & Lacks Type-Level CT Guarantee
- **Evidence:** `crates/ueas-vm/src/vm.rs:886` implements `Op::SelectBranchless` as `if cond { a } else { b }` without validating that both branches have equal representation cost or prohibiting heap allocations inside secret-tainted select expressions.
- **Critique:** While the VM trace recorder omits `branch_taken` for `SelectBranchless`, `if cond` in Track A codegen (`Rust`/`C++`) is not lowered to a constant-time arithmetic/masking select, creating a semantic gap between VM CT verification and compiled output.
- **Better Solution:** Lower `select(cond, a, b)` in Track A Rust/C++ to explicit branchless arithmetic/bitwise selection for integer/bool types (e.g., `let mask = -(cond as i64); (a & mask) | (b & !mask)`), and validate in `ueas-hir` that `select` inside `@ConstantTime` operates on fixed-size scalar types.

---

## Domain 3: Verification Soundness & Security (`crates/ueas-verify`)

### [VER-01] CRITICAL SECURITY FLAW: Fail-Open Dynamic Constant-Time Verifier
- **Evidence:** `crates/ueas-verify/src/constant_time.rs:40-50`:
  ```rust
  let (Ok(trace1), Ok(trace2)) = (t1, t2) else {
      return (ConstantTimeReport { passed: true, ... }, Vec::new());
  };
  ```
- **Critique:** If trace execution fails for *any* reason (runtime trap, unsupported parameter type, step budget), `verify_constant_time` silently reports `passed: true` and seals a passing SHA-256 certificate! In security engineering, a fail-open verifier is a critical vulnerability. Furthermore, testing only 1 fixed pair (`0x00AA_5511` vs `0x7FFF_0001`) misses edge-case secret values (`0`, `-1`, `1`, all-ones, differing lengths/indices).
- **Better Solution:**
  1. **Fail-Closed:** Any execution error in `run_trace` MUST return `passed: false` with `RepairDiagnostic { code: "CT_DYNAMIC_EXEC_ERROR", ... }`.
  2. **Multi-Vector Secret Differential Suite:** Run at least 6 deterministic secret input pairs (`0` vs `1`, `0` vs `-1`, `0x5555` vs `0xAAAA`, `0` vs `i64::MAX`, differing list elements at index 0 and index `N-1`) and require 100% identical `(pc, cache_line, branch_taken)` traces across all pairs.

### [VER-02] L3 Adversarial Complexity Fitter Benchmarks Invalid Inputs Violating `Require`
- **Evidence:** `crates/ueas-verify/src/complexity_fit.rs:149-195` runs `Sorted`, `ReverseSorted`, and `AllEqual` with `enforce_contracts: false` (skipping `Require`).
- **Critique:** For `BinarySearch` (which requires a sorted list), `ReverseSorted` violates `Require` yet is still executed and included in `worst_steps`! For algorithms whose termination depends on `Require` (e.g., `a >= 0`, `m > 1`, sorted input), benchmarking invalid inputs causes false traps or bogus complexity curves.
- **Better Solution:** Two-phase scale harness:
  1. Generate candidate inputs from rich distributions (`Sorted`, `ReverseSorted`, `AllEqual`, `RandomSeeded`, `WorstCaseSearchMiss`).
  2. Validate each candidate against `Require` (or run a precondition-only check); discard invalid distributions for that algorithm and measure body steps + auxiliary heap bytes ONLY on valid inputs. If 0 valid inputs are generated at a scale, emit `L3_NO_VALID_INPUTS` instead of silently substituting `worst_steps = 1`.

### [VER-03] L2 Property Fuzzer Cannot Generate Composite Types
- **Evidence:** `crates/ueas-verify/src/fuzzer.rs:118-155` only generates `Int`, `Int64`, `UInt64`, `Bool`, `Real`, `String`, and `List<Int>`. All other types (`List<List<Int>>`, `List<String>`, `Map<K, V>`, `Set<T>`, `Tuple<...>`, `Record`, `Option<T>`) fall through to `_ => FuzzArg::Unit`.
- **Critique:** Over 30% of `stdlib/` algorithms (graph algorithms, matrix algorithms, arena-based trees/DSU, string/map algorithms) cannot be meaningfully fuzzed at L2.
- **Better Solution:** Replace flat `FuzzArg` with recursive `FuzzValue` (`Int`, `Real`, `Bool`, `Str`, `List(Vec<FuzzValue>)`, `Set(Vec<FuzzValue>)`, `Map(Vec<(FuzzValue, FuzzValue)>)`, `Tuple(Vec<FuzzValue>)`, `Record { name, fields }`, `Option(Option<Box<FuzzValue>>)`) driven by `Type` + `Program.records`, with recursive structural shrinking!

### [VER-04] Missing Certificate Verification CLI/API (`verify-cert`)
- **Evidence:** `crates/ueas-verify/src/certificate.rs` generates `.ueas.cert.json` with `source_sha256`, `hir_sha256`, and `certificate_sha256`, but there is no `verify_certificate(source, cert_json)` API or `ueas verify-cert` command to audit a certificate in CI/CD.
- **Better Solution:** Add `VerificationCertificate::verify_against_source(source: &str, resolver: &dyn ModuleResolver) -> Result<(), Vec<String>>` and expose `ueas verify-cert <file.ueas> <file.ueas.cert.json>`.

---

## Domain 4: Codegen Fidelity & L4 Equivalence (`crates/ueas-codegen`, `tests/differential`)

### [CG-01] Track A Missing Core Language Constructs & Silent `"0"` Fallbacks
- **Evidence:** `crates/ueas-codegen/src/track_a.rs:780` (`_ => "0".to_string()`), `track_a.rs:774` (`_ => "true"`), `track_a.rs:582` (only `Expr::Range` handled in `Stmt::For`), and `emit_algorithm` never emits `Ensure` postconditions.
- **Critique:** Only trivial integer/list-index algorithms like `BinarySearch` compile cleanly in Track A. Any algorithm using `.push()`, `.pop()`, `.swap()`, `.contains()`, `.insert()`, `RecordLit`, `MapLit`, `SetLit`, `TupleLit`, `Slice`, `select()`, `Cast`, `UnaryOp::BitNot`, or `Ensure` fails or miscompiles silently.
- **Better Solution:**
  1. Remove all silent `_ => "0"` / `_ => "true"` fallbacks; return `Result<String, String>` from all emitter functions.
  2. Implement complete HIR emission for `Rust`, `Python`, and `TypeScript`: `MethodCall` (`push`, `pop`, `swap`, `reverse`, `contains`, `insert`, `remove`, `get`, `keys`, `length`, `is_empty`, `unwrap`, `unwrap_or`), `RecordLit`, `MapLit`, `SetLit`, `TupleLit`, `Slice`, `Cast`, `select`, `For` over collections, `BinaryOp` (all bitwise/logical/in/not-in), and `Ensure` postcondition checks on every `Return`!
  3. Expand `tests/differential` to execute L4 compiled differential equivalence across **Rust (`rustc`), Python (`python`), and TypeScript/Node (`node`)** on **at least 6 diverse algorithms** (BinarySearch, MergeSort, InsertionSort, GCD, TwoSum/Map, DisjointSet/Record).

---

## Domain 5: Module System, Type Checker & Extractors (`crates/ueas-hir`, `crates/ueas-extract`)

### [MOD-01] `NullModuleResolver` Hardcoded Everywhere — Multi-File Imports Broken
- **Evidence:** `NullModuleResolver` is passed in `tools/ueas-cli/src/main.rs`, `tools/ueas-lsp/src/lib.rs`, `tools/ueas-dap/src/lib.rs`, `crates/ueas-wasm/src/lib.rs`, and `tests/ucts/src/lib.rs`.
- **Critique:** `import std.math` or `import utils.helpers` cannot resolve in any tool! Users cannot compose algorithms across files or reuse `stdlib/`.
- **Better Solution:**
  1. Implement `EmbeddedStdlibResolver` in a small `ueas-stdlib` (or inside `ueas-hir` via `include_str!` / registry) containing all 44 verified `std.*` modules — works zero-I/O in `wasm32` (Browser Playground & Edge MCP) and native CLI!
  2. Implement `FsModuleResolver` (with `ChainedResolver(FsModuleResolver, EmbeddedStdlibResolver)`) in `ueas-cli`, `ueas-lsp`, and `ueas-dap` so local files + `std.*` resolve seamlessly.

### [HIR-01] `compile_to_hir` Discards Structured `SourceSpan`s into `Vec<String>`
- **Evidence:** `crates/ueas-hir/src/lib.rs:20` returns `Result<Program, Vec<String>>`, forcing `tools/ueas-lsp/src/lib.rs:46` to hardcode `Position::new(0, 0)` for every HIR error!
- **Critique:** In VS Code / Zed, every type error, MVS violation, or CT leak shows up on line 1 column 1 instead of highlighting the offending expression.
- **Better Solution:** Expose `compile_to_hir_detailed(source, resolver) -> Result<Program, Vec<DiagnosticItem>>` where `DiagnosticItem { code, message, span: SourceSpan, hint: Option<String> }`, and wire it into `ueas-lsp`, `ueas-cli`, `ueas-verify`, and `ueas-wasm`.

### [EXT-01] Tree-sitter Extractors Drop Common Constructs & Emit Hardcoded `O(N)`
- **Evidence:** `crates/ueas-extract/src/{python,typescript,rust_lang}.rs` ignore `elif` (`elif_clause`), `for` loops in TS/Rust, compound assignments (`+=`, `-=`), and emit hardcoded `Time: O(N)`.
- **Better Solution:** Support `elif_clause`, `augmented_assignment` (`+=`, `-=`, `*=`), `for_statement` / `for_expression` in TS/Rust, and emit clean `@Extracted` comments without fabricating unverified `O(N)` complexity claims.

---

## Domain 6: Developer Experience, AI-Agent Integration & Tooling (`tools/`)

### [UX-01] MCP Tool Schema & Surface Drift Between CLI (`ueas mcp`) and Edge (`ueas-mcp-edge`)
- **Evidence:**
  - `tools/ueas-cli/src/main.rs:249` lists 4 tools (`ueas_check`, `ueas_verify`, `ueas_transpile`, `ueas_extract`) with NO `inputSchema`.
  - `tools/ueas-mcp-edge/src/index.ts:86` lists 5 tools (`ueas_check`, `ueas_verify`, `ueas_transpile`, `ueas_run`, `ueas_format`) with NO `inputSchema`.
- **Critique:** Modern MCP clients (Claude Desktop, Cursor, Windsurf, Zed) validate tool calls against JSON `inputSchema`. Missing `inputSchema` causes tool-call failures or hallucinated parameter names, and tool divergence between local and edge confuses agents.
- **Better Solution:** Define a single canonical MCP tool catalog (all 6 tools: `ueas_check`, `ueas_run`, `ueas_verify`, `ueas_transpile`, `ueas_format`, `ueas_extract` on CLI; the 5 WASM-compatible tools on Edge) with complete JSON Schema `inputSchema` objects and structured `isError` flags.

### [UX-02] Playground UX Lacks Example Browser, Diagnostics Gutter & Visual Telemetry Charts
- **Evidence:** `tools/ueas-playground/index.html` and `src/main.ts` render only a bare `<textarea>` and `<pre>` JSON dump.
- **Critique:** UEAS's biggest differentiator is seeing live **Big-O empirical curve fitting**, **L1/L2/L3 cache hit rates**, **Work/Span parallelism**, and **L0–L3 verification badges** in real time. Dumping raw JSON into `<pre>` hides the product's magic.
- **Better Solution:** Upgrade `tools/ueas-playground` into a real **UEAS Studio** (still 100% XSS-safe via DOM APIs / `textContent`, zero `innerHTML`):
  - Curated built-in example selector (BinarySearch, QuickSort, Dijkstra, ParallelSum, ConstantTimeSelect, BrokenComplexity demo).
  - Live structured diagnostics panel with line/column jump.
  - Pure-SVG visual charts for **Empirical Big-O Scaling Curve**, **L1/L2/L3 Cache Hit Rates**, and **Work/Span Parallelism**.
  - Shareable URL hash encoding (`#code=...`) and one-click `.ueas.cert.json` download.

### [UX-03] LSP Missing Go-to-Definition, Completions & Span-Accurate Diagnostics
- **Evidence:** `tools/ueas-lsp/src/lib.rs:43-54` puts all HIR errors at `(0, 0)` and only implements hover + document symbols + formatting.
- **Better Solution:** Wire `compile_to_hir_detailed` for exact diagnostic ranges, add `completion` (UEAS keywords, built-in types, `std.*` modules, collection methods), and add `goto_definition` for `Algorithm` and `Record` symbols.

### [UX-04] Stateless Jupyter Kernel Breaks Multi-Cell Notebooks
- **Evidence:** `tools/ueas-jupyter/ueas_jupyter/kernel.py:34-46` writes each cell to an isolated temp file and invokes `ueas run`, so definitions in Cell 1 are lost in Cell 2.
- **Better Solution:** Maintain an accumulated definition buffer (`Record` and `Algorithm` declarations from prior successful cells) in `UeasKernel` (with `%reset`, `%verify`, `%transpile <target>` line magics!) so interactive notebooks work naturally across cells.

---

## Domain 7: Universal Language Extensibility & Breaking the $O(M \times N)$ Bottleneck

### [EXT-ARCH-01] Duplicated High-Level Lowering Across Backends (`HIR -> Target`)
- **Evidence:** `crates/ueas-codegen/src/track_a.rs` and `track_b.rs` each independently lower high-level HIR constructs (`Quantifier`, `Ensure`, `For` over collections, `Match`, `Slice`, `mut` write-backs) directly into target syntax.
- **Critique:** Every new UEAS language feature forces $N$ separate, error-prone re-implementations across every backend.
- **Better Solution:** Introduce a single canonical **`HIR -> LIR` (Lowered Imperative IR)** desugaring pass inside `crates/ueas-codegen/src/lir.rs`. `LIR` lowers `forall`/`exists` -> loops, `Ensure` -> return-site assertions, `For` over collections -> indexed loops, `Match` -> tag checks, and `mut` -> explicit write-backs **once**, shrinking every source emitter by ~70% and guaranteeing semantic consistency.

### [EXT-ARCH-02] No Universal Portable Binary Target (Piece-by-Piece Language Lock-In)
- **Evidence:** Outbound production synthesis currently requires hand-writing a separate transpiler for each language (`Rust`, `Python`, `TypeScript`, `C++`, `Go`), leaving Java, C#, Swift, Zig, Ruby, Kotlin, and browser-native consumers unsupported.
- **Better Solution:** Add a **Universal WebAssembly Target (`WasmTarget` + `.wit` Interface Generator)** in `crates/ueas-codegen` powered by pure-Rust `wasm-encoder` (`Apache-2.0`, `#![forbid(unsafe_code)]`, `wasm32`-compatible). One universal binary backend delivers 100% semantically exact execution to every major language runtime (`wasmtime`, `jco`, `wazero`, browser native) with zero language-specific drift.

### [EXT-ARCH-03] Prover-Specific Track B Fragmentation
- **Evidence:** Formal proof generation is fragmented across 5 prover-specific syntaxes (`Dafny`, `Lean4`, `Verus`, `Quint`, `TLA+`).
- **Better Solution:** Add a **Universal `SMT-LIB2` Verification Condition (VC) Target (`SmtLib2Target`)** in `crates/ueas-codegen` emitting standard SMT-LIB2 `(declare-const ...)`, `(assert ...)`, `(check-sat)` obligations for `Require`/`Ensure` contracts — immediately supporting Z3, CVC5, Alt-Ergo, and Bitwuzla with zero extra dependencies.

### [EXT-ARCH-04] Closed Codegen Architecture (No Community/Enterprise Target Plugins)
- **Evidence:** `TargetKind` in `crates/ueas-codegen/src/lib.rs` is a closed Rust `enum`; adding a new target (e.g., Java, Zig, CUDA, Kotlin) requires modifying UEAS core and recompiling the CLI.
- **Better Solution:** Expose an **Open External Plugin ABI** (`ueas transpile <file> --plugin <cmd>`): streams canonical `ueas-hir.v2.schema.json` (plus optional lowered `LIR` JSON) to the plugin's `stdin` and reads generated source/JSON artifacts from `stdout`. Any language or enterprise backend can plug into UEAS without bloating core.

### [EXT-ARCH-05] Missing Language-Agnostic L4 Differential Oracle (`ueas oracle`)
- **Evidence:** `tests/differential` only validates Rust code emitted by `ueas-codegen`; there is no reusable, language-agnostic differential oracle for arbitrary external implementations (human-written, legacy-extracted, or LLM-synthesized in Zig, Java, C#, CUDA, Go, Python, etc.).
- **Better Solution:** Build **`ueas oracle <spec.ueas> --entry <Algo> --exec "<cmd>"`** (and MCP tool `ueas_oracle_diff`):
  - Uses `ueas-verify`'s precondition-guided fuzzer to generate 64+ diverse valid inputs (edge cases + random + adversarial) encoded as newline-delimited JSON.
  - Feeds each input to both `ueas-vm` and the external executable over stdio, compares canonical outputs, and shrinks any mismatch into a minimal counterexample!
  - Unlocks **universal outbound synthesis verification** AND **universal inbound extraction verification** for *every programming language* without maintaining bespoke per-language AST translators.

---

## Summary Priority Matrix

| Priority | Audit IDs | Theme | Impact |
| :--- | :--- | :--- | :--- |
| **P0 (Critical)** | `VER-01`, `CG-01`, `VM-01`, `VM-04` | Security fail-closed CT, Zero-Silent-Fallback Codegen, CoW VirtualHeap | Eliminates false security passes, silent miscompilation, and O(N) VM distortion |
| **P1 (High)** | `MOD-01`, `PROD-02`, `VM-02`, `VM-03`, `VER-02`, `VER-03` | Real `std.*` + FS Module Resolution, `mut` inout write-back, Direct Bytecode Dispatch, Precondition-valid L3 + Recursive L2 Fuzzer | Makes UEAS genuinely usable for multi-file real algorithms |
| **P1 (Universal Scale)** | `EXT-ARCH-01`..`EXT-ARCH-05` | `HIR -> LIR` desugaring, Universal `WASM/WIT` binary, `SMT-LIB2`, External Plugin ABI, Language-Agnostic `ueas oracle` | Breaks the $O(M \times N)$ language-expansion trap for both humans and AI agents |
| **P2 (High UX)** | `HIR-01`, `UX-01`, `UX-02`, `UX-03`, `UX-04`, `VER-04`, `EXT-01` | Span-accurate LSP, Unified MCP `inputSchema`, Visual Playground Studio, Stateful Jupyter, `verify-cert`, Rich Extractors | Transforms developer & AI-agent adoption experience |
