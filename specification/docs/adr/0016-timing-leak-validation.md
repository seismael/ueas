# ADR 0016: Timing Leak Validation

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

Cryptographic algorithms (AES, RSA) are vulnerable to timing side-channel attacks. If a conditional branch depends on a secret key, and the True path takes longer to execute than the False path, an attacker can deduce the key bits simply by measuring execution time. Traditional testing cannot prove the absence of timing leaks.

## Decision

The UEAS abstract interpreter will introduce a `TIMING_LEAK` trap. We will implement a symbolic execution fork when entering an `If` statement inside a `@ConstantTime` block. 
The interpreter will execute *both* paths in the Virtual Heap abstractly, counting the steps. If the step-counters diverge, it proves a non-constant time branch exists, and the trap is triggered.

## Consequences

**Positive:**
- UEAS becomes the first algorithm standard capable of formally verifying side-channel immunity at the pseudocode level, massive adoption driver for security researchers.

**Negative:**
- Symbolic execution (branching the Virtual Heap to test both paths) exponentially increases interpretation time for deeply nested conditionals. 

## Alternatives Considered

1. **Static Analysis Only:** We considered building a static linter to look for `Secret` variables in `If` statements. However, static analysis often produces false positives. The abstract interpreter's dynamic step-counter provides an exact, mathematically sound verification.
