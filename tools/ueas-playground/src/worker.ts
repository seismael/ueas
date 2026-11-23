/**
 * Dedicated Web Worker running `ueas-wasm` off the main UI thread.
 */
import initWasm, {
  ueas_check,
  ueas_format,
  ueas_verify,
  ueas_transpile,
  ueas_run,
} from "./pkg/ueas_wasm.js";
import wasmUrl from "./pkg/ueas_wasm_bg.wasm?url";

interface WorkerRequest {
  id: number;
  action: "check" | "verify" | "transpile" | "run" | "format";
  source: string;
  target?: string;
  level?: string;
}

interface UeasWasmApi {
  ueas_check(source: string): string;
  ueas_format(source: string): string;
  ueas_verify(source: string, level: string, fuzzIters: number): string;
  ueas_transpile(source: string, target: string): string;
  ueas_run(source: string, entry: string, stepBudget: bigint): string;
}

let wasmPromise: Promise<UeasWasmApi> | null = null;

function loadWasm(): Promise<UeasWasmApi> {
  if (!wasmPromise) {
    wasmPromise = initWasm(wasmUrl).then(() => ({
      ueas_check,
      ueas_format,
      ueas_verify,
      ueas_transpile,
      ueas_run,
    }));
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
