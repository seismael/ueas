# Launch Posts

## Hacker News (Show HN)

**Title:** Show HN: UEAS - A mathematically rigorous, executable algorithm standard

Hi HN,

I've been working on UEAS (Universal Executable Algorithm Standard) for a while now. The core problem it solves is that classic algorithmic pseudocode (like in CLRS) isn't executable, and real programming languages (like C++ or Rust) carry too much machine-specific boilerplate (pointers, memory management, types) that obfuscate pure algorithmic logic.

UEAS bridges this gap. It's a completely language-agnostic algorithmic standard with a strict ANTLR4 grammar that looks almost exactly like LaTeX `algorithmicx`. But beneath it is a highly optimized Rust interpreter and virtual heap that executes the abstract syntax tree directly.

**Key Features:**
- **Deterministic Big-O Verification:** The engine counts exact abstract operations. You can declare `Complexity: "O(log N)"` in the algorithm preamble, and if your code degrades to O(N), the engine intercepts execution and traps with a `ComplexityViolation`.
- **Omni-Transpilation:** A verified UEAS algorithm transpiles seamlessly into C++, Python, Rust, JavaScript, and even formal verification targets like Lean 4 and TLA+.
- **Zero-Boilerplate:** Memory allocation is implicit. `visited <- {}` automatically allocates a hash set on the virtual heap.
- **Advanced Domains:** Natively supports distributed message passing, concurrent work-span profiling, quantum gates, and cryptographic constant-time assertions.

We've deployed a WASM-based browser playground so you can write and execute algorithms locally in your browser. There's also an MCP server for integrating with Claude/AI agents, and a Jupyter Kernel.

Repo: https://github.com/seismael/ueas
Playground: [Playground URL]

I'd love your thoughts on the language design and transpiler architecture!
