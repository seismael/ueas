# UEAS — Continuous Loop Tasks

## From REVIEW.md Section 3 (Implementable Now)
- [ ] Add `bitwise_op()` to eval_binary dispatch table (& | ^ << >> operators)
- [ ] Add string builtins to dispatch_builtin (substring, concat, string-length)
- [ ] Implement break/continue in execute_algorithm loop

## From REVIEW.md Section 2 (Remaining)
- [ ] Add `NOT` unary handling in eval_unary (currently stubbed)
- [ ] Add AST source mapping stub to conformance tests
- [ ] Add eval_binary eq/neq handling for Real values

## Quality
- [ ] 144 tests baseline verified
- [ ] clippy clean, fmt clean
- [ ] Commit + push
