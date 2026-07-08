# UEAS — Comprehensive Refactoring Plan

## Phase 1: Syntax Modernization (Grammar)
- [ ] 1.1 Replace SEMICOLON statement terminator with NEWLINE (keep ; optional)
- [ ] 1.2 Remove LPAREN/RPAREN from ifStmt, whileLoop, forLoop conditions
- [ ] 1.3 Add `elif` keyword as `else if` shorthand
- [ ] 1.4 Add `import IDENTIFIER` production rule (parse-only, no cross-file resolution)
- [ ] 1.5 Add `range(start, end)` to primary expression list
- [ ] 1.6 Add `emptyList()`, `emptySet()`, `emptyMap()` to built-in calls
- [ ] 1.7 Update `.ueas` grammar test files to new syntax
- [ ] 1.8 Update SPEC.md Section 4 (EBNF) and examples to new syntax

## Phase 2: Kernel Completeness (Rust)
- [ ] 2.1 Refactor SymbolTable to store only HeapHandle (remove SymbolValue::Value)
- [ ] 2.2 Implement heap-backed value lifecycle (allocate→write→read→deallocate)
- [ ] 2.3 Implement implicit complexity binding: auto-bind N from first param
- [ ] 2.4 Replace count_collection_items() with actual size computation
- [ ] 2.5 Add builtins: length, cardinality, contains, append, slice, pop, range
- [ ] 2.6 Add builtins: emptyList, emptySet, emptyMap
- [ ] 2.7 Update enforce_complexity() for implicit + explicit binding
- [ ] 2.8 Add integration tests for all 7 benchmark algorithms

## Phase 3: Transpilation & Quality (Backends + Docs)
- [ ] 3.1 Rust backend: statement transpilation (if, while, for, return)
- [ ] 3.2 Transpiler builtins: map kernel builtins to target-language equivalents
- [ ] 3.3 E2E pipeline test: .ueas → parse → AST → kernel → transpile
- [ ] 3.4 Create examples/ directory with 7 benchmark .ueas files
- [ ] 3.5 Update README.md code examples to new syntax
- [ ] 3.6 Update SPEC.md all algorithm examples to new syntax
- [ ] 3.7 Full CI gate: test + clippy + fmt, all clean

## Phase 4: Documentation Finalization
- [ ] 4.1 SPEC.md Section 4 EBNF updated to match final grammar
- [ ] 4.2 SPEC.md Section 5 AST examples updated
- [ ] 4.3 AGENTS.md updated with new toolchain / quality gate commands
- [ ] 4.4 REVIEW.md deleted (feedback incorporated)
- [ ] 4.5 Final workspace CI gate: 100% pass, clippy clean, fmt clean
