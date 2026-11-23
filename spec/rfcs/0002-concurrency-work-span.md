# RFC 0002: Concurrency and the Work/Span Complexity Model

- **Status:** Ratified
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

UEAS is currently a purely sequential standard. However, modern algorithms (e.g., Parallel Merge Sort, MapReduce, GPU-accelerated algorithms) heavily rely on concurrency. 
To maintain UEAS's status as a universal standard, it must support expressing concurrent operations without sacrificing its deterministic step-counting and strict memory safety guarantees. We cannot simply add threads and rely on wall-clock time, as that violates Axiom 2 (Abstract Step-Counting).

We must introduce concurrent grammar constructs and upgrade the Complexity Profiler to track **Work** (the total number of operations across all threads) and **Span** (the length of the longest critical path in the computation DAG).

## Proposed Change

We propose two primary grammatical additions to support both task parallelism and data parallelism.

1. **Task Parallelism (`spawn` and `sync`)**
```ueas
Algorithm ParallelFibonacci(n)
    Require: n: Integer
    Ensure: Integer
    Complexity: "Work: O(2^N), Span: O(N)"

    if n <= 1 then
        return n
    end if

    let x <- spawn ParallelFibonacci(n - 1)
    let y <- spawn ParallelFibonacci(n - 2)
    sync
    return x + y
```

2. **Data Parallelism (`parallel for`)**
```ueas
parallel for each i in [0 .. n-1] do
    A[i] <- A[i] * 2
end for
```

## Impact Analysis

### Grammar changes
- Add `SPAWN`, `SYNC`, and `PARALLEL` keywords to the lexer.
- Add a `spawnExpression` rule: `spawn` functionCall.
- Add a `syncStmt` rule: `sync`.
- Update `forLoop` to allow an optional `parallel` modifier: `parallel for each IDENTIFIER in expression do block end for`.

### AST schema changes
- Add `"Spawn"` node kind. Fields: `expression: Expression`.
- Add `"Sync"` node kind.
- Modify `"ForLoop"` node to include `isParallel: boolean`.

### Kernel changes
- **Complexity Profiler (DAG Model):** The kernel must maintain a Computation Directed Acyclic Graph (DAG) for the current execution trace.
  - Every standard instruction adds a node to the current task's chain (Work += 1, Span += 1).
  - `spawn` creates a new branch in the DAG.
  - `sync` joins all child branches to the parent. 
  - When the algorithm terminates, **Work** is the total number of nodes in the DAG, and **Span** is the longest path from root to sink.
- **Trap Codes:** Add `DEADLOCK_DETECTED` (if a spawn cycle is somehow formed, though statically unlikely in strict trees) and `CONCURRENCY_VIOLATION` (if Work or Span exceeds their respective contracts).
- **Virtual Heap:** Single-ownership must be rigidly enforced. Spawning a task transfers ownership of passed arguments to the spawned task (or passes immutable `const` references). Data races must trap immediately.

### Backward compatibility
- Fully backward compatible. Sequential algorithms simply have `Work == Span`.

## Alternatives Considered

1. **Explicit Threads / Mutexes:** Rejected. Shared-memory concurrency with explicit locking destroys determinism and makes formal verification nearly impossible. The Work/Span DAG model (based on strictly structured `spawn/sync` and `parallel for`) enforces deterministic data flow.
2. **Actor Model (Message Passing):** Considered, but less aligned with classical algorithmic pseudocode (CLRS usually relies on nested parallel loops).

## Reference Implementation Plan

- **Affected Domains:** `grammar/` (parser additions), `kernel/` (DAG profiler and memory isolation rules), and `backends/` (`rayon` for Rust, `std::thread` for C++, `java.util.concurrent` for Java).
- **Epoch Alignment:** Epoch 7.
- **Dependencies:** None for the kernel (abstract execution). Transpilers may require standard libraries for threads.
