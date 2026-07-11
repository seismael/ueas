# UEAS MCP — E2E LLM Test Prompts

Verification prompts for the bidirectional both-ways workflow (M36.3).

## Prompt 1: Basic Reverse Audit

```
Use the `audit_legacy` tool to analyze this Python code:

def binary_search(arr, target):
    left = 0
    right = len(arr) - 1
    while left <= right:
        mid = (left + right) // 2
        if arr[mid] == target:
            return mid
        elif arr[mid] < target:
            left = mid + 1
        else:
            right = mid - 1
    return -1

Expected:
- 1 function found
- Estimated complexity O(N) (contains while + condition)
- No I/O violations
- UEAS mapping should show Algorithm BinarySearch(arr, target) with Require/Ensure/Complexity preamble
```

## Prompt 2: I/O Violation Detection

```
Use `audit_legacy` to check this code for UEAS Axiom compliance:

def process_data(filename):
    import json
    data = open(filename).read()
    print("Processing...")
    result = json.loads(data)
    for item in result:
        if item > 100:
            return item
    return None

Expected:
- 3 I/O violations: import, open(), print()
- Status: axiom_violations_found
- Recommendations: Remove I/O calls, Replace print with return
```

## Prompt 3: Bidirectional Both-Ways Workflow

```
1. Use `audit_legacy` to reverse-audit this Java algorithm for a linear search:
   int search(int[] data, int target) {
       for (int i = 0; i < data.length; i++) {
           if (data[i] == target) return i;
       }
       return -1;
   }

2. The audit should identify: 1 function, O(N) complexity, for loop, if condition

3. Then use `transpile_ueas` to forward-transpile the UEAS equivalent back to Java
   and compare the output for semantic equivalence
```

## Prompt 4: Complexity Estimation Accuracy

```
Use `profile_complexity` to verify the complexity estimate from `audit_legacy`:

Run the following pipeline:
1. audit_legacy on: 
   def quicksort(arr):
       if len(arr) <= 1: return arr
       pivot = arr[0]
       left = [x for x in arr[1:] if x <= pivot]
       right = [x for x in arr[1:] if x > pivot]
       return quicksort(left) + [pivot] + quicksort(right)

2. The audit should estimate O(N log N) or O(N^2)
3. Verify the UEAS mapping is syntactically valid
4. Run parse_ueas on the generated UEAS to confirm validity
```
