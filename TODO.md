# UEAS — V2.0 "Iceberg Architecture" Implementation Loop

## Phase 1: Grammar Modernization (Syntax V2.0)
- [ ] 1.1 Add INDENT/DEDENT tokens to lexer (Pythonic indentation tracking)
- [ ] 1.2 Replace { } braces with INDENT/DEDENT blocks in all rules
- [ ] 1.3 Add `in` operator to comparison precedence
- [ ] 1.4 Add `not in` compound operator
- [ ] 1.5 Add `pass` statement
- [ ] 1.6 Move `@Complexity` above algorithm declaration (decorator pattern)
- [ ] 1.7 Simplify composite literals: `[]` auto-infers List, `{}` auto-infers Set/Map
- [ ] 1.8 Make `let` keyword optional (implicit declaration via `assignmentOrCall`)
- [ ] 1.9 Add method chaining: `target.IDENTIFIER(args)` → desugared to `IDENTIFIER(target, args)`
- [ ] 1.10 Remove mandatory type annotations from algorithm body
- [ ] 1.11 Regenerate ANTLR4 grammar, verify compiles clean
- [ ] 1.12 Update all .ueas test files to V2.0 syntax
- [ ] 1.13 Update all examples/ to V2.0 syntax

## Phase 2: Semantic Engine (Type Inference + Desugaring)
- [ ] 2.1 Create `kernel/src/infer/mod.rs` module
- [ ] 2.2 Implement type inference for primitives (0→Integer, 0.0→Real, true→Boolean, ""→String)
- [ ] 2.3 Implement composite literal inference ([1,2,3]→List<Integer>, {}→Set)
- [ ] 2.4 Implement variable scope resolution (first assignment = declaration)
- [ ] 2.5 Implement method chaining desugaring (a.push(b)→push(a,b))
- [ ] 2.6 Implement `in` operator desugaring (x in s→contains(s,x))
- [ ] 2.7 Implement `not in` operator desugaring (x not in s→not contains(s,x))
- [ ] 2.8 Create multi-pass pipeline: DraftAST → Infer → Validate → Execute
- [ ] 2.9 Declare infer module in lib.rs
- [ ] 2.10 Write unit tests for type inference
- [ ] 2.11 Write unit tests for desugaring

## Phase 3: Kernel Integration
- [ ] 3.1 Integrate inference engine into execute_algorithm pipeline
- [ ] 3.2 Remove `let` requirement from AST (allow implicit VariableDeclaration)
- [ ] 3.3 Update execute_var_decl to handle inferred types
- [ ] 3.4 Wire `pass` statement execution (no-op)
- [ ] 3.5 Wire `in`/`not in` operator through eval_binary
- [ ] 3.6 Write integration tests for full pipeline
- [ ] 3.7 Full workspace: test + clippy + fmt

## Phase 4: Documentation & Quality
- [ ] 4.1 Update SPEC.md with V2.0 grammar and semantics
- [ ] 4.2 Update README.md to V2.0 DFS example
- [ ] 4.3 Update AGENTS.md design rationale
- [ ] 4.4 Update CHANGELOG.md
- [ ] 4.5 Create new examples/: dfs, bfs, quicksort in V2.0 syntax
- [ ] 4.6 Full quality gate: 160+ tests, clippy clean, fmt clean
- [ ] 4.7 Final REVIEW.md evaluation
- [ ] 4.8 Commit + push + tag v0.2.0
