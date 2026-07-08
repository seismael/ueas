# UEAS — Final Implementation Loop

## Phase I: Grammar & AST (Epoch 1) [95%]
- [x] Remove semicolons, modernize control flow, add import rule
- [x] Matrix dimensions accept type variables
- [x] 14/14 grammar tests parse
- [ ] 1.1 Complete Graph/Matrix literal grammar (remove "reserved" comments)
- [ ] 1.2 Fix `#[serde(untagged)]` on AstValue with proper variant ordering
- [ ] 1.3 Fix `count_collection_items()` placeholder — read actual heap metadata

## Phase II: Microkernel (Epoch 2) [90%]
- [x] SymbolTable → HeapHandle only
- [x] Complexity enforcement at algorithm termination
- [x] 15+ built-in functions in dispatcher
- [x] 93 kernel tests pass
- [ ] 2.1 Make complexity enforcement evaluate variableBinding expressions
- [ ] 2.2 Add trap validation: ComplexityViolation, InfiniteLoopDetected, HeapExhaustion, IndexOutOfBounds

## Phase III: Transpilation (Epoch 3) [80%]
- [x] Python + Rust expression transpilation
- [x] Python statement transpilation (var, assign, return, if, while, for)
- [x] MCP endpoint
- [ ] 3.1 Rust backend: full statement transpilation
- [ ] 3.2 Standard prelude mapping (length→len, cardinality→len, etc.)
- [ ] 3.3 Memory lifecycle: Rust .clone() for composite types, Python deepcopy

## Phase IV: Conformance (UCTS) [10%]
- [x] 7 benchmark .ueas examples parse
- [ ] 4.1 Create conformance.rs: positive tests (7 benchmarks → ExitCode::NoError)
- [ ] 4.2 Create trap tests: ComplexityViolation, InfiniteLoopDetected, HeapExhaustion, IndexOutOfBounds
- [ ] 4.3 Create negative .ueas trap test files
- [ ] 4.4 Cross-target equivalence on all 7 benchmarks

## Definition of Done
- [ ] All 10 ExitCode traps verifiable via automated harness
- [ ] Property-based fuzz: 10^6 AST permutations, zero panics
- [ ] Cross-target transpilation yields identical results
- [ ] 100% SPEC.md Section 6.5 error semantics verifiable
