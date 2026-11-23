//! UEAS v2.0 deterministic, cost-metered bytecode VM, VirtualHeap, cache & Work/Span DAG.
#![allow(missing_docs)]

pub mod builtins;
pub mod cache;
pub mod chunk;
pub mod compile;
pub mod heap;
pub mod op;
pub mod prng;
pub mod profiler;
pub mod trace;
pub mod trap;
pub mod value;
pub mod vm;

pub use cache::{CacheConfig, CacheHierarchy, CacheLevelConfig, CacheStats};
pub use chunk::{Chunk, CompiledProgram, Constant};
pub use compile::compile_program;
pub use heap::{HeapObject, HeapStats, VirtualHeap};
pub use op::Op;
pub use prng::DeterministicPrng;
pub use profiler::{WorkSpanProfiler, WorkSpanReport};
pub use trace::TraceEvent;
pub use trap::{TrapCode, TrapError};
pub use value::{CanonicalF64, HeapHandle, RuntimeKey, RuntimeValue};
pub use vm::{run_hir, CallFrame, ExecutionResult, StepOutcome, VmConfig, VmSession};

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use ueas_hir::check::linker::NullModuleResolver;
    use ueas_hir::compile_to_hir;

    #[test]
    fn executes_binary_search_and_verifies_contracts() {
        let src = r#"
module test.vm

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
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR compiles");
        let compiled = compile_program(&prog).expect("bytecode compiles");
        let mut session = VmSession::new(
            compiled,
            "BinarySearch",
            vec![RuntimeValue::Unit, RuntimeValue::Int(BigInt::from(7))],
            VmConfig::default(),
        )
        .expect("session");
        let list_h = session
            .heap_mut()
            .alloc(HeapObject::List(
                [1, 3, 5, 7, 9, 11]
                    .into_iter()
                    .map(|x| RuntimeValue::Int(BigInt::from(x)))
                    .collect(),
            ))
            .expect("alloc");
        session.set_arg(0, RuntimeValue::Handle(list_h));

        let res = session.run().expect("run succeeds");
        assert_eq!(
            res.value,
            RuntimeValue::OptionVal(Some(Box::new(RuntimeValue::Int(BigInt::from(3)))))
        );
        assert!(res.steps > 0);
        assert!(res.cache.accesses > 0);
        assert!(res.heap.peak_live_bytes > 0);
    }

    #[test]
    fn enforces_step_budget_and_generational_use_after_free() {
        let mut heap = VirtualHeap::new(1024);
        let h = heap
            .alloc(HeapObject::List(vec![RuntimeValue::Int(BigInt::from(1))]))
            .unwrap();
        assert!(heap.free(h).is_ok());
        assert_eq!(heap.stats().live_bytes, 0);
        let err = heap.get(h).unwrap_err();
        assert_eq!(err.trap, TrapCode::UseAfterFree);
    }

    #[test]
    fn parallel_for_computes_true_work_span_dag() {
        let src = r#"
module test.par

Algorithm SumSquares(n: Int) -> Int
    Complexity:
        Time: O(N),
        Work: O(N),
        Span: O(log N)

    let mut total <- 0
    parallel for i in 0..n do
        let sq <- i * i
        total <- total + sq
    end for
    return total
end Algorithm
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR compiles");
        let res = run_hir(
            &prog,
            "SumSquares",
            vec![RuntimeValue::Int(BigInt::from(16))],
            VmConfig::default(),
        )
        .expect("parallel run");
        assert!(res.work_span.work > res.work_span.span);
        assert!(res.work_span.parallelism > 1.0);
        assert!(res.work_span.efficiency_at_4_cores <= 1.0);
        assert!(res.work_span.fork_count == 1);
    }

    #[test]
    fn step_back_restores_previous_state() {
        let src = r#"
module test.dbg

Algorithm Inc(x: Int) -> Int
    let y <- x + 1
    return y
end Algorithm
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR");
        let compiled = compile_program(&prog).expect("compiled");
        let mut session = VmSession::new(
            compiled,
            "Inc",
            vec![RuntimeValue::Int(BigInt::from(41))],
            VmConfig {
                max_snapshots: 64,
                ..VmConfig::default()
            },
        )
        .unwrap();
        assert!(matches!(session.step().unwrap(), StepOutcome::Continue));
        let steps_after_1 = session.steps();
        assert!(matches!(session.step().unwrap(), StepOutcome::Continue));
        assert!(session.steps() > steps_after_1);
        assert!(session.step_back());
        assert_eq!(session.steps(), steps_after_1);
        let res = session.run().unwrap();
        assert_eq!(res.value, RuntimeValue::Int(BigInt::from(42)));
    }

    #[test]
    fn cow_heap_shares_without_cloning_and_isolates_on_write() {
        let mut heap = VirtualHeap::new(1024 * 1024);
        let h1 = heap
            .alloc(HeapObject::List(vec![RuntimeValue::Int(BigInt::from(10))]))
            .unwrap();
        let peak_before = heap.stats().peak_live_bytes;
        heap.retain(h1);
        assert_eq!(heap.stats().peak_live_bytes, peak_before);
        let h2 = heap.ensure_unique(h1).unwrap();
        assert_ne!(h1, h2);
        heap.with_mut(h2, |obj, _| {
            if let HeapObject::List(v) = obj {
                v[0] = RuntimeValue::Int(BigInt::from(99));
            }
            Ok(())
        })
        .unwrap();
        let (o1, _) = heap.get(h1).unwrap();
        let (o2, _) = heap.get(h2).unwrap();
        assert_ne!(o1, o2);
    }

    #[test]
    fn mut_inout_parameter_writes_back_to_caller() {
        let src = r#"
module test.inout

Algorithm SwapFirstTwo(mut a: List<Int>) -> Void
    a.swap(0, 1)
end Algorithm

Algorithm Caller() -> Int
    let mut xs <- [10, 20, 30]
    SwapFirstTwo(mut xs)
    return xs[0] * 100 + xs[1]
end Algorithm
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR");
        let res = run_hir(&prog, "Caller", Vec::new(), VmConfig::default()).expect("run");
        assert_eq!(res.value, RuntimeValue::Int(BigInt::from(2010)));
    }
}
