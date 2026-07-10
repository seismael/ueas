# ADR 0014: Interactive Debugger (DAP Protocol)

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

The UEAS kernel runs algorithms abstractly and verifies invariants, but it does so as an opaque black box. To support teaching, debugging, and AI-assisted algorithm generation, developers need to be able to pause the kernel execution and inspect the Virtual Heap state step-by-step.

## Decision

We will build a Debug Adapter Protocol (DAP) server wrapper around the abstract interpreter (`tools/ueas-dap/`). 
The interpreter core will be refactored to support a yield-based `step_execution()` loop rather than a monolithic `execute_program()`. The DAP server will listen for IDE breakpoint events, pause the `step_execution()` loop when line numbers match, and serialize the Virtual Heap state back to the IDE variables pane.

## Consequences

**Positive:**
- Massive adoption boost for computer science education; students can step through sorting algorithms visually.
- Immediate integration into VS Code, Zed, and Cursor without writing IDE-specific extensions beyond a basic `package.json` config.

**Negative:**
- The kernel execution loop must be heavily refactored to support asynchronous pausing without losing stack state.

## Alternatives Considered

1. **Custom Debugger Protocol:** We considered writing a proprietary debugger CLI, but this would require users to learn new terminal commands. DAP is universally supported by modern editors.
