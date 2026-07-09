# UEAS Standard Algorithm Library

47 verified algorithms across 7 categories. All algorithms are written in
UEAS v3.0 academic pseudocode syntax with `Complexity:` contracts.

## Sorting
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| quicksort | O(N log N) | QuickSort with pivot partitioning |
| mergesort | O(N log N) | MergeSort with divide-and-conquer |
| heapsort | O(N log N) | HeapSort with heapify |
| insertion_sort | O(N^2) | Insertion sort for small arrays |
| counting_sort | O(N+k) | Integer counting sort |
| radix_sort | O(d*(N+b)) | LSD radix sort by digit |
| bucket_sort | O(N^2) avg O(N) | Bucket sort with scatter/gather |
| timsort | O(N log N) | Hybrid TimSort (merge+insertion) |

## Searching
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| ternary_search | O(log N) | Ternary search on sorted array |
| jump_search | O(sqrt N) | Jump search with block skipping |
| exponential_search | O(log N) | Exponential doubling + binary search |

## Graph Algorithms
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| bfs | O(V+E) | Breadth-first search |
| dfs | O(V+E) | Depth-first search |
| dijkstra | O((V+E) log V) | Dijkstra's shortest path |
| kruskal | O(E log V) | Kruskal's minimum spanning tree |
| prim | O(V^2) | Prim's minimum spanning tree |
| floyd_warshall | O(V^3) | All-pairs shortest paths |
| bellman_ford | O(VE) | Bellman-Ford with negative cycle detection |
| topological_sort | O(V+E) | Kahn's topological sort |
| kosaraju | O(V+E) | Kosaraju's strongly connected components |

## Dynamic Programming
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| longest_common_subsequence | O(M*N) | Longest common subsequence |
| knapsack | O(N*W) | 0/1 knapsack problem |
| edit_distance | O(M*N) | Levenshtein edit distance |
| matrix_chain_multiplication | O(N^3) | Optimal matrix chain parenthesization |
| coin_change | O(N*A) | Minimum coins to make amount |

## Mathematics
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| gcd | O(log min(a,b)) | Euclidean greatest common divisor |
| sieve_of_eratosthenes | O(N log log N) | Prime number sieve |
| fast_power | O(log N) | Binary exponentiation |
| miller_rabin | O(k log^3 N) | Miller-Rabin primality test |
| fft | O(N log N) | Fast Fourier Transform |
| chinese_remainder | O(N*log M) | Chinese Remainder Theorem |
| extended_euclid | O(log N) | Extended Euclidean algorithm |
| modular_inverse | O(log M) | Modular multiplicative inverse |

## String Algorithms
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| kmp | O(N+M) | Knuth-Morris-Pratt string matching |
| rabin_karp | O(N+M) avg | Rabin-Karp with rolling hash |
| boyer_moore | O(N+M) | Boyer-Moore with bad character rule |
| z_algorithm | O(N) | Z-algorithm for pattern matching |
| suffix_array | O(N log^2 N) | Suffix array construction |

## Data Structures
| Algorithm | Complexity | Description |
|-----------|-----------|-------------|
| binary_search_tree | O(H) | BST insert, search, delete |
| avl_tree | O(log N) | Self-balancing AVL tree |
| heap | O(log N) | Min-heap with insert/extractMin |
| disjoint_set | O(alpha(N)) | Union-Find with path compression |
| trie | O(L) | Trie for string operations |
| segment_tree | O(log N) | Segment tree for range queries |
