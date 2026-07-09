# AGENTS.md — UEAS Development Protocol

This document is the **authoritative, non-negotiable protocol** for all
development work on UEAS. Every agent, contributor, and maintainer MUST
follow this protocol exactly. Deviation is not permitted.

## Architecture & Documentation Stability

The architecture, specification, and core documentation of UEAS are
**sealed** as of the current baseline. This means:

- **Specification (SPEC.md):** Immutable. No changes to grammar rules, AST
  schema, kernel semantics, type system, or transpiler contracts are
  permitted without a ratified RFC. See [RFC Workflow](#rfc-workflow).

- **Architecture (AGENTS.md, domain boundaries):** Immutable. No changes
  to Domain-Driven Design boundaries, SOLID enforcement rules, design pattern
  mandates, or architectural decisions are permitted without a corresponding
  Architecture Decision Record (ADR) and explicit approval from the project
  maintainer.

- **Core Documentation (README.md, LICENSE, NOTICE):** Immutable. No changes
  permitted without explicit approval from the project maintainer.

- **Contributor Documentation (CONTRIBUTING.md, CLA.md):** Changes permitted
  only via the RFC process.

**What IS permitted without approval:**

- Adding new documentation files that **extend and align** with existing
  documentation (e.g., new `docs/specs/` files, tutorial content, meeting
  notes, new ADRs documenting decisions consistent with the current
  architecture, new RFC drafts).
- Adding new code in `grammar/`, `kernel/`, `backends/`, or `tools/` that
  conforms to the existing specification and architecture.
- Adding new tests, benchmarks, fuzzing strategies, CI configurations.
- Fixing typos, broken links, or formatting issues in documentation.

**What IS NOT permitted without approval:**

- Any documentation change that contradicts, removes, or reinterprets
  existing architectural decisions, SOLID/DDD principles, domain boundaries,
  or specification text.
- Any code change that introduces new dependencies, modifies domain
  boundaries, adds I/O to the kernel, or alters the canonical AST schema.
- Any change that relaxes quality gates, coverage thresholds, or testing
  requirements.

**Conflict Resolution:** If an agent discovers a gap, inconsistency, or
potential improvement that conflicts with existing architecture or
specification, the agent MUST:
1. Document the finding as a new RFC draft in `docs/rfcs/`.
2. Present the concern to the project maintainer.
3. Await explicit approval before making any changes.

**Policy Enforcement:** Any pull request that modifies a sealed document
without explicit approval will be rejected. Any agent instruction that
conflicts with this policy must be declined with a reference to this section.

---

## Table of Contents

1. [Project Identity](#project-identity)
2. [Architecture & Documentation Stability](#architecture--documentation-stability)
3. [Domain Boundaries](#domain-boundaries)
4. [Development Principles](#development-principles)
5. [Architecture & Design Enforcement](#architecture--design-enforcement)
6. [Documentation Enforcement](#documentation-enforcement)
7. [Testing Enforcement](#testing-enforcement)
8. [Code Quality Enforcement](#code-quality-enforcement)
9. [RFC Workflow](#rfc-workflow)
10. [Toolchain](#toolchain)
11. [Quality Gates](#quality-gates)
12. [Design Rationale](#design-rationale)
13. [No-Go Zones](#no-go-zones)
14. [Vocabulary](#vocabulary)
15. [Directory Map](#directory-map)

---

## Project Identity

UEAS (Universal Executable Algorithm Standard) is a formal specification and
reference implementation for algorithm representation, validation, and
transpilation. The project is structured as three decoupled domains governed
by Domain-Driven Design (DDD) and SOLID principles.

**Mission:** Provide a mathematically rigorous, language-agnostic standard for
algorithms with guaranteed complexity-invariant enforcement and semantically
equivalent transpilation.

**UEAS is not a programming language.** It is a mathematical blueprint
specification. It does not build applications, manage memory, or handle I/O.
It defines, verifies, and transpiles algorithms. The canonical AST is the
source of truth; implementation follows specification.

**Repository root:** `ueas/`

---

## Domain Boundaries

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

## Development Principles

These principles govern all development work. Process details (task sequencing,
checklists) are managed by `.loop/LOOP_AGENTS.md` and `TODO.md`.

### Core Rules (Non-Negotiable)

1. **Specification before implementation.** Code is never written before the
   corresponding SPEC.md update. If the change alters specification semantics,
   a ratified RFC is required.

2. **Tests before code (TDD).** Every new feature or bug fix begins with a
   failing test. Implementation follows to make the test pass.

3. **Documentation alongside code.** Every `pub` item carries a doc comment.
   Module-level documentation (`//!`) explains architectural role. SPEC.md
   is updated BEFORE implementation.

4. **Quality gate before commit.** Every commit passes:
   ```
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   ```

5. **Refactor while green.** After tests pass, eliminate duplication, improve
   naming, reduce complexity. Never refactor with failing tests.

6. **Verify completely.** Run fuzz tests, cross-target equivalence, and
   grammar benchmarks for any change touching those domains.

7. **Review required.** Every PR needs maintainer approval. CI must be green.
   Squash-merge with Conventional Commits.

8. **Error handling through traps.** No bare `unwrap()`, `expect()`, or
   `panic!()` in production code. All failures route through kernel trap codes.

### Coding Flow

```
Analyze → Test (Red) → Implement (Green) → Refactor → Verify → Commit
```

Each step gates the next. No step is skipped. For the full phase-by-phase
checklist, see `.loop/LOOP_AGENTS.md`.

---

## Architecture & Design Enforcement

### Object-Oriented Design (OOD)

- **Encapsulation:** Internal state is private. Access is through
  well-defined methods. No public fields except in pure data structures
  (AST nodes, DTOs).
- **Inheritance:** Use composition over inheritance. Where inheritance is
  necessary (e.g., `TargetGenerator`), document why composition is
  insufficient.
- **Polymorphism:** Behavior variation through interfaces (traits), not
  through type-checking (`match` on enum with type-specific logic should
  be replaced with trait dispatch where possible).

### Domain-Driven Design (DDD)

- **Ubiquitous Language:** Code identifiers MUST use the terminology from
  SPEC.md Section 2. No synonyms. If SPEC.md says "Virtual Heap," the code
  says `VirtualHeap`, not `MemoryArena` or `Sandbox`.
- **Aggregate Roots:** Each domain module has one aggregate root that owns
  all entities within. External code accesses the domain through this root
  only.
- **Domain Events:** Cross-domain communication is through events, not
  direct function calls. Grammar produces `ParsedAST` event. Kernel consumes
  it. Kernel produces `ExecutedAST` event. Backend consumes it.
- **Bounded Contexts:** `grammar/`, `kernel/`, and `backends/` are separate
  bounded contexts. Each has its own internal model. Translation between
  contexts happens at the boundary via the canonical JSON AST.

### SOLID Principles (Mandatory)

| Principle | Enforcement |
|-----------|-------------|
| **S**RP | One reason to change per class. If a class handles both parsing and validation, split it. |
| **O**CP | New transpilation targets are added via new modules implementing `TargetGenerator` — never by modifying existing generator code. |
| **L**SP | Every `TargetGenerator` implementation must produce semantically equivalent output. Tests verify this. |
| **I**SP | No "God interfaces." `ASTVisitor` has one method per node kind. A visitor that needs only 3 callbacks implements only those 3. |
| **D**IP | High-level policy (kernel) depends on the AST schema abstraction, not on the ANTLR4 concrete parser. Parser depends on grammar, not vice versa. |

### Gang of Four Design Patterns (Mandatory Where Specified)

| Pattern | Where | Why |
|---------|-------|-----|
| **Visitor** | All AST traversal in kernel | Decouples operations from node structure. Adding a new operation never requires modifying node types. |
| **Strategy** | Every transpilation target | Each language backend is a strategy. The kernel selects the appropriate strategy at transpile time. |
| **Observer** | Complexity profiling, invariant monitoring | Kernel emits step events. Multiple observers (profiler, invariant checker, debugger) react independently. |
| **Factory** | All AST node construction | Ensures every node is created with valid invariants. Direct `new()` calls on AST types are forbidden outside the factory module. |
| **Command** | Kernel operations that mutate state | Each operation (allocate, assign, evaluate) is a command object. Enables undo/redo in the debugger and step replay. |
| **Singleton** | Kernel configuration, step counter, trap register | One instance per kernel execution. Global state access through controlled, testable interfaces. |

### Anti-Patterns (Strictly Forbidden)

- **God Class:** No class with more than 10 public methods.
- **Feature Envy:** A method that accesses another object's data more than
  its own must be moved to that object.
- **Shotgun Surgery:** A change that requires modifying more than 3 files
  in unrelated modules indicates poor cohesion. Refactor before proceeding.
- **Primitive Obsession:** Use domain types (`StepCount`, `TrapCode`,
  `VirtualAddress`) — never raw `u64`, `i32`, or `usize` for domain concepts.
- **Magic Numbers:** Every numeric literal in non-test code must be a named
  constant with a doc comment explaining its origin.

---

## Documentation Enforcement

### Documentation Must Precede Code

Documentation is NOT an afterthought. The pipeline enforces doc-before-code
in Phase 2. This is non-negotiable.

### Documentation Update Triggers

| Change Type | Documentation Required |
|-------------|----------------------|
| New grammar rule | SPEC.md Section 4 updated, AST node kind added to Section 5, example added |
| New AST node kind | SPEC.md Section 5 updated, Visitor interface updated in doc comments |
| New kernel operation | SPEC.md Section 6.2 updated (step cost), 6.5 updated (new trap code if any) |
| New transpilation target | SPEC.md Section 7.2 updated, ADR written |
| Modified step cost | SPEC.md Section 6.2 updated, complexity violation tests updated |
| New invariant type | SPEC.md Section 6.3 updated |
| New error condition | SPEC.md Section 6.5 updated (new trap code, description, cause) |
| Public API change | Module doc comments updated, README.md updated if user-facing |
| Architectural decision | ADR written in `docs/adr/NNNN-title.md` |
| RFC implementation | RFC status updated, SPEC.md updated, CHANGELOG entry added |

### Doc Comment Standard (Rust)

```rust
/// Brief one-line summary.
///
/// Detailed description of behavior, preconditions, and postconditions.
///
/// # Arguments
/// * `param_name` — Description of the parameter.
///
/// # Returns
/// Description of the return value.
///
/// # Errors
/// List of trap codes or error conditions this function can produce.
///
/// # Complexity
/// Step-cost impact of this operation.
///
/// # Examples
/// ```
/// // Working example that compiles and runs in doctests
/// ```
///
/// # Panics
/// Only if the function can panic. Prefer trap codes.
///
/// # Safety
/// Only if the function is `unsafe`. Describe the invariants the caller
/// must uphold.
```

---

## Testing Enforcement

### Testing Pyramid

```
           ╱  E2E  ╲          Cross-target equivalence, full pipeline
          ╱──────────╲
         ╱ Integration ╲       Cross-domain: parse→execute, execute→transpile
        ╱────────────────╲
       ╱   Property-Based  ╲    proptest: random valid inputs, zero panics
      ╱──────────────────────╲
     ╱      Unit Tests        ╲  One per function, per code path, per edge case
    ╱──────────────────────────╲
```

### Test Quantity Requirements

| Change Scope | Minimum Tests Required |
|-------------|----------------------|
| New public function (kernel) | 1 happy path + 1 null/empty + 1 error condition = 3 minimum |
| New AST node evaluator (kernel) | 1 normal input + 1 edge case + 1 property-based = 3 minimum |
| New grammar rule (grammar) | 1 valid parse + 1 invalid parse rejection + 1 AST structure verification = 3 minimum |
| New transpiler target (backends) | 1 basic algorithm + 1 complex algorithm + cross-target equivalence on 7 benchmarks = 9 minimum |
| Bug fix | 1 regression test that fails before the fix and passes after |

### Edge Case Coverage Checklist

Every kernel function that accepts input MUST be tested with:

- [ ] Null / None / empty input
- [ ] Single-element input
- [ ] Input at declared maximum size
- [ ] Negative values (for numeric inputs that could be negative)
- [ ] Zero values
- [ ] Duplicate values (for collections)
- [ ] Input that triggers the worst-case complexity path
- [ ] Input that triggers each defined error condition

### Coverage Requirements

| Domain | Minimum Line Coverage | Enforced By |
|--------|----------------------|-------------|
| `kernel/src/interp/` | 95% | CI gate |
| `kernel/src/ast/` | 90% | CI gate |
| `kernel/src/*.rs` (utilities) | 85% | CI gate |
| `backends/<target>/` | 80% | CI gate |
| `grammar/` | 100% benchmark parse accuracy | CI gate |

Coverage regressions block merge. No exceptions.

### Fuzzing Requirements

Before every merge to `main`, the kernel MUST pass:

- `cargo test --test fuzz -- --ignored`
- Minimum 10^6 random inputs generated by `proptest`
- Zero panics, zero crashes, zero undefined behavior

Fuzzing strategies MUST generate:
- Valid ASTs of varying size and structure
- Edge cases: empty programs, single-node programs, maximally nested programs
- Type-correct and type-incorrect inputs
- Random complexity annotations and invariant expressions

---

## Code Quality Enforcement

### Rust (`kernel/`)

```rust
// REQUIRED: Module-level documentation
//! Virtual heap implementation for the UEAS abstract interpreter.
//!
//! The virtual heap is a contiguous byte array isolated from the host
//! operating system. It provides allocation, deallocation, and type-aware
//! access primitives used by the interpreter's execution engine.

use std::collections::HashMap;

/// Handle to an allocation in the virtual heap.
///
/// Wraps a 64-bit address. Not constructable outside the heap module
/// to prevent forged addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapHandle(u64);

/// Manages the isolated memory space for algorithm execution.
///
/// # Complexity
/// Allocations are O(1). Deallocations are O(1). Access is O(1).
///
/// # Errors
/// Returns `HEAP_EXHAUSTION` trap code if allocation exceeds configured size.
pub struct VirtualHeap {
    memory: Vec<u8>,
    allocations: HashMap<HeapHandle, AllocationMetadata>,
    config: HeapConfig,
}

impl VirtualHeap {
    /// Allocates a region of the given size and returns a handle.
    ///
    /// # Arguments
    /// * `size` — Number of bytes to allocate. Must be > 0.
    /// * `type_tag` — The UEAS type of the allocation for metadata tracking.
    ///
    /// # Returns
    /// A `HeapHandle` that can be used to read from or write to the
    /// allocated region.
    ///
    /// # Errors
    /// Returns `TrapCode::HEAP_EXHAUSTION` if the allocation would exceed
    /// the configured heap size.
    ///
    /// # Complexity
    /// O(1) — bump-pointer allocation.
    pub fn allocate(&mut self, size: usize, type_tag: TypeTag) -> Result<HeapHandle, TrapCode> {
        // SAFETY: The allocation size is bounds-checked against the
        // remaining heap capacity. The returned handle is unique.
        // See: https://github.com/seismael/ueas/issues/<issue_number>
        todo!()
    }
}
```

**Rust Conventions:**

- Edition: 2021
- Lint: `cargo clippy -- -D warnings` — zero warnings. No `#[allow(clippy::*)]`
  without a filed GitHub issue.
- Format: `rustfmt` with default settings.
- Module structure: One module per concept. `ast/`, `interp/`, `heap/`,
  `invariants/`, `traps/`, `profiling/`.
- `unsafe`: Forbidden without a filed GitHub issue AND a reviewer-approved
  `// SAFETY:` comment. The issue must remain open as long as `unsafe`
  exists in the codebase.

### ANTLR4 (`grammar/`)

- Version: 4.13.2+
- Single `UEAS.g4` file with combined lexer and parser grammar.
- Rule grouping: Lexer rules, then parser rules grouped as:
    1. Top-level (program, algorithm, complexity annotation)
    2. Statements (variableDecl, assignment, ifStmt, forLoop, whileLoop, etc.)
    3. Expressions (by precedence, lowest to highest)
    4. Types (typeAnnotation, primitiveType, compositeType)
- Alternatives: Each alternative on its own line with `|` prefix.
- Section dividers: `(* ===== Section Name ===== *)` between rule groups.
- Every rule MUST have a comment explaining its purpose.
- Generated parser/lexer files are NOT committed. They are build artifacts.

### Python (`tools/`, `scripts/`)

- Version: 3.11+
- Lint/Format: `ruff check --fix && ruff format`
- Docstrings: Google style. All public functions and classes documented.
- Type hints: Required for ALL function signatures. `mypy --strict` passes.
- Imports: Standard library first, then third-party, then local. Alphabetical
  per group.
- No `except:` without a specific exception type. No bare `except Exception`
  without logging or re-raise.

### Documentation (`docs/`, `*.md`)

- Format: GitHub-Flavored Markdown.
- Line length: 100 characters maximum.
- Headings: ATX-style (`#`, `##`, `###`). Exactly one `#` per file.
- Code blocks: Always specify language. Use `` ```rust `` not `` ``` ``.
- Links: Relative paths for internal links. Full URLs for external.
- Tables: Aligned with spaces. Header separator on the line below headers.

---

## RFC Workflow

When a task involves a specification change:

1. **Check** if an RFC already exists in `docs/rfcs/` covering the change.
2. If no RFC exists, **draft one** using the template in
   `docs/rfcs/README.md`. Name it `docs/rfcs/NNNN-title.md` (next number).
3. Set status to `Draft` and submit for review.
4. **Do not write ANY implementation code** until the RFC status is
   `Ratified`. This includes test code, prototype code, and proof-of-concept
   code. No exceptions.
5. Once ratified, update `SPEC.md` FIRST (Phase 2 of the pipeline).
6. Then proceed through the full pipeline (Phases 3-8).
7. After merge, mark RFC as `Implemented`.

---

## Toolchain

| Component | Tool | Version | Command |
|-----------|------|---------|---------|
| Grammar | ANTLR4 | 4.13.2+ | `antlr4 UEAS.g4` |
| Kernel | Rust | 1.75+ (stable) | `cargo build`, `cargo test`, `cargo clippy` |
| Python | Python | 3.11+ | `python script.py` |
| Lint (Rust) | clippy | bundled | `cargo clippy -- -D warnings` |
| Format (Rust) | rustfmt | bundled | `cargo fmt --check` |
| Lint (Python) | ruff | latest | `ruff check --fix` |
| Format (Python) | ruff | latest | `ruff format --check` |
| Type Check (Python) | mypy | latest | `mypy --strict` |
| Fuzzing | proptest | latest | `cargo test --test fuzz -- --ignored` |
| Coverage | tarpaulin | latest | `cargo tarpaulin` |
| Containers | Docker | 26+ | `docker build`, `docker run` |

---

## Quality Gates

All changes MUST pass these gates before merge. No gate may be skipped.

### Pre-Commit Gates (Local)

Run before committing:

```bash
# Rust (kernel/)
cd kernel
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# Python (tools/, scripts/)
ruff check --fix
ruff format --check
mypy --strict tools/
```

### Pre-Push Gates (Local)

Run before pushing:

```bash
# Property-based fuzzing
cd kernel
cargo test --test fuzz -- --ignored

# Coverage check
cargo tarpaulin --fail-under 85
```

### CI Gates (Automated, Must Pass on PR)

```
┌──────────────────────────────────────────────────────┐
│                    CI PIPELINE                        │
├──────────────┬──────────────┬─────────────────────────┤
│    STAGE 1   │    STAGE 2   │        STAGE 3          │
│  Lint/Format │  Unit Tests  │  Integration & Fuzz     │
├──────────────┼──────────────┼─────────────────────────┤
│ cargo clippy │  cargo test  │ cargo test --test       │
│ cargo fmt    │  ruff check  │   integration           │
│ ruff format  │  mypy        │ cargo test --test       │
│              │              │   cross_target          │
│              │              │ cargo test --test       │
│              │              │   fuzz -- --ignored     │
└──────────────┴──────────────┴─────────────────────────┘
                           │
                    ┌──────┴──────┐
                    │   STAGE 4   │
                    │  Coverage   │
                    ├─────────────┤
                    │ cargo       │
                    │ tarpaulin   │
                    │ --fail-     │
                    │ under 85    │
                    └─────────────┘
```

**Gate thresholds:**

| Gate | Threshold | Blocker? |
|------|-----------|----------|
| `cargo clippy` | Zero warnings | **Yes** |
| `cargo fmt --check` | Clean | **Yes** |
| `ruff check` | Zero issues | **Yes** |
| `ruff format --check` | Clean | **Yes** |
| `mypy --strict` | Zero errors | **Yes** |
| `cargo test` (unit) | 100% pass | **Yes** |
| `cargo test --test conformance` | 100% pass | **Yes** |
| `cargo test --test fuzz` | Zero panics, 10^6 inputs | **Yes** |
| `cargo test --test cross_target` | 100% equivalence | **Yes** |
| `cargo tarpaulin` | >= 85% overall | **Yes** |
| Grammar benchmarks | 100% parse accuracy | **Yes** (Epoch 1) |

A single failing gate blocks the entire PR.

---

## Design Rationale

The following decisions are intentional and non-negotiable.

| Decision | Why |
|----------|-----|
| **Abstract step-counting, not wall-clock time** | Wall-clock profiling is environment-dependent. Step-counting produces an absolute, deterministic complexity curve identical on any hardware. |
| **Zero system I/O in the kernel** | System I/O introduces side effects, non-determinism, and security surface. The kernel sandbox guarantees reproducibility. |
| **Three-domain decoupling** | Coupled domains cause "architecture drift" — the blueprint and implementation fall out of sync. The canonical JSON AST is the only interface contract. |
| **Static explicit typing** | Removes ambiguity about author intent. Enables parser-level rejection of type-mismatch programs. |
| **`unsafe` requires documented justification** | `unsafe` circumvents Rust's memory safety guarantees. Every usage must have a filed issue and a `// SAFETY:` comment. |
| **Kernel never panics — all failures are trap codes** | Panics are uncontrolled crashes. Exit status `0` signals success; non-zero codes (1-11) are controlled, documented error halts that downstream tools can programmatically distinguish. |
| **RFC ratification before implementation** | Specification leads; reference implementation follows. Reversing this is how standards rot. |
| **Documentation before code (Phase 2)** | Ensures specification-code alignment. Prevents "the code is the documentation" anti-pattern. |
| **Design pattern enforcement** | Consistent patterns (Visitor, Strategy, Observer, Factory, Command) make the codebase navigable and predictable. |
| **`const` expressions** | Variables declared with `const` are immutable. This allows the kernel to optimize heap allocation and guarantees side-effect-free evaluation. |
| **Graph directedness (`Directed`/`Undirected`)** | Explicit direction annotation prevents ambiguity in graph traversal algorithms. Undirected graphs treat edges as unordered pairs; directed graphs use ordered pairs. |
| **`@Memory` annotation** | Optional memory complexity bound complements `@Complexity`. The kernel tracks `heap.bytes_allocated()` and traps with `HEAP_EXHAUSTION` if the bound is exceeded. |
| **V2.0 Iceberg Architecture** | Syntax is user-facing (Pythonic pseudocode without braces/semicolons). Semantic engine (`kernel/src/infer/`) performs type inference and operator desugaring transparently. Users write pseudocode; the kernel handles the rigor. |

---

## No-Go Zones

The **Immutable Axioms** defined in [SPEC.md Section 1.4](SPEC.md#14-immutable-axioms)
are non-negotiable. Any change that violates them is automatically rejected.
In addition, the following are **strictly forbidden** in the kernel domain:

- System I/O (file read/write, stdin/stdout)
- Network access (sockets, HTTP)
- Hardware calls (GPU, DMA, peripherals)
- Unsafe blocks without documented, reviewed justification in a filed GitHub issue
- Panics that are not caught and converted to trap codes
- Modifying the AST during execution (the AST is immutable)
- Dependencies outside the Rust standard library (without RFC ratification)

The following are **strictly forbidden** project-wide:

- Bypassing the RFC process for spec changes
- Merging code without corresponding tests
- Skipping the CLA requirement for external contributions
- Introducing non-Apache-2.0-compatible dependencies
- Skipping any phase of the development pipeline
- Merging a PR with any failing quality gate
- Leaving TODO comments without a `TODO(#issue_number)` format
- Modifying sealed architecture or specification documents (SPEC.md, AGENTS.md architecture sections, README.md, LICENSE, NOTICE) without explicit maintainer approval
- Introducing documentation that contradicts, removes, or reinterprets existing architectural decisions, SOLID principles, or domain boundaries

---

## Vocabulary

Use the terminology defined in [SPEC.md Section 2](SPEC.md#2-definitions-and-terminology).
Do not invent new terms. If a concept lacks a term, propose one via RFC.

Key terms (abbreviated reference):

- **AST** — Abstract Syntax Tree
- **Virtual Heap** — Isolated memory arena with zero external access
- **Step Count** — Monotonic counter of primitive operations
- **Trap Code** — Controlled error code (0 = success, 1-11 = trap conditions)
- **Invariant** — Boolean predicate enforced by the kernel at runtime
- **Complexity Contract** — `@Complexity` annotation with Big-O bound
- **TargetGenerator** — Interface for transpilation backends
- **UCTS** — UEAS Conformance Test Suite

---

## Directory Map

```
ueas/
├── AGENTS.md              This file — authoritative development protocol
├── README.md              Project overview, architecture, quick links
├── SPEC.md                Formal specification v1.0.0-draft
├── LICENSE                Apache License 2.0
├── NOTICE                 Apache copyright notice
├── CONTRIBUTORS.md        List of contributors (All Contributors spec)
├── CHANGELOG.md           Release history (Keep a Changelog)
├── TODO.md                Current task list
├── SECURITY.md            Vulnerability reporting policy
├── CODE_OF_CONDUCT.md     Apache Foundation CoC
├── Cargo.toml             Workspace root (kernel + backends)
├── .github/               GitHub PR and issue templates
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       ├── feature_request.md
│       ├── rfc_proposal.yml
│       └── target_generator.yml
├── grammar/               ANTLR4 grammar (Epoch 1)
│   ├── UEAS.g4            Full ANTLR4 grammar
│   └── tests/
│       ├── positive/      7 parse-test .ueas files
│       └── negative/      3 rejection-test .ueas files
├── kernel/                Rust abstract interpreter (Epoch 2)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs         Module declarations
│   │   ├── ast/mod.rs     AST node types, Factory, Visitor, Types, serde
│   │   ├── interp/mod.rs  Expression evaluator, statement executor
│   │   ├── heap/mod.rs    Virtual heap (bump-alloc, bounds-checked)
│   │   ├── traps/mod.rs   Exit codes (0-11), trap register
│   │   ├── profiling/     Step counter, complexity profiler
│   │   │   └── mod.rs
│   │   └── invariants/    Invariant engine
│   │       └── mod.rs
│   └── tests/
│       ├── fuzz.rs         Property-based fuzz (6 proptest + 200K batch)
│       └── conformance.rs  UCTS — 7 conformance tests
├── backends/              Transpiler plugins (Epoch 3)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs         TargetGenerator, PythonTarget, RustTarget
│   │   └── mcp.rs         MCP endpoint (handle_transpile)
│   └── tests/
│       └── cross_target.rs  7 benchmark equivalence tests
├── examples/               Benchmark algorithm .ueas files
│   ├── euclidean.ueas     O(1)
│   ├── linear_search.ueas O(N)
│   ├── binary_search.ueas O(log N)
│   ├── merge_sort.ueas    O(N log N)
│   ├── dijkstra.ueas      O((V+E) log V)
│   ├── dfs.ueas           O(V+E)
│   └── matrix_multiply.ueas  O(R*C*K)
│   └── quicksort_randomized.ueas  O(N log N) with const + randInt
├── tools/                 CI, containers
│   └── Dockerfile         Reproducible CI environment
└── docs/
    ├── CONTRIBUTING.md    Contribution guide (full contributor lifecycle)
    ├── CLA.md             Contributor License Agreement (ICLA + CCLA)
    ├── GOVERNANCE.md      BDFL → TSC transition
    ├── rfcs/              RFC proposals (NNNN-title.md)
    │   └── README.md      RFC lifecycle, template, review criteria
    ├── adr/               Architecture Decision Records
    │   └── README.md      ADR format and index
    ├── specs/             Detailed per-domain specifications
    │   └── README.md      Format and index
    └── meeting-notes/     Community meeting archives
        └── README.md      Format and index
```
