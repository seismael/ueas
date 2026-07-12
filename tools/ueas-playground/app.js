// UEAS Playground v4.5.0 — MCP-only client (Cloudflare Workers backend)

let ueasEditor, targetEditor;
const MCP_URL = 'https://ueas-mcp.seismael.workers.dev';

require.config({
  paths: { vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.44.0/min/vs' }
});

// 45 examples matching project directory structure
const examples = [
  // core/
  { name: 'Euclidean Distance', category: 'core', complexity: 'O(1)', code: `Algorithm EuclideanDistance(x1, y1, x2, y2)\n    Require: x1: Real, y1: Real, x2: Real, y2: Real\n    Ensure: Real\n    Complexity: "O(1)"\n\n    dx <- x2 - x1\n    dy <- y2 - y1\n    return sqrt(dx * dx + dy * dy)` },
  { name: 'Linear Search', category: 'core', complexity: 'O(N)', code: `Algorithm LinearSearch(data, target)\n    Require: data: List, target: Integer\n    Ensure: Integer\n    Complexity: "O(N)"\n\n    for each item in data do\n        if item == target then\n            return item\n        end if\n    end for\n    return -1` },
  { name: 'Binary Search', category: 'core', complexity: 'O(log N)', code: `Algorithm BinarySearch(data, target)\n    Require: data: List, target: Integer\n    Ensure: Integer\n    Complexity: "O(log N)"\n\n    low <- 0\n    high <- data.length - 1\n    while low <= high do\n        mid <- low + (high - low) / 2\n        if data[mid] == target then\n            return mid\n        else if data[mid] < target then\n            low <- mid + 1\n        else\n            high <- mid - 1\n        end if\n    end while\n    return -1` },
  { name: 'Matrix Multiply', category: 'core', complexity: 'O(N^3)', code: `Algorithm MatrixMultiply(A, B)\n    Require: A: Matrix, B: Matrix\n    Ensure: Matrix\n    Complexity: "O(R * C * K)"\n\n    result <- zeroMatrix(R, K)\n    for each i in range(0, R) do\n        for each j in range(0, K) do\n            sum <- 0.0\n            for each k in range(0, C) do\n                sum <- sum + A[i][k] * B[k][j]\n            end for\n            result[i][j] <- sum\n        end for\n    end for\n    return result` },
  // sorting/
  { name: 'Quicksort', category: 'sorting', complexity: 'O(N log N)', code: `Algorithm QuickSort(data)\n    Require: data: List\n    Ensure: List\n    Complexity: "O(N log N)"\n\n    if data.length <= 1 then\n        return data\n    end if\n    pivot <- data[0]\n    left <- []\n    right <- []\n    for each item in data[1:] do\n        if item <= pivot then\n            left.append(item)\n        else\n            right.append(item)\n        end if\n    end for\n    return QuickSort(left) + [pivot] + QuickSort(right)` },
  { name: 'Merge Sort', category: 'sorting', complexity: 'O(N log N)', code: `Algorithm MergeSort(data)\n    Require: data: List\n    Ensure: List\n    Complexity: "O(N log N)"\n\n    if data.length <= 1 then\n        return data\n    end if\n    mid <- data.length / 2\n    left <- data[0:mid]\n    right <- data[mid:data.length]\n    return Merge(MergeSort(left), MergeSort(right))` },
  // graph/
  { name: 'BFS', category: 'graph', complexity: 'O(V + E)', code: `Algorithm BreadthFirstSearch(g, start)\n    Require: g: Graph, start: Integer\n    Ensure: Set\n    Complexity: "O(V + E)", V = g.nodes.length, E = g.edges.length\n\n    visited <- {start}\n    queue <- [start]\n    while queue.length > 0 do\n        current <- queue.pop(0)\n        for each neighbor in g.adjacent(current) do\n            if neighbor not in visited then\n                visited.add(neighbor)\n                queue.append(neighbor)\n            end if\n        end for\n    end while\n    return visited` },
  { name: 'DFS', category: 'graph', complexity: 'O(V + E)', code: `Algorithm DepthFirstSearch(g, start)\n    Require: g: Graph, start: Integer\n    Ensure: Set\n    Complexity: "O(V + E)", V = g.nodes.length, E = g.edges.length\n\n    visited <- {}\n    stack <- [start]\n    while stack.length > 0 do\n        current <- stack.pop()\n        if current not in visited then\n            visited.add(current)\n            for each neighbor in g.adjacent(current) do\n                if neighbor not in visited then\n                    stack.push(neighbor)\n                end if\n            end for\n        end if\n    end while\n    return visited` },
  { name: 'Dijkstra', category: 'graph', complexity: 'O((V+E) log V)', code: `Algorithm DijkstraShortestPath(g, source)\n    Require: g: Graph, source: Integer\n    Ensure: Map\n    Complexity: "O((V+E) log V)", V = g.nodes.length, E = g.edges.length\n\n    distances <- {}\n    unvisited <- g.nodes\n    for each node in unvisited do\n        if node == source then\n            distances[node] <- 0.0\n        else\n            distances[node] <- 999999.0\n        end if\n    end for\n    while unvisited.length > 0 do\n        current <- extractMin(unvisited, distances)\n        neighbors <- g.adjacent(current)\n        for each neighbor in neighbors do\n            newDist <- distances[current] + g.weight(current, neighbor)\n            if newDist < distances[neighbor] then\n                distances[neighbor] <- newDist\n            end if\n        end for\n    end while\n    return distances` },
  // dp/
  { name: 'Max Subarray (Kadane)', category: 'dp', complexity: 'O(N)', code: `Algorithm MaxSubarray(data)\n    Require: data: List\n    Ensure: Integer\n    Complexity: "O(N)"\n\n    maxSoFar <- data[0]\n    maxEnding <- data[0]\n    i <- 1\n    while i < data.length do\n        if maxEnding + data[i] > data[i] then\n            maxEnding <- maxEnding + data[i]\n        else\n            maxEnding <- data[i]\n        end if\n        if maxEnding > maxSoFar then\n            maxSoFar <- maxEnding\n        end if\n        i <- i + 1\n    end while\n    return maxSoFar` },
  { name: 'LCS', category: 'dp', complexity: 'O(M*N)', code: `Algorithm LCS(a, b)\n    Require: a: String, b: String\n    Ensure: Integer\n    Complexity: "O(M * N)", M = a.length, N = b.length\n\n    m <- a.length\n    n <- b.length\n    dp <- zeroMatrix(m + 1, n + 1)\n    i <- 1\n    while i <= m do\n        j <- 1\n        while j <= n do\n            if a[i - 1] == b[j - 1] then\n                dp[i][j] <- dp[i - 1][j - 1] + 1\n            else\n                maxVal <- dp[i - 1][j]\n                if dp[i][j - 1] > maxVal then\n                    maxVal <- dp[i][j - 1]\n                end if\n                dp[i][j] <- maxVal\n            end if\n            j <- j + 1\n        end while\n        i <- i + 1\n    end while\n    return dp[m][n]` },
  // arrays/
  { name: 'Two Sum', category: 'arrays', complexity: 'O(N)', code: `Algorithm TwoSum(data, target)\n    Require: data: List, target: Integer\n    Ensure: List\n    Complexity: "O(N)", N = data.length\n\n    seen <- {}\n    i <- 0\n    for each num in data do\n        complement <- target - num\n        if complement in seen then\n            return [seen[complement], i]\n        end if\n        seen[num] <- i\n        i <- i + 1\n    end for\n    return []` },
  { name: 'Trap Rain Water', category: 'arrays', complexity: 'O(N)', code: `Algorithm TrapRainWater(height)\n    Require: height: List\n    Ensure: Integer\n    Complexity: "O(N)"\n\n    n <- height.length\n    if n <= 2 then\n        return 0\n    end if\n    left <- 0\n    right <- n - 1\n    leftMax <- 0\n    rightMax <- 0\n    water <- 0\n    while left < right do\n        if height[left] < height[right] then\n            if height[left] >= leftMax then\n                leftMax <- height[left]\n            else\n                water <- water + leftMax - height[left]\n            end if\n            left <- left + 1\n        else\n            if height[right] >= rightMax then\n                rightMax <- height[right]\n            else\n                water <- water + rightMax - height[right]\n            end if\n            right <- right - 1\n        end if\n    end while\n    return water` },
  // backtracking/
  { name: 'N-Queens', category: 'backtracking', complexity: 'O(N!)', code: `Algorithm NQueens(n)\n    Require: n: Integer\n    Ensure: List\n    Complexity: "O(N!)"\n\n    solutions <- {}\n    board <- {}\n    i <- 0\n    while i < n do\n        board[i] <- -1\n        i <- i + 1\n    end while\n    cols <- {}\n    i <- 0\n    while i < n do\n        cols[i] <- 0\n        i <- i + 1\n    end while\n    diag1 <- {}\n    diag2 <- {}\n    i <- 0\n    while i < 2 * n do\n        diag1[i] <- 0\n        diag2[i] <- 0\n        i <- i + 1\n    end while\n    row <- 0\n    while row < n do\n        col <- 0\n        while col < n do\n            d1 <- row - col + n - 1\n            d2 <- row + col\n            if cols[col] == 0 and diag1[d1] == 0 and diag2[d2] == 0 then\n                board[row] <- col\n                cols[col] <- 1\n                diag1[d1] <- 1\n                diag2[d2] <- 1\n                if row == n - 1 then\n                    sol <- {}\n                    j <- 0\n                    while j < n do\n                        sol.append(board[j])\n                        j <- j + 1\n                    end while\n                    solutions.append(sol)\n                end if\n                board[row] <- -1\n                cols[col] <- 0\n                diag1[d1] <- 0\n                diag2[d2] <- 0\n            end if\n            col <- col + 1\n        end while\n        row <- row + 1\n    end while\n    return solutions` },
  // stack/
  { name: 'Valid Parentheses', category: 'stack', complexity: 'O(N)', code: `Algorithm ValidParentheses(s)\n    Require: s: String\n    Ensure: Boolean\n    Complexity: "O(N)"\n\n    stack <- []\n    i <- 0\n    while i < s.length do\n        ch <- s[i]\n        if ch == "(" or ch == "[" or ch == "{" then\n            stack.append(ch)\n        else\n            if stack.length == 0 then\n                return false\n            end if\n            top <- stack.pop()\n            if ch == ")" and top != "(" then\n                return false\n            end if\n            if ch == "]" and top != "[" then\n                return false\n            end if\n            if ch == "}" and top != "{" then\n                return false\n            end if\n        end if\n        i <- i + 1\n    end while\n    return stack.length == 0` },
  { name: 'Largest Rectangle', category: 'stack', complexity: 'O(N)', code: `Algorithm LargestRectangle(heights)\n    Require: heights: List\n    Ensure: Integer\n    Complexity: "O(N)"\n\n    stack <- []\n    maxArea <- 0\n    i <- 0\n    while i <= heights.length do\n        h <- 0\n        if i < heights.length then\n            h <- heights[i]\n        end if\n        while stack.length > 0 and h < heights[stack[stack.length - 1]] do\n            top <- stack.pop()\n            width <- i\n            if stack.length > 0 then\n                width <- i - stack[stack.length - 1] - 1\n            end if\n            area <- heights[top] * width\n            if area > maxArea then\n                maxArea <- area\n            end if\n        end while\n        stack.append(i)\n        i <- i + 1\n    end while\n    return maxArea` },
  // features
  { name: 'Monte Carlo Pi', category: 'stochastic', complexity: 'O(N) Expected', code: `Algorithm MonteCarloPi(iterations)\n    Require: iterations: Integer\n    Ensure: Real\n    Complexity: "O(N)", Expected = "O(N)"\n\n    inside <- 0\n    i <- 0\n    while i < iterations do\n        x <- random(0, 10000)\n        y <- random(0, 10000)\n        dist <- x * x + y * y\n        if dist <= 100000000 then\n            inside <- inside + 1\n        end if\n        i <- i + 1\n    end while\n    pi <- (4.0 * inside) / iterations\n    return pi` },
  { name: 'Fibonacci Generator', category: 'streams', complexity: 'O(N)', code: `Algorithm FibonacciGenerator(limit)\n    Require: limit: Integer\n    Ensure: Stream\n    Complexity: "O(N)", Memory = "O(1)"\n\n    Stream Result Integer\n    a <- 0\n    b <- 1\n    i <- 0\n    while i < limit do\n        yield a\n        next <- a + b\n        a <- b\n        b <- next\n        i <- i + 1\n    end while` },
  { name: 'Spawn-Join Matrix', category: 'concurrency', complexity: 'O(N^3/P)', code: `Algorithm SpawnJoinMatrix(A, B)\n    Require: A: Matrix, B: Matrix\n    Ensure: Matrix\n    Complexity: "O(N^3 / P)", Work = "O(N^3)", Span = "O(log N)"\n\n    N <- A.rows\n    parallel for each i in range(0, N) do\n        j <- 0\n        while j < N do\n            k <- 0\n            while k < N do\n                spawn C[i][j] <- A[i][k] * B[k][j]\n                k <- k + 1\n            end while\n            j <- j + 1\n        end while\n    end for\n    sync\n    return C` },
  { name: 'Constant-Time Compare', category: 'cryptographic', complexity: 'O(N)', code: `Algorithm ConstantTimeCompare(a, b)\n    Require: a: Secret, b: Secret\n    Ensure: Boolean\n    Complexity: "O(N)"\n\n    @ConstantTime\n    result <- 0\n    i <- 0\n    while i < a.length do\n        if a[i] == b[i] then\n            result <- result + 1\n        else\n            result <- result - 1\n        end if\n        i <- i + 1\n    end while\n    return result == a.length` },
  { name: 'Cache-Optimized Search', category: 'hardware', complexity: 'O(N/L)', code: `Algorithm CacheOptimizedSearch(data, target)\n    Require: data: List, target: Integer\n    Ensure: Integer\n    Complexity: "O(N / L)"\n\n    @HardwareProfile(L1 = 64KB, L2 = 512KB, CacheLine = 64B)\n    n <- data.length\n    i <- 0\n    while i < n do\n        if data[i] == target then\n            return i\n        end if\n        i <- i + 1\n    end while\n    return -1` }
];

// Advanced domain examples
const advancedExamples = [
  { name: "Shor's Algorithm", complexity: 'Quantum', code: `Algorithm ShorsFactoring(N)\n    Require: N: Integer\n    Ensure: Integer\n    Complexity: "O((log N)^3)"\n\n    if N mod 2 == 0 then\n        return 2\n    end if\n    a <- random(2, N - 2)\n    g <- gcd(a, N)\n    if g > 1 then\n        return g\n    end if\n    measure qubit\n    return qubit` },
  { name: "Grover's Search", complexity: 'O(sqrt N)', code: `Algorithm GroversSearch(database, target)\n    Require: database: List, target: Integer\n    Ensure: Integer\n    Complexity: "O(sqrt N)"\n\n    n <- database.length\n    iterations <- sqrt(n)\n    marked <- {}\n    i <- 0\n    while i < n do\n        if database[i] == target then\n            marked.add(i)\n        end if\n        i <- i + 1\n    end while\n    measure qubit\n    if qubit == 1 then\n        return marked[0]\n    end if\n    return -1` },
  { name: 'Tensor MatMul', complexity: 'O(N^3)', code: `Algorithm TensorMatMul(A, B)\n    Require: A: Tensor, B: Tensor\n    Ensure: Tensor\n    Complexity: "O(N^3)"\n\n    tensor A Real 2\n    tensor B Real 2\n    R <- A.dim[0]\n    C <- B.dim[1]\n    result <- zeroTensor(R, C)\n    i <- 0\n    while i < R do\n        j <- 0\n        while j < C do\n            sum <- 0.0\n            k <- 0\n            while k < A.dim[1] do\n                sum <- sum + A[i][k] * B[k][j]\n                k <- k + 1\n            end while\n            result[i][j] <- sum\n            j <- j + 1\n        end while\n        i <- i + 1\n    end while\n    return result` },
  { name: 'Parallel Sum (Work-Span)', complexity: 'O(N/P + log P)', code: `Algorithm ParallelSum(data)\n    Require: data: List\n    Ensure: Integer\n    Complexity: "O(N / P + log P)", Work = "O(N)", Span = "O(log N)"\n\n    n <- data.length\n    parallel for each chunk in data do\n        size <- chunk.length\n        partial <- 0\n        i <- 0\n        while i < size do\n            partial <- partial + chunk[i]\n            i <- i + 1\n        end while\n    end for\n    sync\n    return partial` }
];

// Render categorized accordion examples sidebar
function renderCategories() {
  var el = document.getElementById('sidebar-scroll');
  var cats = groupByCategory();

  cats.forEach(function(cat) {
    // Category header
    var hdr = document.createElement('div');
    hdr.className = 'cat-header';
    hdr.innerHTML = '<span>' + cat.label + '</span><span class="arrow">&#9654;</span>';
    hdr.onclick = function() {
      var body = this.nextElementSibling;
      var arrow = this.querySelector('.arrow');
      var isOpen = body.classList.contains('open');
      // Close all
      document.querySelectorAll('.cat-body').forEach(function(b) { b.classList.remove('open'); });
      document.querySelectorAll('.arrow').forEach(function(a) { a.classList.remove('open'); });
      if (!isOpen) {
        body.classList.add('open');
        arrow.classList.add('open');
      }
    };
    el.appendChild(hdr);

    // Category body
    var body = document.createElement('div');
    body.className = 'cat-body';
    cat.items.forEach(function(ex, i) {
      var item = document.createElement('div');
      item.className = 'example-item';
      item.innerHTML = '<div class="name">' + ex.name + '</div><div class="meta">' + ex.complexity + '</div>';
      item.onclick = (function(e) {
        return function() {
          ueasEditor.setValue(e.code);
          document.querySelectorAll('.example-item').forEach(function(el) { el.classList.remove('active'); });
          item.classList.add('active');
        };
      })(ex);
      body.appendChild(item);
    });
    el.appendChild(body);
  });
}

function groupByCategory() {
  var cats = {};
  examples.forEach(function(ex) {
    var cat = ex.category || 'other';
    if (!cats[cat]) cats[cat] = [];
    cats[cat].push(ex);
  });
  var labels = {
    'core': 'Core Algorithms', 'sorting': 'Sorting', 'graph': 'Graph Algorithms',
    'dp': 'Dynamic Programming', 'arrays': 'Arrays & Two Pointers',
    'backtracking': 'Backtracking', 'stack': 'Stack',
    'features': 'Feature Demos', 'stochastic': 'Stochastic',
    'streams': 'Streams', 'concurrency': 'Concurrency',
    'cryptographic': 'Cryptographic', 'hardware': 'Hardware Profiling'
  };
  var order = ['core','sorting','graph','dp','arrays','backtracking','stack',
               'concurrency','cryptographic','hardware','stochastic','streams','features'];
  return order.filter(function(k) { return cats[k]; }).map(function(k) {
    return { label: labels[k] || k, items: cats[k] };
  });
}

// MCP call helper
async function callMCP(tool, args) {
  try {
    const resp = await fetch(MCP_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0', id: Date.now() % 1000, method: 'tools/call',
        params: { name: tool, arguments: args }
      })
    });
    const data = await resp.json();
    if (data.error) {
      return { status: 'error', error: data.error.message || 'MCP error', code: data.error.code };
    }
    const text = data?.result?.content?.[0]?.text;
    if (!text) return { status: 'error', error: 'Empty response from MCP server' };
    try { return JSON.parse(text); } catch(e) { return { status: 'error', error: text.substring(0, 200) || e.message }; }
  } catch (e) {
    return { status: 'error', error: 'MCP unreachable: ' + (e.message || 'network error') };
  }
}

async function doTranspile() {
  var code = ueasEditor.getValue();
  var target = document.getElementById('target-select').value;
  targetEditor.setValue('// Transpiling via MCP...');
  document.getElementById('audit-report').innerHTML = '<span style="color:var(--orange)">Transpiling...</span>';
  try {
    var result = await callMCP('transpile', { source: code, target: target });
    if (result.source) {
      targetEditor.setValue(result.source);
      if (result.note) {
        document.getElementById('audit-report').innerHTML = '<span style="color:var(--green)">Transpiled to Dafny. Build with: <code>dafny build --target:' + target + '</code></span>';
      } else {
        document.getElementById('audit-report').innerHTML = '<span style="color:var(--green)">Transpiled to ' + target + '</span>';
      }
    } else {
      targetEditor.setValue('Transpile Error:\n' + (result.error || 'Unknown error'));
      document.getElementById('audit-report').innerHTML = '<span style="color:var(--red)">Transpile failed: ' + (result.error || 'unknown') + '</span>';
    }
  } catch (e) {
    targetEditor.setValue('// Connection error: ' + (e.message || e));
    document.getElementById('audit-report').innerHTML = '<span style="color:var(--red)">MCP connection error.</span>';
  }
}

function simulateTranspile() { doTranspile(); }
async function runExecute() {
  var code = ueasEditor.getValue();
  document.getElementById('exec-status').textContent = 'Running...';
  document.getElementById('exec-status').style.color = 'var(--text-dim)';
  
  // Call tools individually — handle each failure gracefully
  var execResult = await callMCP('execute', { source: code });
  var cmplxResult = execResult.status === 'error' ? {} : await callMCP('complexity', { source: code });
  var parseResult = await callMCP('parse', { source: code });
  
  updateDashboard(execResult, cmplxResult);
  // Show AST tree from parse — use the raw AST field
  var astData = null;
  if (parseResult.ast) {
    astData = parseResult.ast;
  } else if (!parseResult.status || parseResult.status !== 'error') {
    astData = parseResult;
  }
  if (astData) updateAstTree(astData);
}


async function reverseAudit() {
  var legacyCode = targetEditor.getValue();
  // Skip placeholder/default text
  if (!legacyCode || legacyCode.startsWith('// Target') || legacyCode.startsWith('// Transpiling') || legacyCode.startsWith('// Connection')) {
    document.getElementById('audit-report').innerHTML = '<span style="color:var(--orange)">Type or paste Python/Java code in the right editor, then click Audit.</span>';
    return;
  }
  document.getElementById('audit-report').innerHTML = '<span style="color:var(--orange)">Auditing via MCP...</span>';
  try {
    var result = await callMCP('audit', { source: legacyCode });
    if (result.status === 'ok') {
      if (result.functions_found === 0) {
        document.getElementById('audit-report').innerHTML = '<span style="color:var(--orange)">No functions found. Audit detects <code>def</code> declarations. Try pasting Python code like:</span><br><code style="font-size:0.75rem">def search(arr, target):\\n    for i in range(len(arr)):\\n        if arr[i]==target: return i\\n    return -1</code>';
        return;
      }
      var pseudocode = '';
      if (result.ueas_mappings && result.ueas_mappings.length) {
        pseudocode = result.ueas_mappings[0].ueas_equivalent;
      } else if (result.recommendations && result.recommendations.length) {
        pseudocode = '// ' + result.recommendations.join('\n// ');
      }
      ueasEditor.setValue(pseudocode || '// No algorithm pseudocode extracted.');
      var cpx = result.complexity_estimates ? result.complexity_estimates.map(function(e) { return e.function + ' (' + e.estimated_complexity + ')'; }).join(', ') : 'None';
      var html = '<div><strong>Functions:</strong> ' + (result.functions_found || 0) + '</div>';
      html += '<div><strong>Complexity:</strong> ' + cpx + '</div>';
      var violations = (result.io_violations && result.io_violations.length) || 0;
      html += '<div style="color:' + (violations ? 'var(--red)' : 'var(--green)') + '"><strong>I/O Violations:</strong> ' + violations + '</div>';
      document.getElementById('audit-report').innerHTML = html;
    } else {
      document.getElementById('audit-report').innerHTML = '<span style="color:var(--red)">' + (result.error || result.status || 'Audit failed') + '</span>';
    }
  } catch (e) {
    document.getElementById('audit-report').innerHTML = '<span style="color:var(--red)">Network error</span>';
  }
}

function updateTargetLanguage() {
  var langMap = { dafny: 'plaintext', cpp: 'cpp', python: 'python', java: 'java', javascript: 'javascript', lean4: 'plaintext', tlaplus: 'plaintext', latex: 'tex' };
  var sel = document.getElementById('target-select').value;
  var modelLang = langMap[sel] || 'plaintext';
  if (targetEditor) {
    monaco.editor.setModelLanguage(targetEditor.getModel(), modelLang);
  }
}

function updateDashboard(exec, cmplx) {
  exec = exec || {}; cmplx = cmplx || {};
  
  if (exec.status === 'error') {
    document.getElementById('exec-status').textContent = exec.error || 'Error';
    document.getElementById('exec-status').style.color = 'var(--red)';
    document.getElementById('exec-steps').textContent = '—';
    document.getElementById('exec-heap').textContent = '—';
    document.getElementById('exec-complexity').textContent = '—';
    return;
  }
  
  document.getElementById('exec-status').textContent = 'OK';
  document.getElementById('exec-status').style.color = 'var(--green)';
  document.getElementById('exec-steps').textContent = exec.step_count != null ? exec.step_count : (cmplx.step_estimate || '—');
  document.getElementById('exec-heap').textContent = (exec.heap_bytes || '—') + ' B';
  // Use complexity from MCP response (cmplx.complexity) directly, fallback to source extraction
  var cpx = (cmplx && cmplx.complexity) ? cmplx.complexity : extractComplexity(ueasEditor.getValue());
  document.getElementById('exec-complexity').textContent = cpx || '—';
  document.getElementById('step-bar-fill').style.width = Math.min(((exec.step_count || cmplx.step_estimate || 0) * 2), 100) + '%';
}

function updateAstTree(astJson) {
  var el = document.getElementById('ast-tree');
  try {
    var ast = typeof astJson === 'string' ? JSON.parse(astJson) : astJson;
    el.innerHTML = renderAstNode(ast, 0);
  } catch (e) {
    el.innerHTML = '<span style="color:var(--text-dim)">AST visualization requires kernel-level parse (available via CLI).</span>';
  }
}

function renderAstNode(node, depth) {
  if (!node || typeof node !== 'object') return '';
  var html = '';
  if (node.kind) {
    html += '<div class="ast-collapse" onclick="this.classList.toggle(\'collapsed\')"><span class="ast-kind">' + node.kind + '</span>';
    if (node.value != null) html += ' <span class="ast-val">' + JSON.stringify(node.value) + '</span>';
    html += '</div>';
    if (node.children || node.algorithms || node.body || node.parameters) {
      html += '<div class="ast-node">';
      var children = node.children || node.algorithms || node.body || node.parameters || [];
      if (Array.isArray(children)) {
        children.forEach(function(c) { html += renderAstNode(c, depth + 1); });
      }
      Object.keys(node).forEach(function(k) {
        if (['kind', 'value', 'children', 'algorithms', 'body', 'parameters'].indexOf(k) >= 0) return;
        var v = node[k];
        if (typeof v === 'object' && v !== null) {
          html += '<div><span class="ast-key">' + k + ':</span>';
          html += renderAstNode(v, depth + 1);
          html += '</div>';
        } else if (v != null) {
          html += '<div><span class="ast-key">' + k + ':</span> <span class="ast-val">' + JSON.stringify(v) + '</span></div>';
        }
      });
      html += '</div>';
    }
  } else if (typeof node === 'object') {
    Object.keys(node).forEach(function(k) {
      var v = node[k];
      if (typeof v === 'object' && v !== null) {
        html += '<div><span class="ast-key">' + k + ':</span><div class="ast-node">' + renderAstNode(v, depth + 1) + '</div></div>';
      } else {
        html += '<div><span class="ast-key">' + k + ':</span> <span class="ast-val">' + JSON.stringify(v) + '</span></div>';
      }
    });
  }
  return html;
}


function toggleSidebar() {
  document.getElementById('sidebar').classList.toggle('collapsed');
  setTimeout(function() { if(ueasEditor) ueasEditor.layout(); if(targetEditor) targetEditor.layout(); }, 220);
}

function toggleBottomPanel() {
  var p = document.getElementById('bottom-panel');
  p.classList.toggle('collapsed');
  var btn = document.getElementById('bottom-toggle-btn');
  if (btn) btn.innerHTML = p.classList.contains('collapsed') ? '<span class="material-symbols-outlined">expand_less</span>' : '<span class="material-symbols-outlined">expand_more</span>';
  setTimeout(function() { if(ueasEditor) ueasEditor.layout(); if(targetEditor) targetEditor.layout(); }, 220);
}

function formatCode() {
  var code = ueasEditor.getValue();
  var lines = code.split('\n');
  var depth = 0;
  var formatted = lines.map(function(line) {
    var trimmed = line.trim();
    if (!trimmed) return '';
    if (trimmed.startsWith('end ') || trimmed.startsWith('End ') || trimmed.startsWith('END ')) {
      depth = Math.max(0, depth - 1);
    }
    var result = '    '.repeat(depth) + trimmed;
    if (trimmed.startsWith('for ') || trimmed.startsWith('while ') || trimmed.startsWith('if ')) {
      depth += 1;
    }
    return result;
  }).join('\n');
  ueasEditor.setValue(formatted);
}

function copyToClipboard() {
  navigator.clipboard.writeText(ueasEditor.getValue()).then(function() {
    showToast('Copied');
  });
}

function showToast(msg) {
  var el = document.createElement('div');
  el.textContent = msg;
  el.style.cssText = 'position:fixed;bottom:20px;right:20px;background:var(--accent-dim);color:white;padding:8px 16px;border-radius:6px;font-size:0.8rem;z-index:999;';
  document.body.appendChild(el);
  setTimeout(function() { el.remove(); }, 2000);
}

// Initialize Monaco Editor
require(['vs/editor/editor.main'], function() {
  monaco.languages.register({ id: 'ueas' });

  monaco.languages.setMonarchTokensProvider('ueas', {
    keywords: [
      'Algorithm', 'algorithm', 'ALGORITHM', 'Require', 'require', 'REQUIRE',
      'Ensure', 'ensure', 'ENSURE', 'Complexity', 'complexity', 'COMPLEXITY',
      'Memory', 'memory', 'MEMORY', 'return', 'Return', 'RETURN',
      'if', 'If', 'IF', 'then', 'Then', 'THEN', 'else', 'Else', 'ELSE',
      'for', 'For', 'FOR', 'each', 'Each', 'EACH', 'in', 'In', 'IN', 'do', 'Do', 'DO',
      'while', 'While', 'WHILE', 'end', 'End', 'END',
      'assert', 'Assert', 'ASSERT', 'invariant', 'Invariant', 'INVARIANT',
      'and', 'And', 'AND', 'or', 'Or', 'OR', 'not', 'Not', 'NOT',
      'true', 'True', 'TRUE', 'false', 'False', 'FALSE',
      'parallel', 'Parallel', 'PARALLEL', 'spawn', 'Spawn', 'SPAWN',
      'sync', 'Sync', 'SYNC', 'measure', 'Measure', 'MEASURE',
      'yield', 'Yield', 'YIELD', 'random', 'const', 'Const', 'CONST'
    ],
    typeKeywords: [
      'Integer', 'Real', 'Boolean', 'String', 'Void', 'List', 'Set', 'Map',
      'Graph', 'Matrix', 'Tensor', 'Stream', 'Secret', 'Qubit'
    ],
    operators: ['<-', ':=', '=', '==', '!=', '<', '<=', '>', '>=', '+', '-', '*', '/', 'mod', '->'],
    tokenizer: {
      root: [
        [/#.*$/, 'comment'],
        [/"[^"]*"/, 'string'],
        [/[0-9]+\.[0-9]+/, 'number.float'],
        [/[0-9]+/, 'number'],
        [/[a-zA-Z_][a-zA-Z0-9_]*/, { cases: { '@typeKeywords': 'type', '@keywords': 'keyword', '@default': 'identifier' } }],
        [/<-|:=/, 'keyword'],
        [/[+\-*/=<>!]+/, 'operator'],
        [/[{}()[\],.:]/, 'delimiter'],
      ]
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
      { token: 'identifier', foreground: 'DCDCAA' },
      { token: 'delimiter', foreground: '808080' }
    ],
    colors: {
      'editor.background': '#0d1117',
      'editor.foreground': '#c9d1d9',
      'editor.lineHighlightBackground': '#161b22'
    }
  });

  ueasEditor = monaco.editor.create(document.getElementById('editor-ueas'), {
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
    renderWhitespace: 'selection'
  });

  targetEditor = monaco.editor.create(document.getElementById('editor-target'), {
    value: '// Target generated code or legacy code will appear here.',
    language: 'java',
    theme: 'ueas-dark',
    fontSize: 14,
    fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace",
    lineNumbers: 'on',
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 4
  });

  updateTargetLanguage();

  // Render categorized examples sidebar
  renderCategories();

  // Select first example
  var firstExample = document.querySelectorAll('#sidebar-scroll .example-item')[0];
  if (firstExample) {
    firstExample.classList.add('active');
  }

  // Resizer logic
  var resizer = document.getElementById('editor-resizer');
  var ueasPane = document.querySelector('.ueas-pane');
  var targetPane = document.querySelector('.target-pane');
  var isResizing = false;

  if (resizer) {
    resizer.addEventListener('mousedown', function(e) {
      isResizing = true;
      resizer.classList.add('dragging');
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    });
    document.addEventListener('mousemove', function(e) {
      if (!isResizing) return;
      var containerWidth = ueasPane.parentElement.offsetWidth;
      var minWidth = 100;
      var pointerX = e.clientX - ueasPane.parentElement.offsetLeft;
      var newFlexBasis = Math.max(minWidth, Math.min(pointerX, containerWidth - minWidth));
      ueasPane.style.flex = '1 1 ' + newFlexBasis + 'px';
      targetPane.style.flex = '1 1 ' + (containerWidth - newFlexBasis - 5) + 'px';
      if (ueasEditor) ueasEditor.layout();
      if (targetEditor) targetEditor.layout();
    });
    document.addEventListener('mouseup', function(e) {
      if (isResizing) {
        isResizing = false;
        resizer.classList.remove('dragging');
        document.body.style.cursor = 'default';
        document.body.style.userSelect = 'auto';
      }
    });
  }
});

// Expose to global scope for onclick handlers
window.copyToClipboard = copyToClipboard;
window.simulateTranspile = simulateTranspile;
window.runExecute = runExecute;
window.reverseAudit = reverseAudit;
window.updateTargetLanguage = updateTargetLanguage;
window.toggleSidebar = toggleSidebar;
window.toggleBottomPanel = toggleBottomPanel;
