// UEAS Playground — client-side interactive algorithm viewer
import init, { parse_ueas, transpile_ueas } from './wasm/pkg/ueas_wasm.js';

require.config({
  paths: { vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.44.0/min/vs' }
});

let editor;

// Algorithm examples gallery
const examples = [
  {
    name: 'Euclidean Distance',
    complexity: 'O(1)',
    code: `Algorithm EuclideanDistance(x1, y1, x2, y2)
    Require: x1: Real, y1: Real, x2: Real, y2: Real
    Ensure: Real
    Complexity: "O(1)"

    dx <- x2 - x1
    dy <- y2 - y1
    return sqrt(dx * dx + dy * dy)`
  },
  {
    name: 'Linear Search',
    complexity: 'O(N)',
    code: `Algorithm LinearSearch(data, target)
    Require: data: List, target: Integer
    Ensure: Integer
    Complexity: "O(N)"

    for each item in data do
        if item == target then
            return item
        end if
    end for

    return -1`
  },
  {
    name: 'Binary Search',
    complexity: 'O(log N)',
    code: `Algorithm BinarySearch(data, target)
    Require: data: List, target: Integer
    Ensure: Integer
    Complexity: "O(log N)"

    low <- 0
    high <- data.length - 1

    while low <= high do
        mid <- low + (high - low) / 2

        if data[mid] == target then
            return mid
        else if data[mid] < target then
            low <- mid + 1
        else
            high <- mid - 1
        end if
    end while

    return -1`
  },
  {
    name: 'Merge Sort',
    complexity: 'O(N log N)',
    code: `Algorithm MergeSort(data)
    Require: data: List
    Ensure: List
    Complexity: "O(N log N)"

    if data.length <= 1 then
        return data
    end if

    mid <- data.length / 2
    left <- data[0:mid]
    right <- data[mid:data.length]

    return Merge(MergeSort(left), MergeSort(right))`
  },
  {
    name: 'Dijkstra SSSP',
    complexity: 'O((V+E) log V)',
    code: `Algorithm DijkstraShortestPath(g, source)
    Require: g: Graph, source: Integer
    Ensure: Map
    Complexity: "O((V+E) log V)", V = g.nodes.length, E = g.edges.length

    distances <- {}
    unvisited <- g.nodes

    for each node in unvisited do
        if node == source then
            distances[node] <- 0.0
        else
            distances[node] <- 999999.0
        end if
    end for

    while unvisited.length > 0 do
        invariant(unvisited.length <= V)
        current <- extractMin(unvisited, distances)
        neighbors <- g.adjacent(current)

        for each neighbor in neighbors do
            edgeWeight <- g.weight(current, neighbor)
            newDist <- distances[current] + edgeWeight

            if newDist < distances[neighbor] then
                distances[neighbor] <- newDist
            end if
        end for
    end while

    return distances`
  },
  {
    name: 'Quicksort',
    complexity: 'O(N log N)',
    code: `Algorithm QuickSort(data)
    Require: data: List
    Ensure: List
    Complexity: "O(N log N)"

    if data.length <= 1 then
        return data
    end if

    pivot <- data[0]
    left <- []
    right <- []

    for each item in data[1:] do
        if item <= pivot then
            left.append(item)
        else
            right.append(item)
        end if
    end for

    return QuickSort(left) + [pivot] + QuickSort(right)`
  },
  {
    name: 'BFS',
    complexity: 'O(V + E)',
    code: `Algorithm BreadthFirstSearch(g, start)
    Require: g: Graph, start: Integer
    Ensure: Set
    Complexity: "O(V + E)", V = g.nodes.length, E = g.edges.length

    visited <- {start}
    queue <- [start]

    while queue.length > 0 do
        current <- queue.pop(0)

        for each neighbor in g.adjacent(current) do
            if neighbor not in visited then
                visited.add(neighbor)
                queue.append(neighbor)
            end if
        end for
    end while

    return visited`
  }
];

// Helper to run WASM logic
async function doTranspile() {
  const code = editor.getValue();
  const target = document.getElementById('target-select').value;
  
  // Transpile
  try {
    const transpiled = transpile_ueas(code, target);
    document.getElementById('output-transpiled').textContent = transpiled;
  } catch (err) {
    document.getElementById('output-transpiled').textContent = "Transpilation Error:\n" + err;
  }

  // Parse AST
  try {
    const ast = parse_ueas(code);
    document.getElementById('output-ast').textContent = ast;
  } catch (err) {
    document.getElementById('output-ast').textContent = "Parse Error:\n" + err;
  }
  
  switchTab('transpiled');
}
window.doTranspile = doTranspile;

function simulateTranspile() {
  doTranspile();
}
window.simulateTranspile = simulateTranspile;

function formatCode() {
  // Basic formatting: normalize whitespace
  const code = editor.getValue();
  const lines = code.split('\n');
  let depth = 0;
  const formatted = lines.map(line => {
    const trimmed = line.trim();
    if (!trimmed) return '';
    if (trimmed.startsWith('end ') || trimmed.startsWith('End ') || trimmed.startsWith('END ')) {
      depth = Math.max(0, depth - 1);
    }
    const result = '    '.repeat(depth) + trimmed;
    if (trimmed.startsWith('for ') || trimmed.startsWith('while ') || trimmed.startsWith('if ')) {
      depth += 1;
    }
    return result;
  }).join('\n');
  editor.setValue(formatted);
}
window.formatCode = formatCode;

function copyToClipboard() {
  navigator.clipboard.writeText(editor.getValue()).then(function() {
    showToast('Copied to clipboard');
  });
}
window.copyToClipboard = copyToClipboard;

function switchTab(tab) {
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.output').forEach(o => o.classList.remove('active'));
  document.querySelector('[onclick="switchTab(\'' + tab + '\')"]').classList.add('active');
  document.getElementById('output-' + tab).classList.add('active');
}
window.switchTab = switchTab;

function showToast(msg) {
  const el = document.createElement('div');
  el.textContent = msg;
  el.style.cssText = 'position:fixed;bottom:20px;right:20px;background:var(--accent-dim);color:white;padding:8px 16px;border-radius:6px;font-size:0.8rem;z-index:999;';
  document.body.appendChild(el);
  setTimeout(function() { el.remove(); }, 2000);
}

function loadExample(index) {
  const ex = examples[index];
  editor.setValue(ex.code);
  document.querySelectorAll('.example-item').forEach(function(el, i) {
    el.classList.toggle('active', i === index);
  });
}
window.loadExample = loadExample;

// Initialize
async function initializeApp() {
  try {
    await init(); // Initialize WASM
    console.log("WASM Initialized successfully");
  } catch (err) {
    console.error("Failed to initialize WASM:", err);
  }

  require(['vs/editor/editor.main'], function() {
    // Register UEAS language
    monaco.languages.register({ id: 'ueas' });

    monaco.languages.setMonarchTokensProvider('ueas', {
      keywords: [
        'Algorithm', 'algorithm', 'ALGORITHM',
        'Require', 'require', 'REQUIRE',
        'Ensure', 'ensure', 'ENSURE',
        'Complexity', 'complexity', 'COMPLEXITY',
        'Memory', 'memory', 'MEMORY',
        'return', 'Return', 'RETURN',
        'if', 'If', 'IF', 'then', 'Then', 'THEN',
        'else', 'Else', 'ELSE',
        'for', 'For', 'FOR', 'each', 'Each', 'EACH', 'in', 'In', 'IN', 'do', 'Do', 'DO',
        'while', 'While', 'WHILE',
        'end', 'End', 'END',
        'assert', 'Assert', 'ASSERT',
        'invariant', 'Invariant', 'INVARIANT',
        'and', 'And', 'AND', 'or', 'Or', 'OR', 'not', 'Not', 'NOT',
        'true', 'True', 'TRUE', 'false', 'False', 'FALSE',
      ],
      typeKeywords: [
        'Integer', 'Real', 'Boolean', 'String', 'Void',
        'List', 'Set', 'Map', 'Graph', 'Matrix',
      ],
      operators: ['<-', ':=', '=', '==', '!=', '<', '<=', '>', '>=', '+', '-', '*', '/', 'mod', '->'],
      tokenizer: {
        root: [
          [/#.*$/, 'comment'],
          [/"([^"\\]|\\.)*$/, 'string.invalid'],
          [/'([^'\\]|\\.)*$/, 'string.invalid'],
          [/"/, 'string', '@string_double'],
          [/'/, 'string', '@string_single'],
          [/[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?/, 'number.float'],
          [/[0-9]+/, 'number'],
          [/[a-zA-Z_][a-zA-Z0-9_]*/, {
            cases: {
              '@typeKeywords': 'type',
              '@keywords': 'keyword',
              '@default': 'identifier'
            }
          }],
          [/<-|:=/, 'keyword'],
          [/[+\-*/=<>!]+/, 'operator'],
          [/[{}()[\],.:]/, 'delimiter'],
        ],
        string_double: [
          [/[^\\"]+/, 'string'],
          [/\\./, 'string.escape'],
          [/"/, 'string', '@pop'],
        ],
        string_single: [
          [/[^\\']+/, 'string'],
          [/\\./, 'string.escape'],
          [/'/, 'string', '@pop'],
        ],
      }
    });

    monaco.editor.defineTheme('ueas-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'keyword', foreground: '569CD6', fontStyle: 'bold' },
        { token: 'type', foreground: '4EC9B0' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'number.float', foreground: 'B5CEA8' },
        { token: 'operator', foreground: 'D4D4D4' },
        { token: 'identifier', foreground: 'DCDCAA' },
        { token: 'delimiter', foreground: '808080' },
      ],
      colors: {
        'editor.background': '#0d1117',
        'editor.foreground': '#c9d1d9',
        'editor.lineHighlightBackground': '#161b22',
        'editor.selectionBackground': '#264f78',
        'editorCursor.foreground': '#58a6ff',
        'editorLineNumber.foreground': '#484f58',
      }
    });

    editor = monaco.editor.create(document.getElementById('editor'), {
      value: examples[0].code,
      language: 'ueas',
      theme: 'ueas-dark',
      fontSize: 14,
      fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace",
      lineNumbers: 'on',
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 4,
      renderWhitespace: 'selection',
      bracketPairColorization: { enabled: false },
    });

    // Render examples sidebar
    const list = document.getElementById('examples-list');
    examples.forEach(function(ex, i) {
      const div = document.createElement('div');
      div.className = 'example-item' + (i === 0 ? ' active' : '');
      div.innerHTML = '<div class="name">' + ex.name + '</div><div class="meta">' + ex.complexity + '</div>';
      div.onclick = function() { loadExample(i); };
      list.appendChild(div);
    });
  });
}

// Track 6: Execute + Dashboard + AST Tree + Hybrid Mode + Advanced Sandbox

async function runExecute() {
  const code = editor.getValue();
  const inputs = document.getElementById('exec-inputs').value.trim() || '{}';
  const useRemote = document.getElementById('hybrid-mode').checked;
  
  document.getElementById('dashboard').style.display = 'flex';
  document.getElementById('exec-status').textContent = 'Running...';
  
  try {
    const result = useRemote 
      ? await remoteExecute(code, inputs)
      : localExecute(code);
    
    updateDashboard(result);
    updateAstTree(result.ast || code);
  } catch (e) {
    document.getElementById('exec-status').textContent = 'Error';
    document.getElementById('exec-steps').textContent = e.message;
  }
}

function localExecute(code) {
  try {
    const ast = parse_ueas(code);
    const executed = execute_ueas(code);
    const result = JSON.parse(executed);
    result.ast = ast;
    return result;
  } catch (e) {
    return { status: 'error', exit_name: e.toString(), step_count: 0, heap_bytes: 0 };
  }
}

async function remoteExecute(code, inputs) {
  const resp = await fetch('https://ueas-mcp.seismael.workers.dev', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0', id: 1, method: 'tools/call',
      params: { name: 'execute_ueas', arguments: { source: code, inputs: inputs } }
    })
  });
  const data = await resp.json();
  const text = data?.result?.content?.[0]?.text || '{}';
  return JSON.parse(text);
}

function updateDashboard(r) {
  document.getElementById('exec-status').textContent = r.status || r.exit_name || 'OK';
  document.getElementById('exec-status').style.color = r.exit_code > 0 ? 'var(--red)' : 'var(--green)';
  document.getElementById('exec-steps').textContent = r.step_count ?? '—';
  document.getElementById('exec-heap').textContent = (r.heap_bytes ?? '—') + ' B';
  document.getElementById('exec-work').textContent = r.work ?? '—';
  document.getElementById('exec-span').textContent = r.span ?? '—';
  document.getElementById('exec-cache').textContent = (r.cache_l1_hits ?? '—');
  document.getElementById('exec-parallel').textContent = r.parallel_efficiency != null ? (r.parallel_efficiency * 100).toFixed(1) + '%' : '—';
  
  const complexity = extractComplexity(editor.getValue());
  document.getElementById('exec-complexity').textContent = complexity;
  
  // Step bar: ratio of actual steps vs complexity
  const steps = r.step_count || 0;
  document.getElementById('step-bar-fill').style.width = Math.min(steps * 2, 100) + '%';
  
  // Auto-switch to AST tab
  switchTab('ast');
}

function extractComplexity(code) {
  const m = code.match(/Complexity:\s*"([^"]+)"/);
  return m ? m[1] : '?';
}

function updateAstTree(astJson) {
  const el = document.getElementById('ast-tree');
  try {
    const ast = typeof astJson === 'string' ? JSON.parse(astJson) : astJson;
    el.innerHTML = renderAstNode(ast, 0);
  } catch {
    el.innerHTML = '<span style="color:var(--text-dim)">Parse the algorithm first (click Run) to see AST tree.</span>';
  }
}

function renderAstNode(node, depth) {
  if (!node || typeof node !== 'object') return '';
  let html = '';
  
  if (node.kind) {
    html += `<div class="ast-collapse" onclick="this.classList.toggle('collapsed')"><span class="ast-kind">${node.kind}</span>`;
    if (node.value != null) html += ` <span class="ast-val">${JSON.stringify(node.value)}</span>`;
    html += '</div>';
    
    if (node.children || node.algorithms || node.body || node.parameters) {
      html += '<div class="ast-node">';
      const children = node.children || node.algorithms || node.body || node.parameters || [];
      if (Array.isArray(children)) {
        children.forEach(c => { html += renderAstNode(c, depth + 1); });
      }
      // Handle other keys
      Object.keys(node).forEach(k => {
        if (['kind', 'value', 'children', 'algorithms', 'body', 'parameters', 'type', 'initializer', 'name', 'complexity', 'bindings', 'returnType'].includes(k)) return;
        const v = node[k];
        if (typeof v === 'object' && v !== null) {
          html += `<div><span class="ast-key">${k}:</span>`;
          html += renderAstNode(v, depth + 1);
          html += '</div>';
        } else if (v != null) {
          html += `<div><span class="ast-key">${k}:</span> <span class="ast-val">${JSON.stringify(v)}</span></div>`;
        }
      });
      html += '</div>';
    }
  } else if (typeof node === 'object') {
    Object.keys(node).forEach(k => {
      const v = node[k];
      if (typeof v === 'object' && v !== null) {
        html += `<div><span class="ast-key">${k}:</span><div class="ast-node">${renderAstNode(v, depth + 1)}</div></div>`;
      } else {
        html += `<div><span class="ast-key">${k}:</span> <span class="ast-val">${JSON.stringify(v)}</span></div>`;
      }
    });
  }
  return html;
}

function toggleHybrid() {
  const checked = document.getElementById('hybrid-mode').checked;
  document.querySelector('.toggle-label').textContent = checked ? 'Remote' : 'Local';
}

// Advanced domain examples
const advancedExamples = [
  { name: 'Shor\'s Algorithm', complexity: 'Quantum', code: `Algorithm ShorsFactoring(N)
    Require: N: Integer
    Ensure: Integer
    Complexity: "O((log N)^3)"

    # Classical pre-processing
    if N mod 2 == 0 then
        return 2
    end if

    # Quantum period-finding (simulated)
    a <- random(2, N - 2)
    g <- gcd(a, N)
    if g > 1 then
        return g
    end if

    # Measure qubit to find period
    measure qubit
    return qubit` },
  { name: 'Grover\'s Search', complexity: 'O(sqrt N)', code: `Algorithm GroversSearch(database, target)
    Require: database: List, target: Integer
    Ensure: Integer
    Complexity: "O(sqrt N)"

    n <- database.length
    iterations <- sqrt(n)

    # Oracle marking (simulated)
    marked <- {}
    i <- 0
    while i < n do
        if database[i] == target then
            marked.add(i)
        end if
        i <- i + 1
    end while

    # Amplitude amplification
    measure qubit
    if qubit == 1 then
        return marked[0]
    end if
    return -1` },
  { name: 'Matrix Multiply (Tensor)', complexity: 'O(N^3)', code: `Algorithm TensorMatMul(A, B)
    Require: A: Tensor, B: Tensor
    Ensure: Tensor
    Complexity: "O(N^3)"

    tensor A Real 2
    tensor B Real 2

    R <- A.dim[0]
    C <- B.dim[1]
    result <- zeroTensor(R, C)

    i <- 0
    while i < R do
        j <- 0
        while j < C do
            sum <- 0.0
            k <- 0
            while k < A.dim[1] do
                sum <- sum + A[i][k] * B[k][j]
                k <- k + 1
            end while
            result[i][j] <- sum
            j <- j + 1
        end while
        i <- i + 1
    end while

    return result` },
  { name: 'Parallel Sum (Work-Span)', complexity: 'O(N/P + log P)', code: `Algorithm ParallelSum(data)
    Require: data: List
    Ensure: Integer
    Complexity: "O(N / P + log P)", Work = "O(N)", Span = "O(log N)"

    n <- data.length
    parallel for each chunk in data do
        size <- chunk.length
        partial <- 0
        i <- 0
        while i < size do
            partial <- partial + chunk[i]
            i <- i + 1
        end while
    end for

    sync
    return partial` }
];

// Render advanced examples in sidebar
const advList = document.getElementById('advanced-examples');
advancedExamples.forEach(function(ex, i) {
  const div = document.createElement('div');
  div.className = 'example-item';
  div.innerHTML = '<div class="name">' + ex.name + '</div><div class="meta">' + ex.complexity + '</div>';
  div.onclick = function() {
    editor.setValue(ex.code);
    document.querySelectorAll('.example-item').forEach(el => el.classList.remove('active'));
    div.classList.add('active');
  };
  advList.appendChild(div);
});

window.runExecute = runExecute;
window.toggleHybrid = toggleHybrid;

