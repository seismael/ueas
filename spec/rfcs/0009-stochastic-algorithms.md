# RFC 0009: Stochastic Algorithms & Randomness

- **Domain:** grammar | kernel
- **Date:** 2026-07-10
- **Status:** Ratified

## Summary
Introduces primitives for mathematical randomness and probabilistic complexity bounding. This allows the representation and verification of stochastic algorithms such as Monte Carlo simulations and Randomized Quicksort.

## Specification

### 1. Grammar Extension (`UEAS.g4`)
Add the `random` built-in function to expressions:
```antlr4
expression : 'random(' expression ',' expression ')' ;
```
Add the `Expected` block to the `Complexity` annotation:
```antlr4
complexityBlock : 'Complexity:' (complexityRule)+ ;
complexityRule : ('WorstCase' | 'BestCase' | 'Expected') ':' 'O(' expression ')' ;
```

### 2. Kernel PRNG
The Virtual Heap execution state will be initialized with a deterministic pseudo-random number generator (PRNG) seeded at runtime. A new command-line flag `--seed <num>` will allow reproducible fuzzing of stochastic paths.

### 3. Expected Complexity Profiling
When the kernel encounters an `Expected: O(...)` annotation, it will transparently run the algorithm $N$ times (default $N=100$) using different random seeds and average the step-count results to ensure they conform to the expected probability distribution bounds.
