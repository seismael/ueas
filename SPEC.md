# Universal Executable Algorithm Standard (UEAS)

**Version:** 1.0.0-draft  
**Status:** Working Draft  
**License:** Apache License 2.0  
**Copyright:** UEAS Contributors  

---

## 1. Abstract

### 1.1. Purpose

The Universal Executable Algorithm Standard (UEAS) is a formal specification for
representing, validating, and transpiling algorithms in a language-agnostic,
mathematically rigorous format. UEAS defines a canonical Abstract Syntax Tree
(AST) representation, a grammar for human-readable algorithmic pseudocode, an
abstract interpreter with built-in complexity-invariant enforcement, and a
pluggable transpilation target interface.

No existing technology fills this niche. Formal verification languages (TLA+,
Coq, Alloy) prove state-machine correctness but cannot execute, transpile, or
profile algorithms. General-purpose transpilers (Haxe, Nim) carry hardware
baggage — memory management, I/O primitives, and system calls — that have no
place in a pure algorithm definition. Intermediate representations (LLVM IR,
MLIR) are compiler internals, unreadable to algorithm designers. Academic
pseudocode (LaTeX algorithm2e) is purely visual: it cannot be parsed, executed,
tested, or debugged.

UEAS closes this gap. It treats algorithmic logic as a first-class, deployable
asset — decoupled from programming language syntax, hardware architecture, and
execution environment.

### 1.2. Design Principles

The standard ensures that any algorithm expressed in UEAS:

- is syntactically unambiguous and machine-verifiable;
- carries an explicit computational complexity contract;
- can be transpiled to any conformant target language with guaranteed semantic
  equivalence;
- has no side effects beyond its declared scope (pure mathematical semantics
  within the kernel);
- profiles complexity via deterministic logical step-counting, not
  environment-dependent wall-clock time;
- serves as its own specification — the algorithm is the auditable artifact.

### 1.3. Target Domains

UEAS targets domains where algorithmic correctness, auditability, and
reproducibility are non-negotiable:

- **Quantitative Finance:** Eliminating the translation lag between research
  prototypes (Python/MATLAB) and production execution (C++/Rust) while
  maintaining a verifiable validation trail.
- **Aerospace & Defense:** Enabling DO-178C certification by embedding
  pre-conditions, post-conditions, and invariants directly in the algorithm
  definition, supporting automated validation against formal requirements.
- **AI Agent Orchestration:** Providing autonomous coding agents with a
  minimal, logical target grammar that reduces token-space errors and enables
  sandboxed testing before target-language code generation.
- **Scientific Computing & Academic Publishing:** Making published algorithms
  downloadable, executable, and auditable — ending the era of untestable
  paper pseudocode.

---

## 2. Definitions and Terminology

### 2.1. Core Terms

| Term | Definition |
|------|------------|
| **Abstract Syntax Tree (AST)** | A directed, acyclic, rooted tree whose nodes are typed algorithmic constructs and whose edges represent syntactic containment. The AST is the canonical intermediate representation of all UEAS programs. |
| **Invariant** | A logical predicate that must hold true at a specific point during execution. Invariants are declared explicitly in the grammar and enforced by the kernel at runtime. |
| **Complexity Contract** | A mandatory annotation on every algorithmic scope specifying the worst-case asymptotic time complexity bound in standard Big-O notation (e.g., `O(N log N)`). The kernel monitors step-count and traps if the bound is breached. |
| **Transpilation Target** | A conformant language backend that maps the canonical AST to idiomatic source code in a target language. Targets are implemented as plugins against the `TargetGenerator` interface. |
| **Epoch** | A phased delivery milestone defined by the UEAS roadmap. Each epoch delivers a discrete, testable increment of the standard and reference implementation. |
| **RFC (Request for Comments)** | The sole mechanism for proposing changes to the specification. RFCs proceed through a formal state machine from Draft to Ratification before code is written. |
| **Virtual Heap** | An isolated memory region managed by the abstract interpreter. The heap has no access to system I/O, network, filesystem, or hardware. All allocations and deallocations are logged for complexity profiling. |
| **Step Count** | The number of primitive operations executed by the abstract interpreter. The step count is the basis for invariant enforcement and complexity validation. |

### 2.2. Notation Conventions

- **MUST** / **MUST NOT** — Absolute requirement. Non-negotiable for conformance.
- **SHOULD** / **SHOULD NOT** — Strong recommendation. Deviations require documented justification.
- **MAY** — Optional, implementation-defined behavior.
- Terms defined in this section appear in **bold** on first use.

---

## 3. Type System

UEAS enforces a **static, explicit, nominal type system**. Every variable,
parameter, and return value MUST declare its type at the point of introduction.
No type inference is performed by the parser or kernel.

### 3.1. Primitive Types

| Type | Description | Example Literal |
|------|-------------|-----------------|
| `Integer` | Arbitrary-precision signed integer | `42`, `-1` |
| `Real` | IEEE 754 double-precision floating point | `3.14159`, `-2.5e10` |
| `Boolean` | Logical truth value | `true`, `false` |
| `String` | UTF-8 encoded character sequence | `"hello"` |
| `Void` | Absence of a value (used for procedure return types) | — |

### 3.2. Composite Types

| Type | Description |
|------|-------------|
| `Set<T>` | Unordered collection of distinct elements of type `T`. Operations: `union`, `intersection`, `difference`, `cardinality`, `contains`. |
| `List<T>` | Ordered, indexable sequence of elements of type `T`. Operations: `get`, `append`, `prepend`, `length`, `slice`. |
| `Map<K, V>` | Associative mapping from keys of type `K` to values of type `V`. Operations: `get`, `put`, `containsKey`, `keys`, `values`. |
| `Graph<N, E>` | A directed or undirected graph with nodes of type `N` and edges of type `E`. Edges carry an optional weight of type `Real`. Operations: `adjacent`, `neighbors`, `addNode`, `addEdge`, `removeNode`. |
| `Matrix<R, C, T>` | A fixed-size `R` × `C` matrix of elements of type `T`. Indexing is 0-based. Operations: `get`, `set`, `transpose`, `multiply`, `determinant` (square matrices only). |
| `Tuple<T1, T2, ...>` | Fixed-length heterogeneous ordered collection. Elements accessed by position. |
| `Option<T>` | Represents a value that may be present (`Some(value)`) or absent (`None`). |
| `Result<T, E>` | Represents either a success value of type `T` or an error of type `E`. |

### 3.3. Type Safety Rules

1. Implicit type coercion is forbidden. All type conversions MUST be explicit via cast operators.
2. The parser MUST reject any program that contains a type mismatch at compile time.
3. Generic type parameters are invariant by default. Covariance and contravariance are NOT supported in version 1.0.
4. Recursive type definitions (e.g., `Graph<Node, Edge>` where `Node` contains a `Graph`) are permitted but MUST terminate in a primitive type.

---

## 4. Grammar Specification (Epoch 1)

The UEAS grammar is specified in Extended Backus-Naur Form (EBNF). The
reference implementation uses ANTLR4 (`UEAS.g4`) as the parser generator. The
grammar is the normative definition of valid UEAS syntax.

### 4.1. Lexical Tokens

```ebnf
(* Keywords *)
ALGORITHM   ::= 'algorithm'
FUNCTION    ::= 'function'
PROCEDURE   ::= 'procedure'
RETURN      ::= 'return'
IF          ::= 'if'
ELSE        ::= 'else'
FOR         ::= 'for'
WHILE       ::= 'while'
IN          ::= 'in'
LET         ::= 'let'
ASSERT      ::= 'assert'
INVARIANT   ::= 'invariant'
COMPLEXITY  ::= 'complexity'
GRAPH       ::= 'graph'
SET         ::= 'set'
LIST        ::= 'list'
MAP         ::= 'map'
MATRIX      ::= 'matrix'
OPTION      ::= 'option'
RESULT      ::= 'result'
SOME        ::= 'some'
NONE        ::= 'none'
OK          ::= 'ok'
ERR         ::= 'err'
TRUE        ::= 'true'
FALSE       ::= 'false'
AND         ::= 'and'
OR          ::= 'or'
NOT         ::= 'not'
MOD         ::= 'mod'

(* Literals *)
IDENTIFIER  ::= [a-zA-Z_][a-zA-Z0-9_]*
INTEGER_LIT ::= [0-9]+
REAL_LIT    ::= [0-9]+ '.' [0-9]+ ( [eE] [+-]? [0-9]+ )?
STRING_LIT  ::= '"' [^"]* '"'

(* Operators *)
PLUS        ::= '+'
MINUS       ::= '-'
STAR        ::= '*'
SLASH       ::= '/'
ASSIGN      ::= ':='
EQ          ::= '=='
NEQ         ::= '!='
LT          ::= '<'
LE          ::= '<='
GT          ::= '>'
GE          ::= '>='
ARROW       ::= '->'
COLON       ::= ':'
SEMICOLON   ::= ';'
COMMA       ::= ','
DOT         ::= '.'
LBRACE      ::= '{'
RBRACE      ::= '}'
LPAREN      ::= '('
RPAREN      ::= ')'
LBRACKET    ::= '['
RBRACKET    ::= ']'
LANGLE      ::= '<'
RANGLE      ::= '>'
PIPE        ::= '|'

(* Comments and Whitespace *)
LINE_COMMENT ::= '//' [^\n]*
BLOCK_COMMENT ::= '/*' .* '*/'
WS          ::= [ \t\r\n]+ -> skip
```

### 4.2. Production Rules

```ebnf
(* Top-Level *)
program          ::= algorithmDecl+

algorithmDecl    ::= 'algorithm' IDENTIFIER
                     complexityAnnotation
                     LBRACE statement* RBRACE

complexityAnnotation ::= '@Complexity' LPAREN STRING_LIT RPAREN

(* Statements *)
statement        ::= variableDecl
                   | assignment
                   | returnStmt
                   | ifStmt
                   | forLoop
                   | whileLoop
                   | assertStmt
                   | invariantStmt
                   | functionCall SEMICOLON
                   | block

block            ::= LBRACE statement* RBRACE

variableDecl     ::= 'let' IDENTIFIER ':' typeAnnotation
                     (ASSIGN expression)? SEMICOLON

assignment       ::= IDENTIFIER
                     (DOT IDENTIFIER | LBRACKET expression RBRACKET)*
                     ASSIGN expression SEMICOLON

returnStmt       ::= 'return' expression? SEMICOLON

ifStmt           ::= 'if' LPAREN expression RPAREN block
                     ('else' 'if' LPAREN expression RPAREN block)*
                     ('else' block)?

forLoop          ::= 'for' IDENTIFIER 'in' expression block

whileLoop        ::= 'while' LPAREN expression RPAREN block

assertStmt       ::= 'assert' LPAREN expression RPAREN
                     (':' STRING_LIT)? SEMICOLON

invariantStmt    ::= 'invariant' LPAREN expression RPAREN
                     (':' STRING_LIT)? SEMICOLON

(* Expressions *)
expression       ::= logicalOr

logicalOr        ::= logicalAnd ('or' logicalAnd)*

logicalAnd       ::= equality ('and' equality)*

equality         ::= comparison ((EQ | NEQ) comparison)*

comparison       ::= additive ((LT | LE | GT | GE) additive)*

additive         ::= multiplicative ((PLUS | MINUS) multiplicative)*

multiplicative   ::= unary ((STAR | SLASH | MOD) unary)*

unary            ::= (NOT | MINUS)? primary

primary          ::= INTEGER_LIT
                   | REAL_LIT
                   | STRING_LIT
                   | TRUE
                   | FALSE
                   | NONE
                   | IDENTIFIER
                   | functionCall
                   | LPAREN expression RPAREN
                   | compositeLiteral
                   | castExpression

compositeLiteral ::= setLiteral
                   | listLiteral
                   | mapLiteral
                   | graphLiteral
                   | matrixLiteral

setLiteral       ::= LBRACE expression (COMMA expression)* RBRACE

listLiteral      ::= LBRACKET expression (COMMA expression)* RBRACKET

mapLiteral       ::= LBRACE (expression COLON expression
                     (COMMA expression COLON expression)*)? RBRACE

graphLiteral     ::= 'graph' LANGLE typeAnnotation COMMA typeAnnotation RANGLE
                     LPAREN (* nodes *) LPAREN expression (COMMA expression)* RPAREN COMMA
                     (* edges *) LPAREN edgeLiteral (COMMA edgeLiteral)* RPAREN RPAREN
                     (* reserved for Epoch 1 completion *)

edgeLiteral      ::= LPAREN expression COMMA expression
                     (COMMA expression)? RPAREN

matrixLiteral    ::= 'matrix' LANGLE INTEGER_LIT COMMA INTEGER_LIT
                     COMMA typeAnnotation RANGLE
                     LPAREN expression (COMMA expression)* RPAREN

castExpression   ::= expression 'as' typeAnnotation

functionCall     ::= IDENTIFIER LPAREN
                     (expression (COMMA expression)*)? RPAREN

(* Types *)
typeAnnotation   ::= primitiveType
                   | compositeType

primitiveType    ::= 'Integer' | 'Real' | 'Boolean' | 'String' | 'Void'

compositeType    ::= 'Set'    LANGLE typeAnnotation RANGLE
                   | 'List'   LANGLE typeAnnotation RANGLE
                   | 'Map'    LANGLE typeAnnotation COMMA
                                      typeAnnotation RANGLE
                   | 'Graph'  LANGLE typeAnnotation COMMA
                                      typeAnnotation RANGLE
                   | 'Matrix' LANGLE INTEGER_LIT COMMA INTEGER_LIT
                                      COMMA typeAnnotation RANGLE
                   | 'Option' LANGLE typeAnnotation RANGLE
                   | 'Result' LANGLE typeAnnotation COMMA
                                      typeAnnotation RANGLE
                   | 'Tuple'  LANGLE typeAnnotation
                              (COMMA typeAnnotation)* RANGLE
```

### 4.3. Static Semantics (Parser-Level Validation)

The parser MUST reject programs for the following reasons:

1. **Undeclared variable** — an `IDENTIFIER` used before a corresponding `let` declaration in scope.
2. **Type mismatch in assignment** — the right-hand side expression type does not match the declared variable type.
3. **Type mismatch in binary operation** — operands to `+`, `-`, `*`, `/`, `mod`, `==`, `!=`, `<`, `<=`, `>`, `>=` are not both `Integer` or both `Real`.
4. **Type mismatch in unary operation** — operand to `-` is not `Integer` or `Real`; operand to `not` is not `Boolean`.
5. **Invalid cast** — the cast target type is incompatible with the source type per the type compatibility matrix (Appendix A).
6. **Missing complexity annotation** — an `algorithmDecl` lacks a `complexityAnnotation`.
7. **Invalid complexity string** — the complexity annotation string does not match the pattern defined in Appendix C.
8. **Graph literal without edge set** — reserved for future specification.

---

## 5. AST Canonical Representation

The AST is serialized to JSON with a strict schema. Every node has a `"kind"` field.

### 5.1. Node Kinds

| `"kind"` | Description |
|-----------|-------------|
| `"Program"` | Root node. Contains `algorithms: Algorithm[]`. |
| `"Algorithm"` | Top-level algorithm. Fields: `name: string`, `complexity: string`, `body: Statement[]`. |
| `"VariableDeclaration"` | A `let` binding. Fields: `name: string`, `type: Type`, `initializer?: Expression`. |
| `"Assignment"` | Mutation of an existing binding. Fields: `target: LValue`, `value: Expression`. |
| `"Return"` | Return statement. Fields: `value?: Expression`. |
| `"If"` | Conditional. Fields: `condition: Expression`, `consequent: Statement[]`, `alternate?: If | Statement[]`. |
| `"ForLoop"` | Iteration over a collection. Fields: `iterator: string`, `collection: Expression`, `body: Statement[]`. |
| `"WhileLoop"` | Conditional loop. Fields: `condition: Expression`, `body: Statement[]`. |
| `"Assert"` | Runtime assertion. Fields: `condition: Expression`, `message?: string`. |
| `"Invariant"` | Loop/scoped invariant. Fields: `condition: Expression`, `message?: string`. |
| `"IntegerLiteral"` | Integer constant. Fields: `value: string` (arbitrary precision). |
| `"RealLiteral"` | Floating-point constant. Fields: `value: number`. |
| `"StringLiteral"` | String constant. Fields: `value: string`. |
| `"BooleanLiteral"` | Boolean constant. Fields: `value: boolean`. |
| `"NoneLiteral"` | Null/absent value. |
| `"Identifier"` | Variable reference. Fields: `name: string`. |
| `"BinaryExpression"` | Binary operation. Fields: `operator: string`, `left: Expression`, `right: Expression`. |
| `"UnaryExpression"` | Unary operation. Fields: `operator: string`, `operand: Expression`. |
| `"FunctionCall"` | Built-in or user-defined call. Fields: `name: string`, `arguments: Expression[]`. |
| `"CastExpression"` | Explicit type cast. Fields: `expression: Expression`, `targetType: Type`. |
| `"SetLiteral"` | Set construction. Fields: `elementType: Type`, `elements: Expression[]`. |
| `"ListLiteral"` | List construction. Fields: `elementType: Type`, `elements: Expression[]`. |
| `"MapLiteral"` | Map construction. Fields: `keyType: Type`, `valueType: Type`, `entries: {key: Expression, value: Expression}[]`. |
| `"GraphLiteral"` | Graph construction. Reserved for Epoch 1 completion. |

### 5.2. Type Node Schema

Every `Type` node has `"kind": "Type"` and a `"name"` field:

```json
{
  "kind": "Type",
  "name": "Set",
  "typeArguments": [
    { "kind": "Type", "name": "Integer", "typeArguments": [] }
  ]
}
```

### 5.3. Example AST Output

For the algorithm:

```
algorithm EuclideanDistance
    @Complexity("O(1)")
{
    let dx: Real := x2 - x1;
    let dy: Real := y2 - y1;
    return sqrt(dx * dx + dy * dy);
}
```

The canonical JSON AST MUST output:

```json
{
  "kind": "Program",
  "algorithms": [
    {
      "kind": "Algorithm",
      "name": "EuclideanDistance",
      "complexity": "O(1)",
      "body": [
        {
          "kind": "VariableDeclaration",
          "name": "dx",
          "type": { "kind": "Type", "name": "Real", "typeArguments": [] },
          "initializer": {
            "kind": "BinaryExpression",
            "operator": "-",
            "left": { "kind": "Identifier", "name": "x2" },
            "right": { "kind": "Identifier", "name": "x1" }
          }
        },
        {
          "kind": "VariableDeclaration",
          "name": "dy",
          "type": { "kind": "Type", "name": "Real", "typeArguments": [] },
          "initializer": {
            "kind": "BinaryExpression",
            "operator": "-",
            "left": { "kind": "Identifier", "name": "y2" },
            "right": { "kind": "Identifier", "name": "y1" }
          }
        },
        {
          "kind": "Return",
          "value": {
            "kind": "FunctionCall",
            "name": "sqrt",
            "arguments": [
              {
                "kind": "BinaryExpression",
                "operator": "+",
                "left": {
                  "kind": "BinaryExpression",
                  "operator": "*",
                  "left": { "kind": "Identifier", "name": "dx" },
                  "right": { "kind": "Identifier", "name": "dx" }
                },
                "right": {
                  "kind": "BinaryExpression",
                  "operator": "*",
                  "left": { "kind": "Identifier", "name": "dy" },
                  "right": { "kind": "Identifier", "name": "dy" }
                }
              }
            ]
          }
        }
      ]
    }
  ]
}
```

---

## 6. Abstract Interpreter Semantics (Epoch 2)

### 6.1. Execution Model

The abstract interpreter is a stack-based virtual machine operating over the
canonical AST. It maintains:

- a **Virtual Heap** — an isolated memory arena with no access to system I/O,
  network, filesystem, or hardware;
- a **Symbol Table** — a stack of lexical scopes mapping identifiers to heap
  addresses;
- a **Step Counter** — a monotonic 64-bit unsigned integer incremented on every
  primitive operation;
- a **Trap Register** — set non-zero when execution is halted abnormally.

**Rationale for Abstract Step-Counting.** Conventional profiling measures
wall-clock time, which fluctuates with CPU scheduling, cache topology, and
memory architecture. This makes complexity validation environment-dependent
and non-reproducible. The UEAS kernel instead counts abstract operational steps
— each primitive operation has a fixed, spec-defined cost (see Section 6.2).
This produces an absolute, deterministic complexity curve independent of
hardware, operating system, or concurrent workload. An algorithm declared at
`@Complexity("O(N log N)")` can be objectively verified against its contract
regardless of where the kernel runs.

### 6.2. Primitive Operations and Step Cost

Every operation that advances the algorithm state increments the step counter
by a defined weight:

| Operation | Step Cost |
|-----------|-----------|
| Integer addition, subtraction, comparison | 1 |
| Integer multiplication, division, modulo | 1 |
| Real (float) arithmetic or comparison | 1 |
| Boolean logic (`and`, `or`, `not`) | 1 |
| Variable declaration (allocation) | 1 |
| Variable assignment | 1 |
| Array/list element access (`get`, `set`) | 1 |
| Set `contains` | 1 |
| Set `union`, `intersection`, `difference` (size `n`, `m`) | `n + m` |
| Map `get`, `put`, `containsKey` | 1 |
| Graph `adjacent`, `neighbors` | `degree(v)` |
| Graph `addNode`, `addEdge` | 1 |
| Matrix `get`, `set` | 1 |
| Matrix `transpose` (`r` × `c`) | `r * c` |
| Matrix `multiply` (`a` × `b` and `b` × `c`) | `a * b * c` |
| Function call (user-defined or built-in) | 1 + cost of body |
| Loop iteration header | 1 per iteration |

### 6.3. Invariant Enforcement

An `invariant` statement declares a boolean expression that MUST evaluate to
`true` at the point of declaration on every execution that reaches it.

- On first encounter, the kernel evaluates the invariant expression.
- If the expression evaluates to `false`, the kernel MUST set the trap register
  to `INVARIANT_VIOLATION` and halt execution immediately.
- The kernel MUST re-evaluate the invariant at every loop iteration if the
  invariant appears inside a loop body.

### 6.4. Complexity Contract Enforcement

The `@Complexity` annotation declares an asymptotic bound. The kernel
periodically samples the step counter `s` against the input size `n` (the
declared parameter cardinality) and checks:

- For `O(1)`: `s <= C_max` for some configurable constant `C_max` (default 10^6).
- For `O(N)`: `s / n <= C_max` at termination.
- For `O(N^2)`: `s / (n^2) <= C_max` at termination.
- For `O(N log N)`: `s / (n * ceil(log2(n))) <= C_max` at termination.
- For `O(log N)`: `s / ceil(log2(n)) <= C_max` at termination.
- For `O(2^N)`: `log2(s) / n <= C_max` at termination.
- For `O(N!)`: `log2(s) / (n * log2(n)) <= C_max` at termination.

If the bound is breached, the kernel MUST set the trap register to
`COMPLEXITY_VIOLATION` and halt immediately.

### 6.5. Error Semantics

| Trap Code | Name | Cause |
|-----------|------|-------|
| `0` | `NO_ERROR` | Normal termination. |
| `1` | `DIVISION_BY_ZERO` | Division or modulo by zero. |
| `2` | `INDEX_OUT_OF_BOUNDS` | List, Tuple, or Matrix access beyond declared bounds. |
| `3` | `NULL_DEREFERENCE` | Access on an `Option` of `None`. |
| `4` | `INVARIANT_VIOLATION` | An `invariant` expression evaluated to `false`. |
| `5` | `COMPLEXITY_VIOLATION` | Step count breached the declared complexity contract. |
| `6` | `STACK_OVERFLOW` | Recursion depth exceeded configurable limit (default 10^4). |
| `7` | `HEAP_EXHAUSTION` | Virtual heap allocation failed (configured size exceeded). |
| `8` | `ASSERTION_FAILURE` | An `assert` expression evaluated to `false`. |
| `9` | `INFINITE_LOOP_DETECTED` | Step counter exceeded configurable global maximum (default 10^12). |

### 6.6. Memory Model

- The virtual heap is a contiguous byte array of configurable size (default
  256 MiB).
- Each allocation returns a 64-bit handle. The kernel tracks allocation size
  and type metadata.
- There is no garbage collector. Memory is freed eagerly when a scope exits.
- Reference cycles are illegal. The parser MUST reject programs that could
  produce reference cycles in the heap graph.

---

## 7. Transpilation Target Interface (Epoch 3)

### 7.1. TargetGenerator Contract

Every transpilation target MUST implement the `TargetGenerator` interface:

```
interface TargetGenerator {
    /// Returns the target language identifier (e.g., "python", "rust", "cpp").
    language(): String

    /// Returns the target language version (e.g., "3.11", "2021 edition").
    version(): String

    /// Transpile a validated AST node into a string of target source code.
    /// The input AST MUST have passed kernel validation with trap code 0.
    generate(ast: Program): Result<String, TranspilationError>

    /// Returns the set of UEAS AST node kinds this target supports.
    /// Targets MAY decline to support all node kinds.
    supportedKinds(): Set<String>

    /// Returns a mapping of UEAS primitive types to target-language native types.
    typeMap(): Map<String, String>
}
```

### 7.2. Output Language Conformance Rules

Each target's output MUST satisfy:

| Target | Conformance Requirement |
|--------|------------------------|
| Python 3.11+ | Pass `mypy --strict` with zero errors. |
| Rust (2021 edition) | Pass `cargo clippy -- -D warnings` with zero warnings. |
| C++17+ | Pass `clang-tidy` with zero warnings under `-Wall -Wextra -Wpedantic`. |
| Java 17+ | Pass `javac -Xlint:all` with zero warnings. |

### 7.3. Semantic Equivalence Guarantee

Two transpiled programs `P_a` and `P_b` generated from the same UEAS AST for
targets `a` and `b` are **semantically equivalent** if and only if, for all
inputs, `P_a(input)` and `P_b(input)` produce mathematically identical outputs.

The reference implementation MUST include a cross-target equivalence test suite
that verifies this property on a corpus of benchmark algorithms.

### 7.4. MCP (Model Context Protocol) Interface

The transpilation back-end MUST expose a standard MCP API endpoint to allow
autonomous AI agents to ingest a canonical AST and produce target-language
code:

- **Endpoint:** `POST /mcp/v1/transpile`
- **Request Body:** `{ "ast": <Program>, "target": <language>, "options": {...} }`
- **Response Body:** `{ "source": <string>, "warnings": [...], "target_version": <string> }`
- **Authentication:** Bearer token (optional, implementation-defined).

MCP clients MAY register custom transpilation tools as MCP servers that accept
the canonical AST and return generated code.

---

## 8. RFC Process and Governance

### 8.1. RFC Lifecycle State Machine

```
      +----------+      +----------+      +-------------+      +---------------+
      |  Draft   | ---> |  Review  | ---> | Ratified    | ---> | Implemented   |
      +----------+      +----------+      +-------------+      +---------------+
           |                 |                   |                     |
           v                 v                   v                     v
      [Abandoned]      [Rejected]          [Superseded]           [Superseded]
```

### 8.2. State Definitions

| State | Description |
|-------|-------------|
| **Draft** | RFC author submits a proposal as a Markdown file in `docs/rfcs/`. The proposal MUST describe the motivation, proposed change, impact on the grammar/AST/kernel, and backward compatibility considerations. |
| **Review** | At least two approved reviewers (contributors with ratified RFCs to their name) must provide feedback. Review duration: minimum 14 calendar days. |
| **Ratified** | The RFC is accepted by consensus of active reviewers with no unresolved objections. The specification (SPEC.md) is updated **before** any reference implementation code is written. |
| **Implemented** | The reference implementation passes all existing and new conformance tests for the RFC's changes. |
| **Superseded** | A newer RFC explicitly obsoletes this one. The superseding RFC MUST reference the superseded RFC number. |
| **Rejected** | The RFC is declined with a public written rationale. |
| **Abandoned** | The RFC author withdraws the proposal or is unresponsive for 90 days. |

### 8.3. RFC Document Template

Every RFC document in `docs/rfcs/` MUST follow this structure:

```markdown
# RFC NNNN: Title

- **Status:** Draft | Review | Ratified | Implemented | Superseded | Rejected | Abandoned
- **Author:** Name <email>
- **Date:** YYYY-MM-DD
- **Supersedes:** RFC NNNN (if applicable)
- **Superseded By:** RFC NNNN (if applicable)

## Motivation

## Proposed Change

## Impact Analysis
  - Grammar changes:
  - AST schema changes:
  - Kernel changes:
  - Backward compatibility:

## Reference Implementation Plan

## Rejection Rationale (if rejected)
```

### 8.4. Governance Model

- **Benevolent Dictator For Life (BDFL):** The project founder holds final
  decision authority during the bootstrap phase (Epochs 1-3). After Epoch 3
  completion, authority transitions to a Technical Steering Committee (TSC).
- **TSC Composition:** 5 members, elected annually by contributors with at
  least one ratified RFC.
- **Voting:** TSC decisions require a simple majority (3 of 5). RFC
  ratification requires no unresolved objections (consensus model).
- **Code of Conduct:** The project adopts the Apache Software Foundation
  Code of Conduct.

### 8.5. Contributor License Agreement (CLA)

All contributors MUST sign an Apache-style Individual Contributor License
Agreement (ICLA) or Corporate CLA (CCLA) before any contribution is merged.
The CLA text is maintained in `docs/CLA.md`.

---

## 9. Conformance and Compliance

### 9.1. Definition of Conformance

An implementation is **UEAS-conformant** if and only if:

1. It accepts the canonical JSON AST schema defined in Section 5.
2. It produces semantically equivalent output as defined in Section 7.3.
3. It passes 100% of the UEAS Conformance Test Suite (UCTS).
4. It traps with the correct error code for every error condition in Section 6.5.

### 9.2. Conformance Test Suite (UCTS)

The UCTS consists of:

- **Unit Tests:** One test per AST node kind, verifying correct evaluation.
- **Property-Based Tests:** Fuzzing with `proptest` (Rust) feeding randomly
  generated valid ASTs into the kernel, asserting zero panics and correct
  trap codes.
- **Cross-Target Equivalence Tests:** For every benchmark algorithm,
  transpile to all supported targets, execute, and assert identical outputs.
- **Complexity Violation Tests:** Deliberately construct algorithms whose
  step count exceeds their declared complexity bound; assert
  `COMPLEXITY_VIOLATION` trap.

### 9.3. Benchmark Algorithms

Every conformant implementation MUST correctly handle:

1. **Euclidean Distance** (O(1))
2. **Linear Search** (O(N))
3. **Binary Search** (O(log N))
4. **Merge Sort** (O(N log N))
5. **Matrix Multiplication** (O(N^3))
6. **Dijkstra's Shortest Path** (O((V+E) log V))
7. **Traveling Salesman (Held-Karp)** (O(N^2 * 2^N))

### 9.4. CI/CD Compliance Gates

All changes to the reference implementation MUST pass:

| Gate | Threshold |
|------|-----------|
| Grammar parsing accuracy on benchmarks | 100% |
| Kernel trap code correctness on error corpus | 100% |
| Property-based fuzz tests | Zero panics, 10^6 random inputs |
| Cross-target semantic equivalence | 100% match |
| Containerized benchmark reproducibility | Identical results on Linux, macOS, Windows |

---

## Appendix A: Type Compatibility Matrix

| From \ To | Integer | Real | Boolean | String |
|-----------|---------|------|---------|--------|
| Integer   | —       | Allowed | Forbidden | Forbidden |
| Real      | Allowed (truncation) | — | Forbidden | Forbidden |
| Boolean   | Forbidden | Forbidden | — | Forbidden |
| String    | Forbidden | Forbidden | Forbidden | — |

Casts not listed in this matrix MUST be rejected at parse time.

---

## Appendix B: Reserved Words

The following identifiers are reserved and MUST NOT be used as variable or
algorithm names:

```
algorithm, function, procedure, return, if, else, for, while, in, let,
assert, invariant, complexity, graph, set, list, map, matrix, option,
result, some, none, ok, err, true, false, and, or, not, mod, as,
Integer, Real, Boolean, String, Void, Set, List, Map, Graph, Matrix,
Option, Result, Tuple, None
```

---

## Appendix C: Complexity Annotation Pattern

Valid complexity strings match the regular expression:

```
^O\((1|[A-Z](\^[0-9]+)?|log [A-Z]|[A-Z] log [A-Z]|2\^[A-Z]|[A-Z]!)\)$
```

Supported forms include:

| Form | Meaning |
|------|---------|
| `O(1)` | Constant time |
| `O(N)` | Linear time |
| `O(N^2)` | Quadratic time |
| `O(N^3)` | Cubic time |
| `O(N^k)` | Polynomial time of degree k |
| `O(log N)` | Logarithmic time |
| `O(N log N)` | Linearithmic time |
| `O(2^N)` | Exponential time |
| `O(N!)` | Factorial time |

Where `N` is a placeholder for the input size parameter. The exponent is
an unsigned integer. `log` without an explicit base is assumed base-2.

---

*End of Specification — Version 1.0.0-draft*
