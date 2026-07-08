# UEAS — Final Implementation Loop

## CRITICAL (spec drift)
- [ ] 1. SPEC.md Section 4.2 EBNF stale — shows C-style syntax, UEAS.g4 is modern
- [ ] 2. SPEC.md Section 5.1 says GraphLiteral "Reserved" but code implements it

## MEDIUM (buggy/wrong behavior)
- [ ] 3. dispatch_builtin() returns Integer(0) for 16 stubs — should trap with unimplemented
- [ ] 4. invariants::re_evaluate_loop_invariants() — stub, never called
- [ ] 5. enforce_complexity() fragile substring matching (O(N^20) matches O(N^2))
- [ ] 6. Unused import in conformance.rs

## LOW (infrastructure/quality)
- [ ] 7. Doc comments on pub items: evaluate(), execute_program(), ExecContext, SymbolTable
- [ ] 8. prelude_map() dead code in both backends — remove or integrate
- [ ] 9. Quality gate commands reference non-existent targets (tools/, integration test)
- [ ] 10. Missing .github/ directory (referenced in AGENTS.md)
