# RFC 0008: Cache-Aware Hardware Profiling

- **Domain:** grammar | kernel
- **Date:** 2026-07-10
- **Status:** Ratified

## Summary
Introduces the `@HardwareProfile` metadata block to allow algorithm designers to specify physical CPU cache characteristics. The Virtual Heap will track spatial and temporal data locality to simulate cache miss rates alongside logical step counts.

## Specification

### 1. Grammar Extension (`UEAS.g4`)
Add an optional block above the `Algorithm:` header:
```antlr4
hardwareProfile : '@HardwareProfile(' cacheDef (',' cacheDef)* ')' ;
cacheDef : IDENTIFIER '=' NUMBER 'KB' | NUMBER 'MB' ;
```
Example: `@HardwareProfile(L1=64KB, L2=512KB, L3=8MB, CacheLine=64B)`

### 2. Virtual Heap Modification
Currently, `kernel/src/heap/` operates as a flat $O(1)$ memory mapping. With a hardware profile active, the heap will subdivide virtual addresses into mock cache lines. Reading from sequential addresses will hit the simulated L1; strided reads exceeding the cache size will trigger simulated evictions and cache miss penalties.

### 3. Defaults
If omitted, the kernel will execute with flat $O(1)$ memory access to maintain backwards compatibility with theoretical algorithms.
