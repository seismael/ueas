# Cloudflare Workers Deployment (`ueas-mcp-edge`)

The Edge MCP server runs on Cloudflare Workers and executes `ueas-wasm` directly at the edge with strict payload and fuel limits.

## Build & Typecheck

```bash
wasm-pack build crates/ueas-wasm --target web --out-dir ../../tools/ueas-mcp-edge/src/pkg
cd tools/ueas-mcp-edge
npm ci && npm run typecheck
npm run deploy
```

## Verify

```bash
curl https://<your-worker>.workers.dev/healthz
# -> {"ok":true,"engine":"ueas-wasm-2.0"}

curl -X POST https://<your-worker>.workers.dev/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```
