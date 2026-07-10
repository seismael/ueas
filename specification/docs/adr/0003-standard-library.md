# ADR 0003: Standard Algorithm Library (Epoch 5)

- **Status:** Accepted
- **Date:** 2026-07-10
- **Deciders:** UEAS maintainer
- **Supersedes:** N/A

## Context

UEAS had 10 example algorithm files but no organized standard library.
Potential adopters (LeetCode-style platforms, university courses, technical
interview tools) need a comprehensive, pre-verified corpus of algorithms to
make the standard immediately useful. Without a library, each adopter must
write their own algorithms from scratch.

## Decision

Create a `library/` directory with 45 verified UEAS algorithm files across 7
categories:

| Category | Count | Rationale |
|----------|-------|-----------|
| Sorting | 8 | Core CS curriculum, interview staples |
| Searching | 3 | Complement to sorting, interview staples |
| Graph Algorithms | 9 | Graph theory foundation, LeetCode staple |
| Dynamic Programming | 5 | DP is a distinct algorithmic paradigm |
| Mathematics | 8 | Number theory, cryptography, signal processing |
| String Algorithms | 5 | Text processing, bioinformatics |
| Data Structures | 6 | Building blocks for other algorithms |

### Quality Standards

Every algorithm in the library MUST:
1. Use v3.0 academic pseudocode syntax (`<-` assignment, `then`/`do`/`end`
   closures, `Require:`/`Ensure:`/`Complexity:` preamble)
2. Declare a correct `Complexity:` contract with bindings
3. Parse successfully via `ueas check`
4. Be self-contained (no imports for core logic)

### Organization

```
library/
  INDEX.md               — Complete catalog with name, category, complexity
  sorting/*.ueas         — 8 files
  searching/*.ueas       — 3 files
  graph/*.ueas           — 9 files (3 existing + 6 new)
  dp/*.ueas              — 5 files
  math/*.ueas            — 8 files
  strings/*.ueas         — 5 files
  data_structures/*.ueas — 6 files
```

Existing example files (`examples/bfs.ueas`, `examples/dfs.ueas`,
`examples/dijkstra.ueas`) are also available in `library/graph/` for
convenience.

## Consequences

### Positive
- Immediate utility for adopters — 45 pre-written algorithms
- Covers the full CS undergraduate algorithms curriculum
- All files use v3.0 academic syntax (consistent)
- `library/INDEX.md` provides a searchable catalog

### Negative
- 45 files to maintain as the grammar evolves
- Some complex algorithms may not execute in the current interpreter due to
  parser limitations (parse-verified but not execute-verified)
- Library does not include execution test suite (each algorithm has `Complexity:`
  but no automated conformance test against it)
