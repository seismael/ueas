# Vercel Playground Deployment (`ueas-playground`)

The UEAS v2.0 Playground is a Vite + TypeScript SPA executing `ueas-wasm` locally inside a browser Web Worker (`src/worker.ts`) with zero `innerHTML` sinks.

## Build & Deploy

| Field | Value |
| :--- | :--- |
| **Root Directory** | `tools/ueas-playground` |
| **Framework Preset** | Vite |
| **Build Command** | `npm run build` |
| **Output Directory** | `dist` |

Build `ueas-wasm` into `tools/ueas-playground/public/pkg` before deploying:

```bash
wasm-pack build crates/ueas-wasm --target web --out-dir ../../tools/ueas-playground/public/pkg
cd tools/ueas-playground
npm ci && npm run build
```
