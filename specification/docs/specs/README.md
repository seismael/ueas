# Detailed Per-Domain Specifications

This directory contains detailed specifications for each UEAS domain.
These documents supplement [SPEC.md](../../SPEC.md) with implementation-level
detail without duplicating the formal specification.

## Directory Contents

| File | Domain | Purpose |
|------|--------|---------|
| — | (No specs yet) | — |

## When to Create a Domain Spec

Create a domain specification document when:

1. A ratified RFC introduces significant implementation complexity that
   warrants detailed documentation beyond what fits in the RFC or SPEC.md.
2. A design decision requires coordination across multiple domains.
3. The reference implementation needs a document that bridges the formal
   specification and the codebase.

## Format

Domain specs follow the same header conventions as RFCs and ADRs:

```markdown
# Domain Spec: Short Title

- **Domain:** grammar | kernel | backends
- **RFC Reference:** RFC NNNN
- **Date:** YYYY-MM-DD
- **Status:** Draft | Active | Superseded

## Overview

## Detailed Design

## Domain Interfaces

## Testing Strategy
```
