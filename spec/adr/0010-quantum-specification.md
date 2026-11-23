# ADR 0010: Quantum Algorithm Specification

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

Quantum algorithms (e.g., Shor's Algorithm, Grover's Search) require data types and operations that do not exist in classical computing. To ensure UEAS remains a future-proof, universal standard, we must incorporate quantum primitives.

## Decision

We will introduce Quantum Data Types (`Qubit`, `QRegister`) and Quantum Gate operations (`Hadamard`, `CNOT`, `Measure`) into the UEAS EBNF Grammar and Canonical JSON AST.
Crucially, the abstract interpreter **will not** attempt to simulate the quantum state vectors (which requires exponential classical memory). Instead, the kernel will perform **Static Circuit Validation** (ensuring adherence to the No-Cloning Theorem, checking qubit reference bounds) and track **Quantum Circuit Depth** as the complexity metric. 
The validated AST will then be transpiled by new backends into `Qiskit` (Python) or `OpenQASM` for actual quantum execution.

## Consequences

**Positive:**
- UEAS becomes one of the only language-agnostic standards capable of describing both classical and quantum algorithms with deterministic profiling.
- Transpiling to OpenQASM bridges UEAS directly to real quantum hardware.

**Negative:**
- The kernel cannot natively assert the *correctness* of quantum output via `invariant` statements (since it cannot simulate the wave function collapse), relying instead on post-transpilation execution on a real QPU or classical simulator like Aer.

## Alternatives Considered

1. **Full Quantum Simulation in the Kernel:** Rejected. Simulating more than ~40 qubits requires supercomputer-level RAM. The UEAS kernel is a lightweight profiler, not a heavy quantum physics simulator. Static validation and transpilation is the correct domain mapping.
