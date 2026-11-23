# ADR 0015: Jupyter Kernel Integration

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

Data scientists and academic researchers live in Jupyter Notebooks. The current CLI-based UEAS workflow forces them out of their data-exploration environment. If we want UEAS to be used for algorithmic benchmarking in papers, it must run inside `.ipynb` cells natively.

## Decision

We will implement a custom Jupyter Kernel (`tools/ueas-jupyter/`). 
This will be a lightweight server that implements the Jupyter Messaging Protocol over ZeroMQ. It will accept cell text, pass it to the UEAS interpreter, and return rich HTML payload (containing complexity graphs and step counts) via the `display_data` protocol message.

## Consequences

**Positive:**
- Seamless integration into Python data science workflows. Researchers can use Python to generate datasets, and a UEAS cell to process the algorithm, comparing the complexity graphs visually.

**Negative:**
- Implementing the ZeroMQ heartbeat and messaging protocol in Rust introduces a new, heavy network dependency into the `tools/` workspace.

## Alternatives Considered

1. **Jupyter Magic Command in Python:** We considered writing an `ipython` extension (`%%ueas`) instead of a full kernel. However, this relies on Python running in the background and shelling out to the `ueas` binary, which is brittle and introduces IPC lag. A native kernel provides significantly better performance.
