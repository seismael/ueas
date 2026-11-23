# UEAS v2.0 Deployment Architecture

Three decoupled deployment targets aligned with UEAS v2.0:

| Tier | Service | Source | Purpose |
| :--- | :--- | :--- | :--- |
| **Presentation** | **Vercel (Vite SPA)** | `tools/ueas-playground` | Browser IDE running `ueas-wasm` in a dedicated Web Worker |
| **Edge API** | **Cloudflare Workers** | `tools/ueas-mcp-edge` | Stateless MCP 2025-03-26 JSON-RPC server backed by `ueas-wasm` |
| **Formal Prover** | **Google Cloud Run** | `tools/ueas-cloud-verify` | Hardened Axum service running Dafny 4 / Lean 4 with 1 MB limit & 15s timeout |

## Setup Guides

1. [Vercel Playground](VERCEL.md)
2. [Cloudflare Edge MCP Worker](CLOUDFLARE.md)
3. [Google Cloud Run Formal Verifier](GOOGLE_CLOUD.md)
