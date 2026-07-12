# Vercel Playground Deployment

The browser-based bidirectional editor runs on Vercel (static HTML/JS/CSS).

## Prerequisites
- A Vercel account (free at https://vercel.com/signup)
- GitHub repo connected

## Step 1: Configure

1. Go to **https://vercel.com** → Import Project → select `ueas` repo
2. Fill in:

| Field | Value |
|-------|-------|
| **Root Directory** | `tools/ueas-playground` |
| **Framework Preset** | Other / Static |
| **Build Command** | *(leave empty — no build step)* |
| **Output Directory** | `.` |

3. Click **Deploy**

## Step 2: Verify

Visit `https://ueas-three.vercel.app` — the playground should load with:
- Monaco editor with UEAS syntax highlighting
- Accordion examples sidebar
- Transpile to Dafny, Lean 4, TLA+, LaTeX
- Evaluate + Audit buttons calling CF Workers MCP

## How It Works

The playground is a **pure MCP client** — zero WASM, zero build step. All computation happens via `fetch()` to the Cloudflare Workers MCP server at `ueas-mcp.seismael.workers.dev`.
