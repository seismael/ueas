// UEAS Playground — client-side interactive algorithm viewer
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

// Transpile simulation (static demo — full WASM version runs kernel)
const transpileSimulations = {
  python: function(code) {
    const lines = [];
    const name = extractName(code);
    const params = extractParams(code);
    lines.push('import math');
    lines.push('');
    lines.push(`def ${name}(${params.join(', ')}):`);
    lines.push('    # UEAS Algorithm — transpiled by ueas playground');
    lines.push(`    # Complexity: ${extractComplexity(code)}`);
    lines.push('    pass  # (kernel execution available via ueas CLI)');
    return lines.join('\n');
  },
  rust: function(code) {
    const name = extractName(code);
    const params = extractParams(code);
    const lines = [];
    lines.push(`fn ${name}(${params.map(p => p + ': i64').join(', ')}) -> i64 {`);
    lines.push('    // UEAS Algorithm — transpiled by ueas playground');
    lines.push(`    // Complexity: ${extractComplexity(code)}`);
    lines.push('    unimplemented!()  // (kernel execution available via ueas CLI)');
    lines.push('}');
    return lines.join('\n');
  },
  cpp: function(code) {
    const name = extractName(code);
    const params = extractParams(code);
    const lines = [];
    lines.push('#include <cstdint>');
    lines.push('');
    lines.push(`int64_t ${name}(${params.map(p => 'int64_t ' + p).join(', ')}) {`);
    lines.push('    // UEAS Algorithm — transpiled by ueas playground');
    lines.push(`    // Complexity: ${extractComplexity(code)}`);
    lines.push('    return 0;  // (kernel execution available via ueas CLI)');
    lines.push('}');
    return lines.join('\n');
  },
  java: function(code) {
    const name = extractName(code);
    const params = extractParams(code);
    const lines = [];
    lines.push('import java.util.*;');
    lines.push('');
    lines.push(`public static long ${name}(${params.map(p => 'long ' + p).join(', ')}) {`);
    lines.push('    // UEAS Algorithm — transpiled by ueas playground');
    lines.push(`    // Complexity: ${extractComplexity(code)}`);
    lines.push('    return 0L;  // (kernel execution available via ueas CLI)');
    lines.push('}');
    return lines.join('\n');
  },
  javascript: function(code) {
    const name = extractName(code);
    const params = extractParams(code);
    const lines = [];
    lines.push(`function ${name}(${params.join(', ')}) {`);
    lines.push('    // UEAS Algorithm — transpiled by ueas playground');
    lines.push(`    // Complexity: ${extractComplexity(code)}`);
    lines.push('    return 0;  // (kernel execution available via ueas CLI)');
    lines.push('}');
    return lines.join('\n');
  },
  lean4: function(code) {
    const name = extractName(code);
    const params = extractParams(code);
    const lines = [];
    lines.push('/- Algorithm: ' + name);
    lines.push('   Complexity: ' + extractComplexity(code) + ' -/');
    lines.push('');
    lines.push(`def ${name} (${params.map(p => p + ' : ℕ').join(') (')}) : ℕ :=`);
    lines.push('  -- (full Lean 4 theorem generation via ueas CLI --target lean4)');
    if (params.length > 0) {
      lines.push('  ' + params[0]);
    } else {
      lines.push('  0');
    }
    return lines.join('\n');
  },
  tlaplus: function(code) {
    const name = extractName(code);
    const params = extractParams(code);
    const lines = [];
    lines.push('---- MODULE ' + name + ' ----');
    lines.push('EXTENDS Naturals, Sequences');
    lines.push('');
    lines.push(`\\* Algorithm: ${name}`);
    lines.push(`\\* Complexity: ${extractComplexity(code)}`);
    lines.push('');
    lines.push('VARIABLES ' + (params.length ? params.join(', ') + ', result' : 'result'));
    lines.push('');
    lines.push('Init ==');
    lines.push('    /\\ result = 0');
    params.forEach(p => lines.push('    /\\ ' + p + ' = 0'));
    lines.push('');
    lines.push('Next ==');
    lines.push('    /\\ TRUE  -- (full TLA+ spec via ueas CLI --target tlaplus)');
    lines.push('    /\\ UNCHANGED <<' + (params.length ? params.join(', ') + ', result' : 'result') + '>>');
    lines.push('');
    lines.push('====');
    return lines.join('\n');
  }
};

function extractName(code) {
  const m = code.match(/Algorithm\s+(\w+)/);
  return m ? m[1] : 'unnamed';
}

function extractParams(code) {
  const m = code.match(/Algorithm\s+\w+\(([^)]*)\)/);
  return m ? m[1].split(',').map(s => s.trim()).filter(s => s) : [];
}

function extractComplexity(code) {
  const m = code.match(/Complexity:\s*"([^"]+)"/);
  return m ? m[1] : 'O(?)';
}

function simulateTranspile() {
  const code = editor.getValue();
  const target = document.getElementById('target-select').value;
  const fn = transpileSimulations[target];
  const output = fn ? fn(code) : 'Target not supported in static playground.';
  document.getElementById('output-transpiled').textContent = output;
  switchTab('transpiled');
}

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

function copyToClipboard() {
  navigator.clipboard.writeText(editor.getValue()).then(function() {
    showToast('Copied to clipboard');
  });
}

function switchTab(tab) {
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.output').forEach(o => o.classList.remove('active'));
  document.querySelector('[onclick="switchTab(\'' + tab + '\')"]').classList.add('active');
  document.getElementById('output-' + tab).classList.add('active');
}

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

// Initialize
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
