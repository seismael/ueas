# RFC 0006: Tensor Grammar and Broadcasting Semantics

- **Status:** Draft
- **Author:** UEAS Automated Agent
- **Date:** 2026-07-10
- **Supersedes:** None
- **Superseded By:** None

## Motivation

ADR 0011 establishes the decision to support N-Dimensional Tensors for Machine Learning algorithms. We must amend the EBNF grammar to parse Tensor type signatures and provide built-in operations for them.

## Proposed Change

### 1. Primitive Type Addition
Introduce `Tensor` to the `compositeType` rule in `UEAS.g4`.

```ueas
Algorithm Softmax(logits)
    Require: logits: Tensor<Real, N, C>
    Ensure: Tensor<Real, N, C>
    Complexity: "Work: O(N * C)"

    let max_vals <- reduce(logits, axis=1, op=MAX)
    let exps <- exp(logits - max_vals)
    return exps / reduce(exps, axis=1, op=SUM)
```

### 2. Grammar Updates

**Parser:**
```ebnf
compositeType ::= ...
                | 'Tensor' LANGLE typeAnnotation COMMA (IDENTIFIER | INTEGER_LIT) 
                           (COMMA (IDENTIFIER | INTEGER_LIT))* RANGLE
```

### 3. Built-in Function Updates
The standard library of built-in functions will be expanded to include:
- `reduce(tensor, axis, op)`
- `matmul(tensorA, tensorB)`
- `convolve(tensor, filter, stride, padding)`

### 4. AST Schema Updates
- **Type**: Extend the `Type` node to handle dynamic/variadic dimensionality parameters for Tensors.

## Impact Analysis
- **Kernel Type Checker:** The semantic analyzer must verify that binary operations between two `Tensor` instances follow strict numpy-style broadcasting rules. If shapes are incompatible, it must throw a compilation error.
- **Transpilers:** The Python target must prepend `import numpy as np` if `Tensor` types are detected in the AST.
