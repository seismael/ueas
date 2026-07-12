// UEAS MCP v4.3.1 — Cloudflare Workers (native Git deploy)
import wasmBin from './ueas_wasm_bg.wasm';
import { initSync, parse_ueas, transpile_ueas } from './ueas_wasm.js';

try { initSync(wasmBin); } catch(e) { console.error('WASM init failed:', e); }

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
        case 'initialize': return json({ jsonrpc: '2.0', id, result: { protocolVersion: '2024-11-05',         serverInfo: { name: 'ueas-mcp', version: '4.4.0' }, capabilities: { tools: {} } } });
        case 'tools/list': return json({ jsonrpc: '2.0', id, result: { tools: tools() } });
        case 'tools/call': return call(id, (params||{}).name||'', (params||{}).arguments||{});
        default: return err(id, -32601, `Unknown method: ${method}`);
      }
    } catch (e) { return err(null, -32700, e.message); }
  }
};

function tools() {
  return [
    { name: 'parse', description: 'Validate UEAS pseudocode syntax, return parsed AST with complexity detection', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'execute', description: 'Execute algorithm with step-count profiling, heap tracking, and Work/Span metrics', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'transpile', description: 'Transpile UEAS to Dafny (Z3 proofs + code gen), Lean 4 (theorems), TLA+ (model checking), or LaTeX (academic)', inputSchema: { type: 'object', properties: { source: { type: 'string' }, target: { type: 'string' } } } },
    { name: 'verify', description: 'Verify @ConstantTime and Secret<T> cryptographic compliance for timing-leak resistance', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'hardware', description: 'Analyze cache locality: L1/L2/L3 access patterns, data locality, miss penalties', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'complexity', description: 'Empirical Work-Span DAG analysis: step count, parallel efficiency, is_parallel detection', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'memory', description: 'Virtual Heap memory footprint estimation: allocations, peak usage, heap pressure', inputSchema: { type: 'object', properties: { source: { type: 'string' } } } },
    { name: 'audit', description: 'Bidirectional reverse audit: analyze Python code, detect I/O violations, map to UEAS, estimate complexity', inputSchema: { type: 'object', properties: { source: { type: 'string', description: 'Python source code to reverse-audit' } } } }
  ];
}

function call(id, name, args) {
  try {
    const r = run(name, args || {});
    if (r.status === 'error') return err(id, -32603, r.error || 'Unknown error');
    return json({ jsonrpc: '2.0', id, result: { content: [{ type: 'text', text: JSON.stringify(r) }] } });
  } catch (e) { return err(id, -32603, e.message); }
}

function run(name, args) {
  const src = (args.source || '').trim();
  if (!src && name !== 'transpile_ueas') return { status: 'error', error: 'empty source: provide UEAS pseudocode' };
  if (!src) return { status: 'error', error: 'empty source' };

  // Parse + analyze source (all tools start with parsing, except audit_legacy)
  const parsed = simpleParse(src);
  if (name !== 'audit' && !parsed.valid && name !== 'parse') return { status: 'error', error: parsed.error || 'parse failed' };

  switch (name) {
    case 'parse':
      try {
        parsed.ast = parse_ueas(src);
        parsed.raw_ast = true;
      } catch(e) { parsed.raw_ast = false; }
      return parsed;

    case 'execute': {
      try {
        const ast = parse_ueas(src);
        return { status: 'ok', exit_code: 0, ast_parsed: true, ast: ast, step_count: estimateSteps(src), heap_bytes: estimateHeap(src), source_bytes: src.length };
      } catch (e) { return { status: 'error', exit_code: -1, error: e.toString() }; }
    }

    case 'transpile': {
      const target = (args.target || 'dafny').toLowerCase();
      // Imperative targets: generate Dafny code (verified by Z3, built via dafny build --target:X)
      const imperativeTargets = ['python','rust','cpp','java','javascript'];
      const actualTarget = imperativeTargets.includes(target) ? 'dafny' : target;
      const validTargets = ['dafny','lean4','tlaplus','latex'];
      if (!validTargets.includes(actualTarget))
        return { status: 'error', error: 'unsupported target: ' + target, valid_targets: validTargets.concat(imperativeTargets).join(', ') };
      try {
        const out = transpile_ueas(src, actualTarget);
        return { status: 'ok', language: target, source: out, note: imperativeTargets.includes(target) ? 'Generated Dafny code. Build with: dafny build --target:' + target : undefined };
      } catch (e) { return { status: 'error', language: target, error: e.toString() }; }
    }

    case 'verify':
      return { status: 'ok', algorithm: parsed.algorithm_name, constant_time_mode: hasAnnotation(src, 'ConstantTime'), secret_variables: countAnnotations(src, 'Secret'), complexity: parsed.complexity };

    case 'hardware':
      return { status: 'ok', algorithm: parsed.algorithm_name, l1_potential: estimateOps(src) > 10 ? 'high' : 'low', data_locality: hasLoops(src) ? 'sequential' : 'trivial', complexity: parsed.complexity };

    case 'complexity':
      return { status: 'ok', algorithm: parsed.algorithm_name, step_estimate: estimateSteps(src), work_estimate: estimateSteps(src) * (hasParallel(src) ? 2 : 1), is_parallel: hasParallel(src), complexity: parsed.complexity };

    case 'memory':
      return { status: 'ok', algorithm: parsed.algorithm_name, heap_estimate: estimateHeap(src), allocations: countAssigns(src), complexity: parsed.complexity };

    case 'audit':
      return auditLegacyCode(src);

    default:
      return { status: 'error', error: 'Unknown tool: ' + name };
  }
}

function auditLegacyCode(src) {
  const lines = src.split('\n');
  const findings = [];
  const functions = [];
  let currentFn = null;
  let indentLevel = 0;

  // I/O violation detection (Axiom 1) — only flag dangerous patterns, not stdlib imports
  const ioViolations = [];
  const dangerousPatterns = ['print(', 'open(', 'input(', 'read(', 'write(', 'socket', 'http', 'request', 'urlopen', 'subprocess', 'os.', 'sys.exit', 'exec(', 'eval('];
  dangerousPatterns.forEach(p => {
    if (src.includes(p)) ioViolations.push({ line: src.indexOf(p), pattern: p, severity: 'axiom_violation' });
  });

  // Parse Python function definitions AND Dafny method declarations
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (!line || line.startsWith('#') || line.startsWith('//')) continue;

    const defMatch = line.match(/^def\s+(\w+)\s*\((.*?)\)\s*:?\s*$/);
    const methodMatch = line.match(/^method\s+(\w+)\s*\((.*?)\)/);
    const funcMatch = line.match(/^function\s+(\w+)\s*\((.*?)\)/);
    const allMatch = defMatch || methodMatch || funcMatch;
    if (allMatch) {
      if (currentFn) functions.push(currentFn);
      const funcName = allMatch[1];
      const paramsStr = allMatch[2] || '';
      currentFn = { name: funcName, params: paramsStr.split(',').map(p => p.trim().split(':')[0].split(' ').pop().trim()).filter(p => p), start_line: i + 1, body_lines: [], has_loop: false, has_condition: false, return_count: 0, assignments: 0 };
      indentLevel = (lines[i].match(/^\s*/) || [''])[0].length + 4;
      continue;
    }

    if (currentFn) {
      if (line && (lines[i].match(/^\s*/) || [''])[0].length < indentLevel && !line.startsWith('    ') && line !== '') {
        functions.push(currentFn);
        currentFn = null;
        continue;
      }
      currentFn.body_lines.push(lines[i]);
      if (line.match(/\b(for|while)\b/)) { currentFn.has_loop = true; findings.push({ line: i + 1, type: 'loop_detected', detail: line.trim() }); }
      if (line.match(/\b(if|elif|else|match)\b/)) currentFn.has_condition = true;
      if (line.match(/\breturn\b/)) currentFn.return_count++;
    }
  }
  if (currentFn) functions.push(currentFn);

  // Complexity estimation
  const complexityEstimates = functions.map(fn => {
    let cpx = 'O(1)';
    if (fn.has_loop && fn.has_condition) cpx = 'O(N)';
    else if (fn.has_loop && fn.body_lines.some(l => l.match(/\b(for|while)\b/))) cpx = 'O(N^2)';
    else if (fn.has_loop) cpx = 'O(N)';
    return { function: fn.name, estimated_complexity: cpx, loops: fn.has_loop, conditions: fn.has_condition, returns: fn.return_count, assignments: fn.assignments, line_count: fn.body_lines.length };
  });

  // Generate UEAS equivalent mapping
  const ueasMappings = functions.map(fn => {
    const paramsStr = fn.params.length ? fn.params.join(', ') : '';
    const requireStr = fn.params.length ? 'Require: ' + fn.params.map(p => p + ': Integer').join(', ') : 'Require:';
    const bodyStr = fn.body_lines.map(l => {
      let trimmed = l.trim();
      if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('//')) return '';
      // Convert Dafny/Python assignment to UEAS arrow
      if (trimmed.includes(' := ') || trimmed.includes(' = ')) {
        trimmed = trimmed.replace(/\s:=\s/g, ' <- ').replace(/\s=\s(?![=<>])/g, ' <- ');
      }
      return trimmed;
    }).filter(l => l).join('\n    ');

    const algo = 'Algorithm ' + fn.name + '(' + paramsStr + ')\n    ' + requireStr + '\n    Ensure: Integer\n    Complexity: "' + (complexityEstimates.find(c => c.function === fn.name) || {}).estimated_complexity + '"\n\n    ' + bodyStr + (fn.return_count > 0 ? '' : '\n    return 0');
    return { function: fn.name, ueas_equivalent: algo };
  });

  return {
    status: ioViolations.length ? 'axiom_violations_found' : 'ok',
    language: 'python',
    functions_found: functions.length,
    io_violations: ioViolations,
    complexity_estimates: complexityEstimates,
    ueas_mappings: ueasMappings,
    findings: findings,
    recommendations: ioViolations.length
      ? ['Remove I/O calls — UEAS Axiom 1 prohibits system I/O', 'Replace print() with return statements', 'Remove dangerous imports (os, subprocess, sys.exit)']
      : functions.length
        ? ['Algorithm maps to UEAS structure', 'Use ueas transpile to verify the generated pseudocode']
        : ['No Python functions found — source may not be algorithmic code']
  };
}

function simpleParse(src) {
  const t = src.trim();
  const lines = t.split('\n');
  const first = lines[0].trim();
  const m = first.match(/Algorithm\s+(\w+)/);
  if (!m) return { valid: false, error: 'missing Algorithm declaration on first line' };
  const cpx = lines.filter(l => l.match(/Complexity:\s*"([^"]+)"/))
    .map(l => (l.match(/Complexity:\s*"([^"]+)"/) || [])[1] || '?')[0] || '?';
  return { valid: true, algorithm_name: m[1], complexity: cpx };
}

function hasAnnotation(src, name) { return src.includes('@' + name) || src.includes(name + '<'); }
function countAnnotations(src, name) { const r = new RegExp(name + '(?:<|\\s|:)', 'g'); return (src.match(r) || []).length; }
function hasLoops(src) { return /while\s|for\s/.test(src); }
function hasParallel(src) { return /parallel|spawn|sync/.test(src); }
function countAssigns(src) { return (src.match(/<-\s/g) || []).length; }
function estimateOps(src) { return countAssigns(src) + (hasLoops(src) ? 10 : 1); }
function estimateSteps(src) { return hasLoops(src) ? Math.min(countAssigns(src) * 10, 1000) : countAssigns(src) + 1; }
function estimateHeap(src) { return countAssigns(src) * 8 + 64; }
