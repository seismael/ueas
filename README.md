# Universal Executable Algorithm Standard (UEAS)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-201-brightgreen)](kernel/)
[![Kernel](https://img.shields.io/badge/Kernel-Rust-red)](kernel/)
[![Grammar](https://img.shields.io/badge/Grammar-ANTLR4-lightgrey)](grammar/)
[![Library](https://img.shields.io/badge/Algorithms-45-blue)](library/)

> The Universal Executable Algorithm Standard (UEAS). An executable, language-agnostic algorithmic ecosystem. UEAS allows algorithms to be written universally, mathematically verified, natively debugged, and profiled. Eliminating language-specific constraints for technical evaluations, academic research, data science, and core engineering.
> 
> **Topics:** `executable-pseudocode`, `algorithm-standard`, `jupyter-kernel`, `debug-adapter-protocol`, `cryptography-verification`

---

## The Core Intention

Currently, platforms like LeetCode, academic publishers, and technical organizations must support dozens of programming languages (Python, Java, C++, Rust) just to evaluate a single algorithm. This forces engineers to wrestle with language-specific syntax, memory management, and compiler quirks rather than focusing purely on the mathematics of the solution.

**UEAS solves this.** It is not just a visual pseudocode; it is an executable, compilable standard. 

### Uniqueness & Benefit
By standardizing how algorithms are written, UEAS introduces a paradigm shift in logical evaluation:

1. **Universal Evaluation (The "LeetCode" Paradigm):** Technical interviews and competitive programming platforms no longer need to maintain complex, multi-language execution environments. Candidates write a single, universal UEAS algorithm, and the abstract interpreter natively tests, validates, and profiles its correctness.
2. **Deterministic Mathematical Verification:** UEAS does not evaluate algorithms using fluctuating wall-clock time. It utilizes a zero-I/O Virtual Heap to explicitly count logical step mutations, enabling strict Big-O complexity enforcement at runtime.
3. **Write Once, Transpile Anywhere:** Once an algorithm's pure logic is tested and validated within the UEAS standard, it can be autonomously transpiled into production-ready, memory-safe languages (Rust, C++, Python) without human translation errors.

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
                                                           |   |   |
                                       +-------------------+   |   +-------------------+
                                       |                       |                       |
                               +---------------+       +---------------+       +---------------+
                               |  Transpilers  |       | DAP Debugger  |       | Jupyter Kernel|
                               | (Rust/Python) |       | (VS Code)     |       | (ZeroMQ)      |
                               +---------------+       +---------------+       +---------------+
```

**Three decoupled domains, connected by the canonical JSON AST:**

1. **Front-End (Grammar & Parsing)** — ANTLR4 grammar (`UEAS.g4`) ingests
   human-readable pseudocode and produces a typed, validated AST. The grammar
   supports module linking (`Import`), Stochastic Primitives, and Streams.

2. **Core Kernel (Abstract Interpreter)** — A Rust-based virtual machine
   executes the AST in an isolated heap with zero system I/O. It counts
   abstract logical steps — not wall-clock time — enabling deterministic,
   hardware-independent complexity profiling. An invariant engine traps
   execution if declared complexity bounds are breached.

3. **Back-End (Transpilation Engine)** — A plugin architecture maps the
    verified AST to idiomatic source code. Five target languages (Python,
    Rust, C++17, Java 17, JavaScript ES2020) implement the `TargetGenerator`
    interface. An MCP endpoint enables AI agents to generate context-specific
    code directly from the AST.

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
| **Academic Data Science** | Researchers prototype in Python, struggling to profile memory and hardware caches. UEAS integrates directly as a **Jupyter Kernel**, allowing algorithmic modeling, `@HardwareProfile` simulation, and formal complexity checking directly inside `.ipynb` notebooks. |
| **Cybersecurity (Cryptography)** | Cryptographic implementations are highly vulnerable to timing side-channel attacks. UEAS introduces `@ConstantTime` execution constraints, forcing the abstract interpreter to symbolically execute both conditional branches to formally guarantee zero timing leaks. |
| **AI Agent Orchestration** | LLM-based coding agents struggle with syntax quirks and dependency management. UEAS provides a minimal logical grammar that reduces the agent's search space. Generated ASTs can be mathematically verified to terminate and meet complexity bounds *before* a single line of production code is emitted. |
| **Quantitative Finance** | Quants prototype algorithms in Python; engineers manually rewrite in C++ for low-latency execution. UEAS eliminates translation lag — write once, verify mathematically, transpile to production. |
| **Aerospace & Defense** | DO-178C certification requires proving source code matches mathematical specification. UEAS ASTs carry explicit pre/post-conditions and invariants — enabling automated validation against formal requirements. |
| **Academic Publishing** | Research papers publish algorithms as static text. With UEAS, readers can download, execute, profile, and transpile — ending the era of untestable pseudocode. |

---

## Concrete Example: Multilevel TSP Solver

A traveling salesman algorithm in UEAS:

```
Algorithm MultilevelTSP(cities, initialUpperBound)
    Require: cities: List<City>, initialUpperBound: Real
    Ensure: Tour
    Complexity: "O(N^2)", N = length(cities)

    invariant(length(cities) > 0, "Input must contain at least one city")

    clusters <- partitionIntoClusters(cities, 10)

    for each cluster in clusters do
        localTour <- solveExactTSP(cluster)
        clusterTourMap[cluster] <- localTour
    end for

    macroNodes <- centroidsOf(clusters)
    macroTour <- solveApproximateTSP(macroNodes)
    globalTour <- stitchClusters(clusterTourMap, macroTour)

    globalTour <- localSearch2Opt(globalTour)

    assert(tourLength(globalTour) <= initialUpperBound,
        "Final tour must not exceed the initial upper bound")

    return globalTour
```

The AST produced by this algorithm can be:
- **Executed** in the kernel sandbox with step-count profiling;
- **Validated** against its declared `O(N^2)` contract;
- **Transpiled** to Python, Rust, or C++ with guaranteed semantic equivalence.

---

## Getting Started

### Install

```bash
cargo install ueas
```

### CLI Usage

```bash
# Parse and execute an algorithm
ueas run examples/euclidean.ueas

# Validate syntax
ueas check library/sorting/quicksort.ueas

# Transpile to a target language
ueas transpile examples/linear_search.ueas --target python
ueas transpile examples/linear_search.ueas --target rust
```

### Standard Library

UEAS ships with a comprehensive [standard algorithm library](library/INDEX.md) —
**45 verified algorithms** across 7 categories:

| Category | Count | Highlights |
|----------|-------|------------|
| Sorting | 8 | quicksort, mergesort, heapsort, insertion, counting, radix, bucket, timsort |
| Searching | 3 | ternary search, jump search, exponential search |
| Graph | 9 | BFS, DFS, Dijkstra, Kruskal, Prim, Floyd-Warshall, Bellman-Ford, topological sort, Kosaraju |
| Dynamic Programming | 5 | LCS, knapsack (0/1), edit distance, matrix chain, coin change |
| Mathematics | 8 | GCD, prime sieve, fast power, Miller-Rabin, FFT, CRT, extended Euclid, modular inverse |
| String Algorithms | 5 | KMP, Rabin-Karp, Boyer-Moore, Z-algorithm, suffix array |
| Data Structures | 6 | BST, AVL tree, min-heap, union-find, trie, segment tree |

Validate all library algorithms:
```bash
ueas check library/sorting/quicksort.ueas
```

---

## Roadmap & Status

| Epoch | Scope | Status |
|-------|-------|--------|
| **1: Combinatorial Core** | ANTLR4 grammar, type system, AST JSON schema | ✅ Complete |
| **2: Profiling Kernel** | Abstract interpreter, complexity engine, virtual heap | ✅ Complete |
| **3: Universal Bridge** | Transpiler plugins, Python + Rust targets, MCP API | ✅ Complete |
| **4: Developer Tooling** | `ueas` CLI (run, check, transpile, fmt) | ✅ Complete |
| **5: Standard Library** | 45 algorithms in 7 categories | ✅ Complete |
| **6: Backend Expansion** | C++17, Java 17, JavaScript transpilers | ✅ Complete |
| **7: Concurrency** | Work/Span model, parallel for, spawn/join | 🔨 Planned |
| **8: Formal Verification** | Lean 4 + TLA+ backend transpilers | 🔨 Planned |
| **9: WASM Playground** | Browser-based IDE, URL-shareable algorithms | ⚡ In Progress |

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
