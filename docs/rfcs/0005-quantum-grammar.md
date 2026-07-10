# RFC 0005: Quantum Algorithm Grammar Specification

- **Status:** Ratified
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

ADR 0010 establishes the strategic decision to natively support Quantum Algorithm specifications in UEAS, targeting transpilation to Qiskit and OpenQASM. To accomplish this, we must amend the `UEAS.g4` grammar and canonical AST to include quantum types and primitive gates.

## Proposed Change

### 1. Quantum Primitive Types
- `Qubit`: Represents a single quantum state.
- `QRegister`: Represents a fixed-size collection of entangled or unentangled Qubits.

### 2. Quantum Gate Operations
Quantum gates are modeled as built-in unary or binary operations, rather than standard function calls, to allow the kernel's static circuit validator to enforce the No-Cloning theorem.

```ueas
Algorithm BellState()
    Require: None
    Ensure: Tuple<Integer, Integer>
    Complexity: "QuantumDepth: O(1)"

    let q1: Qubit <- allocateQubit()
    let q2: Qubit <- allocateQubit()

    apply Hadamard to q1
    apply CNOT to q1, q2

    return (measure q1, measure q2)
```

### 3. Grammar Updates

**Lexer:**
```ebnf
APPLY    ::= 'apply' | 'Apply' | 'APPLY'
MEASURE  ::= 'measure' | 'Measure' | 'MEASURE'
HADAMARD ::= 'Hadamard' | 'HADAMARD'
CNOT     ::= 'CNOT'
QUBIT    ::= 'Qubit'
QREGISTER::= 'QRegister'
```

**Parser:**
```ebnf
statement ::= ...
            | applyGateStmt NEWLINE

applyGateStmt ::= 'apply' IDENTIFIER 'to' expression (COMMA expression)*

expression ::= ...
             | 'measure' expression
```

### 4. AST Schema Updates
- **ApplyGateStatement**: `{ "kind": "ApplyGateStatement", "gate": string, "targets": Expression[] }`
- **MeasureExpression**: `{ "kind": "MeasureExpression", "target": Expression }`

## Impact Analysis
- **Backward Compatibility:** Fully backward compatible.
- **Kernel:** The kernel must track Qubit bindings tightly to ensure that variables of type `Qubit` are never duplicated/aliased, as that would violate the No-Cloning theorem of quantum mechanics.
