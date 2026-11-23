# UEAS v2.0 Studio Playground

Vite + TypeScript browser playground executing `ueas-wasm` inside a dedicated Web Worker (`src/worker.ts`).

```bash
wasm-pack build ../../crates/ueas-wasm --target web --out-dir public/pkg
npm install
npm run typecheck
npm run dev
```
