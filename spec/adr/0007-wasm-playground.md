# ADR 0007: WebAssembly (WASM) Playground Architecture

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

Epoch 9 focuses on massive adoption. The biggest barrier to trying a new language or standard is toolchain friction (requiring users to run `cargo install ueas` or download binaries). To make UEAS accessible to researchers, educators, and students instantly, we need a zero-install browser playground that offers the full power of the UEAS pipeline (Parse -> Validate -> Execute -> Transpile).

## Decision

The UEAS Rust Kernel and Backend Transpilers will be compiled to a `wasm32-unknown-unknown` target. 
We will build a frontend Single Page Application (SPA) named `ueas-playground` using a modern framework (React or Vue) integrated with the Monaco Editor.

Because the UEAS kernel strictly enforces **Zero System I/O** (Axiom 1) and memory is purely isolated in the Virtual Heap, the kernel is already 100% compatible with WASM without requiring WASI (WebAssembly System Interface) polyfills for file systems or network access.

## Consequences

**Positive:**
- Instant global access to the standard.
- Validates the purity of the kernel architecture: because the kernel has no system side effects, it runs flawlessly in a browser sandbox.
- Users can share executable algorithms via base64-encoded URLs.

**Negative:**
- Adds a completely new tech stack (TypeScript/Vue/React + Webpack/Vite) to the project repository.
- Managing memory boundaries between JS (Monaco Editor text) and WASM (Rust parser bridge) introduces pointer management overhead.

## Alternatives Considered

1. **Server-Side Playground API:** Running a Rust backend server (e.g., Axum) that receives the code, executes it, and returns the result. Rejected due to hosting costs, security risks of remote code execution, and latency. Client-side WASM is free to host (GitHub Pages) and completely secure.
