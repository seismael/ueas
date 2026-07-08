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
|-----|-------|--------|------|
| —   | (No decisions recorded yet) | — | — |

---

*ADRs are immutable once accepted. Superseded ADRs remain in the index for historical reference.*
