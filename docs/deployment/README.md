# UEAS Deployment Architecture

Three cloud services, all free tier, zero cost at idle.

```
                    ┌─────────────────────────┐
                    │   Vercel (Edge)          │
                    │   Playground UI          │
                    │   ueas-three.vercel.app  │
                    │   Static HTML/JS/CSS     │
                    └──────────┬──────────────┘
                               │ fetch()
                               ▼
                    ┌─────────────────────────┐
                    │ Cloudflare Workers       │
                    │ MCP Server (8 tools)     │
                    │ ueas-mcp.seismael.workers.dev │
                    │ WASM kernel at edge      │
                    └──────────┬──────────────┘
                               │ POST /verify (coming)
                               ▼
                    ┌─────────────────────────┐
                    │ Google Cloud Run         │
                    │ Dafny + Z3 Verifier      │
                    │ ueas-verify-...run.app   │
                    │ Scale-to-zero, $0 idle   │
                    └─────────────────────────┘
```

| Tier | Service | URL | Purpose | Cost |
|------|---------|-----|---------|------|
| Presentation | **Vercel** | `ueas-three.vercel.app` | Playground UI | Free |
| API | **Cloudflare Workers** | `ueas-mcp.seismael.workers.dev` | 8 MCP tools | Free |
| Verification | **Google Cloud Run** | TBD (after deploy) | Dafny/Z3 proofs + code gen | Free tier |

## Setup Guides

1. [Vercel Playground](VERCEL.md) — static HTML deploy
2. [Cloudflare MCP Worker](CLOUDFLARE.md) — WASM kernel at edge
3. [Google Cloud Run Dafny Backend](GOOGLE_CLOUD.md) — Z3 verification
