# UEAS MCP Server — Cloudflare Workers

Always-on, globally distributed MCP server for AI agent integration.
Zero cold starts, 100K req/day free tier.

## Deploy

```bash
cd implementation/tools/ueas-mcp-cf
npm install -g wrangler
wrangler login
wrangler deploy
```

## Local Dev

```bash
wrangler dev
# → http://localhost:8787

# Test tools/list
curl -X POST http://localhost:8787 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

## Architecture

Edge-deployed JavaScript Worker wrapping WASM-compiled Rust kernel.
All computation runs at the Cloudflare edge — no containers, no cold starts.
