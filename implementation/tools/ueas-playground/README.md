# UEAS Playground — MCP Client

Browser-based bidirectional editor for UEAS (Vercel-deployed, static HTML/JS/CSS).

**Architecture:**
```
Playground (Vercel) → fetch() → MCP Server (Cloudflare Workers / WASM kernel)
```

The WASM kernel runs ONCE at the edge, not in every user's browser.
All algorithm execution, transpilation, and reverse-auditing is powered
exclusively by the Cloudflare Workers MCP server.

## Features

- **Bidirectional Editor** — UEAS pseudocode ↔ target language (Python, Java, Rust, C++, JavaScript)
- **Transpile** — translate UEAS to 8 production targets via MCP
- **Reverse Audit** — analyze legacy code, map to UEAS, detect I/O violations
- **Execute** — run algorithms with step-count profiling and complexity verification
- **Zero local WASM** — all computation is remote, keeping the client lightweight

## Deployment

Static HTML/JS/CSS deployed to Vercel. No build step, no npm, no WASM bundle.

```bash
# Serve locally
python -m http.server 8080

# Deploy to Vercel
vercel deploy
```

## Roadmap

- [x] MCP-only architecture (removed local WASM)
- [x] Bidirectional editor (UEAS ↔ target)
- [x] Reverse audit (Python → UEAS)
- [ ] Multi-file project support
- [ ] Shareable algorithm links
