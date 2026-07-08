# AGENTS.md — UEAS Development Conventions

## Project Identity

UEAS (Universal Executable Algorithm Standard) is a formal specification and
reference implementation for algorithm representation, validation, and
transpilation. The project is structured as three decoupled domains governed
by Domain-Driven Design (DDD) and SOLID principles.

**Mission:** Provide a mathematically rigorous, language-agnostic standard for
algorithms with guaranteed complexity-invariant enforcement and semantically
equivalent transpilation.

**Repository root:** `ueas/`

---

## Domain Boundaries (DDD)

```
 grammar/          kernel/           backends/
 (ANTLR4)          (Rust)            (Plugin System)
     |                 |                  |
     v                 v                  v
  Parsing &        Execution &       Code Generation
  AST Validation   Invariant Check   & MCP API
```

- **`grammar/`** — The ANTLR4 grammar (`UEAS.g4`), lexer/parser rules, AST
  listener/visitor stubs. Owns the definition of valid UEAS syntax.
- **`kernel/`** — The Rust abstract interpreter. Owns the virtual heap, step
  counter, trap register, invariant engine, and complexity enforcement. Must
  have zero system I/O, network, or hardware dependencies.
- **`backends/`** — Transpiler plugins implementing `TargetGenerator`. Each
  target is an isolated crate/module. Owns target-specific code generation.
- **`docs/`** — RFCs, ADRs, and domain specifications. The source of truth
  for the standard text.
- **`tools/`** — CI scripts, fuzzing harnesses, benchmark runners, container
  definitions.

**No cross-domain imports.** `grammar/` does not depend on `kernel/`. `kernel/`
does not depend on `backends/`. Communication is via the canonical JSON AST.

---

## Strict Conventions

### SOLID Principles

| Principle | Application |
|-----------|-------------|
| **S**ingle Responsibility | Each file does exactly one thing. AST nodes are pure data. Visitors are separate. |
| **O**pen/Closed | The `TargetGenerator` interface is open for new language backends, closed for modification. |
| **L**iskov Substitution | Any `TargetGenerator` implementation must produce semantically equivalent output. |
| **I**nterface Segregation | AST visitors implement only the callbacks they need. No fat interfaces. |
| **D**ependency Inversion | The kernel depends on the AST schema, not on the parser. The parser depends on the grammar, not on the kernel. |

### Design Patterns

- **Visitor Pattern:** AST traversal in the kernel uses the Visitor pattern.
  Each node kind has an `accept(visitor)` method.
- **Strategy Pattern:** Transpilation targets are Strategy implementations of
  `TargetGenerator`.
- **Observer Pattern:** Complexity profiling subscribes to kernel step events.
- **Factory Pattern:** AST node construction is centralized in a factory module
  to enforce schema invariants.

### Test-Driven Development (TDD)

**Red-Green-Refactor is mandatory for all kernel logic.**

1. Write a failing test for the desired behavior.
2. Write the minimum code to make the test pass.
3. Refactor while keeping tests green.

Tests for AST evaluation logic MUST be written before kernel parser
implementation. See [SPEC.md Section 9](SPEC.md#9-conformance-and-compliance)
for the UCTS requirements.

---

## Toolchain

| Component | Tool | Notes |
|-----------|------|-------|
| Grammar | ANTLR4 4.13+ | Generates parser in target language of choice |
| Kernel | Rust 1.75+ (stable) | `cargo build`, `cargo test`, `cargo clippy` |
| Python utilities | Python 3.11+ | `ruff check --fix`, `ruff format` |
| Fuzzing | `proptest` (Rust) | Property-based testing for kernel robustness |
| Containers | Docker | CI/CD reproducibility |
| Linting (Rust) | `cargo clippy -- -D warnings` | Zero warnings required |
| Formatting (Rust) | `cargo fmt --check` | Standard Rust style |
| Linting (Python) | `ruff check --fix` | Zero issues required |
| Formatting (Python) | `ruff format` | Standard Python style |

---

## Quality Gates

Before merging any PR, the following MUST pass:

```
# Rust (kernel/)
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# Python (tools/, scripts/)
ruff check --fix
ruff format --check

# Property-based fuzzing
cargo test --test fuzz -- --ignored

# Grammar benchmarks
# (to be defined in Epoch 1)
```

A PR that does not pass all gates MUST NOT be merged.

---

## RFC Workflow for Agents

When tasked with a spec change or new feature:

1. **Check** if an RFC already exists in `docs/rfcs/` covering the change.
2. If no RFC exists, **draft one** using the template in
   `docs/rfcs/README.md`. Name it `docs/rfcs/NNNN-title.md` (next available
   number).
3. Set status to `Draft` and submit for review.
4. **Do not write implementation code** until the RFC status is `Ratified`.
5. Once ratified, update `SPEC.md` to reflect the change.
6. Implement in the appropriate domain (`grammar/`, `kernel/`, `backends/`).
7. Write tests per TDD convention.
8. Run all quality gates.
9. Mark RFC as `Implemented`.

---

## Directory Map

```
ueas/
├── AGENTS.md              This file
├── README.md              Project overview
├── SPEC.md                Formal specification
├── LICENSE                Apache License 2.0
├── NOTICE                 Apache copyright notice
├── CONTRIBUTORS.md        List of contributors
├── .github/               GitHub templates
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       └── feature_request.md
├── grammar/               ANTLR4 grammar files (Epoch 1)
│   └── UEAS.g4            (future)
├── kernel/                Rust abstract interpreter (Epoch 2)
│   └── Cargo.toml         (future)
├── backends/              Transpiler plugins (Epoch 3)
│   ├── python/            (future)
│   ├── rust/              (future)
│   └── cpp/               (future)
├── tools/                 CI, fuzzing, benchmarks
│   └── Dockerfile         (future)
└── docs/
    ├── CONTRIBUTING.md    Contribution guide and CLA
    ├── CLA.md             Contributor License Agreement
    ├── rfcs/              RFC proposals (numbered)
    │   └── README.md      RFC template and process
    ├── adr/               Architecture Decision Records
    │   └── README.md      ADR format and index
    ├── specs/             Detailed per-domain specifications
    │   └── README.md
    └── meeting-notes/     Community meeting archives
        └── README.md
```

---

## Design Rationale

The following decisions are intentional and non-negotiable. Understanding the
rationale prevents drift into architectural anti-patterns.

| Decision | Why |
|----------|-----|
| **Abstract step-counting, not wall-clock time** | Wall-clock profiling is environment-dependent — it fluctuates with CPU scheduling, cache topology, and concurrent workload. Step-counting produces an absolute, deterministic complexity curve that is identical on any hardware. This is necessary for the kernel to objectively enforce `@Complexity` contracts. |
| **Zero system I/O in the kernel** | The kernel is the verification layer. System I/O introduces side effects, non-determinism, and security surface. An algorithm that depends on file reads or network calls cannot be verified in isolation. The kernel sandbox guarantees reproducibility. |
| **Three-domain decoupling (grammar, kernel, backends)** | Each domain has a distinct concern. Coupling them creates the "architecture drift" problem that killed UML-based code generation — when the blueprint and the implementation fall out of sync, contributors abandon the standard. The canonical JSON AST is the only interface contract between domains. |
| **Static explicit typing** | Type inference creates ambiguity about what the algorithm author intended. A variable that the parser infers as `Integer` may have been intended as `Real`. Explicit types make the algorithm self-documenting and enable the parser to reject type-mismatch programs at parse time rather than at kernel runtime. |
| **`unsafe` requires documented justification** | The kernel claims mathematical certainty. `unsafe` blocks circumvent Rust's memory safety guarantees. Every `unsafe` usage must have a written bug filed and a `// SAFETY:` comment explaining invariant preservation. This is the Apache standard for auditable code. |
| **Kernel never panics — all failures are trap codes** | A panic is an uncontrolled crash. A trap code is a controlled, documented error. Downstream tools (CI, fuzzing, transpilers) must be able to programmatically distinguish between "the algorithm is wrong" (trap) and "the kernel is broken" (panic). |
| **RFC ratification before implementation** | Reversing the standard-engineering relationship — where the spec is updated after code is written — is how standards rot. The Apache model requires the specification to lead and the reference implementation to follow. |

---

## No-Go Zones

The following are **strictly forbidden** in the kernel domain:

- System I/O (file read/write, stdin/stdout)
- Network access (sockets, HTTP)
- Hardware calls (GPU, DMA, peripherals)
- Unsafe blocks (Rust `unsafe`) without documented, reviewed justification
- Panics that are not caught and converted to trap codes
- Modifying the AST during execution (the AST is immutable)

The following are **strictly forbidden** project-wide:

- Bypassing the RFC process for spec changes
- Merging code without corresponding tests
- Skipping the CLA requirement for external contributions
- Introducing non-Apache-2.0-compatible dependencies

---

## Vocabulary

Use the terminology defined in [SPEC.md Section 2](SPEC.md#2-definitions-and-terminology).
Do not invent new terms. If a concept lacks a term, propose one via RFC.
