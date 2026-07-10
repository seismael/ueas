# Universal Executable Algorithm Standard (UEAS)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-277-brightgreen)](kernel/)
[![Kernel](https://img.shields.io/badge/Kernel-Rust-red)](kernel/)  
[![Grammar](https://img.shields.io/badge/Grammar-ANTLR4-lightgrey)](grammar/)
[![Library](https://img.shields.io/badge/Algorithms-45-blue)](library/)
[![Examples](https://img.shields.io/badge/Examples-30-orange)](examples/)

> **An executable, language-agnostic algorithmic ecosystem.** UEAS allows algorithms to be written universally, mathematically verified, natively debugged, and profiled. Eliminating language-specific constraints for technical evaluations, academic research, data science, and core engineering.

**Topics:** `executable-pseudocode`, `algorithm-standard`, `jupyter-kernel`, `debug-adapter-protocol`, `cryptography-verification`

---

## 1. The Problem & The Solution

Algorithms form the intellectual foundation of modern software systems, yet there is no standard way to write them down. 

- **Academics** publish pseudocode in LaTeX — unparsable, untestable, locked on paper.
- **Engineers** re-implement the same pseudocode in Python, Rust, C++ — each translation introducing semantic drift and regression risk.
- **Interviewers** evaluate algorithmic thinking on whiteboards — offering no validation or context-free correctness guarantee.

Existing formal verification tools (TLA+, Coq) demand PhD-level expertise and cannot execute code. General-purpose transpilers (Haxe, Nim) carry hardware baggage — memory management, network I/O, system calls — that have no place in a pure algorithm definition.

**UEAS solves this.** UEAS treats algorithmic logic as a first-class, deployable asset decoupled entirely from programming language syntax, hardware constraints, and execution environments. 

### What UEAS Is NOT
UEAS is **not a new programming language.** It does not build web servers, manage databases, or render UIs. It has zero network sockets and zero filesystem access. It is a **mathematical blueprint** — a pure, verified logic specification that complies *into* your existing programming languages. If you need a general-purpose language, use Rust or Python. If you need to mathematically prove that your algorithm is correct, use UEAS.

---

## 2. Architecture

```text
 +----------+     +-----------+     +----------+     +------------------+
 | .ueas    | --> |  ANTLR4   | --> |  Canonical | --> |  Rust Abstract   |
 | Source   |     |  Parser   |     |  JSON AST  |     |  Interpreter     |
 +----------+     +-----------+     +----------+     +------------------+
                                                           |   |   |
                                       +-------------------+   |   +-------------------+
                                       |                       |                       |
                               +---------------+       +---------------+       +---------------+
                               |  Transpilers  |       | DAP Debugger  |       | Jupyter Kernel|
                               | (Rust/Python) |       |  (VS Code)    |       |   (ZeroMQ)    |
                               +---------------+       +---------------+       +---------------+
```

The UEAS ecosystem is divided into three decoupled domains, connected by the canonical JSON AST:

1. **Front-End (Grammar & Parsing):** The ANTLR4 grammar (`UEAS.g4`) ingests human-readable pseudocode and produces a typed, validated AST. The grammar supports advanced algorithmic primitives including Module Linking (`Import`), Stochastic Primitives (`random`), and Infinite Streams.
2. **Core Kernel (Abstract Interpreter):** A Rust-based virtual machine executes the AST in an isolated Virtual Heap with zero system I/O. It counts abstract logical steps rather than wall-clock time, enabling deterministic complexity profiling regardless of your CPU speed. 
3. **Ecosystem Tooling:** The verified AST feeds into Transpiler plugins (generating idiomatic Python, Rust, C++, Java), a Debug Adapter Protocol (DAP) server for step-through debugging, and a ZeroMQ Jupyter Kernel for academic data science workflows.

---

## 3. High-Impact Domains

| Domain | Problem UEAS Solves |
|--------|-------------------|
| **Academic Data Science** | Researchers prototype in Python, struggling to profile memory and hardware caches. UEAS integrates directly as a **Jupyter Kernel**, allowing algorithmic modeling, `@HardwareProfile` simulation, and formal complexity checking directly inside `.ipynb` notebooks. |
| **Cybersecurity (Cryptography)** | Cryptographic implementations are highly vulnerable to timing side-channel attacks. UEAS introduces `@ConstantTime` execution constraints, forcing the abstract interpreter to symbolically execute both conditional branches to formally guarantee zero timing leaks. |
| **AI Agent Orchestration** | LLM-based coding agents struggle with syntax quirks and dependency management. UEAS provides a minimal logical grammar that reduces the agent's search space. Generated ASTs can be mathematically verified to terminate and meet complexity bounds *before* a single line of production code is emitted. |
| **Quantitative Finance** | Quants prototype algorithms in Python; engineers manually rewrite in C++ for low-latency execution. UEAS eliminates translation lag — write once, verify mathematically, transpile to production. |
| **Aerospace & Defense** | DO-178C certification requires proving source code matches mathematical specification. UEAS ASTs carry explicit pre/post-conditions and invariants — enabling automated validation against formal requirements. |
| **Academic Publishing** | Research papers publish algorithms as static text. With UEAS, readers can download, execute, profile, and transpile — ending the era of untestable pseudocode. |

---

## 4. Concrete Example: Multilevel TSP Solver

A traveling salesman algorithm in UEAS:

```ueas
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
- **Executed** in the kernel sandbox with step-count profiling.
- **Validated** against its declared `O(N^2)` contract.
- **Transpiled** to Python, Rust, or C++ with guaranteed semantic equivalence.

---

## 5. Getting Started

### Installation
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
UEAS ships with a comprehensive [standard algorithm library](library/INDEX.md) containing **45 verified algorithms** across 7 categories (Sorting, Searching, Graphs, Dynamic Programming, Mathematics, Strings, Data Structures).

### Examples
**30 reference implementations** organized by algorithmic technique:

| Category | Count | Highlights |
|----------|-------|------------|
| core | 4 | Euclidean, Linear/Binary Search, Matrix Multiply |
| sorting | 3 | Quicksort, Merge Sort, Randomized QuickSort |
| graph | 3 | BFS, DFS, Dijkstra |
| dynamic_programming | 4 | Kadane, Regex Match, Longest Substring, Median of Two Sorted |
| backtracking | 4 | Subsets, N-Queens, Sudoku, Word Ladder |
| arrays | 7 | Two Sum, Sliding Window, Trap Rain Water, Dutch Flag |
| stack | 2 | Valid Parentheses, Largest Rectangle |
| heap | 1 | Top K Frequent |
| design | 1 | LRU Cache |
| intervals | 1 | Merge Intervals |

All examples validated with `ueas check examples/<category>/<name>.ueas`.

---

## 6. Roadmap & Status

| Phase | Scope | Status |
|-------|-------|--------|
| **Phase 1: Combinatorial Core** | ANTLR4 grammar, type system, AST JSON schema | ✅ Complete |
| **Phase 2: Profiling Kernel** | Abstract interpreter, complexity engine, virtual heap | ✅ Complete |
| **Phase 3: Universal Bridge** | Transpilers (Python, Rust, C++, Java), CLI, Standard Library | ✅ Complete |
| **Phase 4: Real-World Ecosystem** | DAP Debugger, Jupyter Kernel, `@HardwareProfile`, Modules | 🔨 Planned |
| **Phase 5: Core Math Frontiers** | Stochastic `random`, Infinite `Stream<T>`, `@ConstantTime` Cryptography | 🔨 Planned |
| **Phase 6: Formal Verification** | Lean 4 + TLA+ backend transpilers | 🔨 Planned |

---

## 7. Quick Links

| Document | Purpose |
|----------|---------|
| [SPEC.md](SPEC.md) | Formal specification — the mathematical source of truth |
| [AGENTS.md](AGENTS.md) | Conventions for AI agents working on UEAS |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute, setup, quality gates |
| [CONTRIBUTORS.md](CONTRIBUTORS.md) | List of contributors to the project |
| [CLA.md](CLA.md) | Contributor License Agreement |
| [RFC Process](docs/rfcs/README.md) | How to propose changes to the standard |
| [ADR Log](docs/adr/README.md) | Architecture Decision Records |

---

## 8. License & Governance

UEAS is released under the [Apache License 2.0](LICENSE). All contributors must sign the Contributor License Agreement (CLA) before submitting pull requests.

Governance follows a formal RFC process: **Draft → Review → Ratification → Implementation**. The specification is strictly updated **before** code is written.

---

*UEAS is a community-driven standard. Contributions, proposals, and feedback are welcome.*
