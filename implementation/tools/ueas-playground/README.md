# UEAS Playground

Browser-based interactive editor for UEAS (Universal Executable Algorithm
Standard). Write, format, and preview transpiled output for 7 target languages.

## Features

- **Monaco Editor** with UEAS syntax highlighting
- **7 target previews** — Python, Rust, C++17, Java 17, JavaScript, Lean 4, TLA+
- **6 example algorithms** pre-loaded from the standard library
- **One-click copy** and code formatting
- **Dark theme** matching GitHub's code aesthetic
- **Zero install** — works in any modern browser

## Deployment

The playground is fully configured for Vercel deployment and uses a WebAssembly (WASM) build of the Rust kernel to parse and transpile UEAS algorithms natively in the browser.

```bash
# Build the WASM module locally
npm run build

# Serve locally
npx serve
```

## Roadmap

- [x] WASM-compiled kernel for real-time execution
- [ ] Step-count profiler visualization
- [ ] Complexity contract enforcement in-browser
- [ ] Cross-target equivalence verification
- [ ] Shareable URL encoding (base64 algorithm snippets)
- [ ] Full standard library browser (45 algorithms)

## License

Apache License 2.0 — see [LICENSE](../../LICENSE)
