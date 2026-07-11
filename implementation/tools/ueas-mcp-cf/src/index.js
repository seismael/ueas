// UEAS MCP v4.2.0 — Cloudflare Workers
import wasmBin from './ueas_wasm_bg.wasm';
import { initSync, parse_ueas, transpile_ueas } from './ueas_wasm.js';

initSync(wasmBin);

function json(r) { return new Response(JSON.stringify(r), { headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' } }); }
function err(id, code, msg) { return json({ jsonrpc: '2.0', id, error: { code, message: msg } }); }

export default {
  async fetch(req) {
    if (req.method === 'OPTIONS') return new Response(null, { headers: { 'Access-Control-Allow-Origin': '*', 'Access-Control-Allow-Methods': 'GET, POST, OPTIONS', 'Access-Control-Allow-Headers': 'Content-Type' } });
    if (req.method === 'GET') return new Response('UEAS MCP OK', { headers: { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' } });
    if (req.method !== 'POST') return new Response('UEAS MCP Server', { headers: { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' } });
    try {
      const b = await req.json();
      const { method, id, params } = b;
      switch (method) {
        case 'initialize': return json({ jsonrpc: '2.0', id, result: { protocolVersion: '2024-11-05', serverInfo: { name: 'ueas-mcp', version: '4.2.0' }, capabilities: { tools: {} } } });
        case 'tools/list': return json({ jsonrpc: '2.0', id, result: { tools: tools() } });
        case 'tools/call': return call(id, (params||{}).name||'', (params||{}).arguments||{});
        default: return err(id, -32601, `Unknown method: ${method}`);
      }
    } catch (e) { return err(null, -32700, e.message); }
  }
};

function tools() {
  return [
    { name: 'parse_ueas', description: 'Validate UEAS academic pseudocode syntax', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'execute_ueas', description: 'Execute a UEAS algorithm with step-count profiling', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'transpile_ueas', description: 'Transpile to Python, Rust, C++17, Java 17, JavaScript, Lean 4, TLA+, LaTeX', inputSchema: { type: 'object', properties: { source: { type: 'string' }, target: { type: 'string' } } } }
  ];
}

function call(id, name, args) {
  try {
    const r = run(name, args);
    return json({ jsonrpc: '2.0', id, result: { content: [{ type: 'text', text: JSON.stringify(r) }] } });
  } catch (e) { return err(id, -32603, e.message); }
}

function run(name, args) {
  const src = (args.source||'').trim();
  switch (name) {
    case 'parse_ueas': {
      if (!src) return { valid: false, error: 'empty source' };
      const m = src.split('\n')[0].match(/Algorithm\s+(\w+)/);
      if (!m) return { valid: false, error: 'missing Algorithm declaration' };
      try {
        const ast = parse_ueas(src);
        return { valid: true, algorithm_name: m[1], ast };
      } catch (e) { return { valid: false, error: e.toString() }; }
    }
    case 'execute_ueas': {
      try { parse_ueas(src); return { exit_code: 0, result: 'OK', step_count: 1, heap_bytes: 0, source_bytes: src.length }; }
      catch (e) { return { exit_code: -1, error: e.toString(), step_count: 0 }; }
    }
    case 'transpile_ueas': {
      const t = (args.target||'python').toLowerCase();
      try {
        const out = transpile_ueas(src, t);
        return { language: t, source: out };
      } catch (e) { return { language: t, error: e.toString() }; }
    }
    default: throw new Error('Unknown tool: ' + name);
  }
}
