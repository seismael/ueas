# RFC Process — UEAS Specification Changes

The RFC (Request for Comments) process is the sole mechanism for proposing
changes to the UEAS specification. No specification change may be implemented
without a ratified RFC.

## RFC Numbering

RFCs are numbered sequentially starting from `0001`. Check the `docs/rfcs/`
directory for the highest existing number and increment by one.

RFC file names follow the pattern: `NNNN-short-title.md`

Examples:
- `0001-graph-literal-syntax.md`
- `0002-tensor-primitives.md`
- `0003-distributed-execution-scopes.md`

## RFC Lifecycle

```
      +----------+      +----------+      +-------------+      +---------------+
      |  Draft   | ---> |  Review  | ---> | Ratified    | ---> | Implemented   |
      +----------+      +----------+      +-------------+      +---------------+
           |                 |                   |                     |
           v                 v                   v                     v
      [Abandoned]      [Rejected]          [Superseded]           [Superseded]
```

### State Descriptions

| State | Description |
|-------|-------------|
| **Draft** | Proposal submitted but not yet under review. |
| **Review** | Under active review by at least two approved reviewers. Minimum 14 days. |
| **Ratified** | Accepted by consensus. Specification updated. Code implementation may begin. |
| **Implemented** | Reference implementation passes all conformance tests for the RFC. |
| **Superseded** | Obsoleted by a newer RFC. |
| **Rejected** | Declined with public written rationale. |
| **Abandoned** | Author withdrew or was unresponsive for 90 days. |

## Submitting an RFC

1. Copy the template below into a new file `docs/rfcs/NNNN-title.md`.
2. Fill in all sections marked "REQUIRED".
3. Set status to `Draft`.
4. Submit a pull request with the RFC file only. No implementation code.
5. Notify the reviewers.

## Review Criteria

RFCs are evaluated on:

| Criterion | Question |
|-----------|----------|
| **Motivation** | Does the proposal solve a real problem? Is the problem clearly described? |
| **Correctness** | Is the proposed change mathematically sound? Does it preserve existing invariants? |
| **Complexity Impact** | Does the change complicate or simplify the standard? Does it introduce new complexity classes that cannot be enforced? |
| **Backward Compatibility** | Does the change break existing UEAS programs? If so, what is the migration path? |
| **Implementability** | Can the change be implemented in the kernel and all supported targets? |

## Template

```markdown
# RFC NNNN: Short Descriptive Title

- **Status:** Draft
- **Author:** Name <email>
- **Date:** YYYY-MM-DD
- **Supersedes:** (if applicable)
- **Superseded By:** (if applicable)

## Motivation

REQUIRED. Why is this change needed? What problem does it solve?

## Proposed Change

REQUIRED. Describe the change precisely. Use formal notation where
applicable.

## Impact Analysis

REQUIRED. Address each:

- **Grammar changes:** What production rules are added, modified, or removed?
- **AST schema changes:** What node kinds are affected?
- **Kernel changes:** What new trap codes, operations, or invariants are needed?
- **Backward compatibility:** Do existing UEAS programs remain valid?

## Alternatives Considered

OPTIONAL. What other approaches were evaluated and why were they rejected?

## Reference Implementation Plan

REQUIRED. High-level plan for the reference implementation.

- Which domain(s) are affected: `grammar/`, `kernel/`, `backends/`?
- Estimated epoch alignment: Epoch 1, 2, or 3?
- New dependencies required?

## Rejection Rationale

(Filled by reviewers if rejected.)
```

## Reviewers

Reviewers are contributors who have at least one ratified RFC to their name.
The reviewer list is maintained by the project maintainers.

---

*For questions about the RFC process, open an issue in the project tracker.*
