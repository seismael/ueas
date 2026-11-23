# ADR 0012: Academic Bridge (LaTeX Target)

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS Architectural Review
- **Supersedes:** None
- **Superseded By:** None

## Context

UEAS claims to be the definitive standard for "Academic Pseudocode". However, without a native way to export algorithms into a format suitable for academic publishing (e.g., IEEE/ACM conferences), researchers will be forced to manually translate UEAS code into LaTeX by hand. This introduces transcription errors and massive friction, heavily bottlenecking adoption in academia.

## Decision

We will build a native `TargetGenerator` for LaTeX (`backends/src/latex.rs`).
This backend will take the canonical UEAS JSON AST and transpile it directly into the syntax expected by the `algorithm2e` LaTeX package, which is the industry standard for typesetting algorithms in computer science papers.

## Consequences

**Positive:**
- Zero-friction academic adoption: Researchers can profile an algorithm using the UEAS kernel and immediately export it for publication.
- Ensures that the pseudocode printed in a published paper is 100% semantically equivalent to the profiled execution, eliminating "pseudocode drift."

**Negative:**
- The LaTeX generator is fundamentally different from other backends (Python, Rust, C++) because LaTeX is not an executable programming language in this context; it is a typesetting language. The semantic mapping logic will be purely cosmetic.

## Alternatives Considered

1. **algorithmicx / algpseudocode package:** We selected `algorithm2e` because it maps more cleanly to UEAS's `Require/Ensure` preamble structure (`\KwIn`, `\KwOut`) and block formatting.
