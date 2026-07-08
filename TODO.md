# UEAS — v0.1.0 Released

## All Done
- [x] A.1-A.4: Grammar — Directed/Undirected, const, @Memory, rand
- [x] B.1-B.4: Kernel — const storage, @Memory hook, rand builtins, Graph tags
- [x] C.1-C.7: Documentation — SPEC.md, README.md, AGENTS.md, examples
- [x] D.1-D.3: Quality — 151 tests, clippy clean, fmt clean, tagged

## Metrics
  Tests: 151 (116 kernel + 22 backend + 7 conformance + 6 fuzz)
  Grammar: 15/15 parse tests (8 examples + 7 test files)
  AST node kinds: 33
  TypeTag variants: 18
  Builtins: 22+
  Exit codes: 12 (0-11)
  REPO: https://github.com/seismael/ueas
  TAG: v0.1.0

## Deferred (requires RFC)
- Generic algorithms (algorithm sort<T>)
- Enumerations (enum keyword)
- Standard library module system
- CLI / LSP / UCTS harness

## V2.0 (deferred per REVIEW.md Section 5)
- Indentation syntax, dot-notation, compound assignment, etc.
