# ADR 0017: Physical Decoupling of Specification and Implementation

**Status:** Accepted  
**Date:** 2026-07-10  

## Context

As the UEAS project expands into Phase 4 (Ecosystem) and Phase 5 (Core Math Frontiers), we are approaching a point where the standard needs to be governed independently of the reference implementation. The goal is to prepare UEAS for dual foundation adoption:
1. **ECMA/IEEE** for the mathematical algorithmic specification (`SPEC.md`, `grammar/`, `docs/rfcs/`).
2. **Linux Foundation / Apache** for the reference engine and ecosystem (`kernel/`, `backends/`, `tools/`, `library/`).

Previously, the grammar and the engine were tightly coupled in the same root monorepo, sharing the same version tags (e.g., `v3.0.0`) and the same root `Cargo.toml`. This makes it impossible for the standard to evolve independently of the Rust interpreter, and confuses potential foundation adopters regarding what is a standard versus what is a software implementation.

## Decision

We will implement a **Physical Monorepo Split**. 

We will fundamentally restructure the repository into two isolated root directories, ensuring that the Standard and the Engine are completely decoupled physically, conceptually, and operationally:

1. **`/specification/` directory:** Contains `SPEC.md`, `grammar/`, `docs/rfcs/`, and `docs/adr/`. No Rust engine code is allowed here.
2. **`/implementation/` directory:** Contains the root `Cargo.toml` workspace, `kernel/`, `backends/`, `tools/`, `library/`, and `examples/`.
3. **Root Directory:** Stripped clean, containing only governance files (`README.md`, `AGENTS.md`, `CLA.md`, `CONTRIBUTING.md`, `LICENSE`).

## Consequences

### Positive
- **Foundation Readiness:** We can formally pitch the `/specification` and the `/implementation` as two distinct deliverables to different foundations.
- **Architectural Clarity:** Strict physical boundary prevents the implementation side from accidentally altering the standard without going through the specification RFC process.
- **Independent CI/CD:** We can easily split GitHub Actions to test the grammar independently of the Rust workspace.

### Negative
- **Temporary Migration Friction:** Any existing uncommitted local work or open PRs that rely on the old root-level paths will encounter merge conflicts and path errors.
- **Tooling Adjustments:** Implementation agents and developers must now `cd implementation` before running `cargo build` or `cargo test`.
