# ADR 0005: Concurrency via Work/Span DAG Profiling

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

Epoch 7 introduces concurrency and parallelism to the UEAS standard to support modern algorithms (e.g., Parallel Merge Sort, MapReduce). However, UEAS's core value proposition is **deterministic step-counting (Axiom 2)**. If we allow arbitrary multi-threading governed by wall-clock time, complexity verification becomes non-deterministic and hardware-dependent, completely breaking the standard. We must find a way to model concurrent computation mathematically.

## Decision

We will adopt the **Work/Span (DAG) Complexity Model**.
Instead of tracking execution as a linear counter, the UEAS abstract interpreter will track execution as a Directed Acyclic Graph (DAG) of logical steps. 
- **Work** represents the total number of operations performed across all branches of the DAG.
- **Span** (or critical path) represents the longest path through the DAG from root to sink.

The standard will introduce `spawn` and `sync` for task parallelism, and `parallel for` for data parallelism. Each thread of execution maintains its own step counter, which is joined back to the main branch upon synchronization.

## Consequences

**Positive:**
- Complexity profiling remains 100% deterministic and hardware-agnostic.
- Algorithms can assert dual complexity bounds, e.g., `Complexity: Work = O(N log N), Span = O(log^2 N)`.
- Naturally prevents data races by mapping well to UEAS's existing strict single-ownership memory model (spawned tasks take ownership of passed arguments).

**Negative:**
- The kernel's `StepCounter` must be fundamentally rewritten to support a stack of DAG branch counters, increasing the runtime overhead of the abstract interpreter.

## Alternatives Considered

1. **Wall-clock threading (`std::thread` equivalent):** Rejected because it violates deterministic profiling. 
2. **Actor Model (Message Passing):** Rejected. While mathematically pure, it does not cleanly map to classical divide-and-conquer algorithm textbooks (like CLRS) which rely heavily on fork-join parallelism.
