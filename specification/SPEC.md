# Universal Executable Algorithm Standard (UEAS)

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

- **AI Agent Orchestration:** Providing autonomous coding agents with a
  minimal, logical target grammar that reduces token-space errors and enables
  sandboxed testing before target-language code generation.
- **Academic Data Science:** UEAS natively embeds into Jupyter Notebooks via ZeroMQ,
  allowing researchers to profile algorithms with memory constraints and `@HardwareProfile`
  caching simulations without leaving the Python ecosystem.
- **Cybersecurity (Cryptography):** By natively supporting `@ConstantTime`
  block constraints, UEAS mathematically verifies immunity to timing side-channel attacks
  via abstract symbolic branching.
- **Aerospace & Defense:** Enabling DO-178C certification by embedding
  pre-conditions, post-conditions, and invariants directly in the algorithm
  definition, supporting automated validation against formal requirements.
- **Scientific Computing & Academic Publishing:** Making published algorithms
  downloadable, executable, and auditable — ending the era of untestable
  paper pseudocode.

### 1.4. Immutable Axioms

The following constraints are foundational to the standard. They are not
temporary limitations — they are the architectural guarantees that enable
mathematical verification of algorithmic correctness.

1. **Zero System I/O.** The abstract interpreter has no access to network
   sockets, filesystem operations, or hardware interfaces. Algorithms execute
   as pure state mutations within an isolated virtual heap. This guarantees
   deterministic, environment-independent execution profiles.

2. **Abstract Step-Counting.** Wall-clock time profiling is forbidden. Every
   primitive operation carries a fixed, spec-defined step cost (Section 6.2).
   Complexity contracts (`Complexity:`) are enforced by counting abstract
   logical mutations, producing identical complexity curves on any hardware.

3. **Specification Before Implementation.** The canonical JSON AST (Section 5)
   is the normative source of truth. No implementation code may be written
   for a specification change until the corresponding RFC is ratified and
   this document is updated. Behavior not described in this specification has
   no place in the reference implementation.

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
| `Set<T>` | Unordered collection of distinct elements of type `T`. Iteration order MUST be deterministic (insertion order). Operations: `union`, `intersection`, `difference`, `cardinality`, `contains`. |
| `List<T>` | Ordered, indexable sequence of elements of type `T`. Operations: `get`, `append`, `prepend`, `length`, `slice`. |
| `Map<K, V>` | Associative mapping from keys of type `K` to values of type `V`. Iteration order MUST be deterministic (insertion order of keys). Operations: `get`, `put`, `containsKey`, `keys`, `values`. |
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
5. All internal representations of `Set<T>` and `Map<K, V>` within the Virtual Heap MUST maintain deterministic insertion-order traversal to guarantee reproducible step-count profiles across hardware architectures and kernel implementations.
6. `Graph<N, E>` is undirected by default. The keywords `Directed` and `Undirected` MAY be used as type modifiers to declare graph directedness explicitly. Directed graphs treat edges as ordered pairs; undirected graphs treat edges as unordered.
7. Variables declared with `const` are immutable. Once bound, their value SHALL NOT be reassigned. Attempting to assign to a `const` variable MUST be rejected at compile time.

### 3.4. Randomization

The standard provides two built-in functions for non-deterministic algorithm specification:

- `randInt(min: Integer, max: Integer) -> Integer` — Returns a value in the range [min, max].
- `randReal() -> Real` — Returns a value in the range [0.0, 1.0).

Randomization is provided for algorithmic completeness (e.g., Randomized QuickSort, Monte Carlo simulations). The kernel's random number generator is implementation-defined and does NOT guarantee cryptographic security.

---

## 4. Grammar Specification (Epoch 3)

The UEAS grammar is specified in Extended Backus-Naur Form (EBNF). The
reference implementation uses ANTLR4 (`UEAS.g4`) as the parser generator. The
grammar is the normative definition of valid UEAS syntax.

UEAS v3.0 adopts academic pseudocode conventions inspired by CLRS textbooks
and the LaTeX `algorithmicx` package. The syntax uses `Require:`/`Ensure:` preambles,
`<-` assignment, and `then`/`do`/`end` block closures to produce human-readable
algorithms that are both executable and recognizable to any computer scientist.

### 4.1. Lexical Tokens

All structural keywords accept lowercase, TitleCase, and UPPERCASE variants for
academic flexibility (e.g., `if`, `If`, `IF` all produce the `IF` token).

```ebnf
(* Keywords — case-insensitive)
ALGORITHM   ::= 'algorithm' | 'Algorithm' | 'ALGORITHM'
FUNCTION    ::= 'function'  | 'Function'  | 'FUNCTION'
PROCEDURE   ::= 'procedure' | 'Procedure' | 'PROCEDURE'
RETURN      ::= 'return'    | 'Return'    | 'RETURN'
IF          ::= 'if'        | 'If'        | 'IF'
ELSE        ::= 'else'      | 'Else'      | 'ELSE'
FOR         ::= 'for'       | 'For'       | 'FOR'
WHILE       ::= 'while'     | 'While'     | 'WHILE'
BREAK       ::= 'break'     | 'Break'     | 'BREAK'
CONTINUE    ::= 'continue'  | 'Continue'  | 'CONTINUE'
IN          ::= 'in'        | 'In'        | 'IN'
EACH        ::= 'each'      | 'Each'      | 'EACH'
LET         ::= 'let'       | 'Let'       | 'LET'
CONST       ::= 'const'     | 'Const'     | 'CONST'
ASSERT      ::= 'assert'    | 'Assert'    | 'ASSERT'
INVARIANT   ::= 'invariant' | 'Invariant' | 'INVARIANT'
REQUIRE     ::= 'require'   | 'Require'   | 'REQUIRE'
ENSURE      ::= 'ensure'    | 'Ensure'    | 'ENSURE'
COMPLEXITY  ::= 'complexity'| 'Complexity'| 'COMPLEXITY'
THEN        ::= 'then'      | 'Then'      | 'THEN'
DO          ::= 'do'        | 'Do'        | 'DO'
END         ::= 'end'       | 'End'       | 'END'
MEMORY      ::= 'memory'    | 'Memory'    | 'MEMORY'
IMPORT      ::= 'import'    | 'Import'    | 'IMPORT'
TRUE        ::= 'true'      | 'True'      | 'TRUE'
FALSE       ::= 'false'     | 'False'     | 'FALSE'
AND         ::= 'and'       | 'And'       | 'AND'
OR          ::= 'or'        | 'Or'        | 'OR'
NOT         ::= 'not'       | 'Not'       | 'NOT'
MOD         ::= 'mod'       | 'Mod'       | 'MOD'
AS          ::= 'as'        | 'As'        | 'AS'
DIRECTED    ::= 'directed'  | 'Directed'  | 'DIRECTED'
UNDIRECTED  ::= 'undirected'| 'Undirected'| 'UNDIRECTED'
INFINITY    ::= 'infinity'  | 'Infinity'  | 'INFINITY'
NAN         ::= 'nan'       | 'NaN'       | 'NAN'

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
ASSIGN      ::= '<-' | ':='
BIND        ::= '='
EQ          ::= '=='
NEQ         ::= '!='
LT          ::= '<'
LE          ::= '<='
GT          ::= '>'
GE          ::= '>='
ARROW       ::= '->'
COLON       ::= ':'
COMMA       ::= ','
DOT         ::= '.'
LPAREN      ::= '('
RPAREN      ::= ')'
LBRACKET    ::= '['
RBRACKET    ::= ']'
AMP         ::= '&'
CARET       ::= '^'
LSHIFT      ::= '<<'
RSHIFT      ::= '>>'

(* Comments and Whitespace *)
LINE_COMMENT  ::= '#' [^\n]*
BLOCK_COMMENT ::= '/*' .* '*/'
NEWLINE      ::= '\r'? '\n' -> skip
WS           ::= [ \t]+ -> skip
```

### 4.2. Production Rules

```ebnf
(* Top-Level *)
program          ::= importDecl* algorithmDecl+

importDecl       ::= 'import' IDENTIFIER

algorithmDecl    ::= complexityDecorator?
                     'algorithm' IDENTIFIER
                     LPAREN (IDENTIFIER (COMMA IDENTIFIER)*)? RPAREN NEWLINE
                     requireBlock?
                     ensureBlock?
                     memoryDecorator?
                     block
                     (END ALGORITHM? NEWLINE?)?

requireBlock     ::= 'Require' COLON? paramTypeDecl (COMMA paramTypeDecl)* NEWLINE

paramTypeDecl    ::= IDENTIFIER ':' typeAnnotation

ensureBlock      ::= 'Ensure' COLON? typeAnnotation NEWLINE

complexityDecorator ::= 'Complexity' COLON? STRING_LIT
                         (COMMA variableBinding)* NEWLINE?

memoryDecorator  ::= 'Memory' COLON? STRING_LIT NEWLINE?

variableBinding  ::= IDENTIFIER '=' expression

(* Statements *)
statement        ::= assignmentOrCall NEWLINE
                   | returnStmt NEWLINE
                   | letDecl NEWLINE
                   | constDecl NEWLINE
                   | ifStmt
                   | forLoop
                   | whileLoop
                   | assertStmt NEWLINE
                   | invariantStmt NEWLINE
                   | BREAK NEWLINE
                   | CONTINUE NEWLINE
                   | PASS NEWLINE

letDecl          ::= 'let' IDENTIFIER (':' typeAnnotation)? ASSIGN expression

constDecl        ::= 'const' IDENTIFIER (':' typeAnnotation)? ASSIGN expression

block            ::= INDENT statement+ DEDENT

assignmentOrCall ::= target ASSIGN expression
                   | expression

target           ::= IDENTIFIER
                   | target LBRACKET expression RBRACKET
                   | target DOT IDENTIFIER

returnStmt       ::= 'return' expression?

assertStmt       ::= 'assert' LPAREN expression RPAREN
                     (COMMA STRING_LIT)?

invariantStmt    ::= 'invariant' LPAREN expression RPAREN
                     (COMMA STRING_LIT)?

(* Control Flow — academic textbook style: then/do/end closures *)
ifStmt           ::= 'if' expression 'then' NEWLINE block
                     ('else' 'if' expression 'then' NEWLINE block)*
                     ('else' NEWLINE block)?
                     'end' 'if' NEWLINE

forLoop          ::= 'for' 'each'? IDENTIFIER 'in' expression 'do'
                     NEWLINE block 'end' 'for' NEWLINE

whileLoop        ::= 'while' expression 'do' NEWLINE block
                     'end' 'while' NEWLINE

(* Expressions *)
expression       ::= logicalOr

logicalOr        ::= logicalAnd ('or' logicalAnd)*

logicalAnd       ::= equality ('and' equality)*

equality         ::= comparison ((EQ | NEQ) comparison)*

comparison       ::= additive ((LT | LE | GT | GE | IN) additive)*

additive         ::= multiplicative ((PLUS | MINUS) multiplicative)*

multiplicative   ::= unary ((STAR | SLASH | MOD) unary)*

bitwise          ::= multiplicative ((AMP | CARET | LSHIFT | RSHIFT) multiplicative)*

unary            ::= (NOT | MINUS)? primary

primary          ::= INTEGER_LIT
                   | REAL_LIT
                   | STRING_LIT
                   | TRUE
                   | FALSE
                   | INFINITY
                   | NAN
                   | LPAREN expression RPAREN
                   | dataStructure
                   | methodCallOrId

(* Composite Data Structures *)
dataStructure    ::= LBRACKET (expression (COMMA expression)*)? RBRACKET
                   | LBRACE (expression (COMMA expression)*)? RBRACE
                   | LBRACE (expression COLON expression
                       (COMMA expression COLON expression)*)? RBRACE

(* Method Chaining *)
methodCallOrId   ::= IDENTIFIER
                   | methodCallOrId DOT IDENTIFIER
                       LPAREN (expression (COMMA expression)*)? RPAREN
                   | methodCallOrId LBRACKET expression RBRACKET
                   | IDENTIFIER LPAREN (expression (COMMA expression)*)? RPAREN

matrixDim        ::= INTEGER_LIT | IDENTIFIER

(* Types *)
typeAnnotation   ::= primitiveType
                   | compositeType
                   | IDENTIFIER

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

1. **Variable used before first assignment** — the kernel auto-declares variables
   on first `<-` assignment; however, a variable read before any assignment in
   scope remains a runtime error (`NullDereference` trap code 4).
2. **Type mismatch in binary operation** — operands to `+`, `-`, `*`, `/`, `mod`, `==`, `!=`, `<`, `<=`, `>`, `>=` are not both `Integer` or both `Real`.
3. **Type mismatch in unary operation** — operand to `-` is not `Integer` or `Real`; operand to `not` is not `Boolean`.
4. **Invalid cast** — the cast target type is incompatible with the source type per the type compatibility matrix (Appendix A).
5. **Missing complexity annotation** — an `algorithmDecl` lacks a `Complexity:` preamble.
6. **Invalid complexity string** — the complexity annotation string does not match any supported form defined in Appendix C.
7. **Complexity binding mismatch** — a variable in the complexity string appears in a form that requires a binding (e.g., `V` in `O((V+E) log V)`) but no corresponding `variableBinding` is provided.
8. **Missing Require block** — an `algorithmDecl` with typed parameters lacks a `Require:` preamble.
9. **Missing parameter type** — a `paramTypeDecl` in `Require:` must include a type annotation for each parameter.

---

## 5. AST Canonical Representation

The AST is serialized to JSON with a strict schema. Every node has a `"kind"` field.

### 5.1. Node Kinds

| `"kind"` | Description |
|-----------|-------------|
| `"Program"` | Root node. Contains `algorithms: Algorithm[]`. |
| `"Algorithm"` | Top-level algorithm. Fields: `name: string`, `parameters: Parameter[]`, `returnType?: Type`, `complexity: string`, `bindings: VariableBinding[]`, `body: Statement[]`. |
| `"Parameter"` | Algorithm parameter. Fields: `name: string`, `type: Type`. |
| `"VariableBinding"` | Complexity variable binding. Fields: `variable: string`, `expression: Expression`. |
| `"VariableDeclaration"` | An explicit `let` binding (optional in v3.0; the kernel auto-declares variables on first `Assignment`). Fields: `name: string`, `type: Type`, `initializer?: Expression`. |
| `"ConstDeclaration"` | A `const` binding. Fields: `name: string`, `type: Type`, `initializer: Expression`. |
| `"Assignment"` | Mutation of an existing binding. Fields: `target: LValue`, `value: Expression`. |
| `"Return"` | Return statement. Fields: `value?: Expression`. |
| `"If"` | Conditional. Fields: `condition: Expression`, `consequent: Statement[]`, `alternate?: If | Statement[]`. |
| `"ForLoop"` | Iteration over a collection. Fields: `iterator: string`, `collection: Expression`, `body: Statement[]`. |
| `"WhileLoop"` | Conditional loop. Fields: `condition: Expression`, `body: Statement[]`. |
| `"Assert"` | Runtime assertion. Fields: `condition: Expression`, `message?: string`. |
| `"Invariant"` | Loop/scoped invariant. Fields: `condition: Expression`, `message?: string`. |
| `"IntegerLiteral"` | Integer constant. Fields: `value: number` (i64). |
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
| `"GraphLiteral"` | Graph construction. Fields: `nodeType: Type`, `edgeType: Type`, `nodes: Expression[]`, `edges: EdgeLiteral[]`. |
| `"EdgeLiteral"` | Directed edge within a graph. Fields: `from: Expression`, `to: Expression`, `weight?: Expression`. |
| `"InfinityLiteral"` | IEEE 754 positive infinity. Fields: `value: Infinity`. |
| `"NanLiteral"` | IEEE 754 NaN (not-a-number). Fields: `value: NaN`. |
| `"MatrixLiteral"` | Matrix construction. Fields: `rows: number`, `cols: number`, `elementType: Type`, `elements: Expression[]`. |
| `"Type"` | Type annotation node per Section 5.2. Fields: `name: string`, `typeArguments: Type[]`. |

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
Algorithm EuclideanDistance(x1, y1, x2, y2)
    Require: x1: Real, y1: Real, x2: Real, y2: Real
    Ensure: Real
    Complexity: "O(1)"

    dx <- x2 - x1
    dy <- y2 - y1
    return sqrt(dx * dx + dy * dy)
```

The canonical JSON AST MUST output:

```json
{
  "kind": "Program",
  "algorithms": [
    {
      "kind": "Algorithm",
      "name": "EuclideanDistance",
      "parameters": [
        { "kind": "Parameter", "name": "x1", "type": { "kind": "Type", "name": "Real", "typeArguments": [] } },
        { "kind": "Parameter", "name": "y1", "type": { "kind": "Type", "name": "Real", "typeArguments": [] } },
        { "kind": "Parameter", "name": "x2", "type": { "kind": "Type", "name": "Real", "typeArguments": [] } },
        { "kind": "Parameter", "name": "y2", "type": { "kind": "Type", "name": "Real", "typeArguments": [] } }
      ],
      "returnType": { "kind": "Type", "name": "Real", "typeArguments": [] },
      "complexity": "O(1)",
      "bindings": [],
      "body": [
        {
          "kind": "Assignment",
          "target": { "kind": "Identifier", "name": "dx" },
          "value": {
            "kind": "BinaryExpression",
            "operator": "-",
            "left": { "kind": "Identifier", "name": "x2" },
            "right": { "kind": "Identifier", "name": "x1" }
          }
        },
        {
          "kind": "Assignment",
          "target": { "kind": "Identifier", "name": "dy" },
          "value": {
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
  addresses. Variables are declared implicitly on first assignment — the
  kernel allocates a heap slot and inserts the identifier into the current
  scope automatically when `execute_assignment` encounters an unbound name;
- a **Step Counter** — a monotonic 64-bit unsigned integer incremented on every
  primitive operation;
- a **Trap Register** — set non-zero when execution is halted abnormally.

**Rationale for Abstract Step-Counting.** Conventional profiling measures
wall-clock time, which fluctuates with CPU scheduling, cache topology, and
memory architecture. This makes complexity validation environment-dependent
and non-reproducible. The UEAS kernel instead counts abstract operational steps
— each primitive operation has a fixed, spec-defined cost (see Section 6.2).
This produces an absolute, deterministic complexity curve independent of
hardware, operating system, or concurrent workload. An algorithm declared with `Complexity: "O(N log N)"` can be objectively verified against its contract
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
| Variable declaration (allocation, implicit on first assignment) | 1 |
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

The `Complexity:` annotation declares an asymptotic bound. The complexity
string `S` describes the Big-O form (e.g., `"O(N^2)"`, `"O((V+E) log V)"`).
Optional `variableBinding` entries (`V = expression`) map each abstract variable
in `S` to a concrete expression evaluated at algorithm entry.

**Evaluation Procedure:**

1. At algorithm entry, evaluate each `variableBinding` expression in declaration
   order to produce concrete integer values `n_1, n_2, ..., n_k`.
2. At algorithm termination, let `s` be the final step count.
3. Substitute the concrete values into the complexity formula and compute the
   expected asymptotic cost `E(n_1, ..., n_k)`.
4. Verify `s <= C_max * E` where `C_max` is a configurable constant (default 10^4).

**Supported Complexity Forms and Their Cost Functions:**

| Form | Cost `E(s)` | Example Binding |
|------|------------|-----------------|
| `O(1)` | 1 | (none needed) |
| `O(N)` | `n` | `N = length(items)` |
| `O(N^2)` | `n^2` | `N = rows` |
| `O(N^3)` | `n^3` | `N = dimension` |
| `O(N^k)` | `n^k` | `N = size` |
| `O(log N)` | `ceil(log2(n))` | `N = nodeCount` |
| `O(N log N)` | `n * ceil(log2(n))` | `N = elements` |
| `O((V+E) log V)` | `(v+e) * ceil(log2(v))` | `V = length(graph.nodes)`, `E = length(graph.edges)` |
| `O(V+E)` | `v + e` | `V = length(graph.nodes)`, `E = length(graph.edges)` |
| `O(2^N)` | `2^n` | `N = cities` |
| `O(N!)` | factorial of `n` | `N = cities` |

If the bound is breached — i.e., `s > C_max * E(n_1, ..., n_k)` — the kernel
MUST set the trap register to `COMPLEXITY_VIOLATION` and halt immediately.

If the complexity string contains variables that are not bound by a
`variableBinding`, the kernel MUST trap with `INVALID_COMPLEXITY_BINDING`
before execution begins.

### 6.5. Memory Complexity Enforcement

The optional `Memory:` annotation declares an asymptotic memory bound. The
kernel tracks the total bytes allocated on the Virtual Heap (`heap.bytes_allocated()`)
and verifies the allocation does not exceed the configured maximum (default 256 MiB).
If the maximum is exceeded, the kernel traps with `HEAP_EXHAUSTION` (code 7).

### 6.6. Exit Status Codes

The kernel produces a non-negative integer exit status at termination.
Code `0` indicates successful completion. Non-zero codes are termed
**trap codes** and indicate controlled error halts.

| Code | Name | Type | Cause |
|------|------|------|-------|
| `0` | `NO_ERROR` | Exit | Normal termination. |
| `1` | `DIVISION_BY_ZERO` | Trap | Division or modulo by zero. |
| `2` | `INDEX_OUT_OF_BOUNDS` | Trap | List, Tuple, or Matrix access beyond declared bounds. |
| `3` | `NULL_DEREFERENCE` | Trap | Read access on an undeclared variable or an `Option` of `None`. Note: Assignment to an undeclared variable triggers implicit declaration (see Section 6.1). |
| `4` | `INVARIANT_VIOLATION` | Trap | An `invariant` expression evaluated to `false`. |
| `5` | `COMPLEXITY_VIOLATION` | Trap | Step count breached the declared complexity contract. |
| `6` | `STACK_OVERFLOW` | Trap | Recursion depth exceeded configurable limit (default 10^4). |
| `7` | `HEAP_EXHAUSTION` | Trap | Virtual heap allocation failed (configured size exceeded). |
| `8` | `ASSERTION_FAILURE` | Trap | An `assert` expression evaluated to `false`. |
| `9` | `INFINITE_LOOP_DETECTED` | Trap | Step counter exceeded configurable global maximum (default 10^12). |
| `10` | `INVALID_COMPLEXITY_BINDING` | Trap | The complexity string references a variable not bound by a `variableBinding`. |
| `11` | `INVALID_OPERATION` | Trap | Unsupported operation, type mismatch, or unimplemented built-in function. |

### 6.7. Memory Model

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
    /// The input AST MUST have passed kernel validation with exit code 0.
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
| JavaScript ES2020+ | Valid per ECMAScript strict mode; no lint warnings with ESLint recommended config. |

### 7.3. Semantic Equivalence Guarantee

Two transpiled programs `P_a` and `P_b` generated from the same UEAS AST for
targets `a` and `b` are **semantically equivalent** if and only if, for all
inputs, `P_a(input)` and `P_b(input)` produce mathematically identical outputs.

The reference implementation MUST include a cross-target equivalence test suite
that verifies this property on a corpus of benchmark algorithms.

### 7.4. Memory Lifecycle Mapping

The UEAS memory model defines a strictly hierarchical ownership regime: memory
is freed when the declaring scope exits, and reference cycles are forbidden
(Section 6.7). Transpilers targeting systems languages MUST map UEAS
scope-based memory to single-ownership semantics:

| Target Language | Recommended Mapping |
|-----------------|-------------------|
| **Rust** | Pass by value or move semantics. For shared substructures (e.g., a `Graph` node referenced from multiple scopes), use explicit deep copies. |
| **C++** | Pass by value or `std::unique_ptr`. `std::shared_ptr` MUST NOT be generated as the default because it violates UEAS's deterministic scope-based deallocation. |
| **Python / Java** | Garbage-collected runtimes are exempt from explicit ownership mapping but SHOULD generate copy-on-write semantics for mutable composite types (List, Map, Matrix) to preserve the illusion of isolation between scopes. |

This policy ensures that transpiled code does not introduce aliasing bugs,
use-after-free errors, or memory leaks that are absent from the UEAS abstract
model.

### 7.5. MCP (Model Context Protocol) Interface

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
4. It traps with the correct error code for every error condition in
   Sections 6.5 and 6.6.

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

Every conformant implementation MUST correctly handle the following benchmark
algorithms (see `examples/` and `library/` for reference implementations):

1. **Euclidean Distance** (O(1))
2. **Linear Search** (O(N))
3. **Binary Search** (O(log N))
4. **Merge Sort** (O(N log N))
5. **Matrix Multiplication** (O(N^3))
6. **Dijkstra's Shortest Path** (O((V+E) log V))
7. **Traveling Salesman (Held-Karp)** (O(N^2 * 2^N))

The [Standard Algorithm Library](library/INDEX.md) extends this core set with
45 additional verified algorithms covering sorting, searching, graph theory,
dynamic programming, number theory, string processing, and data structures.
All library algorithms are specified in v3.0 academic pseudocode syntax with
`Complexity:` contracts.

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
algorithm, function, procedure, return, if, else, for, while,
break, continue, in, each, let, const, pass, assert, invariant,
require, ensure, complexity, then, do, end, memory, import,
and, or, not, mod, as,
Integer, Real, Boolean, String, Void, Set, List, Map, Graph,
Matrix, Option, Result, Tuple
```

The following words may be used as identifiers despite appearing in
syntactic contexts (they are handled by the `identifier` parser rule):

```
graph, matrix, some, none, true, false, Directed, Undirected
```

---

## Appendix C: Complexity Annotation Pattern

The complexity string in `Complexity:` MUST match one of the supported forms.
The grammar of complexity strings is:

```
Form ::= 'O(' Term ')'
Term ::= '1'
       | Variable ('^' Exponent)?
       | Term '+' Term | Term '*' Term
       | Term 'log' Variable | 'log' Variable
       | '2^' Variable
       | Variable '!'

Variable ::= [A-Z]+
Exponent ::= [0-9]+
```

Where `Variable` corresponds to variable names bound in `variableBinding`
entries. A subset of idiomatic forms is shown below:

| Form | Name | Requires Bindings |
|------|------|-------------------|
| `O(1)` | Constant | No |
| `O(N)` | Linear | `N` |
| `O(N^2)` | Quadratic | `N` |
| `O(N^3)` | Cubic | `N` |
| `O(log N)` | Logarithmic | `N` |
| `O(N log N)` | Linearithmic | `N` |
| `O(V+E)` | Sum (e.g., graph) | `V`, `E` |
| `O((V+E) log V)` | Mixed (e.g., Dijkstra) | `V`, `E` |
| `O(2^N)` | Exponential | `N` |
| `O(N!)` | Factorial | `N` |

`log` without explicit base is assumed base-2.

---

## Appendix D: Draft Specifications (Upcoming in v5.x)

The following grammar extensions have been formally ratified (Phase 4 & 5) and are currently pending implementation into the core `UEAS.g4` grammar and abstract interpreter:

### D.1 Module Linking (`Import`)
Algorithms will support AST composition across files:
```antlr4
importDecl : 'Import:' IDENTIFIER ('.' IDENTIFIER)* ;
```
*Example:* `Import: math.fourier`

### D.2 Hardware Profiling (`@HardwareProfile`)
The Virtual Heap will simulate cache line hits and evictions for memory-bound HPC profiling:
```antlr4
hardwareProfile : '@HardwareProfile(' cacheDef (',' cacheDef)* ')' ;
cacheDef : IDENTIFIER '=' NUMBER 'KB' | NUMBER 'MB' ;
```
*Example:* `@HardwareProfile(L1=64KB, L2=512KB)`

### D.3 Stochastic Primitives (`random`)
Allows probabilistic logic verification and Monte Carlo simulations:
```antlr4
expression : 'random(' expression ',' expression ')' ;
complexityRule : ('WorstCase' | 'BestCase' | 'Expected') ':' 'O(' expression ')' ;
```

### D.4 Infinite Data Streams (`Stream<T>`)
Enables algorithm processing on continuous, unbounded data constrained by `Space: O(1)` rather than Time boundaries:
```antlr4
typeAnnotation : 'Stream<' primitiveType '>' ;
statement : 'yield' expression | 'await' 'next' IDENTIFIER ;
```

### D.5 Cryptographic Verification (`@ConstantTime`)
Allows the abstract interpreter to verify immunity against timing side-channel leaks via symbolic branch testing:
```antlr4
typeAnnotation : 'Secret<' primitiveType '>' ;
blockConstraint : '@ConstantTime' block ;
```

---

*End of Specification*
