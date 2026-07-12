# Cloudflare Workers Deployment

The MCP server runs on Cloudflare Workers (always-on, globally distributed).

## Prerequisites
- A Cloudflare account (free at https://dash.cloudflare.com/sign-up)
- GitHub repo already connected via Workers & Pages

## Step 1: Configure Worker

1. Go to **https://dash.cloudflare.com** → **Workers & Pages** → select `ueas-mcp`
2. Click **Settings** → **Build**

3. Fill in:

| Field | Value |
|-------|-------|
| **Root directory** | `tools/ueas-mcp` |
| **Build command** | `echo 'Static ES module — no build'` |
| **Framework preset** | None (Custom) |

4. Click **Save**

## Step 2: Deploy

Cloudflare auto-deploys on every push to `master`. To trigger immediately:

1. **Deployments** → **Deploy latest commit**

## Step 3: Verify

```bash
curl https://ueas-mcp.seismael.workers.dev
# → "UEAS MCP OK"

curl -X POST https://ueas-mcp.seismael.workers.dev \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
# → 8 tools listed
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Build fails | Delete node_modules, retry deploy |
| 404 on all paths | Root directory might be wrong — verify `tools/ueas-mcp` |
| WASM not loaded | Push to master to trigger CF native Git deploy |
