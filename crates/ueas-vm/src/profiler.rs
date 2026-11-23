//! True fork-join Work/Span DAG profiler (Amdahl/Brent-compliant).

use serde::{Deserialize, Serialize};

/// Fork-join Work/Span summary for an execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkSpanReport {
    pub work: u64,
    pub span: u64,
    pub parallelism: f64,
    pub efficiency_at_4_cores: f64,
    pub efficiency_at_8_cores: f64,
    pub fork_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScopeFrame {
    seq_work: u64,
    seq_span: u64,
    branches: Vec<(u64, u64)>,
    in_branch: bool,
    cur_branch_work: u64,
    cur_branch_span: u64,
}

/// Tracks sequential vs parallel fork-join Work (`W`) and Span (`D`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSpanProfiler {
    stack: Vec<ScopeFrame>,
    fork_count: u64,
}

impl Default for WorkSpanProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkSpanProfiler {
    /// Create a fresh Work/Span DAG profiler with a root sequential frame.
    pub fn new() -> Self {
        Self {
            stack: vec![ScopeFrame {
                seq_work: 0,
                seq_span: 0,
                branches: Vec::new(),
                in_branch: false,
                cur_branch_work: 0,
                cur_branch_span: 0,
            }],
            fork_count: 0,
        }
    }

    /// Charge `steps` of work & span to the active scope or branch.
    pub fn record_step(&mut self, steps: u64) {
        if let Some(top) = self.stack.last_mut() {
            if top.in_branch {
                top.cur_branch_work += steps;
                top.cur_branch_span += steps;
            } else {
                top.seq_work += steps;
                top.seq_span += steps;
            }
        }
    }

    /// Open a parallel fork scope (`ForkScope` or `ParallelForBegin`).
    pub fn fork_scope(&mut self) {
        self.fork_count += 1;
        self.stack.push(ScopeFrame {
            seq_work: 0,
            seq_span: 0,
            branches: Vec::new(),
            in_branch: false,
            cur_branch_work: 0,
            cur_branch_span: 0,
        });
    }

    /// Start a new parallel branch inside the current fork scope.
    pub fn begin_branch(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            if top.in_branch {
                top.branches
                    .push((top.cur_branch_work, top.cur_branch_span));
            }
            top.in_branch = true;
            top.cur_branch_work = 0;
            top.cur_branch_span = 0;
        }
    }

    /// Finish the active parallel branch.
    pub fn end_branch(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            if top.in_branch {
                top.branches
                    .push((top.cur_branch_work, top.cur_branch_span));
                top.in_branch = false;
                top.cur_branch_work = 0;
                top.cur_branch_span = 0;
            }
        }
    }

    /// Synchronize/join the current parallel fork scope into its parent.
    ///
    /// `Work_total = sum(branch_work)`, `Span_total = max(branch_span) + ceil(log2(N + 1))`.
    pub fn sync_scope(&mut self) {
        if self.stack.len() <= 1 {
            return;
        }
        if let Some(mut frame) = self.stack.pop() {
            if frame.in_branch {
                frame
                    .branches
                    .push((frame.cur_branch_work, frame.cur_branch_span));
            }
            let n = frame.branches.len();
            let branch_work: u64 = frame.branches.iter().map(|(w, _)| *w).sum();
            let max_branch_span: u64 = frame.branches.iter().map(|(_, s)| *s).max().unwrap_or(0);
            let sync_overhead = if n <= 1 {
                0
            } else {
                ((n + 1) as f64).log2().ceil() as u64
            };
            let total_work = frame.seq_work + branch_work;
            let total_span = frame.seq_span + max_branch_span + sync_overhead;

            if let Some(parent) = self.stack.last_mut() {
                if parent.in_branch {
                    parent.cur_branch_work += total_work;
                    parent.cur_branch_span += total_span;
                } else {
                    parent.seq_work += total_work;
                    parent.seq_span += total_span;
                }
            }
        }
    }

    /// Produce the final Work/Span report.
    pub fn report(&self) -> WorkSpanReport {
        let root = self.stack.first();
        let (work, span) = match root {
            Some(r) => (r.seq_work, r.seq_span.max(1)),
            None => (0, 1),
        };
        let work_f = work as f64;
        let span_f = span as f64;
        let parallelism = work_f / span_f;

        // Brent's Theorem bound: T_p <= ceil(W / P) + D => Speedup = W / T_p => Eff = Speedup / P <= 1.0
        let eff = |cores: f64| -> f64 {
            if work == 0 {
                return 1.0;
            }
            let tp = (work_f / cores).max(span_f);
            let speedup = work_f / tp;
            (speedup / cores).clamp(0.0, 1.0)
        };

        WorkSpanReport {
            work,
            span,
            parallelism,
            efficiency_at_4_cores: eff(4.0),
            efficiency_at_8_cores: eff(8.0),
            fork_count: self.fork_count,
        }
    }
}
