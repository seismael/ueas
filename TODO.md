# UEAS — Production Readiness (25 items, 4 phases)

## Phase 1 — Spec Compliance
- [ ] 1.1 SPEC 4.1: Add ELIF, AS, BIND, IMPORT to keyword list
- [ ] 1.2 SPEC 5.1: Add EdgeLiteral, MatrixLiteral, Type to node kind table
- [ ] 1.3 SPEC 5.3: Align AST JSON example with factory structure
- [ ] 1.4 SPEC App.B: Fix reserved words — code allows them as identifiers
- [ ] 1.5 Grammar: Remove reserved words from `identifier` rule
- [ ] 1.6 SPEC 5.1: Update GraphLiteral (no longer reserved)
- [ ] 1.7 SPEC 6.2: Note step-cost flatness as known limitation
- [ ] 1.8 SPEC 9.3: Mark TSP as deferred (requires combinatorial solver)
- [ ] 1.9 Fix `conformance_complexity_violation` — currently asserts is_ok()

## Phase 2 — Infrastructure
- [ ] 2.1 Create `.github/workflows/ci.yml` — test + clippy + fmt
- [ ] 2.2 Create `tools/Dockerfile` — reproducible CI environment
- [ ] 2.3 Create `SECURITY.md` — vulnerability reporting policy
- [ ] 2.4 Create `CODE_OF_CONDUCT.md` — Apache Foundation CoC
- [ ] 2.5 Create `.github/workflows/fuzz.yml` — 10^6 fuzz batch nightly
- [ ] 2.6 Create `docs/GOVERNANCE.md` — BDFL → TSC transition

## Phase 3 — Missing Features
- [ ] 3.1 Implement 4 core builtins: `length`, `contains`, `append`, `pop`
- [ ] 3.2 Implement complexity enforcement for Sum + MixedLogLinear
- [ ] 3.3 Invariant loop re-evaluation (execute_while + execute_for)
- [ ] 3.4 Use `enforce_complexity` with actual parameter sizes

## Phase 4 — Quality
- [ ] 4.1 Add doc comments on pub items: evaluate(), execute_program(), ExecContext
- [ ] 4.2 Test: step_weighted(), exit_recursion(), capacity()
- [ ] 4.3 Test: execute_while, execute_for, execute_if directly
- [ ] 4.4 Final workspace gate: 150+ tests, clippy, fmt
