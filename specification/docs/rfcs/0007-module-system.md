# RFC 0007: Module & Package System (`Import`)

- **Domain:** grammar | kernel
- **Date:** 2026-07-10
- **Status:** Ratified

## Summary
Introduces a dependency injection syntax (`Import: <namespace>`) to the UEAS grammar, allowing multi-file algorithm compositions. The abstract interpreter will implement an AST Linker to merge referenced modules before execution.

## Specification

### 1. Grammar Extension (`UEAS.g4`)
Add the following tokens to the Header block:
```antlr4
importDecl : 'Import:' IDENTIFIER ('.' IDENTIFIER)* ;
```

### 2. AST Linker (`kernel/src/linker.rs`)
The kernel will no longer parse and execute single files blindly. When it encounters an `Import` node, it will search the configured `library_path` for the corresponding `.ueas` file, parse it into an AST, and merge its functions/types into the main execution tree.

### 3. Namespace Resolution
We will avoid filesystem-specific imports (e.g., `../math/fft.ueas`). Instead, we will use dotted namespaces (`math.fft`) which map directly to directory structures, keeping the standard OS-agnostic.
