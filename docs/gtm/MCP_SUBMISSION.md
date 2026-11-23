# UEAS v2.0 MCP Server — Directory Submission

## Manifest

```yaml
name: ueas
display_name: UEAS v2.0 — Unified Executable Algorithmic Specification
description: |
  Parse, verify (L0-L3 progressive verification, property fuzzing, adversarial Big-O
  complexity fitting, observational constant-time checking), extract from Python/TS/Rust,
  and transpile to Rust, Python, TypeScript, C++20, Go, Dafny 4, Lean 4, Verus, Quint,
  TLA+, LaTeX, and Typst.
version: 2.0.0
license: Apache-2.0
tags:
  - algorithms
  - formal-verification
  - transpiler
  - wasm
  - rust
tools:
  - name: ueas_check
    description: Static L0 check (lossless CST parse, canonical HIR, bidirectional types, MVS, CT lattice)
  - name: ueas_verify
    description: Progressive L0-L3 verification & cryptographic SHA-256 certificate
  - name: ueas_transpile
    description: Dual-track transpilation across 12 Track A/B/C targets
  - name: ueas_extract
    description: Tree-sitter extraction from Python, TypeScript, or Rust with Axiom 1 purity audit
status: stable
```

## Installation

```bash
# Local stdio transport via unified CLI
cargo install --path tools/ueas-cli

# Claude Desktop / MCP client config:
{
  "mcpServers": {
    "ueas": {
      "command": "ueas",
      "args": ["mcp"]
    }
  }
}
```
