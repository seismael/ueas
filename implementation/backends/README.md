# UEAS Transpilation Backends

Four transpilation targets implementing the `TargetGenerator`
interface (GoF Strategy).

## Target Overview

| Type | Target | Language | Version | Module | Purpose |
|------|--------|----------|---------|--------|---------|
| Imperative | Dafny | dafny | 4.6.0 | `backends/src/dafny.rs` | Z3 proofs + code gen (C++, Python, Java, Go, C#, JS) |
| Formal | Lean 4 | lean4 | 4.0 | `backends/src/lean4.rs` | Theorem proving |
| Formal | TLA+ | tlaplus | 2.18 | `backends/src/tla.rs` | Model checking |
| Academic | LaTeX | latex | algorithm2e/v5.2 | `backends/src/latex.rs` | Academic publishing |

## Architecture

```
UEAS AST → DafnyTarget → .dfy source → Z3 SMT Solver → Proof verified
                                                ↓
                                         dafny build --target:cpp → C++
                                         dafny build --target:py  → Python
                                         dafny build --target:java → Java
                                         dafny build --target:go   → Go
                                         dafny build --target:cs   → C#
                                         dafny build --target:js   → JavaScript
```

The DafnyTarget replaces 5 deprecated direct transpilers (Python, Rust, C++, Java, JS)
with a single verifiable pipeline. The Z3 theorem prover guarantees mathematical
correctness before any code is generated.

## Type Mappings

| UEAS Type | Python | Rust | C++ | Java | JavaScript |
|-----------|--------|------|-----|------|------------|
| Integer | `int` | `i64` | `int64_t` | `long` | `Number` |
| Real | `float` | `f64` | `double` | `double` | `Number` |
| Boolean | `bool` | `bool` | `bool` | `boolean` | `boolean` |
| String | `str` | `String` | `std::string` | `String` | `string` |

Formal/academic targets map to their native type systems: ℕ/ℝ/Bool (Lean 4), Int/Real/Bool/String (TLA+), and academic names (LaTeX).

## Control Flow Patterns

| UEAS Construct | Python | Other Imperative | Lean 4 | TLA+ | LaTeX |
|----------------|--------|-----------------|--------|------|-------|
| `if/then/end if` | `if cond:` | `if (cond) { }` | `if cond then ... else ...` | `cond => ... /\\ ...` | `\\eIf{...}` |
| `for/in/do/end for` | `for x in range(N):` | `for (auto x : col) { }` | `List.map` | `\\A i \\in S:` | `\\ForEach{...}` |
| `while/do/end while` | `while cond:` | `while (cond) { }` | `partial def` | fairness condition | `\\While{...}` |

## Adding a New Target

1. Create `backends/src/<lang>.rs` with `pub struct <Lang>Target;`
2. Implement `TargetGenerator` trait (5 required methods)
3. Implement `generate_algo`, `generate_statement`, `generate_node` helper methods
4. Add `pub mod <lang>;` to `backends/src/lib.rs`
5. Add `pub use` re-export for CLI access
6. Add tests verifying: language name, version, integer literal, addition, function definition
7. Update this file with the new target's type and function mappings
