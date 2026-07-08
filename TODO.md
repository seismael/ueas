# UEAS Development — Epoch 1 Task List

> Process tasks sequentially, top to bottom. Mark `[x]` only after verification passes.
> Each task follows the 8-phase pipeline from AGENTS.md.

## Scaffolding

- [ ] 1.1 Create `kernel/Cargo.toml` — Rust workspace with edition 2021, dependencies
- [ ] 1.2 Create `kernel/src/lib.rs` with module declarations (`ast`, `interp`, `heap`, `traps`, `profiling`)
- [ ] 1.3 Create `grammar/UEAS.g4` — ANTLR4 grammar skeleton with lexer rules from SPEC.md Section 4.1
- [ ] 1.4 Create `backends/Cargo.toml` — workspace with plugin trait stub

## AST Foundation

- [ ] 2.1 Define `AstNodeKind` enum per SPEC.md Section 5.1 (all 20+ node kinds)
- [ ] 2.2 Define `AstNode` struct with `kind`, `children`, `location`, `type_id` fields
- [ ] 2.3 Implement `AstNodeFactory` (GoF Factory) — validates invariants on construction
- [ ] 2.4 Write unit tests for factory: valid creation, invalid kind rejection, type-checking
- [ ] 2.5 Implement `AstVisitor` trait (GoF Visitor) with one method per node kind
- [ ] 2.6 Implement canonical JSON serialization (`serde::Serialize` for all AST types)
- [ ] 2.7 Implement canonical JSON deserialization (`serde::Deserialize` for all AST types)
- [ ] 2.8 Write round-trip serde tests: serialize → deserialize → assert structural equality

## Type System

- [ ] 3.1 Define `PrimitiveType` enum (Integer, Real, Boolean, String, Void)
- [ ] 3.2 Define `CompositeType` enum (Set, List, Map, Graph, Matrix, Option, Result, Tuple)
- [ ] 3.3 Implement type compatibility checker per Appendix A (type compatibility matrix)
- [ ] 3.4 Write unit tests: valid type matches, invalid type rejection, cast validation

## Virtual Heap

- [ ] 4.1 Define `HeapHandle(u64)` newtype
- [ ] 4.2 Define `HeapConfig` struct with `max_size`, `alignment` constants
- [ ] 4.3 Implement `VirtualHeap` with bump-pointer allocation
- [ ] 4.4 Implement `allocate(size, type_tag)` → `Result<HeapHandle, TrapCode>`
- [ ] 4.5 Implement `deallocate(handle)` — scope-exit eager deallocation
- [ ] 4.6 Implement typed read/write with bounds checking
- [ ] 4.7 Write unit tests: allocate, deallocate, read, write, exhaustion trap
- [ ] 4.8 Write property-based tests: random alloc/dealloc sequences, assert zero panics

## Interpreter Core

- [ ] 5.1 Define `StepCount(u64)` newtype and `StepCounter` with monotonic increment
- [ ] 5.2 Define `TrapCode` enum per SPEC.md Section 6.5 (codes 0-10)
- [ ] 5.3 Implement `TrapRegister` — set, clear, query trap state
- [ ] 5.4 Implement arithmetic expression evaluator (+, -, *, /, mod)
- [ ] 5.5 Implement comparison evaluator (==, !=, <, >, <=, >=)
- [ ] 5.6 Implement logical expression evaluator (and, or, not)
- [ ] 5.7 Write unit tests: happy path, zero, negative, boundary, division-by-zero trap
- [ ] 5.8 Write complexity violation tests: O(N^2) contract breached on nested loops

## Grammar (ANTLR4)

- [ ] 6.1 Define all lexical tokens in `UEAS.g4` (keywords, operators, literals)
- [ ] 6.2 Define parser rules for algorithm declarations with parameters and complexity
- [ ] 6.3 Define parser rules for statements (let, assign, return, if, for, while, assert, invariant)
- [ ] 6.4 Define parser rules for expressions by precedence (logical → comparison → additive → multiplicative → unary → primary)
- [ ] 6.5 Define parser rules for composite literals (set, list, map, matrix)
- [ ] 6.6 Define type annotation parser rules (primitive and composite types)
- [ ] 6.7 Generate parser with `antlr4 -Dlanguage=Rust UEAS.g4`
- [ ] 6.8 Write positive parse tests: valid .ueas programs parse successfully
- [ ] 6.9 Write negative parse tests: invalid syntax, type mismatches, missing annotations — all rejected

## Backend Plugin System

- [ ] 7.1 Define `TargetGenerator` trait (GoF Strategy) with `language()`, `version()`, `generate()`, `supportedKinds()`, `typeMap()`
- [ ] 7.2 Implement Python backend stub with basic arithmetic expression generation
- [ ] 7.3 Write basic transpilation test: arithmetic AST → Python source → execute → verify output
- [ ] 7.4 Implement Rust backend stub
- [ ] 7.5 Write cross-target equivalence test: same AST → Python output matches Rust output

---

## Epoch 1 Success Criteria

- [ ] ANTLR4 parser ingests all 7 benchmark algorithms from SPEC.md Section 9.3
- [ ] All unit tests pass (`cargo test`)
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — clean
- [ ] Property-based fuzz tests pass (`cargo test --test fuzz -- --ignored`)
- [ ] Cross-target equivalence verified for at least 2 targets on 7 benchmarks
