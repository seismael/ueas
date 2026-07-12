# UEAS MCP Server — Cloudflare Workers

Always-on MCP server with 8 tools deployed at the edge.

## Tools

| Tool | Description |
|------|-------------|
| `parse` | Validate UEAS syntax, return parsed AST |
| `execute` | Execute with step-count profiling |
| `transpile` | Transpile to Dafny, Lean 4, TLA+, LaTeX |
| `verify` | @ConstantTime / Secret<T> compliance |
| `hardware` | Cache locality analysis |
| `complexity` | Work-Span DAG analysis |
| `memory` | Heap footprint estimation |
| `audit` | Bidirectional reverse audit (Python→UEAS) |

## Endpoint

https://ueas-mcp.seismael.workers.dev

## Architecture

WASM-compiled Rust kernel at the edge. No Docker, no cold starts.
Auto-deploys via Cloudflare native Git integration on push to master.

## Deploy

1. Cloudflare Dashboard → Workers & Pages → ueas-mcp
2. Settings → Build → Root directory: `tools/ueas-mcp`
3. Push to master → auto-deploy in ~15s
