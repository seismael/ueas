# Changelog

All notable changes to UEAS follow [Keep a Changelog](https://keepachangelog.com/) format.
This project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] — Unreleased (Epoch 1)

### Added
- Formal specification (SPEC.md v1.0.0-draft, 933 lines)
- ANTLR4 grammar (UEAS.g4) with lexer and parser rules
- AST node types (29 kinds), Factory, Visitor, JSON serde
- Type system: primitive types, composite types, type compatibility
- Virtual heap: isolated bump-pointer allocation
- Exit codes (0-10) per Section 6.5
- Step counter and complexity profiler (all Big-O forms)
- Abstract interpreter: expression evaluator, statement executor
- Symbol table with stacked lexical scopes
- Invariant and assertion enforcement with trap codes
- TargetGenerator trait (GoF Strategy)
- Python transpiler target
- Rust transpiler target
- MCP (Model Context Protocol) endpoint
- Cross-target equivalence tests (7 benchmarks)
- Property-based fuzz tests (6 proptest + 200K batch)
- Comprehensive test suite (130 tests)
- Project documentation (README, AGENTS.md, CONTRIBUTING.md, CLA, LICENSE, NOTICE)
- RFC process, ADR format, meeting-notes, domain specs
- GitHub issue and PR templates
