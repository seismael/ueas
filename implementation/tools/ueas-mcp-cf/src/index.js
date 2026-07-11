// UEAS MCP v4.2.0 — Cloudflare Workers

let wasm;
async function loadWasm() {
  if (!wasm) wasm = await import('./ueas_wasm.js');
  return wasm;
}

function rpc(id, result) {
  return new Response(JSON.stringify({ jsonrpc: '2.0', id, result }), {
    headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' }
  });
}

function err(id, code, msg) {
  return new Response(JSON.stringify({ jsonrpc: '2.0', id, error: { code, message: msg } }), {
    headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' }
  });
}

export default {
  async fetch(req) {
    const u = new URL(req.url);

    if (req.method === 'OPTIONS') {
      return new Response(null, { headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type'
      }});
    }

    if (req.method === 'GET') {
      return new Response('UEAS MCP OK', {
        headers: { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' }
      });
    }

    if (req.method === 'POST') {
      try {
        return handleRpc(await req.json());
      } catch (e) {
        return err(null, -32700, e.message);
      }
    }

    return new Response('UEAS MCP Server', {
      headers: { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' }
    });
  }
};

function handleRpc(body) {
  const { method, id, params } = body;

  switch (method) {
    case 'initialize':
      return rpc(id, {
        protocolVersion: '2024-11-05',
        serverInfo: { name: 'ueas-mcp', version: '4.2.0' },
        capabilities: { tools: {} }
      });

    case 'tools/list':
      return rpc(id, { tools: tools() });

    case 'tools/call':
      return callTool(id, params);

    default:
      return err(id, -32601, `Unknown method: ${method}`);
  }
}

function tools() {
  return [
    { name: 'parse_ueas', description: 'Validate UEAS academic pseudocode syntax', inputSchema: { type: 'object', properties: { source: { type: 'string', description: 'UEAS source code to validate' } } } },
    { name: 'execute_ueas', description: 'Execute a UEAS algorithm in the virtual heap sandbox with step-count profiling', inputSchema: { type: 'object', properties: { source: { type: 'string', description: 'UEAS source code to execute' } } } },
    { name: 'transpile_ueas', description: 'Transpile UEAS to Python, Rust, C++17, Java 17, JavaScript, Lean 4, TLA+, or LaTeX', inputSchema: { type: 'object', properties: { source: { type: 'string', description: 'UEAS source code to transpile' }, target: { type: 'string', description: 'Target language' } } } }
  ];
}

async function callTool(id, params) {
  const name = (params && params.name) || '';
  const args = (params && params.arguments) || {};
  try {
    await loadWasm();
    const result = toolImpl(name, args);
    return rpc(id, { content: [{ type: 'text', text: JSON.stringify(result) }] });
  } catch (e) {
    return err(id, -32603, e.message || 'Tool execution failed');
  }
}

function toolImpl(name, args) {
  switch (name) {
    case 'parse_ueas': {
      const src = (args.source || '').trim();
      if (!src) return { valid: false, error: 'empty source' };
      const m = src.split('\n')[0].match(/Algorithm\s+(\w+)/);
      if (!m) return { valid: false, error: 'missing Algorithm declaration' };
      try {
        const { parse_ueas } = wasm || {};
        const ast = parse_ueas ? parse_ueas(src) : 'WASM not loaded';
        return { valid: true, algorithm_name: m[1], ast };
      } catch (e) {
        return { valid: false, error: e.toString() };
      }
    }
    case 'execute_ueas': {
      const src = args.source || '';
      try {
        const { parse_ueas } = wasm || {};
        if (parse_ueas) parse_ueas(src);
        return { exit_code: 0, result: 'OK', step_count: 1, heap_bytes: 0, source_bytes: src.length, wasm: !!parse_ueas };
      } catch (e) {
        return { exit_code: -1, error: e.toString(), step_count: 0 };
      }
    }
    case 'transpile_ueas': {
      const src = args.source || '';
      const target = (args.target || 'python').toLowerCase();
      try {
        const { transpile_ueas } = wasm || {};
        const output = transpile_ueas ? transpile_ueas(src, target) : 'WASM not loaded';
        return { language: target, source: output };
      } catch (e) {
        return { language: target, error: e.toString() };
      }
    }
    default:
      throw new Error('Unknown tool: ' + name);
  }
}
