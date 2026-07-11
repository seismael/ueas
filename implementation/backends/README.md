# UEAS Transpilation Backends

Eight transpilation targets implementing the `TargetGenerator`
interface (GoF Strategy). All targets produce idiomatic code from the
canonical UEAS JSON AST with guaranteed semantic equivalence.

## Target Overview

| Target | Language | Version | Module | Type |
|--------|----------|---------|--------|------|
| Python | python | 3.11 | `backends/src/lib.rs` | Imperative |
| Rust | rust | 2021 | `backends/src/lib.rs` | Imperative |
| C++ | cpp | 17 | `backends/src/cpp.rs` | Imperative |
| Java | java | 17 | `backends/src/java.rs` | Imperative |
| JavaScript | javascript | ES2020 | `backends/src/javascript.rs` | Imperative |
| Lean 4 | lean4 | 4.0 | `backends/src/lean4.rs` | Formal Verification (theorem proving) |
| TLA+ | tlaplus | 2.18 | `backends/src/tla.rs` | Formal Verification (model checking) |
| LaTeX | latex | algorithm2e/v5.2 | `backends/src/latex.rs` | Academic Publishing |

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
