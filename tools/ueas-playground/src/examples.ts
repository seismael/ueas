export interface ExampleItem {
  id: string;
  label: string;
  source: string;
}

export const STUDIO_EXAMPLES: ExampleItem[] = [
  {
    id: "binary-search",
    label: "BinarySearch (O(log N) + Contracts)",
    source: `module playground.searching

Algorithm BinarySearch(data: List<Int>, target: Int) -> Option<Int>
    Require:
        forall i in 0..(data.length - 1) :: data[i] <= data[i + 1]
    Complexity:
        Time: O(log N) where N = data.length,
        Space: O(1)

    let mut low <- 0
    let mut high <- data.length - 1
    while low <= high do
        invariant 0 <= low
        decreases high - low + 1
        let mid <- low + (high - low) / 2
        if data[mid] == target then
            return Some(mid)
        else if data[mid] < target then
            low <- mid + 1
        else
            high <- mid - 1
        end if
    end while
    return None
end Algorithm

@Test "binary search"
    assert BinarySearch([1, 3, 5, 7, 9], 7) == Some(3)
end Test
`,
  },
  {
    id: "parallel-sum",
    label: "ParallelSum (Work/Span DAG)",
    source: `module playground.parallel

Algorithm ParallelSum(n: Int) -> Int
    Complexity:
        Time: O(N),
        Work: O(N),
        Span: O(log N)

    let mut total <- 0
    parallel for i in 0..n do
        total <- total + i
    end for
    return total
end Algorithm

@Test "parallel sum"
    assert ParallelSum(10) == 45
end Test
`,
  },
  {
    id: "ct-select",
    label: "ConstantTimeSelectMax (Public ⊑ Secret)",
    source: `module playground.crypto

@ConstantTime
Algorithm ConstantTimeSelectMax(a: Secret<Int>, b: Secret<Int>) -> Secret<Int>
    Complexity:
        Time: O(1),
        Space: O(1)

    return select(a >= b, a, b)
end Algorithm

@Test "constant time select"
    assert ConstantTimeSelectMax(10, 25) == 25
end Test
`,
  },
  {
    id: "bad-complexity",
    label: "BrokenComplexity Demo (Adversarial L3 Catch)",
    source: `module playground.bad_complexity

Algorithm ClaimedFast(data: List<Int>) -> Int
    Complexity:
        Time: O(N log N) where N = data.length,
        Space: O(1)

    let mut s <- 0
    for i in 0..data.length do
        for j in 0..data.length do
            s <- s + data[i] + data[j]
        end for
    end for
    return s
end Algorithm
`,
  },
];
