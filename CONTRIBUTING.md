# Contributing to UEAS

First off, thank you for considering contributing to the Universal Executable Algorithm Standard (UEAS)! It's people like you that make UEAS such a powerful, mathematically rigorous standard.

## 1. The Golden Rule: Read AGENTS.md
The architectural governance of this project is extremely strict to maintain formal mathematical verification guarantees. Before you open a Pull Request, you **MUST** read the [AGENTS.md](AGENTS.md) manifesto.

This document outlines:
- The strict separation between `grammar/` (Specification) and `kernel/` (Execution).
- The non-negotiable Domain-Driven Design (DDD) and SOLID principles we enforce.
- The testing pyramid (100% test coverage and zero fuzzing panics required).

## 2. Contributor License Agreement (CLA)
Before we can merge your first Pull Request, you must sign our [Contributor License Agreement (CLA)](CLA.md). This is a standard legal requirement to ensure the project can eventually be adopted by the Linux Foundation or Apache Software Foundation.

You do not need to print or email anything. Simply add the following text to the description of your first Pull Request:

> I have read the CLA Document and I hereby sign the CLA.

## 3. Contribution Workflow

### A. Core Specification Changes (Grammar)
If you want to add a new algorithm primitive or alter the `UEAS.g4` grammar:
1. **Do not write code.** Open an Issue or Draft PR with an **RFC (Request for Comments)**. See `docs/rfcs/README.md`.
2. Wait for the RFC to be ratified by the architectural maintainers.
3. Once ratified, update `SPEC.md` first, then write the tests, then write the code.

### B. Interpreter & Backend Changes (Kernel/Transpilers)
If you are fixing a bug in the Rust interpreter or writing a new transpiler target (e.g., Python, C++):
1. **Test-Driven Development (TDD):** Write a failing test in the Rust test suite first.
2. Implement the fix.
3. Run the strict quality gates locally:
   ```bash
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   cargo test --workspace
   ```
4. If your change affects execution complexity, you must run the fuzzer:
   ```bash
   cargo test --test fuzz -- --ignored
   ```

### C. Developer Ecosystem (Tooling)
If you are contributing to Phase 4 (DAP Debugger, Jupyter Kernel) or Phase 5:
- Ensure your architecture is documented in an **ADR (Architecture Decision Record)** under `docs/adr/`.

## 4. Submitting a Pull Request
- Use [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat: added monte carlo support`, `fix: heap allocation overflow`).
- Ensure CI passes. We do not merge PRs with failing tests or clippy warnings under any circumstances.
- A project maintainer will review your code. 

Thank you for helping us build the future of algorithmic standards!
