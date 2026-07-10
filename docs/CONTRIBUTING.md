# Contributing to UEAS

Welcome, and thank you for your interest in contributing to the Universal
Executable Algorithm Standard (UEAS). This document is the single source of
truth for how to participate in the project — whether you write code, review
RFCs, hunt bugs, improve documentation, or organize the community.

UEAS is built on the conviction that algorithms deserve the same engineering
rigor as physical infrastructure. Every contribution moves us closer to a
world where algorithmic correctness is machine-verifiable and mathematically
enforceable.

---

## Table of Contents

1. [Project Philosophy](#1-project-philosophy)
2. [Community & Communication](#2-community--communication)
3. [Ways to Contribute](#3-ways-to-contribute)
4. [Reporting Issues](#4-reporting-issues)
5. [Development Environment](#5-development-environment)
6. [Branching & Workflow](#6-branching--workflow)
7. [Pull Request Process](#7-pull-request-process)
8. [Coding Conventions](#8-coding-conventions)
9. [Testing Requirements](#9-testing-requirements)
10. [Contributor License Agreement](#10-contributor-license-agreement)
11. [Release Process](#11-release-process)
12. [Maintainer Guide](#12-maintainer-guide)
13. [Recognition](#13-recognition)

---

## 1. Project Philosophy

UEAS exists to eliminate ambiguity from algorithmic specification. Every
contribution — whether a grammar rule, a kernel optimization, or a
documentation fix — must serve this mission.

We hold these principles:

- **Mathematics First.** Implementation follows specification, never the
  reverse. No line of code is written until the corresponding RFC is ratified.
- **Correctness Over Convenience.** A transpiler that silently produces
  incorrect code is worse than no transpiler at all. Trap early, trap loudly.
- **Isolation by Design.** The kernel has zero external dependencies. No I/O,
  no network, no hardware access. This is not a limitation — it is the
  guarantee.
- **Open Governance.** The standard belongs to its contributors. Governance
  transitions from BDFL to TSC after Epoch 3. Every RFC is public, every
  decision has a written rationale.

Before contributing, read [SPEC.md](../SPEC.md) Sections 1 and 2 to
understand the foundational definitions.

---

## 2. Community & Communication

### Channels

| Channel | Purpose |
|---------|---------|
| **GitHub Issues** | Bug reports, feature requests, task tracking |
| **GitHub Discussions** | Design conversations, RFC brainstorming before formal submission, Q&A |
| **Project Mailing List** (to be configured) | Release announcements, governance votes, security advisories |
| **Community Chat** (to be configured) | Real-time collaboration, newcomer onboarding, pair debugging |

### Meeting Cadence

| Meeting | Frequency | Purpose |
|---------|-----------|---------|
| **Community Call** | Bi-weekly | Progress updates, RFC status, contributor demos |
| **RFC Review** | Weekly | Active RFC discussion, blocking issue resolution |
| **Release Planning** | Per-epoch milestone | Scope negotiation, risk assessment, go/no-go |

Meeting notes are published in `docs/meeting-notes/` within 48 hours.

### Asking Questions

- "How do I...?" → GitHub Discussions
- "Is this a bug?" → GitHub Issues with `question` label
- "I have a security concern." → Private disclosure (see Section 4.3)
- "I want to propose a spec change." → Start in Discussions, then formal RFC

---

## 3. Ways to Contribute

You do not need to write Rust to contribute. Here are all the contribution
paths, organized by skill area:

### Code Contributions

| Domain | Skill | Impact |
|--------|-------|--------|
| `grammar/` | ANTLR4, language design, formal grammars | Defines what UEAS *is* |
| `kernel/` | Rust, VM design, algorithms analysis | Enforces the standard |
| `backends/` | Python/Rust/C++/Java code generation | Brings UEAS to the world |
| `tools/` | CI/CD, benchmarking, fuzzing, containers | Keeps quality gates honest |

### Non-Code Contributions

| Role | What You Do |
|------|-------------|
| **RFC Reviewer** | Peer-review proposals for mathematical soundness and backward compatibility |
| **Documentation Writer** | Improve SPEC.md, tutorials, API references, onboarding guides |
| **Test Writer** | Expand the UCTS (UEAS Conformance Test Suite) with edge cases |
| **Fuzzing Engineer** | Write `proptest` strategies that find kernel panics |
| **Community Organizer** | Moderate Discussions, triage issues, onboard newcomers |
| **Benchmark Designer** | Create algorithmic workloads that stress the interpreter |
| **Security Researcher** | Audit the kernel for sandbox escapes, fuzz the MCP endpoint |

---

## 4. Reporting Issues

### 4.1. Bug Reports

Open a GitHub Issue using the **Bug Report** template. A good bug report
includes:

1. **Minimal Reproducer:** The smallest `.ueas` source or AST JSON that
   triggers the bug.
2. **Expected Behavior:** What the spec says should happen.
3. **Actual Behavior:** What actually happened (trap code, panic, incorrect
   output).
4. **Environment:** OS, Rust version, ANTLR4 version, Docker image tag.

Example:

```
## Description
Kernel traps with COMPLEXITY_VIOLATION on an O(N) linear search
over an empty list.

## Reproducer
Algorithm LinearSearchEmpty()
    Ensure: Integer
    Complexity: "O(N)"

    data <- []
    target <- 5
    i <- 0
    while i < length(data) do
        if data[i] == target then
            return i
        end if
        i <- i + 1
    end while
    return -1
end Algorithm

## Expected
Trap code 0 (NO_ERROR), returns -1.

## Actual
Trap code 5 (COMPLEXITY_VIOLATION).

## Environment
- Ubuntu 24.04, Rust 1.80.0, ANTLR4 4.13.2, Docker 26.1.3
```

### 4.2. Feature Requests

Open a GitHub Issue using the **Feature Request** template. A good feature
request includes:

1. **Motivation:** What problem does this solve? Who benefits?
2. **Proposed Syntax/Semantics:** How would this look in the grammar and AST?
3. **Impact on Complexity Model:** Does this introduce new step-cost models?
4. **Existing Alternatives:** How do other formal specification languages
   handle this?

Feature requests that alter the specification MUST follow the RFC process
after discussion. Do not submit implementation PRs for spec changes without a
ratified RFC.

### 4.3. Security Vulnerabilities

**Do not open a public issue for security vulnerabilities.**

Send a detailed report to the project security contact (to be configured).
Include:

- Affected component(s): grammar parser, kernel sandbox, MCP endpoint, CI/CD
- Attack vector description
- Proof-of-concept exploit
- Suggested mitigation

The security team will acknowledge receipt within 48 hours and provide a
timeline for remediation. Critical vulnerabilities will trigger an out-of-band
security release.

### 4.4. Issue Triage Labels

| Label | Meaning |
|-------|---------|
| `bug` | Confirmed incorrect behavior |
| `enhancement` | Feature request or improvement |
| `question` | Needs clarification, not actionable yet |
| `good first issue` | Suitable for new contributors |
| `help wanted` | Available for community pickup |
| `rfc-needed` | Requires an RFC before implementation |
| `epoch-1` / `epoch-2` / `epoch-3` | Roadmap alignment |
| `blocked` | Waiting on another issue or RFC |
| `security` | Security-sensitive, private handling |

---

## 5. Development Environment

### 5.1. Prerequisites

| Component | Minimum Version | Installation |
|-----------|-----------------|--------------|
| Rust | 1.75.0 (stable) | `rustup install stable` |
| ANTLR4 | 4.13.2 | [ANTLR Downloads](https://www.antlr.org/download.html) |
| Python | 3.11+ | `python.org` or `pyenv` |
| Docker | 26+ | `docker.com` |
| Git | 2.40+ | `git-scm.com` |

### 5.2. Setup by Platform

**Linux (Ubuntu/Debian):**

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup component add clippy rustfmt

# ANTLR4
sudo apt install default-jre
curl -O https://www.antlr.org/download/antlr-4.13.2-complete.jar
sudo mv antlr-4.13.2-complete.jar /usr/local/lib/
echo 'alias antlr4="java -jar /usr/local/lib/antlr-4.13.2-complete.jar"' >> ~/.bashrc
source ~/.bashrc

# Python
sudo apt install python3 python3-pip
pip install ruff
```

**macOS:**

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup component add clippy rustfmt

# ANTLR4
brew install openjdk
curl -O https://www.antlr.org/download/antlr-4.13.2-complete.jar
sudo mkdir -p /usr/local/lib
sudo mv antlr-4.13.2-complete.jar /usr/local/lib/
echo 'alias antlr4="java -jar /usr/local/lib/antlr-4.13.2-complete.jar"' >> ~/.zshrc
source ~/.zshrc

# Python
brew install python@3.11
pip install ruff
```

**Windows (PowerShell):**

```powershell
# Rust
# Download and run https://rustup.rs
rustup component add clippy rustfmt

# ANTLR4
# Install Java from https://adoptium.net/
Invoke-WebRequest -Uri "https://www.antlr.org/download/antlr-4.13.2-complete.jar" `
    -OutFile "$env:USERPROFILE\antlr-4.13.2-complete.jar"
# Add to PowerShell profile:
# function antlr4 { java -jar "$env:USERPROFILE\antlr-4.13.2-complete.jar" $args }

# Python
# Install from https://python.org
pip install ruff
```

### 5.3. Docker Alternative

If you prefer a containerized environment:

```bash
# Build the development image (once tools/Dockerfile is created)
docker build -t ueas-dev -f tools/Dockerfile .

# Enter a development shell with all toolchains pre-configured
docker run -it --rm -v "$PWD:/ueas" -w /ueas ueas-dev bash

# Inside the container, run quality gates
cargo test && cargo clippy -- -D warnings && cargo fmt --check && ruff check --fix && ruff format --check
```

### 5.4. Editor Configuration

Recommended extensions for IDE integration:

| Editor | Extensions |
|--------|-----------|
| VS Code | `rust-analyzer`, `mike-lischke.vscode-antlr4`, `ms-python.python`, `charliermarsh.ruff` |
| JetBrains (CLion/IntelliJ) | Rust plugin, ANTLR v4 plugin |
| Vim/Neovim | `rust-analyzer` (LSP), `ale` or `nvim-lint` for lint-on-save |
| Emacs | `rustic`, `eglot` with rust-analyzer |

Recommended settings for VS Code (`.vscode/settings.json`):

```json
{
  "editor.formatOnSave": true,
  "rust-analyzer.check.command": "clippy",
  "[python]": {
    "editor.defaultFormatter": "charliermarsh.ruff"
  },
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### 5.5. CLI Development

The `ueas` CLI lives in `tools/ueas-cli/` as a workspace member.

```bash
# Build the CLI
cargo build --workspace

# Run the CLI directly
cargo run -- run examples/euclidean.ueas
cargo run -- check library/sorting/quicksort.ueas
cargo run -- transpile examples/linear_search.ueas --target python

# Install globally
cargo install --path tools/ueas-cli
```

### 5.6. Library Contribution

New algorithms in `library/` must follow these rules:

1. Use v3.0 academic pseudocode syntax (`<-`, `then`/`do`/`end` closures,
   `Require:`/`Ensure:`/`Complexity:` preamble)
2. Place in the correct category directory (`sorting/`, `graph/`, `dp/`, etc.)
3. Declare an accurate `Complexity:` contract with bindings
4. Pass `ueas check <file>` — no parse errors
5. Update `library/INDEX.md` with the new entry
6. Use descriptive algorithm and parameter names

**Library quality checklist:**

- [ ] Algorithm header uses `Algorithm Name(params)` format
- [ ] `Require:` block declares all parameter types
- [ ] `Ensure:` block declares the return type
- [ ] `Complexity:` matches the algorithm's theoretical bound
- [ ] Bindings provided for all complexity variables (e.g., `N = data.length`)
- [ ] All assignments use `<-` operator
- [ ] Control flow uses `then`/`do`/`end if`/`end for`/`end while` closures
- [ ] No semicolons, no curly braces, no `let` keyword
- [ ] Self-contained (no imports for core logic)
- [ ] `index <- 0` initialized before `for each` loops needing iteration tracking

---

## 6. Branching & Workflow

### 6.1. Trunk-Based Development

UEAS uses trunk-based development with short-lived feature branches.

- **`main`** — Always deployable. All quality gates pass. Protected branch.
- **Feature branches** — Created from and merged back to `main` within days,
  not weeks.

### 6.2. Branch Naming Convention

```
<type>/<description>
```

| Type | Use Case |
|------|----------|
| `feature/` | New functionality (non-spec-changing, or RFC already ratified) |
| `fix/` | Bug fixes |
| `docs/` | Documentation only (SPEC.md, README, comments) |
| `rfc/` | RFC document submission (contains no implementation code) |
| `chore/` | CI/CD, tooling, dependency bumps |
| `test/` | Test-only changes (expanding test coverage) |
| `refactor/` | Code restructuring without behavior change |

Examples:

```
feature/matrix-determinant-optimization
fix/kernel-trap-on-empty-set-union
docs/epoch-1-grammar-examples
rfc/0001-graph-literal-syntax
chore/update-rust-toolchain-1.81
test/fuzz-matrix-multiplication
refactor/extract-visitor-from-interpreter
```

### 6.3. Fork-and-PR Model

1. **Fork** the repository to your GitHub account.
2. **Clone** your fork: `git clone https://github.com/YOUR-USERNAME/ueas.git`
3. **Add upstream remote:** `git remote add upstream https://github.com/ueas/ueas.git`
4. **Sync before branching:** `git fetch upstream && git checkout main && git merge upstream/main`
5. **Create a branch:** `git checkout -b <type>/<description>`
6. **Commit and push:** `git push -u origin <type>/<description>`
7. **Open a Pull Request** from your branch to `ueas/main`.

Keep your branch in sync with upstream during review:

```bash
git fetch upstream
git merge upstream/main
git push
```

### 6.4. Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/)
specification with UEAS-specific scopes:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

| Type | Meaning |
|------|---------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation |
| `test` | Test addition or modification |
| `refactor` | Code restructuring |
| `perf` | Performance improvement |
| `chore` | Maintenance |
| `ci` | CI/CD changes |

| Scope | Meaning |
|-------|---------|
| `grammar` | ANTLR4 grammar, lexer, parser |
| `kernel` | Rust abstract interpreter |
| `backends` | Transpiler plugins |
| `tools` | CI, fuzzing, benchmarks |
| `docs` | Documentation files |

Examples:

```
feat(grammar): add Graph literal syntax production rules
fix(kernel): correct step-cost accounting for empty Set union
docs(spec): clarify invariant re-evaluation semantics in loop bodies
test(kernel): add property-based fuzz tests for Matrix transpose
refactor(kernel): extract InvariantEngine from Interpreter
chore(ci): upgrade Docker base image to Ubuntu 24.04
```

---

## 7. Pull Request Process

### 7.1. Before Opening a PR

Run these checks locally. The CI pipeline runs the same checks — catching
issues before pushing saves everyone time.

```bash
# Rust (kernel/)
cd kernel
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo test --test fuzz -- --ignored

# Python (tools/, scripts/)
ruff check --fix
ruff format --check
```

### 7.2. PR Template

Every PR description MUST include:

```markdown
## Summary
<!-- 1-2 sentences describing the change -->

## Motivation
<!-- Why is this needed? Link to related issue or RFC. -->

## Type of Change
<!-- Check all that apply -->
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation
- [ ] Refactoring
- [ ] Performance improvement
- [ ] CI/CD

## Domain(s) Affected
<!-- Check all that apply -->
- [ ] grammar/
- [ ] kernel/
- [ ] backends/
- [ ] tools/
- [ ] docs/

## Testing
<!-- Describe the tests you added or ran -->
- [ ] Unit tests pass
- [ ] Property-based fuzz tests pass
- [ ] Cross-target equivalence verified

## Checklist
- [ ] All quality gates pass locally
- [ ] New code follows project conventions (AGENTS.md)
- [ ] Tests added for new functionality (TDD)
- [ ] Documentation updated if applicable
- [ ] RFC ratified if this is a spec change (link to RFC: ____)
```

### 7.3. Review Requirements

- **At least one maintainer approval** is required before merge.
- RFC-related code requires the RFC author's approval in addition to a
  maintainer.
- Reviewers check: correctness, style, test coverage, documentation clarity,
  and alignment with SPEC.md.
- A single reviewer may request changes. Address all comments before
  re-requesting review.
- CI checks must be green. A PR with failing CI MUST NOT be merged.

### 7.4. Merge Strategy

- **Squash and merge** is the default. Clean history on `main`.
- Squashed commit message follows the Conventional Commits format.
- The PR description body becomes the squash commit body.
- Merge commits are permitted only for integrating long-running feature
  branches with explicit maintainer approval.

### 7.5. After Merge

- Delete your feature branch (GitHub does this automatically when you select
  the option).
- If implementing an RFC, update the RFC status from `Ratified` to
  `Implemented`.
- Close the related GitHub issue(s) with a comment referencing the merge
  commit.

---

## 8. Coding Conventions

### 8.1. Rust (`kernel/`)

- **Edition:** 2021
- **Lint level:** `cargo clippy -- -D warnings` — zero warnings required
- **Formatting:** `rustfmt` with default settings
- **Documentation:** Every `pub` item MUST have a doc comment (`///`).
  Module-level docs (`//!`) required for each `src/` submodule.
- **`unsafe`:** Forbidden without a documented, maintainer-reviewed
  justification filed as a GitHub issue. Each `unsafe` block must carry a
  `// SAFETY:` comment explaining why the invariants hold.
- **Error Handling:** Use `Result<T, E>` and the `?` operator. No bare
  `unwrap()` or `expect()` in production code. Panics go through the
  kernel trap handler.
- **Module Structure:** One Rust module per conceptual unit. AST node types
  in `ast/`, interpreter in `interp/`, invariants in `invariants/`.

### 8.2. ANTLR4 (`grammar/`)

- **Version:** 4.13.2+
- **File structure:** Single `UEAS.g4` file with combined lexer and parser
  grammar.
- **Rule grouping:** Lexer rules first, then parser rules grouped by:
    1. Top-level (program, algorithm)
    2. Statements
    3. Expressions (by precedence)
    4. Types
- **Alternatives:** One per line with `|` prefix for readability.
- **Comments:** Section dividers (`(* ===== Section ===== *)`) between
  rule groups.

### 8.3. Python (`tools/`, `scripts/`)

- **Version:** 3.11+
- **Linting/Formatting:** `ruff check --fix && ruff format`
- **Docstrings:** Google style for all public functions and classes.
- **Type Hints:** Required for all function signatures. `mypy --strict` must
  pass for any Python code that ships as part of the project toolchain.
- **Imports:** Standard library first, then third-party, then local.
  Alphabetical within each group.

### 8.4. Documentation (`docs/`, `*.md`)

- **Format:** GitHub-Flavored Markdown (GFM).
- **Line length:** Wrap at 100 characters for readability in terminal editors.
- **Headings:** ATX-style (`#`, `##`, `###`). One `#` per file.
- **Code blocks:** Always specify the language for syntax highlighting.
- **Links:** Use relative paths for internal links.
- **RFC and ADR documents:** Follow the templates in `docs/rfcs/README.md`
  and `docs/adr/README.md`.

---

## 9. Testing Requirements

UEAS follows Test-Driven Development (TDD). The rule is absolute:

> **No production code without a failing test first.**

### 9.1. Test Categories

| Category | Scope | Tool | When Required |
|----------|-------|------|---------------|
| **Unit** | Single function or AST node evaluator | `cargo test` | Always |
| **Integration** | End-to-end: parse → execute → verify | `cargo test --test integration` | For new AST node kinds |
| **Property-Based** | Random valid AST inputs; expect zero panics | `proptest` | For all kernel evaluators |
| **Complexity Violation** | Deliberately over-budget algorithms | Manual `#[test]` | For each complexity class |
| **Cross-Target Equivalence** | Same AST → Python and Rust outputs match | `cargo test --test cross_target` | For new transpiler features |
| **Grammar** | 100% parse accuracy on benchmark algorithms | ANTLR4 TestRig | For grammar changes |
| **Fuzzing** | Chaotic input to catch crashes | `cargo test --test fuzz -- --ignored` | Before every release |

### 9.2. Writing a Good Test

```rust
// Example: unit test for Matrix transpose
#[test]
fn matrix_transpose_3x2() {
    // Arrange
    let matrix = Matrix::new(3, 2, vec![1, 2, 3, 4, 5, 6]);

    // Act
    let result = matrix.transpose();

    // Assert
    assert_eq!(result.rows(), 2);
    assert_eq!(result.cols(), 3);
    assert_eq!(result.get(0, 0), 1);
    assert_eq!(result.get(0, 1), 3);
    assert_eq!(result.get(0, 2), 5);
    assert_eq!(result.get(1, 0), 2);
    assert_eq!(result.get(1, 1), 4);
    assert_eq!(result.get(1, 2), 6);
}

#[test]
fn matrix_transpose_panics_on_non_matrix_input() {
    // Property-based: proptest generates random dimensions
    // Kernel must never panic, even on degenerate inputs
}
```

### 9.3. Running Tests

```bash
# All unit tests
cargo test

# Specific test
cargo test matrix_transpose

# Integration tests (parse + execute)
cargo test --test integration

# Property-based fuzz tests (may run for hours)
cargo test --test fuzz -- --ignored --test-threads=1

# Cross-target equivalence (requires Python and Rust targets built)
cargo test --test cross_target

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html --output-dir target/coverage
```

### 9.4. Coverage Expectations

| Domain | Minimum Line Coverage |
|--------|----------------------|
| Kernel (`kernel/src/interp/`) | 95% |
| Kernel (`kernel/src/ast/`) | 90% |
| Kernel (utilities) | 85% |
| Backends (each target) | 80% |
| Grammar (ANTLR4) | 100% (benchmark parse accuracy) |

Coverage is measured per PR. Regressions that drop coverage below these
thresholds block merge.

---

## 10. Contributor License Agreement (CLA)

All contributors MUST sign a Contributor License Agreement before any
contribution can be merged. This is a legal requirement inherited from the
Apache Software Foundation model.

- **Individual contributors** sign the Individual CLA (ICLA).
- **Organizations** sign the Corporate CLA (CCLA) for employee contributions.

The full CLA text is in [`docs/CLA.md`](CLA.md).

**Submission process** (to be configured):
1. Read the CLA text.
2. Sign via the project's CLA assistant (GitHub bot or web form).
3. Once verified, your PRs will be labeled `cla-signed`.

Contributions from unsigned contributors will be politely held until the CLA
is on file.

---

## 11. Release Process

### 11.1. Versioning

UEAS follows [Semantic Versioning 2.0.0](https://semver.org/).

| Version Component | Changes When |
|-------------------|--------------|
| **MAJOR** (`1.0.0` → `2.0.0`) | Breaking changes to the grammar, AST schema, or kernel semantics |
| **MINOR** (`1.0.0` → `1.1.0`) | New AST node kinds, new transpilation targets, new invariant types |
| **PATCH** (`1.0.0` → `1.0.1`) | Bug fixes, performance improvements, documentation updates |

SPEC.md version and the reference implementation version are synchronized.
Every release ships a versioned SPEC.md artifact.

### 11.2. Release Cadence

| Release Type | Cadence | Example |
|--------------|---------|---------|
| **Epoch Milestones** | Every 3 months | `0.1.0` (Epoch 1), `0.2.0` (Epoch 2), `1.0.0` (Epoch 3 complete) |
| **Feature Releases** | Monthly (between milestones) | `0.1.1`, `0.2.3` |
| **Patch Releases** | As needed (bug/security fixes) | `0.2.3` → `0.2.4` |

### 11.3. Release Checklist

The release manager (rotated among maintainers) executes:

1. **Verify all quality gates** on `main` — CI green, fuzz tests pass, coverage
   above threshold.
2. **Update version** in SPEC.md header and `kernel/Cargo.toml`.
3. **Generate CHANGELOG** from Conventional Commits since last release.
4. **Create release branch** `release/x.y.z` from `main`.
5. **Tag** `vx.y.z` with GPG signature.
6. **Build release artifacts** — Docker image, standalone binaries (future).
7. **Publish** to package registries (future: Homebrew, apt, crates.io for kernel
   library).
8. **Announce** on mailing list and GitHub Discussions.

### 11.4. Release Candidate Testing

Before a MAJOR or MINOR release:

1. A release candidate (`vx.y.z-rc1`) is tagged 2 weeks before release.
2. The community tests against their own algorithmic workloads.
3. Any regressions found during the RC period block the release.
4. If fixes are needed, a new RC (`vx.y.z-rc2`) is tagged and the 2-week
   clock resets.

### 11.5. CHANGELOG

The CHANGELOG follows [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [0.2.0] - YYYY-MM-DD

### Added
- Graph literal syntax in grammar (RFC 0001)
- Step-cost accounting for graph adjacency operations

### Changed
- Invariant re-evaluation now mandatory inside loop bodies

### Fixed
- Empty Set union now correctly costs 0 steps

### Security
- Kernel sandbox hardened against recursive type DoS
```

---

## 12. Maintainer Guide

### 12.1. What Is a Maintainer?

Maintainers are contributors who have demonstrated sustained, high-quality
contributions and have been granted write access to the repository.
Maintainers:

- Review and merge pull requests
- Triage issues
- Participate in RFC voting
- Shepherd release candidates
- Enforce the Code of Conduct

### 12.2. Becoming a Maintainer

Candidates for maintainer status must:

1. Have at least **5 merged pull requests** spanning at least 2 domains
   (e.g., `kernel/` and `docs/`).
2. Have **reviewed at least 3 pull requests** with constructive feedback.
3. Be nominated by an existing maintainer.
4. Be approved by majority vote of current maintainers.

New maintainers undergo a 30-day mentorship period with an experienced
maintainer before receiving full merge rights.

### 12.3. Maintainer Responsibilities

- **Code Review SLA:** Review assigned PRs within 3 business days.
- **Issue Triage:** All new issues should receive a label and acknowledgment
  within 1 business day.
- **RFC Participation:** Vote (approve/abstain/reject with rationale) on all
  RFCs under review.
- **Release Rotation:** Take turns as release manager.
- **Community Health:** Enforce the Code of Conduct; escalate violations to
  the TSC.

### 12.4. Voting

| Decision Type | Threshold | Example |
|---------------|-----------|---------|
| RFC Ratification | Consensus (no unresolved objections) | Spec changes |
| Maintainer Addition | Majority | New committer |
| Release Go/No-Go | Majority + CI green | Shipping a release |
| Code of Conduct Enforcement | 2/3 supermajority | Banning a contributor |

### 12.5. Stepping Down

Maintainers may step down at any time by notifying the group. Inactive
maintainers (no activity for 6 months) are moved to emeritus status with
gratitude.

---

## 13. Recognition

### 13.1. Contributors List

All contributors — code, documentation, testing, design, community — are
recognized in `CONTRIBUTORS.md` at the repository root. This list follows the
[All Contributors](https://allcontributors.org/) specification.

To add yourself after your first merged PR, comment on the PR:

```
@all-contributors please add @<username> for <contribution type>
```

### 13.2. Contribution Types

| Emoji/Key | Contribution |
|-----------|--------------|
| 💻 `code` | Code |
| 📖 `doc` | Documentation |
| 🧪 `test` | Tests |
| 🐛 `bug` | Bug reports |
| 💡 `ideas` | RFCs, feature requests |
| 🔍 `review` | PR reviews |
| 🛡️ `security` | Security research |
| 🎨 `design` | Specification design |
| 📦 `infra` | CI/CD, tooling |
| 🚇 `fuzz` | Fuzzing |
| 🏋️ `benchmark` | Benchmarking |

### 13.3. Contributor Covenant

This project adopts the [Apache Software Foundation Code of
Conduct](https://www.apache.org/foundation/policies/conduct.html). All
contributors, maintainers, and participants in UEAS community spaces are
expected to uphold this code.

Reports of Code of Conduct violations should be sent to the project conduct
contact (to be configured).

---

*This CONTRIBUTING.md is a living document. Proposals to change it follow the
same RFC process as specification changes.*
