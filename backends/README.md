# UEAS Transpilation Backends

Five production-grade transpilation targets implementing the `TargetGenerator`
interface (GoF Strategy). All targets produce idiomatic source code from the
canonical UEAS JSON AST with guaranteed semantic equivalence.

## Target Overview

| Target | Language | Version | Module | Variable Tracking |
|--------|----------|---------|--------|-------------------|
| Python | python | 3.11 | `backends/src/lib.rs` | None (Python is dynamically typed) |
| Rust | rust | 2021 | `backends/src/lib.rs` | `HashSet<String>` → `let mut` |
| C++ | cpp | 17 | `backends/src/cpp.rs` | `HashSet<String>` → `auto` |
| Java | java | 17 | `backends/src/java.rs` | `HashSet<String>` → `var` |
| JavaScript | javascript | ES2020 | `backends/src/javascript.rs` | `HashSet<String>` → `let` |

## Type Mappings

| UEAS Type | Python | Rust | C++ | Java | JavaScript |
|-----------|--------|------|-----|------|------------|
| Integer | `int` | `i64` | `int64_t` | `long` | `Number` |
| Real | `float` | `f64` | `double` | `double` | `Number` |
| Boolean | `bool` | `bool` | `bool` | `boolean` | `boolean` |
| String | `str` | `String` | `std::string` | `String` | `string` |

## Function Name Mappings

| UEAS Built-in | Python | Rust | C++ | Java | JavaScript |
|--------------|--------|------|-----|------|------------|
| `sqrt` | `math.sqrt` | `f64::sqrt` | `std::sqrt` | `Math.sqrt` | `Math.sqrt` |
| `length` | `len` | `len` | `size()` | `size()` | `length` |
| `cardinality` | `len` | `len` | `size()` | `size()` | `length` |

## Control Flow Patterns

| UEAS Construct | Python | Rust | C++ | Java | JavaScript |
|----------------|--------|------|-----|------|------------|
| `if/then/end if` | `if cond:` | `if cond { }` | `if (cond) { }` | `if (cond) { }` | `if (cond) { }` |
| `for/in/do/end for` | `for x in range(N):` | `for x in 0..N { }` | `for (auto x : col) { }` | `for (var x : col) { }` | `for (let x of col) { }` |
| `while/do/end while` | `while cond:` | `while cond { }` | `while (cond) { }` | `while (cond) { }` | `while (cond) { }` |

## Adding a New Target

1. Create `backends/src/<lang>.rs` with `pub struct <Lang>Target;`
2. Implement `TargetGenerator` trait (5 required methods)
3. Implement `generate_algo`, `generate_statement`, `generate_node` helper methods
4. Add `pub mod <lang>;` to `backends/src/lib.rs`
5. Add tests verifying: language name, version, integer literal, addition, function definition
6. Update this file with the new target's type and function mappings
