# UEAS — Version 1.0 Final Implementation

## Phase A — Grammar Completeness
- [ ] A.1 Add `Directed`/`Undirected` keyword to grammar + Graph type marker
- [ ] A.2 Add `const` keyword + `constDecl` grammar rule
- [ ] A.3 Add `@Memory` annotation grammar + parsing
- [ ] A.4 Add `randInt(min, max)`, `randReal()` to builtins registry

## Phase B — Kernel Support
- [ ] B.1 `const` variable storage: immutable scope entry in SymbolTable
- [ ] B.2 `@Memory` profiler hook: enforce_memory_contract() in execute_algorithm
- [ ] B.3 `randInt`, `randReal` in dispatch_builtin with step costing
- [ ] B.4 Directed/undirected Graph type tag extension

## Phase C — Documentation Alignment
- [ ] C.1 SPEC.md Section 3: add `const`, `@Memory`, Graph direction, rand
- [ ] C.2 SPEC.md Section 4: update EBNF for new grammar rules
- [ ] C.3 SPEC.md Section 6: update error semantics + step costs
- [ ] C.4 README.md: update feature list + version badge
- [ ] C.5 AGENTS.md: update directory map, design rationale
- [ ] C.6 examples/: add const, direction, rand to benchmarks
- [ ] C.7 REVIEW.md: mark all Section 3.1 items as evaluated

## Phase D — Final Quality Gate
- [ ] D.1 Full workspace: 160+ tests, clippy clean, fmt clean
- [ ] D.2 Grammar: 14+ parse tests pass with new features
- [ ] D.3 Commit + push, tag v0.1.0

## Deferred (requires RFC)
- Generic algorithms (algorithm sort<T>)
- Enumerations (enum keyword)
- Standard library module system
- CLI / LSP / UCTS harness (separate projects)

## V2.0 (deferred per REVIEW.md Section 5)
- Indentation syntax, dot-notation, compound assignment, etc.
