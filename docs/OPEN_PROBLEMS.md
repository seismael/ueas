# UEAS Open Problems

The Universal Executable Algorithm Standard (UEAS) aims to mathematically formalize algorithmic logic independent of hardware or programming language constraints. 

To achieve a production-ready **v1.0.0** standard, we need highly specialized domain experts to solve several deep architectural, mathematical, and grammatical challenges. This board serves as a "honeypot" for researchers and engineers looking for high-impact open-source problems.

---

## Challenge 1: Generic Algorithms and the AST Type Checker
**Domain:** Compiler Theory, Programming Language Theory (PLT)  
**Status:** Unsolved  

### The Problem
Currently, the `UEAS.g4` grammar supports generic *data types* (e.g., `Set<City>`), but lacks the formal syntax to declare a *generic algorithm*. 

For example, we cannot currently specify:
```
algorithm sort<T>(items: List<T>)
```

### Why It's Hard
If we introduce `<T>` at the algorithm definition level, the static type checker (which runs on the AST prior to execution) must support monomorphization or type-erasure. Because UEAS is a formal standard, the type resolution rules must be mathematically unambiguous.

### How You Can Help
We need an expert in PLT or ANTLR4 to propose an EBNF modification via RFC that introduces generic algorithm declarations while keeping the static type checker deterministic.

---

## Challenge 2: Deterministic Space Complexity Profiling (`@Memory`)
**Domain:** Algorithm Analysis, VM Design  
**Status:** Unsolved  

### The Problem
UEAS natively supports deterministic `@Complexity("O(N^2)")` annotations for time complexity, which mathematically traps if the abstract step counter exceeds the bound. However, we have not formalized **Space Complexity**.

### Why It's Hard
Unlike time complexity (which counts logical mutations), space complexity must measure the footprint of objects on the `VirtualHeap`. How do we abstractly measure the memory footprint of a `Graph` node across Python, Rust, and C++ transpilers so that the Big-O limit is enforced identically across all of them?

### How You Can Help
We need an algorithms expert to propose a deterministic mathematical model for measuring abstract heap usage, allowing us to introduce `@Memory("O(N)")` contracts into the kernel.

---

## Challenge 3: Syntax v2.0 - Indentation vs. Braces
**Domain:** Language Design, ANTLR4  
**Status:** Unsolved  

### The Problem
UEAS currently uses C-style braces `{}` for scoping. To make UEAS the ultimate, readable pseudocode standard (similar to Python or textbook pseudocode), we want to transition to significant whitespace (indentation-based scoping).

### Why It's Hard
Implementing `INDENT` and `DEDENT` tokens in ANTLR4 can be fragile and context-dependent. Is the loss of copy/paste resilience worth the massive readability gain?

### How You Can Help
We need ANTLR4 parser experts to architect the transition to significant whitespace in `UEAS.g4` and evaluate the parsing complexity.

---

## Ready to tackle one?
If you have a solution or want to discuss one of these problems, please open a **Request for Comments (RFC)** issue on our GitHub using the `rfc_proposal.yml` template, or join the discussion on our Discord/Mailing List.
