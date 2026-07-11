# UEAS MCP Server — Anthropic Directory Submission

## Manifest

```yaml
name: ueas
display_name: UEAS — Universal Executable Algorithm Standard
description: |
  Parse, execute, and transpile mathematically-verified algorithms.
  Formal complexity guarantees (O(1) to O(N!)) enforced by a Rust
  abstract interpreter with zero system I/O. 8 transpiler targets
  including Lean 4 theorem proofs and TLA+ model checking.
version: 4.1.0
repository: https://github.com/seismael/ueas
license: Apache-2.0
tags:
  - algorithms
  - formal-verification
  - transpiler
  - computational-science
  - education
tools:
  - name: parse_ueas
    description: Validate UEAS academic pseudocode syntax
  - name: execute_ueas
    description: Run an algorithm in the virtual heap sandbox with step-count profiling
  - name: transpile_ueas
    description: Transpile to Python, Rust, C++17, Java 17, JavaScript, Lean 4, TLA+, or LaTeX
contact: maintainers@ueas.dev
status: stable
```

## Installation

```bash
# Local (stdio transport)
cd implementation && cargo install --path tools/ueas-mcp

# Add to Claude Desktop config:
{
  "mcpServers": {
    "ueas": {
      "command": "ueas-mcp"
    }
  }
}
```

