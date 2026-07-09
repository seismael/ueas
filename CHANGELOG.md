# Changelog

All notable changes to UEAS follow [Keep a Changelog](https://keepachangelog.com/) format.

## [0.2.0] — July 2026 (V2.0 Iceberg Architecture)

### Added
- V2.0 grammar: INDENT/DEDENT blocks (Pythonic pseudocode syntax)
- `in`/`not in` operators with desugaring to contains()
- `pass` statement, NEWS statement terminator (no semicolons)
- Method chaining: target.method(args) → method(target, args)
- Simplified composite literals: [] for List, {} for Set/Map
- `@Complexity` + `@Memory` decorators above algorithm declaration
- Semantic engine: kernel/src/infer/mod.rs (type inference + desugaring)
- Implicit variable declaration (first assignment = VariableDeclaration)
- V2.0 examples: dfs.ueas, bfs.ueas, quicksort.ueas, binary_search.ueas
- `const` keyword for immutable variables
- `Directed`/`Undirected` graph type markers
- `randInt`/`randReal` randomization builtins
- `@Memory` annotation for memory complexity enforcement
- Infinity/NaN IEEE 754 literal tokens
- Bitwise operators (& | ^ << >>)
- String manipulation: substring(), concat(), strlen()

### Changed
- Control flow: `if expr :`, `while expr :`, `for x in expr :` (no parentheses)
- Statements: NEWS-terminated (no semicolons)
- Blocks: INDENT/DEDENT (no curly braces)

## [0.1.0] — July 2026

### Added
- Formal specification (SPEC.md v1.0.0-draft, 980 lines)
- ANTLR4 grammar (UEAS.g4) with Modern Mathematical syntax
- AST node types (32 kinds), Factory, Visitor, JSON serde
- Type system: primitive types, composite types, type compatibility
- Virtual heap: isolated bump-pointer allocation with 16 type tags
- Exit codes (0-11) per Section 6.5/6.6
- Step counter and complexity profiler (all Big-O forms)
- Abstract interpreter: expression evaluator, statement executor
- Symbol table with stacked scopes + recursion depth trapping
- Invariant and assertion enforcement with trap codes
- For-loop iteration step counting (SPEC 6.2 compliance)
- @Memory memory complexity enforcement (heap.bytes_allocated())
- const declarations with immutability enforcement
- TargetGenerator trait (GoF Strategy)
- Python transpiler target with full statement transpilation
- Rust transpiler target with full statement transpilation
- MCP (Model Context Protocol) endpoint
- Cross-target equivalence tests (7 benchmarks)
- Property-based fuzz tests (6 proptest + 200K batch)
- Comprehensive test suite (151 tests)
- Composite literal evaluation (Set, List, Map)
- 22+ composite builtins: union, intersection, difference, get, put, containsKey, keys, values, transpose, substring, concat, strlen, randInt, randReal, etc.
- Dynamic step costing + bitwise operators (& | ^ << >>)
- Infinity/NaN literals + break/continue statements
- Directed/Undirected Graph type variants
- Project documentation (README, AGENTS.md, CONTRIBUTING.md, CLA, LICENSE, NOTICE)
- RFC process, ADR format, meeting-notes, domain specs
- GitHub issue and PR templates
- CI workflows (ci.yml, fuzz.yml), Dockerfile, SECURITY.md, CODE_OF_CONDUCT.md, GOVERNANCE.md
- 8 benchmark example .ueas files
