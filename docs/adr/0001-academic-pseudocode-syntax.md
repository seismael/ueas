# ADR 0001: Academic Pseudocode Syntax (v3.0)

- **Status:** Accepted
- **Date:** 2026-07-09
- **Deciders:** UEAS maintainer
- **Supersedes:** N/A

## Context

The UEAS v2.0 grammar used systems-programming conventions (`let`, `{ }` braces,
`@Complexity` decorators, inline type annotations) that were unfamiliar to
academic computer scientists and made algorithm transcription from classical
textbooks (CLRS, LaTeX `algorithmicx`) error-prone. Mass adoption requires
syntax that is instantly recognizable to anyone who has read a CS paper.

## Decision

Refactor the UEAS frontend to adopt academic pseudocode conventions while
preserving the JSON AST schema, VirtualHeap, step counter, trap register, and
backend transpilers byte-for-byte. The Iceberg Architecture ensures the rigorous
Rust kernel is untouched while the user-facing grammar shifts to textbook style.

### Specific Changes

| Concern | Before (v2.0) | After (v3.0) |
|---------|---------------|--------------|
| Algorithm header | `algorithm Name(p: T) -> T` | `Algorithm Name(p)` with `Require:`/`Ensure:` preamble |
| Complexity annotation | `@Complexity("O(N)")` | `Complexity: "O(N)"` |
| Memory annotation | `@Memory("O(N)")` | `Memory: "O(N)"` |
| Assignment operator | `:=` | `<-` (academic arrow); `:=` retained for backward compat |
| Variable declaration | `let x: T := v` | `x <- v` (implicit declaration on first assignment) |
| Block delimiters | `{ ... }` / Pythonic `:` | `then`/`do`...`end if`/`end for`/`end while` |
| Comments | `//` | `#` |
| Case sensitivity | Strict lowercase | Case-insensitive keywords (e.g., `if`/`If`/`IF`) |
| Semicolons | Supported | Removed |

### What Was NOT Changed

- JSON AST schema — all node kinds preserved, AST output structurally identical
- `AstNodeKind` enum — zero changes
- `VirtualHeap`, `SymbolTable`, `StepCounter`, `TrapRegister` — zero changes
- Complexity profiler — zero changes
- `PythonTarget` — zero changes
- `TargetGenerator` trait — zero changes

## Consequences

### Positive
- **Immediate recognition** by academics and CS professionals
- **Eliminates the `let` learning curve** — variables are declared implicitly on first assignment
- **Case-insensitive keywords** accommodate personal style preferences
- **`end if` / `end for` / `end while`** closures are immune to copy-paste whitespace errors
- **`Require:` / `Ensure:` / `Complexity:` preamble** matches LaTeX `algorithmicx` conventions

### Negative
- **`RustTarget` must track variables** to emit `let mut` on first use (implemeted via `HashSet<String>`)
- **Interpreter must handle implicit declaration** — `execute_assignment` now auto-declares on `NullDereference`
- **Grammar complexity increased** — 6 new keyword tokens with case-insensitive variants
- **Legacy `.ueas` files incompatible** — all examples and test files required rewriting

## Scope

| Domain | Changes |
|--------|---------|
| `grammar/UEAS.g4` | Complete rewrite (v2.0 → v3.0 academic syntax) |
| `kernel/src/interp/mod.rs` | `execute_assignment` auto-declare |
| `backends/src/lib.rs` | `RustTarget` `HashSet<String>` var tracking |
| `examples/*.ueas` (10 files) | Rewritten to v3.0 syntax |
| `grammar/tests/` (10 files) | Rewritten to v3.0 syntax |
| `SPEC.md` | Sections 4, 5.1, 5.3, 6.1, 6.2, 6.4, 6.5, 6.6, App B, App C updated |
| `kernel/src/ast/mod.rs` | No changes (backward-compat) |
| `kernel/src/heap/`, `kernel/src/traps/`, `kernel/src/profiling/` | No changes |
