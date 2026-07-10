# ADR 0013: AI Agent Integration (MCP Server)

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

AI coding assistants and orchestration agents are rapidly becoming the primary authors of boilerplate code. If UEAS is to become the universal standard, AI agents must be able to natively interact with it. Currently, agents can write `.ueas` files, but they have no formal, structured protocol to execute the abstract interpreter and parse the complexity profiles programmatically.

## Decision

We will implement a Model Context Protocol (MCP) server wrapper around the UEAS Kernel (`tools/ueas-mcp/`).
This server will expose discrete, stateless tools to AI clients:
- `parse_ueas`: Validate syntax and return AST structure.
- `execute_ueas`: Run the code in the Virtual Heap and return deterministic step counts and exit codes.
- `transpile_ueas`: Return the target language code.

## Consequences

**Positive:**
- AI agents can use UEAS to empirically prove the time complexity of their generated algorithms before transpiling them into the user's target codebase, effectively preventing LLM hallucinations in algorithmic logic.
- MCP is an open standard, ensuring compatibility with Claude Desktop, Cursor, Zed, and other AI IDEs.

**Negative:**
- Adds maintenance overhead for a networking/IPC layer outside the core `kernel/`.

## Alternatives Considered

1. **Raw CLI Parsing:** We could force AI agents to use the `ueas-cli` via terminal commands and regex-parse the stdout. This is brittle and error-prone compared to MCP's structured JSON RPC interface.
