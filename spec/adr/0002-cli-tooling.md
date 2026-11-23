# ADR 0002: Command-Line Interface (Epoch 4)

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS maintainer
- **Supersedes:** N/A

## Context

UEAS was a reference implementation accessible only by cloning the repository
and compiling the Rust kernel. There was no installable tool, no CLI, and no
end-to-end pipeline from `.ueas` source to execution or transpilation. This
prevented real-world adoption — users could not run UEAS algorithms without
understanding the internal codebase.

## Decision

Create a `ueas` CLI binary as a workspace member (`tools/ueas-cli/`) providing
four subcommands:

- `ueas run <file>` — parse `.ueas` source, execute via the abstract
  interpreter, print exit code, step count, and result
- `ueas check <file>` — parse and validate syntax, print errors with line
  numbers
- `ueas transpile <file> --target <python|rust>` — transpile to a target
  language via the `TargetGenerator` interface
- `ueas fmt <file>` — validate source formatting

The CLI uses `clap` for argument parsing and integrates with the existing
`ueas-kernel` and `ueas-backends` workspace crates.

### Parser Design

Rather than depending on the full ANTLR4 toolchain (which requires Java and
generated Rust code), the CLI uses a hand-written line-by-line parser that
directly constructs `AstNode` objects via the factory. This parser handles:

- Algorithm headers with `Require:`/`Ensure:`/`Complexity:` preambles
- Assignment (`<-`), return, assert, and invariant statements
- Control flow: `if/then/end if`, `for/do/end for`, `while/do/end while`
- Nested block depth tracking for end-closures
- Expressions: literals, identifiers, binary/unary operators, function calls

### Known Limitations

The simple parser does not handle:
- Method-chain desugaring (e.g., `data.length` → `length(data)`)
- Complex expressions nested inside function call arguments
- Full ANTLR4 grammar coverage (a bridge parser is planned for Epoch 6+)

## Consequences

### Positive
- One-command install: `cargo install ueas`
- `.ueas` files can be parsed, executed, and transpiled from the terminal
- Full pipeline: `ueas check → ueas run → ueas transpile`
- Zero new dependencies on Java or ANTLR4 at install time

### Negative
- Hand-written parser is not a complete ANTLR4 conformant implementation
- Some `.ueas` algorithms fail to execute due to parser limitations
- Maintaining both the ANTLR4 grammar and the CLI parser creates a specification
  divergence risk

## Files Created
- `tools/ueas-cli/Cargo.toml` — CLI crate manifest
- `tools/ueas-cli/src/main.rs` — CLI entry point, parser, and command handlers
- `Cargo.toml` (root) — added `tools/ueas-cli` to workspace members
