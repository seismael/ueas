# Architecture Decision Records (ADR)

Architecture Decision Records capture significant architectural decisions
made during the development of the UEAS standard and reference
implementation.

## ADR Format

Each ADR is a Markdown file in this directory following the naming pattern:

```
NNNN-short-title.md
```

Where `NNNN` is a zero-padded sequential number.

## ADR Template

```markdown
# ADR NNNN: Short Descriptive Title

- **Status:** Proposed | Accepted | Deprecated | Superseded
- **Date:** YYYY-MM-DD
- **Deciders:** Names of decision-makers
- **Supersedes:** ADR NNNN (if applicable)
- **Superseded By:** ADR NNNN (if applicable)

## Context

What is the issue motivating this decision? What constraints are at play?

## Decision

What is the decision? What will we do and not do?

## Consequences

What becomes easier or more difficult because of this decision? What are
the positive, negative, and neutral outcomes?

## Alternatives Considered

What other options were evaluated? Why were they not chosen?
```

## ADR Index

| ADR | Title | Status | Date |
| ADR | Title | Status | Date |
|-----|-------|--------|------|
| 0001 | Academic Pseudocode Syntax | Accepted | 2026-07-09 |
| 0002 | CLI Tooling | Accepted | 2026-07-10 |
| 0003 | Standard Library | Accepted | 2026-07-10 |
| 0004 | Backend Expansion | Accepted | 2026-07-10 |
| 0005 | Concurrency via Work/Span DAG Profiling | Accepted | 2026-07-10 |
| 0006 | Formal Verification Target Generation | Accepted | 2026-07-10 |
| 0007 | WebAssembly (WASM) Playground Architecture | Accepted | 2026-07-10 |

---

*ADRs are immutable once accepted. Superseded ADRs remain in the index for historical reference.*
