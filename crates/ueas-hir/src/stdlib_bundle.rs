//! Zero-I/O embedded `std.*` module resolver (`wasm32`-compatible).

use crate::check::linker::ModuleResolver;

/// Resolves `std.*` modules from compile-time embedded `stdlib/` `.ueas` sources.
pub struct EmbeddedStdlibResolver;

impl ModuleResolver for EmbeddedStdlibResolver {
    fn resolve(&self, dotted_path: &str) -> Result<String, String> {
        let src = match dotted_path {
            "std.math.gcd" => Some(include_str!("../../../stdlib/math/gcd.ueas")),
            "std.math.extended_euclid" => {
                Some(include_str!("../../../stdlib/math/extended_euclid.ueas"))
            }
            "std.math.fast_power" => Some(include_str!("../../../stdlib/math/fast_power.ueas")),
            "std.math.modular_inverse" => {
                Some(include_str!("../../../stdlib/math/modular_inverse.ueas"))
            }
            "std.math.chinese_remainder" => {
                Some(include_str!("../../../stdlib/math/chinese_remainder.ueas"))
            }
            "std.math.sieve_of_eratosthenes" => Some(include_str!(
                "../../../stdlib/math/sieve_of_eratosthenes.ueas"
            )),
            "std.math.miller_rabin" => Some(include_str!("../../../stdlib/math/miller_rabin.ueas")),
            "std.math.fft" => Some(include_str!("../../../stdlib/math/fft.ueas")),
            "std.searching.exponential_search" => Some(include_str!(
                "../../../stdlib/searching/exponential_search.ueas"
            )),
            "std.searching.jump_search" => {
                Some(include_str!("../../../stdlib/searching/jump_search.ueas"))
            }
            "std.searching.ternary_search" => Some(include_str!(
                "../../../stdlib/searching/ternary_search.ueas"
            )),
            "std.sorting.insertion_sort" => {
                Some(include_str!("../../../stdlib/sorting/insertion_sort.ueas"))
            }
            "std.sorting.mergesort" => Some(include_str!("../../../stdlib/sorting/mergesort.ueas")),
            "std.sorting.quicksort" => Some(include_str!("../../../stdlib/sorting/quicksort.ueas")),
            "std.sorting.heapsort" => Some(include_str!("../../../stdlib/sorting/heapsort.ueas")),
            "std.sorting.counting_sort" => {
                Some(include_str!("../../../stdlib/sorting/counting_sort.ueas"))
            }
            "std.sorting.radix_sort" => {
                Some(include_str!("../../../stdlib/sorting/radix_sort.ueas"))
            }
            "std.sorting.bucket_sort" => {
                Some(include_str!("../../../stdlib/sorting/bucket_sort.ueas"))
            }
            "std.sorting.timsort" => Some(include_str!("../../../stdlib/sorting/timsort.ueas")),
            "std.data_structures.disjoint_set" => Some(include_str!(
                "../../../stdlib/data_structures/disjoint_set.ueas"
            )),
            "std.data_structures.heap" => {
                Some(include_str!("../../../stdlib/data_structures/heap.ueas"))
            }
            "std.data_structures.segment_tree" => Some(include_str!(
                "../../../stdlib/data_structures/segment_tree.ueas"
            )),
            "std.data_structures.binary_search_tree" => Some(include_str!(
                "../../../stdlib/data_structures/binary_search_tree.ueas"
            )),
            "std.data_structures.avl_tree" => Some(include_str!(
                "../../../stdlib/data_structures/avl_tree.ueas"
            )),
            "std.data_structures.trie" => {
                Some(include_str!("../../../stdlib/data_structures/trie.ueas"))
            }
            "std.graph.bfs" => Some(include_str!("../../../stdlib/graph/bfs.ueas")),
            "std.graph.dfs" => Some(include_str!("../../../stdlib/graph/dfs.ueas")),
            "std.graph.dijkstra" => Some(include_str!("../../../stdlib/graph/dijkstra.ueas")),
            "std.graph.bellman_ford" => {
                Some(include_str!("../../../stdlib/graph/bellman_ford.ueas"))
            }
            "std.graph.floyd_warshall" => {
                Some(include_str!("../../../stdlib/graph/floyd_warshall.ueas"))
            }
            "std.graph.topological_sort" => {
                Some(include_str!("../../../stdlib/graph/topological_sort.ueas"))
            }
            "std.graph.kruskal" => Some(include_str!("../../../stdlib/graph/kruskal.ueas")),
            "std.graph.prim" => Some(include_str!("../../../stdlib/graph/prim.ueas")),
            "std.graph.kosaraju" => Some(include_str!("../../../stdlib/graph/kosaraju.ueas")),
            "std.dp.coin_change" => Some(include_str!("../../../stdlib/dp/coin_change.ueas")),
            "std.dp.edit_distance" => Some(include_str!("../../../stdlib/dp/edit_distance.ueas")),
            "std.dp.knapsack" => Some(include_str!("../../../stdlib/dp/knapsack.ueas")),
            "std.dp.longest_common_subsequence" => Some(include_str!(
                "../../../stdlib/dp/longest_common_subsequence.ueas"
            )),
            "std.dp.matrix_chain_multiplication" => Some(include_str!(
                "../../../stdlib/dp/matrix_chain_multiplication.ueas"
            )),
            "std.strings.kmp" => Some(include_str!("../../../stdlib/strings/kmp.ueas")),
            "std.strings.z_algorithm" => {
                Some(include_str!("../../../stdlib/strings/z_algorithm.ueas"))
            }
            "std.strings.rabin_karp" => {
                Some(include_str!("../../../stdlib/strings/rabin_karp.ueas"))
            }
            "std.strings.boyer_moore" => {
                Some(include_str!("../../../stdlib/strings/boyer_moore.ueas"))
            }
            "std.strings.suffix_array" => {
                Some(include_str!("../../../stdlib/strings/suffix_array.ueas"))
            }
            _ => None,
        };
        src.map(ToString::to_string)
            .ok_or_else(|| format!("embedded stdlib module '{dotted_path}' not found"))
    }
}
