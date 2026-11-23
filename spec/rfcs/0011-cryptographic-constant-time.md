# RFC 0011: Cryptographic Constant-Time Guarantees

- **Domain:** grammar | kernel
- **Date:** 2026-07-10
- **Status:** Ratified

## Summary
Introduces semantic constraints for cryptographic algorithm verification. By tagging variables as `Secret` and wrapping execution in a `@ConstantTime` block, the language can mathematically guarantee immunity to timing side-channel attacks.

## Specification

### 1. Grammar Extension (`UEAS.g4`)
Add `Secret` type modifier:
```antlr4
typeAnnotation : 'Secret<' primitiveType '>' ;
```
Add the `@ConstantTime` block constraint:
```antlr4
blockConstraint : '@ConstantTime' block ;
```

### 2. Constraint Verification
When executing inside a `@ConstantTime` block, the abstract interpreter enters a specialized strict mode. If an `If / Else` branch conditional relies on a `Secret` variable, the interpreter records a fork. Both forks must execute and result in the exact same logical step-count constraint. If they diverge (e.g., one branch terminates in $O(1)$ and the other in $O(N)$), the interpreter raises a `TIMING_LEAK` trap.
