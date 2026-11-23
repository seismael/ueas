/**
 * UEAS v2.0 Edge MCP Server (Cloudflare Worker / Streamable HTTP).
 * Backed 100% by compiled `ueas-wasm` — zero regex heuristics or mocked metrics.
 */

export interface UeasWasmModule {
  ueas_check(source: string): string;
  ueas_format(source: string): string;
  ueas_verify(source: string, level: string, fuzzIterations: number): string;
  ueas_transpile(source: string, target: string): string;
  ueas_run(source: string, entry: string, stepBudget: number): string;
}

interface JsonRpcRequest {
  jsonrpc?: string;
  id?: string | number | null;
  method?: string;
  params?: {
    name?: string;
    arguments?: Record<string, unknown>;
  };
}

const MAX_SOURCE_BYTES = 128 * 1024;
const MAX_FUZZ_ITERS = 64;
const MAX_STEP_BUDGET = 1_000_000;

let wasmLoader: Promise<UeasWasmModule> | null = null;

async function getWasm(): Promise<UeasWasmModule> {
  if (!wasmLoader) {
    // Expects `wasm-pack build crates/ueas-wasm --target web --out-dir ../../tools/ueas-mcp-edge/src/pkg`
    const modPath = "./pkg/ueas_wasm.js";
    wasmLoader = import(/* @vite-ignore */ modPath) as Promise<UeasWasmModule>;
  }
  return wasmLoader;
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "access-control-allow-origin": "*",
    },
  });
}

export default {
  async fetch(request: Request): Promise<Response> {
    if (request.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: {
          "access-control-allow-origin": "*",
          "access-control-allow-methods": "POST, GET, OPTIONS",
          "access-control-allow-headers": "content-type",
        },
      });
    }

    const url = new URL(request.url);
    if (url.pathname === "/healthz") {
      return jsonResponse({ ok: true, engine: "ueas-wasm-2.0" });
    }

    if (request.method !== "POST") {
      return jsonResponse({ error: "Method Not Allowed" }, 405);
    }

    let req: JsonRpcRequest;
    try {
      req = (await request.json()) as JsonRpcRequest;
    } catch {
      return jsonResponse({ jsonrpc: "2.0", id: null, error: { code: -32700, message: "Parse error" } }, 400);
    }

    const id = req.id ?? null;
    if (req.method === "initialize") {
      return jsonResponse({
        jsonrpc: "2.0",
        id,
        result: {
          protocolVersion: "2025-03-26",
          serverInfo: { name: "ueas-mcp-edge", version: "2.0.0" },
          capabilities: { tools: {} },
        },
      });
    }

    if (req.method === "tools/list") {
      const sourceProp = { type: "string", description: "UEAS v2.1 source code" };
      return jsonResponse({
        jsonrpc: "2.0",
        id,
        result: {
          tools: [
            {
              name: "ueas_check",
              description: "Static L0 check (parse, HIR, types, MVS, CT)",
              inputSchema: { type: "object", properties: { source: sourceProp }, required: ["source"] },
            },
            {
              name: "ueas_verify",
              description: "Progressive L0-L3 verification & SHA-256 certificate",
              inputSchema: {
                type: "object",
                properties: {
                  source: sourceProp,
                  level: { type: "string", enum: ["L0", "L1", "L2", "L3"] },
                  fuzzIterations: { type: "integer" },
                },
                required: ["source"],
              },
            },
            {
              name: "ueas_transpile",
              description: "Dual-track transpilation across Track A/B/C/WASM/SMT-LIB2 targets",
              inputSchema: {
                type: "object",
                properties: { source: sourceProp, target: { type: "string" } },
                required: ["source", "target"],
              },
            },
            {
              name: "ueas_run",
              description: "Execute @Test suite or Algorithm in deterministic VM",
              inputSchema: {
                type: "object",
                properties: {
                  source: sourceProp,
                  entry: { type: "string" },
                  stepBudget: { type: "integer" },
                },
                required: ["source"],
              },
            },
            {
              name: "ueas_format",
              description: "Format UEAS v2.1 source code",
              inputSchema: { type: "object", properties: { source: sourceProp }, required: ["source"] },
            },
          ],
        },
      });
    }

    if (req.method === "tools/call") {
      const toolName = req.params?.name ?? "";
      const args = req.params?.arguments ?? {};
      const source = typeof args.source === "string" ? args.source : "";
      if (source.length > MAX_SOURCE_BYTES) {
        return jsonResponse({
          jsonrpc: "2.0",
          id,
          error: { code: -32602, message: `source exceeds ${MAX_SOURCE_BYTES} byte edge limit` },
        }, 400);
      }

      const wasm = await getWasm();
      let output: string;
      switch (toolName) {
        case "ueas_check":
          output = wasm.ueas_check(source);
          break;
        case "ueas_format":
          output = wasm.ueas_format(source);
          break;
        case "ueas_verify": {
          const level = typeof args.level === "string" ? args.level : "L3";
          const iters = typeof args.fuzzIterations === "number" ? Math.min(args.fuzzIterations, MAX_FUZZ_ITERS) : 32;
          output = wasm.ueas_verify(source, level, iters);
          break;
        }
        case "ueas_transpile": {
          const target = typeof args.target === "string" ? args.target : "rust";
          output = wasm.ueas_transpile(source, target);
          break;
        }
        case "ueas_run": {
          const entry = typeof args.entry === "string" ? args.entry : "";
          const budget = typeof args.stepBudget === "number" ? Math.min(args.stepBudget, MAX_STEP_BUDGET) : 500_000;
          output = wasm.ueas_run(source, entry, budget);
          break;
        }
        default:
          return jsonResponse({
            jsonrpc: "2.0",
            id,
            error: { code: -32601, message: `Unknown tool: ${toolName}` },
          }, 404);
      }

      return jsonResponse({
        jsonrpc: "2.0",
        id,
        result: { content: [{ type: "text", text: output }] },
      });
    }

    return jsonResponse({ jsonrpc: "2.0", id, result: {} });
  },
};
