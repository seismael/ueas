# RFC 0001: User-Defined Record Types

- **Status:** Ratified
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

Epoch 5 (`PLAN.md`) mandates the implementation of a Standard Algorithm Library that includes classical data structures such as Binary Search Trees, AVL Trees, Heaps, and Tries. 
Currently, `SPEC.md` does not support User-Defined Types (Structs, Records, or Classes). The grammar only supports `algorithmDecl`, and the type system strictly defines a set of primitive and collection types (`List`, `Map`, `Graph`, etc.).

Without the ability to define a custom type (e.g., a `TreeNode` with `left` and `right` fields), implementing these data structures natively in UEAS is impossible. However, UEAS explicitly enforces a **strict single-ownership, value-semantic memory model** (Section 7.4) and forbids reference cycles (Section 6.7). Therefore, we cannot implement textbook data structures using direct recursive object pointers (which would cause infinite copying or illegal cycles in a value-semantic language).

To support Epoch 5 while rigorously preserving the UEAS memory axioms, we must introduce **Value-Semantic Record Types**. This allows algorithms to define standard structs and use the **Arena Pattern** (storing nodes in a flat `Map` or `List` and using `Integer` indices as pointers) to safely model cyclic graphs or trees with parent pointers.

## Proposed Change

We propose adding a `record` declaration to the top-level UEAS grammar, allowing users to define named, non-recursive composite types consisting of typed fields. 

Example syntax (using the Arena Pattern):
```ueas
record TreeNode
    value: Integer
    left: Integer    # Arena index
    right: Integer   # Arena index
    parent: Integer  # Arena index
end record
```

These records can then be instantiated and their fields accessed via dot-notation. Because they are strict value types, they can be safely stored in the virtual heap without violating single-ownership.
```ueas
let root <- TreeNode(value = 5, left = -1, right = -1, parent = -1)
arena.put(0, root)
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
- **Virtual Heap:** Needs support for allocating custom record sizes based on the number of fields. Records are strict value types (pass-by-value).
- **Symbol Table:** Needs to track record definitions in the global scope to enforce type-checking upon instantiation.
- **Reference Cycle Prevention:** The parser must strictly forbid recursive record definitions (e.g., `record Node { next: Node }`). All self-referential structures MUST use the Arena pattern (e.g., `next: Integer`).
- **New Trap Codes:** `MISSING_FIELD_INITIALIZATION` if a record is instantiated without all fields, or `INVALID_FIELD_ACCESS` for unrecognized fields.

### Backward compatibility
- Existing UEAS programs remain 100% valid. This is a strictly additive change.

## Alternatives Considered

1. **Recursive Object Pointers (`Option<Node>`):** *Rejected.* While this is common in GC'd languages (Java/Python), it catastrophically violates UEAS's single-ownership axiom (Section 7.4). A tree with a parent pointer would create a reference cycle, breaking deterministic memory deallocation and causing infinite loops in pass-by-value assignments. The Arena pattern with flat Records is the only mathematically sound approach.
2. **Tuple Indexing:** Using `Tuple<Integer, Integer, Integer, Integer>` instead of Records in the Arena. *Rejected* because positional indexing (`node[1]`) is unreadable and error-prone compared to academic field names (`node.left`).

## Reference Implementation Plan

- **Affected Domains:** `grammar/` (parser updates), `kernel/` (heap and type inference updates), and `backends/` (transpiling records to Python `dataclass`, Rust `struct`, C++ `struct`, etc.).
- **Epoch Alignment:** Must be implemented in a new sub-epoch (Epoch 4.5) to unblock Epoch 5.
- **Dependencies:** No new external dependencies required.
