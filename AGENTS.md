# AGENTS.md — UEAS v2.0 Engineering Protocol

This document is the **authoritative development protocol** for all contributors and AI agents working on UEAS v2.0.

## 1. Core Architectural Invariants

1. **Physical Domain Separation:**
   - `spec/` — Formal specification (`SPEC.md`), JSON Schemas (`spec/schemas/`), `tree-sitter-ueas`, ADRs, and RFCs. **Zero Rust implementation code** is permitted in `spec/`.
   - `crates/` — Pure, zero-host-I/O, `wasm32`-compatible Rust reference engine (`ueas-syntax`, `ueas-hir`, `ueas-vm`, `ueas-verify`, `ueas-codegen`, `ueas-extract`, `ueas-wasm`).
   - `tools/` — User-facing applications (`ueas-cli`, `ueas-lsp`, `ueas-dap`, `ueas-mcp-edge`, `ueas-playground`, `ueas-vscode`, `ueas-jupyter`, `ueas-cloud-verify`).
   - `stdlib/` & `examples/` — Verified `.ueas` algorithms with `@Test` and `@Benchmark` blocks.
   - `tests/` — Workspace-wide UCTS corpus tests, L4 compiled differential equivalence tests, and property/fuzz suites.
2. **Single Source of Truth:**
   - Exactly one parser (`ueas-syntax`), one typed IR (`ueas-hir`), one execution VM (`ueas-vm`), one verification engine (`ueas-verify`), one WASM crate (`ueas-wasm`), and one root `Cargo.lock`.
   - Zero duplicate parsers, zero JS regex heuristics in MCP/Playground, zero mocked metrics.
3. **Idiomatic Rust & Memory Safety:**
   - `#![forbid(unsafe_code)]` in all core `crates/*`.
   - `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` in all non-test library code. All runtime failures route through structured `Trap` codes or typed `Result` errors.
   - **Zero Global Mutable State / Singletons:** All execution and verification runs inside explicit, owned `VmSession` / `VerificationContext` structs for deterministic, race-free parallelism.
   - Use exhaustive Algebraic Data Types (`enum`/`struct`) and trait polymorphism (`CodeGenerator`, `ModuleResolver`, `Extractor`).
4. **Curated Dependency Policy:**
   - Pure-Rust, `Apache-2.0` / `MIT` / `BSD` / `ISC` / `Zlib` crates only, audited via `cargo deny check`.
   - Core crates (`ueas-syntax`, `ueas-hir`, `ueas-vm`, `ueas-verify`, `ueas-codegen`) must compile cleanly to `wasm32-unknown-unknown` with zero filesystem, network, or system clock access.
5. **Browser Web-Worker WASM & Edge MCP:**
   - `tools/ueas-playground` executes `ueas-wasm` locally inside a browser Web Worker.
   - `tools/ueas-mcp-edge` executes `ueas-wasm` at the edge with strict payload and fuel limits.
6. **Executable Specification:**
   - `spec/SPEC.md` is continuously verified against `tests/ucts/` and `tests/differential/` in CI. Spec changes require an ADR/RFC update alongside implementation and tests.

## 2. Quality Gates (Mandatory Before Any Merge)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo xtask verify-clean
```

- All Python code (`tools/ueas-jupyter`) MUST pass `ruff check`, `ruff format --check`, and `mypy --strict`.
- All TypeScript code (`tools/ueas-mcp-edge`, `tools/ueas-playground`, `tools/ueas-vscode`) MUST pass `tsc --noEmit` with `strict: true`.
- Never leave `todo!()`, `unimplemented!()`, `Box::leak`, or `#[ignore]`d equivalence tests in the codebase.

## 3. Contribution & Governance

- Contributions use **Developer Certificate of Origin (DCO)** (`Signed-off-by:` in commit messages) under Apache License 2.0.
- Commit messages follow **Conventional Commits** (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`).
