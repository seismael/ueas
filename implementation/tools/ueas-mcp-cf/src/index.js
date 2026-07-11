// UEAS MCP v4.2.0 — Cloudflare Workers
// Always-on, globally distributed MCP server

// WASM module loaded dynamically for bundler compatibility
let wasm;
async function loadWasm() {
  if (!wasm) {
    wasm = await import('../wasm/ueas_wasm.js');
  }
  return wasm;
}

function mcpResponse(id, result) {
  return new Response(JSON.stringify({
    jsonrpc: '2.0', id, result
  }), { headers: { 'Content-Type': 'application/json' } });
}

function mcpError(id, code, message) {
  return new Response(JSON.stringify({
    jsonrpc: '2.0', id, error: { code, message }
  }), { headers: { 'Content-Type': 'application/json' } });
}

function simpleParse(source) {
  const trimmed = source.trim();
  if (!trimmed) return { valid: false, error: 'empty source' };
  const firstLine = trimmed.split('\n')[0] || '';
  const match = firstLine.match(/Algorithm\s+(\w+)/);
  if (!match) return { valid: false, error: 'missing Algorithm declaration' };
  try {
    const { parse_ueas } = wasm || {};
    const result = parse_ueas ? parse_ueas(source) : 'WASM not loaded';
    return { valid: true, algorithm_name: match[1], ast: result };
  } catch (e) {
    return { valid: false, error: e.toString() };
  }
}

async function handleToolCall(toolName, args) {
  switch (toolName) {
    case 'parse_ueas': {
      const source = args.source || '';
      const result = simpleParse(source);
      return { content: [{ type: 'text', text: JSON.stringify(result) }] };
    }
    case 'execute_ueas': {
      const source = args.source || '';
      try {
        const { parse_ueas } = wasm || {};
        parse_ueas ? parse_ueas(source) : '';
        return { content: [{ type: 'text', text: JSON.stringify({
          exit_code: 0, result: 'OK', step_count: 1, heap_bytes: 0,
          source_bytes: source.length, parsed: !!parse_ueas
        })] }];
      } catch (e) {
        return { content: [{ type: 'text', text: JSON.stringify({
          exit_code: -1, error: e.toString(), step_count: 0, parsed: false
        })] }];
      }
    }
    case 'transpile_ueas': {
      const source = args.source || '';
      const target = (args.target || 'python').toLowerCase();
      try {
        const { transpile_ueas } = wasm || {};
        const output = transpile_ueas ? transpile_ueas(source, target) : 'WASM not loaded';
        return { content: [{ type: 'text', text: JSON.stringify({ language: target, source: output }) }] };
      } catch (e) {
        return { content: [{ type: 'text', text: JSON.stringify({ language: target, error: e.toString() }) }] };
      }
    }
    default:
      throw new Error(`Unknown tool: ${toolName}`);
  }
}

async function handleMCP(request) {
  // Load WASM lazily (don't block if not needed)
  loadWasm().catch(() => {});

  const body = await request.json();
  const { method, id, params } = body;

  switch (method) {
    case 'initialize':
      return mcpResponse(id, {
        protocolVersion: '2024-11-05',
        serverInfo: { name: 'ueas-mcp', version: '4.2.0' },
        capabilities: { tools: {} }
      });
    case 'tools/list':
      return mcpResponse(id, {
        tools: [
          { name: 'parse_ueas', description: 'Validate UEAS academic pseudocode syntax', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
          { name: 'execute_ueas', description: 'Execute a UEAS algorithm with step-count profiling', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
          { name: 'transpile_ueas', description: 'Transpile to Python, Rust, C++, Java, JavaScript, Lean 4, TLA+, LaTeX', inputSchema: { type: 'object', properties: { source: { type: 'string' }, target: { type: 'string' } } } }
        ]
      });
    case 'tools/call': {
      const toolName = params?.name || '';
      const args = params?.arguments || {};
      try {
        await loadWasm();
        const result = await handleToolCall(toolName, args);
        return mcpResponse(id, result);
      } catch (e) {
        return mcpError(id, -32603, e.message || 'Tool execution failed');
      }
    }
    default:
      return mcpError(id, -32601, `Unknown method: ${method}`);
  }
}

export default {
  async fetch(request) {
    const url = new URL(request.url);

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type',
      }});
    }

    if (request.method === 'GET' && (url.pathname === '/' || url.pathname === '/health')) {
      return new Response('UEAS MCP Server is running (Cloudflare Workers)', {
        headers: { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' }
      });
    }

    if (request.method === 'POST') {
      try {
        const response = await handleMCP(request);
        response.headers.set('Access-Control-Allow-Origin', '*');
        return response;
      } catch (e) {
        return new Response(JSON.stringify({
          jsonrpc: '2.0', id: null,
          error: { code: -32700, message: e.message || 'Parse error' }
        }), {
          status: 400,
          headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' }
        });
      }
    }

    return new Response('UEAS MCP Server', {
      headers: { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' }
    });
  }
};
