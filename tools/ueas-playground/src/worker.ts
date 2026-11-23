/**
 * Dedicated Web Worker running `ueas-wasm` off the main UI thread.
 */
interface WorkerRequest {
  id: number;
  action: "check" | "verify" | "transpile" | "run" | "format";
  source: string;
  target?: string;
  level?: string;
}

interface UeasWasmApi {
  default?: () => Promise<unknown>;
  ueas_check(source: string): string;
  ueas_format(source: string): string;
  ueas_verify(source: string, level: string, fuzzIters: number): string;
  ueas_transpile(source: string, target: string): string;
  ueas_run(source: string, entry: string, stepBudget: bigint): string;
}

let wasmPromise: Promise<UeasWasmApi> | null = null;

async function loadWasm(): Promise<UeasWasmApi> {
  if (!wasmPromise) {
    const pkgPath = "/pkg/ueas_wasm.js";
    wasmPromise = (import(/* @vite-ignore */ pkgPath) as Promise<UeasWasmApi>).then(async (mod) => {
      if (typeof mod.default === "function") {
        await mod.default();
      }
      return mod;
    });
  }
  return wasmPromise;
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const req = event.data;
  try {
    const wasm = await loadWasm();
    let payload: string;
    switch (req.action) {
      case "check":
        payload = wasm.ueas_check(req.source);
        break;
      case "format":
        payload = wasm.ueas_format(req.source);
        break;
      case "verify":
        payload = wasm.ueas_verify(req.source, req.level ?? "L3", 32);
        break;
      case "transpile":
        payload = wasm.ueas_transpile(req.source, req.target ?? "rust");
        break;
      case "run":
        payload = wasm.ueas_run(req.source, "", 1_000_000n);
        break;
    }
    self.postMessage({ id: req.id, ok: true, payload });
  } catch (err) {
    self.postMessage({
      id: req.id,
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    });
  }
};
