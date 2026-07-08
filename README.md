# Universal Executable Algorithm Standard (UEAS)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Spec Version](https://img.shields.io/badge/Spec-1.0.0--draft-orange)](SPEC.md)

**Write once. Verify complexity. Run everywhere.**

UEAS is a formal specification and reference implementation for
representing, validating, and transpiling algorithms in a language-agnostic,
mathematically rigorous format. It defines a canonical Abstract Syntax Tree
(AST), a grammar for human-readable algorithmic pseudocode, an abstract
interpreter with built-in complexity-invariant enforcement, and a pluggable
transpilation target interface.

---

## The Problem

Algorithms form the intellectual foundation of modern software systems. Yet
there is no standard way to write them down.

- **Academics** publish pseudocode in LaTeX — unparsable, untestable, locked
  on paper.
- **Engineers** re-implement the same pseudocode in Python, Rust, C++ — each
  translation introducing semantic drift and regression risk.
- **Interviewers** evaluate algorithmic thinking on whiteboards — no
  validation, no profiling, no context-free correctness guarantee.

Meanwhile, formal verification tools (TLA+, Coq) demand PhD-level expertise,
and general-purpose transpilers (Haxe, Nim) carry hardware baggage — memory
management, I/O, system calls — that has no place in a pure algorithm
definition.

**There is no standard for algorithms themselves.**

UEAS fills this gap. It treats algorithmic logic as a first-class, deployable
asset — decoupled entirely from programming language syntax, hardware
constraints, and execution environment.

### What UEAS Is NOT

UEAS is **not a new programming language.** It does not build web servers,
manage databases, or render user interfaces. It has no network sockets, no
filesystem access, no hardware primitives. It is a **mathematical blueprint**
— a pure, verified logic specification that compiles *into* your existing
programming languages. If you need a general-purpose language, use Rust,
Python, or C++. If you need to prove that your algorithm is correct, use UEAS.

---

## Architecture

```
 +----------+     +-----------+     +----------+     +------------------+
 | .ueas    | --> |  ANTLR4   | --> |  Canonical | --> |  Rust Abstract   |
 | Source   |     |  Parser   |     |  JSON AST  |     |  Interpreter     |
 +----------+     +-----------+     +----------+     +------------------+
                                                           |
                                             +-------------+-------------+
                                             |             |             |
                                        +--------+   +--------+   +--------+
                                        | Python |   |  Rust  |   |  C++   |
                                        | Target |   | Target |   | Target |
                                        +--------+   +--------+   +--------+
```

**Three decoupled domains, connected by the canonical JSON AST:**

1. **Front-End (Grammar & Parsing)** — ANTLR4 grammar (`UEAS.g4`) ingests
   human-readable pseudocode and produces a typed, validated AST. The grammar
   is the normative definition of valid UEAS syntax.

2. **Core Kernel (Abstract Interpreter)** — A Rust-based virtual machine
   executes the AST in an isolated heap with zero system I/O. It counts
   abstract logical steps — not wall-clock time — enabling deterministic,
   hardware-independent complexity profiling. An invariant engine traps
   execution if declared complexity bounds are breached.

3. **Back-End (Transpilation Engine)** — A plugin architecture maps the
   verified AST to idiomatic source code. Each target language (Python, Rust,
   C++, Java) implements the `TargetGenerator` interface. An MCP endpoint
   enables AI agents to generate context-specific code directly from the AST.

---

## Why This Is Different

Most tools manage hardware, network traffic, or user interfaces. UEAS treats
pure logic as a first-class asset.

| Existing Paradigm | Why It Falls Short |
|-------------------|-------------------|
| **Formal Verification (TLA+, Coq, Alloy)** | Prove state-machine correctness for distributed systems. Impossible to execute, transpile, or profile. Steep learning curve. |
| **General-Purpose Transpilers (Haxe, Nim)** | Full programming languages. Include I/O, memory management, platform APIs. Too heavy for pure algorithm definition. |
| **Intermediate Representations (LLVM IR, MLIR)** | Built for compilers, not humans. Unreadable to algorithm designers. |
| **Academic Pseudocode (LaTeX, algorithm2e)** | Purely visual. Cannot be parsed, executed, tested, or debugged. |
| **UEAS** | Human-readable, machine-executable, complexity-profiling, language-agnostic. Algorithm logic as a deployable, testable artifact. |

### Immutable Axioms

To guarantee mathematical verification, UEAS enforces these design boundaries as
matters of principle. They are not temporary constraints — they are the source
of the standard's power.

1. **Zero System I/O.** The kernel has no access to network, filesystem, or
   hardware. Algorithms are isolated, pure state mutations. This guarantees
   deterministic, reproducible complexity profiling regardless of environment.

2. **Abstract Step-Counting.** Wall-clock profiling is forbidden. All
   computational cost is measured in abstract step mutations — each primitive
   operation has a fixed, spec-defined cost. An `O(N log N)` contract yields
   identical complexity bounds on any hardware.

3. **Specification Before Implementation.** The canonical AST is the source of
   truth. Implementation follows the specification, never the reverse.
   Behavior not specified in SPEC.md has no place in the reference
   implementation.

---

## High-Impact Domains

| Domain | Problem UEAS Solves |
|--------|-------------------|
| **AI Agent Orchestration** | LLM-based coding agents struggle with syntax quirks and dependency management. UEAS provides a minimal logical grammar that reduces the agent's search space. Generated ASTs can be mathematically verified to terminate and meet complexity bounds *before* a single line of production code is emitted — directly addressing the hallucination problem in AI code generation. |
| **Quantitative Finance** | Quants prototype algorithms in Python; engineers manually rewrite in C++ for low-latency execution. UEAS eliminates translation lag — write once, verify mathematically, transpile to production. |
| **Aerospace & Defense** | DO-178C certification requires proving source code matches mathematical specification. UEAS ASTs carry explicit pre/post-conditions and invariants — enabling automated validation against formal requirements. |
| **Academic Publishing** | Research papers publish algorithms as static text. With UEAS, readers can download, execute, profile, and transpile — ending the era of untestable pseudocode. |

---

## Concrete Example: Multilevel TSP Solver

A traveling salesman algorithm in UEAS:

```
algorithm MultilevelTSP(cities: List<City>, initialUpperBound: Real) -> Tour
    @Complexity("O(N^2)", N = length(cities))
{
    let clusters: List<Set<City>> := partitionIntoClusters(cities, 10);

    invariant(length(cities) > 0):
        "Input must contain at least one city";

    for cluster: Set<City> in clusters {
        let localTour: Tour := solveExactTSP(cluster);
        clusterTourMap[cluster] := localTour;
    }

    let macroNodes: List<Point> := centroidsOf(clusters);
    let macroTour: Tour := solveApproximateTSP(macroNodes);
    let globalTour: Tour := stitchClusters(clusterTourMap, macroTour);

    globalTour := localSearch2Opt(globalTour);

    assert(tourLength(globalTour) <= initialUpperBound):
        "Final tour must not exceed the initial upper bound";

    return globalTour;
}
```

The AST produced by this algorithm can be:
- **Executed** in the kernel sandbox with step-count profiling;
- **Validated** against its declared `O(N^2)` contract;
- **Transpiled** to Python, Rust, or C++ with guaranteed semantic equivalence.

---

## Roadmap

| Epoch | Timeline | Scope | Deliverable |
|-------|----------|-------|-------------|
| **1: Combinatorial Core** | Months 1-3 | ANTLR4 grammar, static type system, AST JSON schema | Parser with 100% accuracy on algorithmic benchmarks |
| **2: Profiling Kernel** | Months 4-6 | Rust abstract interpreter, complexity-invariant engine, virtual heap, error traps | Kernel that flags infinite loops and complexity violations |
| **3: Universal Bridge** | Months 7-9 | Transpiler plugin system, Python + Rust targets, MCP API, cross-target equivalence suite | Mathematically identical outputs across all targets |

---

## Quick Links

| Document | Purpose |
|----------|---------|
| [SPEC.md](SPEC.md) | Formal specification — the mathematical source of truth |
| [AGENTS.md](AGENTS.md) | Conventions for AI agents working on UEAS |
| [CONTRIBUTING.md](docs/CONTRIBUTING.md) | How to contribute, setup, quality gates |
| [CONTRIBUTORS.md](CONTRIBUTORS.md) | List of contributors to the project |
| [CLA.md](docs/CLA.md) | Contributor License Agreement |
| [RFC Process](docs/rfcs/README.md) | How to propose changes to the standard |
| [ADR Log](docs/adr/README.md) | Architecture Decision Records |

---

## License & Governance

UEAS is released under the [Apache License 2.0](LICENSE). All contributors
must sign a Contributor License Agreement (CLA).

Governance follows a formal RFC process: **Draft → Review → Ratification →
Implementation**. The specification is updated **before** code is written.

---

*UEAS is a community-driven standard. Contributions, proposals, and feedback are welcome.*
