# Universal Executable Algorithm Standard (UEAS)

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-292-brightgreen)](implementation/kernel/)
[![Kernel](https://img.shields.io/badge/Kernel-Rust-red)](implementation/kernel/)  
[![Grammar](https://img.shields.io/badge/Grammar-ANTLR4-lightgrey)](specification/grammar/)
[![Library](https://img.shields.io/badge/Algorithms-45-blue)](implementation/library/)
[![Examples](https://img.shields.io/badge/Examples-45-orange)](examples/)
[![Playground](https://img.shields.io/badge/Playground-Live-success)](https://ueas-three.vercel.app/)
[![MCP Server](https://img.shields.io/badge/MCP%20Server-Live-success)](https://ueas-mcp.seismael.workers.dev)
[![Version](https://img.shields.io/badge/Version-5.0.1-blue)](VERSION)

> **An executable, language-agnostic algorithmic ecosystem.** UEAS allows algorithms to be written universally, mathematically verified, natively debugged, and profiled. Eliminating language-specific constraints for technical evaluations, academic research, data science, and core engineering.

**Topics:** `executable-pseudocode`, `algorithm-standard`, `jupyter-kernel`, `debug-adapter-protocol`, `cryptography-verification`, `reverse-audit`

---

## Live Deployments

- **Interactive Playground (Vercel):** [https://ueas-three.vercel.app/](https://ueas-three.vercel.app/) — Write UEAS algorithms directly in your browser with a bidirectional editor (UEAS ↔ target language). All execution, transpilation, and auditing is powered exclusively by the MCP server — no local WASM required. The playground is a pure API client.
- **MCP AI Agent Server (Cloudflare Workers):** [https://ueas-mcp.seismael.workers.dev](https://ueas-mcp.seismael.workers.dev) — Connect your AI agents (Claude, Cursor, etc.) to the UEAS interpreter via the Model Context Protocol. Always-on, globally distributed edge deployment via WASM-compiled Rust kernel.

---

## 1. The Problem & The Solution

Algorithms form the intellectual foundation of modern software systems, yet there is no standard way to write them down. 

- **Academics** publish pseudocode in LaTeX — unparsable, untestable, locked on paper.
- **Engineers** re-implement the same pseudocode in Python, Rust, C++ — each translation introducing semantic drift and regression risk.
- **Interviewers** evaluate algorithmic thinking on whiteboards — offering no validation or context-free correctness guarantee.

Existing formal verification tools (TLA+, Coq) demand PhD-level expertise and cannot execute code. General-purpose transpilers (Haxe, Nim) carry hardware baggage — memory management, network I/O, system calls — that have no place in a pure algorithm definition.

**UEAS solves this.** UEAS treats algorithmic logic as a first-class, deployable asset decoupled entirely from programming language syntax, hardware constraints, and execution environments. 

### Why UEAS? (Strategic Focus)
UEAS is currently being optimized for two high-leverage domains critical for enterprise adoption:
1. **AI Interoperability:** LLMs and autonomous coding agents struggle with language quirks. UEAS serves as the mathematically rigorous intermediary—an agent generates a UEAS algorithm, the MCP Server validates its Big-O complexity, then transpiles it to Dafny for verification or directly to Lean 4, TLA+, or LaTeX.
2. **Cryptographic Rigor:** Cryptographic implementations require strict mathematical bounds. UEAS introduces `@ConstantTime` execution and `Secret` variables, using symbolic execution to catch timing leaks and prove hardware-level security guarantees before deployment.

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
                               | Dafny+Formal |       |  (VS Code)    |       |   (ZeroMQ)    |
                               +---------------+       +---------------+       +---------------+
                                       |
                               +---------------+
                               |  MCP Server   |  <-- Cloudflare Workers (WASM kernel at edge)
                               |  (8 tools)    |
                               +---------------+
                                       ^
                                       |  fetch()
                               +---------------+
                               |  Playground   |  <-- Vercel (pure API client, no local WASM)
                               +---------------+
```

The UEAS ecosystem is divided into three decoupled domains, connected by the canonical JSON AST:

1. **Front-End (Grammar & Parsing):** The ANTLR4 grammar (`UEAS.g4`) ingests human-readable pseudocode and produces a typed, validated AST. The grammar supports advanced algorithmic primitives including Module Linking (`Import:`), Stochastic Primitives (`random`), Infinite Streams (`Stream<T>`), and Cryptographic guarantees (`Secret<T>`, `@ConstantTime`).
2. **Core Kernel (Abstract Interpreter):** A Rust-based virtual machine executes the AST in an isolated Virtual Heap with zero system I/O. It counts abstract logical steps rather than wall-clock time, enabling deterministic complexity profiling regardless of your CPU speed. The kernel now includes cache simulation (`@HardwareProfile`), symbolic execution for timing leak detection, and a pseudo-random number generator for stochastic algorithms.
3. **Dafny Transpilation Bridge & Formal Verification:** A single DafnyTarget provides Z3 theorem proofs and generates production code in C++, Python, Java, Go, C#, and JavaScript via `dafny build`. Lean 4, TLA+, and LaTeX targets serve formal verification and academic publishing as distinct concerns.

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
- **Transpiled** to Dafny (Z3-verified code for C++, Python, Java, Go, C#, JavaScript).

---

## 5. Getting Started

### Installation
```bash
cd implementation
cargo install --path ../tools/ueas-cli
```

### CLI Usage
```bash
# Parse and execute an algorithm
ueas run examples/core/euclidean.ueas

# Validate syntax
ueas check implementation/library/sorting/quicksort.ueas

# Transpile to a target language
ueas transpile examples/core/linear_search.ueas --target dafny
ueas transpile examples/core/linear_search.ueas --target lean4
```

### Standard Library
UEAS ships with a comprehensive [standard algorithm library](implementation/library/INDEX.md) containing **45 verified algorithms** across 7 categories (Sorting, Searching, Graphs, Dynamic Programming, Mathematics, Strings, Data Structures).

### MCP Server (AI Agents)

UEAS provides an always-on MCP server for AI agent integration, deployed on
Cloudflare Workers (8 tools, globally distributed, zero cold starts). Z3 mathematical
proofs and Dafny code generation run on Google Cloud Run (scale-to-zero, $0 idle):

```
https://ueas-mcp.seismael.workers.dev
```

**Dafny Verification Backend (GCP):**
```
https://ueas-verify-504087134780.us-central1.run.app/health
```

Available tools: `parse`, `execute`, `transpile`, `verify`, `hardware`, `complexity`, `memory`, `audit`.
Connect via Claude Desktop, Cursor, or any MCP-compatible client.

### Examples
**45 reference implementations** (30 algorithmic + 15 feature demos) organized across 20 categories:

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

### Feature Examples
**15 specification capability demos** organized by feature domain:

| Category | Count | Demonstrated Features |
|----------|-------|----------------------|
| concurrency | 2 | spawn/sync, parallel for, Work-Span |
| distributed | 1 | send/receive messaging |
| quantum | 1 | Qubit type, Measure statement |
| tensor | 1 | Tensor<T,Dims>, broadcasting |
| cryptographic | 1 | Secret<T>, @ConstantTime |
| stochastic | 1 | random(), Expected complexity |
| streams | 2 | Stream<T>, yield/await |
| hardware | 1 | @HardwareProfile cache config |
| modules | 1 | Import: namespace resolution |
| advanced | 4 | const, Graph, Directed, Memory, Infinity, NaN |

All examples validated with `ueas check examples/<category>/<name>.ueas`.

---

## 6. Roadmap & Status

| Phase | Scope | Status |
|-------|-------|--------|
| **Phase 1-6** | Grammar, Kernel, Backends, CLI, Library, Frontends | ✅ Complete |
| **Phase 4-6 (Ecosystem)** | DAP, Jupyter, Hardware, Modules, Cryptography, Streams | ✅ Complete |
| **Phase 7 (Concurrency, Quantum, ML)** | Work-Span, distributed, qubit, tensor primitives | ✅ Complete |
| **Phase 8 (Bidirectional Auditing)** | audit tool, Python→UEAS reverse audit, I/O violation detection | ✅ Complete |

---

## 7. Quick Links

| Document | Purpose |
|----------|---------|
| [SPEC.md](specification/SPEC.md) | Formal specification — the mathematical source of truth |
| [AGENTS.md](AGENTS.md) | Conventions for AI agents working on UEAS |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute, setup, quality gates |
| [CONTRIBUTORS.md](CONTRIBUTORS.md) | List of contributors to the project |
| [CLA.md](CLA.md) | Contributor License Agreement |
| [RFC Process](specification/docs/rfcs/README.md) | How to propose changes to the standard |
| [ADR Log](specification/docs/adr/README.md) | Architecture Decision Records |
| [Library](implementation/library/INDEX.md) | 45 standard algorithms across 7 categories |
| [Examples](examples/) | 45 reference implementations in 20 categories |

---

## 8. License & Governance

UEAS is released under the [Apache License 2.0](LICENSE). All contributors must sign the Contributor License Agreement (CLA) before submitting pull requests.

Governance follows a formal RFC process: **Draft → Review → Ratification → Implementation**. The specification is strictly updated **before** code is written.

---

*UEAS is a community-driven standard. Contributions, proposals, and feedback are welcome.*
