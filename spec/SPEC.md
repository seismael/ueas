# Universal Executable Algorithm Standard (UEAS) — Specification v2.0.0

**License:** Apache License 2.0  
**Version:** `2.0.0`  
**Status:** Executable Normative Standard

---

## 1. Purpose & Design Axioms

UEAS is the **Verification-Aware Algorithm Blueprint (VAAB)** standard. It bridges human-readable academic pseudocode, deterministic resource-metered execution, progressive formal verification, and idiomatic multi-language production code generation.

### 1.1 Immutable Axioms

1. **Axiom 1 — Pure Deterministic Execution (Zero Host I/O):** The UEAS VM has zero access to filesystem, network, system clock, or OS syscalls. Stochasticity is exclusively via an explicitly seeded deterministic PRNG (`PRNG(seed)`).
2. **Axiom 2 — Normative Abstract Cost Semantics:** Every bytecode instruction and collection primitive carries a spec-defined step cost, live heap accounting, fork-join Work/Span DAG cost, and virtual-address cache cost. Identical program + inputs + seed produces identical outputs and resource profiles on any CPU/OS.
3. **Axiom 3 — Mutable Value Semantics (MVS) & Zero Reference Cycles:** No aliasing, no garbage collector, no cyclic object graphs. Parameters are immutable by default; in-place mutations require explicit `mut` at declaration and call site. Recursive data structures use flat `List`/`Map` arenas + value-semantic `Record`s + `Option<Int>` indices.
4. **Axiom 4 — Explicit Boundaries, Inferred Internals:** Algorithm signatures and `Record` fields carry explicit static types (with optional trait bounds `<T: Ord>`). Local variables use bidirectional Hindley-Milner type inference.
5. **Axiom 5 — Contracts & Tests Are First-Class Syntax:** `Require:`, `Ensure:`, `invariant`, `decreases`, `forall`/`exists`, `@Test`, and `@Benchmark` are native AST/HIR constructs enforced by the VM, Property Fuzzer, Complexity Fitter, and Formal Provers.

---

## 2. Lexical & Context-Free Grammar (Normative EBNF)

UEAS uses explicit block delimiters (`then`/`do`/`=>` ... `end if`/`end while`/`end for`/`end match`/`end Algorithm`/`end Record`/`end Test`/`end Benchmark`) rather than whitespace indentation tokens. Structural keywords accept lowercase, TitleCase, or UPPERCASE (`if`, `If`, `IF`).

```ebnf
program        ::= moduleDecl? importDecl* topDecl* EOF
moduleDecl     ::= 'module' dottedIdent
importDecl     ::= 'import' ':'? dottedIdent
dottedIdent    ::= IDENT ('.' IDENT)*

topDecl        ::= recordDecl | algorithmDecl | testDecl | benchmarkDecl

recordDecl     ::= 'Record' IDENT typeParams?
                   (IDENT ':' typeExpr)*
                   'end' 'Record'

algorithmDecl  ::= decorator*
                   'Algorithm' IDENT typeParams? '(' paramList? ')' ('->' typeExpr)?
                   requireBlock?
                   ensureBlock?
                   complexityBlock?
                   stmt*
                   'end' 'Algorithm'

decorator      ::= '@ConstantTime'
                 | '@HardwareProfile' '(' hwParam (',' hwParam)* ')'

paramList      ::= param (',' param)*
param          ::= 'mut'? IDENT ':' typeExpr

requireBlock   ::= 'Require' ':' (logicalOr | paramTypeCompat)*
ensureBlock    ::= 'Ensure' ('(' IDENT ')')? ':' (logicalOr | typeExpr)*
complexityBlock::= 'Complexity' ':' complexityEntry (',' complexityEntry)*

testDecl       ::= '@Test' STRING_LIT stmt* 'end' 'Test'
benchmarkDecl  ::= '@Benchmark' STRING_LIT
                   'scales' '[' INT_LIT (',' INT_LIT)* ']'
                   'generator' '(' IDENT ')' '=>' expr
                   'end' 'Benchmark'
```

### 2.1 Operator Precedence (Lowest to Highest)

| Level | Operators | Associativity |
|---|---|---|
| 1 | `==>`, `<==>` | Right |
| 2 | `or` | Left |
| 3 | `and` | Left |
| 4 | `==`, `!=`, `<`, `<=`, `>`, `>=`, `in`, `not in` | Left |
| 5 | `\|` (Bitwise OR) | Left |
| 6 | `^` (Bitwise XOR) | Left |
| 7 | `&` (Bitwise AND) | Left |
| 8 | `<<`, `>>` | Left |
| 9 | `+`, `-` | Left |
| 10 | `*`, `/`, `mod`, `%` | Left |
| 11 | `not`, `-` (unary), `~`, `spawn` | Right (Prefix) |
| 12 | `as Type` | Left (Postfix) |
| 13 | `f(args)`, `a[i]`, `a[s:e]`, `x.f`, `x.m(args)` | Left (Postfix) |

---

## 3. Type System

- **Primitives:** `Int` (arbitrary-precision $\mathbb{Z}$; alias `Integer`), `Int64`, `UInt64`, `UInt32`, `Byte`, `Real` (canonical IEEE-754 binary64), `Bool` (alias `Boolean`), `String`, `Void`.
- **Algebraic & Composite:** `Option<T>` (`Some(v)` | `None`), `Result<T, E>` (`Ok(v)` | `Err(e)`), `Tuple<T1, ...>`, `List<T>`, `Set<T: Eq + Hash>`, `Map<K: Eq + Hash, V>`, `Graph<N: Eq + Hash, W>` (`DirectedGraph` / `UndirectedGraph`), `Matrix<T>`, `Record`.
- **Security:** `Secret<T>` — tracked by the `Public ⊑ Secret` information-flow lattice inside `@ConstantTime` algorithms (`select(cond, a, b)` for branchless conditional selection, `declassify(x)` for audited release).
- **Trait Bounds:** `Eq`, `Ord`, `Hash`, `Numeric`.

---

## 4. Canonical JSON HIR Schema

All tooling exchanges typed HIR conforming to `spec/schemas/ueas-hir.v2.schema.json`. Every node uses named fields and an explicit `"kind"` discriminant (`#[serde(tag = "kind")]`).

---

## 5. Abstract Cost Semantics & Trap Table

| Operation | Step Cost |
|---|---|
| Arithmetic, comparison, boolean, bitwise, local load/store | `1` |
| `List` index get/set, `append`, `pop`, `length` | `1` |
| `List` `slice` / `concat` / `contains` (length `n`) | `n` |
| `Set` `contains`, `add`, `remove` | `1` |
| `Set` `union`, `intersection`, `difference` (sizes `n`, `m`) | `n + m` |
| `Map` `get`, `put`, `containsKey`, `remove` | `1` |
| `Graph` `addNode`, `addEdge`, `weight`, `degree` | `1` |
| `Graph` `adjacent`, `neighbors` | `max(1, degree(v))` |
| `Matrix` `get`, `set` | `1` |
| `Matrix` `transpose` (`r * c`), `multiply` (`a * b * c`) | `r * c` / `a * b * c` |
| Function call overhead | `1 + body` |

### 5.1 Trap Codes

| Code | Name | Cause |
|---|---|---|
| `0` | `NoError` | Normal termination |
| `1` | `DivisionByZero` | Division or modulo by zero |
| `2` | `IndexOutOfBounds` | Out-of-bounds `List`/`Matrix`/`Tuple`/`String` access |
| `3` | `UnwrapNone` | Unwrapping `None` or missing `Map` key |
| `4` | `PreconditionViolation` | `Require:` predicate evaluated to `false` |
| `5` | `PostconditionViolation` | `Ensure:` predicate evaluated to `false` |
| `6` | `InvariantViolation` | `invariant` predicate evaluated to `false` |
| `7` | `DecreasesViolation` | `decreases` rank `< 0` or not strictly decreasing |
| `8` | `ComplexityViolation` | Time complexity contract breached |
| `9` | `SpaceComplexityViolation` | Auxiliary space complexity contract breached |
| `10` | `StackOverflow` | Call recursion depth exceeded limit |
| `11` | `HeapExhaustion` | Virtual heap byte limit exceeded |
| `12` | `AssertionFailure` | `assert` predicate evaluated to `false` |
| `13` | `StepBudgetExceeded` | Global fuel/step limit exceeded |
| `14` | `IntegerOverflow` | Fixed-width `Int64`/`UInt64` overflow |
| `15` | `TimingLeak` | Observational constant-time trace divergence |
| `16` | `InvalidComplexityBinding` | Unbound complexity variable |

---

## 6. 5-Level Verification Ladder

- **L0 (`StaticSoundness`):** Type check, MVS `mut`/cycle check, purity, `@ConstantTime` security lattice.
- **L1 (`ContractAndProperty`):** `@Test` suite + precondition-guided property fuzzing with shrinking.
- **L2 (`AdversarialComplexity`):** Multi-scale ($N \in \{8 \dots 1024\}$) adversarial input families + BIC-penalized regression for Time & Auxiliary Space + Work/Span DAG + Observational Constant-Time trace equality.
- **L3 (`FormalProof`):** Machine-checked proof via Dafny 4.6 (Z3), Lean 4, Verus, or Quint/TLA+.
- **L4 (`DifferentialEquivalence`):** Compiled cross-target output equivalence across Rust, Python, TypeScript, C++20, and Go.
