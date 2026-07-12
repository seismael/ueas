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

> **A formal, machine-executable standard for algorithms.** Write once in academic pseudocode. Execute in an isolated virtual machine that counts logical steps, enforces complexity contracts, and detects timing leaks. Transpile to 6+ languages through a mathematically-verified Dafny bridge. No hardware baggage. No I/O. Just pure algorithmic logic, proven correct.

**Topics:** `algorithm-standard`, `formal-verification`, `academic-pseudocode`, `transpiler`

---

## Live Deployments

| Service | URL | Purpose |
|---------|-----|---------|
| **Playground** | [ueas-three.vercel.app](https://ueas-three.vercel.app) | Write, transpile, audit algorithms in your browser |
| **MCP Server** | [ueas-mcp.seismael.workers.dev](https://ueas-mcp.seismael.workers.dev) | 8 API tools for AI agents (parse, execute, transpile, audit...) |
| **Dafny Verifier** | [ueas-verify-504087134780.us-central1.run.app](https://ueas-verify-504087134780.us-central1.run.app/health) | Z3 theorem proofs + code generation for C++, Python, Java, etc. |

---

## 1. The Problem

Algorithms are the intellectual foundation of software, but there is no standard way to write them down.

- **Academics** publish pseudocode in LaTeX — beautiful, but unparsable, untestable, and locked on paper.
- **Engineers** re-implement the same algorithm in Python, Rust, C++ — each translation introducing bugs, regressions, and semantic drift.
- **AI agents** generate code with no mathematical guarantee of correctness or complexity.

Existing formal verification tools (TLA+, Coq, Lean) require PhD-level expertise and don't produce runnable code. General-purpose transpilers (Haxe) carry hardware baggage — memory management, I/O, system calls — that have no place in a pure algorithm definition.

## What UEAS Is

UEAS is **not a programming language.** It doesn't build web servers, manage databases, or render UIs. It has zero network sockets and zero filesystem access.

It is a **mathematical blueprint** — a formal, machine-executable specification for algorithms. Write once in academic pseudocode that looks like a textbook. Run it in an isolated virtual machine that counts **abstract logical steps** — the same algorithm produces the same step count on any hardware. Deploy to production through a single Dafny transpilation bridge that provides Z3 theorem proofs before generating C++, Python, Java, Go, C#, or JavaScript.

### What You Get

| Capability | How |
|-----------|-----|
| **Write algorithms universally** | Academic pseudocode with `Require:`/`Ensure:`/`Complexity:` preambles — looks like a textbook |
| **Execute deterministically** | Virtual Heap with zero I/O. Logical step counting, not wall-clock time |
| **Enforce complexity contracts** | `Complexity: "O(N log N)"` is NOT a comment — it's enforced at runtime (trap code 5) |
| **Prove correctness mathematically** | Transpile to Dafny, run Z3 theorem prover (via Google Cloud Run, scale-to-zero) |
| **Detect timing leaks** | `@ConstantTime` block with `Secret<T>` variables triggers symbolic execution of both branches |
| **Profile hardware impact** | `@HardwareProfile(L1=64KB, L2=512KB)` simulates real cache hierarchy |
| **Deploy anywhere** | One source → Dafny → C++, Python, Java, Go, C#, JavaScript |
| **Work with AI agents** | MCP server exposes 8 tools for LLM integration via standard protocol |

---

## 2. Architecture

```text
 .ueas Source → ANTLR4 Parser → JSON AST → Rust Abstract Interpreter
                                               |     |     |
                                     Transpilers  DAP   Jupyter
                                          |
                                      MCP Server (8 tools, CF Workers)
                                          ^
                                          |  fetch()
                                      Playground (Vercel)
```

Three decoupled domains connected by the canonical JSON AST:

1. **Grammar** — ANTLR4 parser that ingests academic pseudocode and produces a validated AST. Case-insensitive keywords. Supports `import`, `random()`, `Stream<T>`, `Secret<T>`, `@ConstantTime`, `@HardwareProfile`, `parallel`, `spawn`, `measure`, `yield`, and 13 trap codes.

2. **Abstract Interpreter** — Rust virtual machine executing in an isolated Virtual Heap with zero system I/O. Counts **abstract logical steps** — the same algorithm produces identical step counts on any hardware. Includes Work/Span DAG profiler for parallel algorithms and LRU-based cache simulation.

3. **Dafny Bridge** — Single transpiler generating verifiable Dafny code. `requires`/`ensures`/`invariant` clauses map directly to UEAS preambles. Z3 SMT solver proves correctness before `dafny build` generates production code. Lean 4, TLA+, and LaTeX targets serve theorem proving, model checking, and academic publishing as separate concerns.

---

## 3. Who Needs This

| Domain | Why |
|--------|-----|
| **Academic Research** | Publish algorithms that readers can execute, profile, and verify — ending the era of untestable pseudocode |
| **AI Agent Developers** | LLMs generate UEAS → MCP server verifies complexity → proven code before deployment |
| **Cryptography** | `@ConstantTime` forces symbolic execution of both branches — mathematically guarantees zero timing leaks |
| **Technical Interviews** | Candidates write algorithms; the system verifies correctness and complexity — no subjective grading |
| **Aerospace & Defense** | DO-178C certification via explicit pre/post-conditions and formal proofs |
| **Quantitative Finance** | Write once in UEAS, transpile to C++ for low-latency execution |
| **Computer Science Education** | Students learn algorithms in a syntax that looks like their textbook, with instant execution feedback |

---

## 4. Example

```ueas
Algorithm EuclideanDistance(x1, y1, x2, y2)
    Require: x1: Real, y1: Real, x2: Real, y2: Real
    Ensure: Real
    Complexity: "O(1)"

    dx <- x2 - x1
    dy <- y2 - y1
    return sqrt(dx * dx + dy * dy)
```

What happens when you run this:
- **Parse** → validated AST with 4 parameters, return type Real, complexity O(1)
- **Execute** → 3 abstract steps, 80 bytes heap, deterministic on any hardware
- **Transpile** → Dafny source → Z3 verifies → generates C++, Python, Java, or JavaScript
- **Audit** → Reverse-engineer existing Python/Dafny code back into UEAS

---

## 5. Getting Started

```bash
cd implementation
cargo install --path ../tools/ueas-cli

# Parse and execute an algorithm
ueas run examples/core/euclidean.ueas          # → steps=3, heap=80B, exit_code=0

# Transpile to Dafny (then dafny build --target:cpp)
ueas transpile examples/core/euclidean.ueas --target dafny

# Reverse-audit existing Python code
ueas check examples/core/linear_search.ueas
```

**Library:** 45 verified algorithms across 7 categories — [browse them](implementation/library/INDEX.md).

**MCP Server:** 8 tools for AI agents — [test live](https://ueas-mcp.seismael.workers.dev).

---

## 6. Roadmap — All Complete

| Phase | Scope |
|-------|-------|
| **1-3** | Grammar v3.0, Abstract Interpreter, Python+Rust transpilers |
| **4-6** | CLI, 45-algo Library, C++/Java/JS transpilers, DAP, Jupyter, Cryptographic, Streams |
| **7** | Concurrency (spawn/sync/parallel for), Quantum (Qubit/Measure), Tensor/ML |
| **8** | Dafny Single-Target Architecture, Bidirectional Auditing, GCP Verification Pipeline |

---

## 7. Quick Links

| Document | Purpose |
|----------|---------|
| [SPEC.md](specification/SPEC.md) | Formal specification — the mathematical source of truth |
| [AGENTS.md](AGENTS.md) | Development protocol for AI agents |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Setup, quality gates, contribution workflow |
| [RFC Process](specification/docs/rfcs/README.md) | How to propose specification changes |
| [ADR Log](specification/docs/adr/README.md) | Architecture Decision Records (17 entries) |
| [Library](implementation/library/INDEX.md) | 45 algorithms across 7 categories |
| [Examples](examples/) | 45 reference implementations in 20 categories |
| [Deployment Guides](docs/deployment/) | Vercel, Cloudflare, Google Cloud Run setup |

---

## 8. License & Governance

Apache License 2.0. Governance follows a formal RFC process: **Draft → Review → Ratification → Implementation**. Specification updated **before** code is written. All contributors must sign the CLA.

*UEAS is a community-driven standard. Contributions, proposals, and feedback are welcome.*
