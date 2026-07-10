# RFC 0001: User-Defined Record Types

- **Status:** Draft
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

Epoch 5 (`PLAN.md`) mandates the implementation of a Standard Algorithm Library that includes classical data structures such as Binary Search Trees, AVL Trees, Heaps, and Tries. 
Currently, `SPEC.md` does not support User-Defined Types (Structs, Records, or Classes). The grammar only supports `algorithmDecl`, and the type system strictly defines a set of primitive and collection types (`List`, `Map`, `Graph`, etc.).

Without the ability to define a custom type (e.g., a `Node` with `left` and `right` fields), implementing these data structures natively in UEAS is impossible. Workarounds involving parallel arrays or maps violate the design principles of producing human-readable, academic pseudocode. This RFC proposes adding `Record` types to the UEAS specification to close this gap.

## Proposed Change

We propose adding a `record` declaration to the top-level UEAS grammar, allowing users to define named composite types consisting of typed fields. 

Example syntax:
```ueas
record Node
    value: Integer
    left: Option<Node>
    right: Option<Node>
end record
```

These records can then be instantiated and their fields accessed via dot-notation:
```ueas
let root <- Node(value = 5, left = None, right = None)
root.value <- 10
```

## Impact Analysis

### Grammar changes
- Add `RECORD` to lexical keywords.
- Update top-level program rule:
  ```ebnf
  program ::= importDecl* (algorithmDecl | recordDecl)+
  ```
- Add `recordDecl` rule:
  ```ebnf
  recordDecl ::= 'record' IDENTIFIER NEWLINE
                 INDENT (IDENTIFIER ':' typeAnnotation NEWLINE)+ DEDENT
                 'end' 'record' NEWLINE
  ```
- Update expressions to support record instantiation (using named arguments).
- Ensure field access via dot-notation (`target DOT IDENTIFIER`) is fully supported in `assignmentOrCall` and `primary`.

### AST schema changes
- Add `"RecordDeclaration"` node kind to the top level. Fields: `name: string`, `fields: {name: string, type: Type}[]`.
- Add `"RecordLiteral"` node kind. Fields: `recordName: string`, `fieldValues: {name: string, value: Expression}[]`.
- Update `"Type"` schema to resolve custom record names.

### Kernel changes
- **Virtual Heap:** Needs support for allocating custom record sizes based on the number of fields.
- **Symbol Table:** Needs to track record definitions in the global scope to enforce type-checking upon instantiation.
- **New Trap Codes:** `MISSING_FIELD_INITIALIZATION` if a record is instantiated without all fields, or `INVALID_FIELD_ACCESS` for unrecognized fields.

### Backward compatibility
- Existing UEAS programs remain 100% valid. This is a strictly additive change.

## Alternatives Considered

1. **Tuple Indexing:** Using `Tuple<Integer, Option<Tuple>, Option<Tuple>>` instead of Records. *Rejected* because positional indexing (`node[1]`) is unreadable and error-prone compared to field names (`node.left`).
2. **Parallel Collections:** Using `Map<Integer, Integer>` to track left and right children. *Rejected* as it introduces manual memory management and index tracking into what should be abstract algorithmic logic.

## Reference Implementation Plan

- **Affected Domains:** `grammar/` (parser updates), `kernel/` (heap and type inference updates), and `backends/` (transpiling records to Python `dataclass`, Rust `struct`, C++ `struct`, etc.).
- **Epoch Alignment:** Must be implemented in a new sub-epoch (Epoch 4.5) to unblock Epoch 5.
- **Dependencies:** No new external dependencies required.
