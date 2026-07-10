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

initializeApp();
