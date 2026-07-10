# ADR 0009: Distributed Systems Simulation Model

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

Distributed algorithms (e.g., Paxos, Raft, Byzantine Fault Tolerance) represent a massive class of computer science algorithms. Currently, UEAS cannot specify or profile these because Axiom 1 explicitly forbids network I/O to guarantee determinism.

## Decision

To support Epoch 11, we will introduce a **Virtual Network Topology** within the abstract interpreter. 
Instead of opening real network sockets, the kernel will simulate multiple independent "nodes" (each with its own Virtual Heap and step counter) inside a single process.
Algorithms will use new grammar constructs: `send message to node` and `on receive`. The kernel acts as a deterministic virtual switch, passing messages between node heaps in memory.

## Consequences

**Positive:**
- We can officially specify and test distributed algorithms deterministically.
- We introduce two new profiling metrics: **Message Complexity** (total bytes sent) and **Round Complexity** (network hops).

**Negative:**
- The kernel becomes significantly more complex, as it now manages a multi-node simulation environment rather than a single monolithic heap.

## Alternatives Considered

1. **Real Network I/O (Sockets):** Rejected. Breaks Axiom 1, making execution non-deterministic and destroying reproducibility.
2. **Actor Model (Erlang style):** This is essentially what the Virtual Network Topology is, but specialized for algorithmic step-counting rather than general-purpose concurrent programming.
