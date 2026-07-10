# ADR 0011: Tensor Architecture for Machine Learning Workloads

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

While UEAS supports a generic `Matrix<R, C, T>` type, modern Machine Learning algorithms (e.g., Transformers, Convolutional Neural Networks) require operations over highly dimensional data structures with implicit broadcasting semantics. Attempting to specify these using nested `for` loops and primitive `Matrix` structures results in extremely verbose pseudocode that fails to map cleanly to industry-standard tools (like NumPy, PyTorch, or JAX).

## Decision

We will introduce a native `Tensor<T, Dims...>` primitive to the UEAS canonical AST.
- **Implicit Broadcasting:** Tensors of compatible shapes will support implicit broadcasting for element-wise operations (`+`, `*`).
- **Complexity Profiling:** The abstract interpreter will automatically compute the broadcasted footprint and multiply the step count accordingly. For example, broadcasting a `[10, 1]` tensor with a `[1, 5]` tensor yields a step cost of `50`.
- **Transpilation:** Tensors will map natively to `numpy.ndarray` in Python, and `ndarray::Array` or `tch::Tensor` in Rust.

## Consequences

**Positive:**
- Immediately captures the massive Machine Learning and Data Science audience.
- AI orchestration agents (a key demographic for UEAS per `SPEC.md`) can write pure mathematical ML pipelines that are deterministically verified before being transpiled to GPU-accelerated PyTorch code.

**Negative:**
- The kernel's type checker becomes significantly more complex, as it must now perform shape inference and broadcasting compatibility checks at compile time.

## Alternatives Considered

1. **Relying solely on `List<List<...>>`:** Rejected. Multi-dimensional lists require manual loop unrolling, ruining code readability and making complexity tracking extremely verbose for standard ML operations.
