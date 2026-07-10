//! Step counter and complexity profiling for the UEAS abstract interpreter.
//!
//! The step counter is the basis for invariant enforcement and complexity
//! validation. Each primitive operation increments a monotonic u64 counter.
//! The kernel enforces `Complexity:` contracts by comparing step counts
//! against declared asymptotic bounds.
//!
//! # Rationale
//! Wall-clock profiling is environment-dependent. Abstract step-counting
//! produces deterministic complexity curves identical on any hardware.

use crate::traps::ExitCode;
use serde::{Deserialize, Serialize};

/// Monotonic counter of primitive operations executed by the interpreter.
///
/// Every operation that advances algorithm state increments this counter
/// by a defined weight per SPEC.md Section 6.2.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepCount(u64);

impl StepCount {
    /// Create a new step counter at zero.
    pub fn new() -> Self {
        Self(0)
    }

    /// Increment the step count by a single step.
    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// Increment the step count by a weighted amount.
    pub fn increment_by(&mut self, weight: u64) {
        self.0 = self.0.saturating_add(weight);
    }

    /// Returns the current step count value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Configuration for complexity profiling and PRNG seeding.
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Maximum constant multiplier before complexity violation is flagged.
    /// Default 10^4 per SPEC.md Section 6.4.
    pub c_max: u64,
    /// Global maximum step count before INFINITE_LOOP_DETECTED trap.
    /// Default 10^12 per SPEC.md Section 6.6.
    pub global_max_steps: u64,
    /// Maximum recursion depth before STACK_OVERFLOW trap.
    /// Default 10^4.
    pub max_recursion_depth: u64,
    /// PRNG seed for reproducible stochastic paths (RFC 0009).
    pub prng_seed: u64,
    /// Stream mode — shift verification from Time to Space complexity.
    pub stream_mode: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            c_max: 10_000,
            global_max_steps: 1_000_000_000_000,
            max_recursion_depth: 10_000,
            prng_seed: 0xCAFE_F00D_D15C_0001,
            stream_mode: false,
        }
    }
}

/// Complexity class variants per SPEC.md Section 6.4 and Appendix C.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityKind {
    /// O(1) — constant time.
    Constant,
    /// O(N) — linear in n.
    Linear { n: u64 },
    /// O(N^2) — quadratic in n.
    Quadratic { n: u64 },
    /// O(N^k) — polynomial of degree k in n.
    Polynomial { n: u64, k: u32 },
    /// O(log N) — logarithmic in n.
    Logarithmic { n: u64 },
    /// O(N log N) — linearithmic in n.
    Linearithmic { n: u64 },
    /// O(V+E) — sum of two variables (graph algorithms).
    Sum { terms: Vec<u64> },
    /// O((V+E) log V) — mixed form (e.g., Dijkstra).
    MixedLogLinear { sum: u64, log_term: u64 },
    /// O(2^N) — exponential in n.
    Exponential { n: u64 },
    /// O(N!) — factorial in n.
    Factorial { n: u64 },
}

/// Complexity contract with optional expected (stochastic) bound (RFC 0009).
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexityContract {
    pub kind: ComplexityKind,
    /// Optional expected-case complexity annotation for stochastic algorithms.
    pub expected_complexity: Option<String>,
}

impl ComplexityContract {
    /// Compute the expected asymptotic cost E(n_1, ..., n_k).
    pub fn expected_cost(&self) -> u64 {
        self.kind.expected_cost()
    }

    /// Check whether the step count `s` violates this contract.
    pub fn is_violated(&self, step_count: u64, c_max: u64) -> bool {
        self.kind.is_violated(step_count, c_max)
    }
}

impl ComplexityKind {
    /// Compute the expected asymptotic cost E(n_1, ..., n_k).
    pub fn expected_cost(&self) -> u64 {
        match self {
            Self::Constant => 1,
            Self::Linear { n } => *n,
            Self::Quadratic { n } => n.saturating_mul(*n),
            Self::Polynomial { n, k } => {
                let mut result: u64 = 1;
                for _ in 0..*k {
                    result = result.saturating_mul(*n);
                }
                result
            }
            Self::Logarithmic { n } => {
                if *n <= 1 {
                    1
                } else {
                    n.ilog2().max(1) as u64
                }
            }
            Self::Linearithmic { n } => {
                let log = if *n <= 1 { 1 } else { n.ilog2().max(1) as u64 };
                n.saturating_mul(log)
            }
            Self::Sum { terms } => terms.iter().sum(),
            Self::MixedLogLinear { sum, log_term } => {
                let log = if *log_term <= 1 {
                    1
                } else {
                    log_term.ilog2().max(1) as u64
                };
                sum.saturating_mul(log)
            }
            Self::Exponential { n } => {
                if *n >= 64 {
                    u64::MAX
                } else {
                    1u64.saturating_mul(2u64.pow(*n as u32))
                }
            }
            Self::Factorial { n } => {
                let mut result: u64 = 1;
                let mut i: u64 = 1;
                while i <= *n {
                    result = result.saturating_mul(i);
                    i += 1;
                }
                result
            }
        }
    }

    /// Check whether the step count `s` violates this contract.
    pub fn is_violated(&self, step_count: u64, c_max: u64) -> bool {
        let expected = self.expected_cost();
        let bound = c_max.saturating_mul(expected);
        if bound == u64::MAX && expected > 0 {
            return false;
        }
        step_count > bound
    }
}

/// A node in the parallel execution DAG for work-span analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub id: u64,
    pub work_cost: u64,
    pub span_cost: u64,
    pub dependencies: Vec<u64>,
}

/// Manages step counting and complexity enforcement.
#[derive(Debug, Clone)]
pub struct Profiler {
    step_count: StepCount,
    config: ProfilingConfig,
    recursion_depth: u64,
    work: u64,
    span: u64,
    dag_nodes: Vec<DagNode>,
}

impl Profiler {
    /// Create a new profiler with default configuration.
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            step_count: StepCount::new(),
            config,
            recursion_depth: 0,
            work: 0,
            span: 0,
            dag_nodes: Vec::new(),
        }
    }

    /// Record a single step operation.
    pub fn step(&mut self) -> Result<(), ExitCode> {
        self.step_count.increment();
        if self.step_count.value() > self.config.global_max_steps {
            return Err(ExitCode::InfiniteLoopDetected);
        }
        Ok(())
    }

    /// Record a weighted operation.
    pub fn step_weighted(&mut self, weight: u64) -> Result<(), ExitCode> {
        self.step_count.increment_by(weight);
        if self.step_count.value() > self.config.global_max_steps {
            return Err(ExitCode::InfiniteLoopDetected);
        }
        Ok(())
    }

    /// Enter a recursion level. Returns error if max depth exceeded.
    pub fn enter_recursion(&mut self) -> Result<(), ExitCode> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.config.max_recursion_depth {
            return Err(ExitCode::StackOverflow);
        }
        Ok(())
    }

    /// Exit a recursion level.
    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// Return the current step count.
    pub fn step_count(&self) -> u64 {
        self.step_count.value()
    }

    /// Verify complexity contract at completion.
    pub fn verify_complexity(&self, contract: &ComplexityContract) -> Result<(), ExitCode> {
        if contract.is_violated(self.step_count.value(), self.config.c_max) {
            return Err(ExitCode::ComplexityViolation);
        }
        Ok(())
    }

    pub fn record_work(&mut self, cost: u64) {
        self.work = self.work.saturating_add(cost);
    }

    pub fn record_span(&mut self, cost: u64) {
        self.span = self.span.saturating_add(cost);
    }

    pub fn work(&self) -> u64 {
        self.work
    }

    pub fn span(&self) -> u64 {
        self.span
    }

    pub fn add_dag_node(&mut self, node: DagNode) {
        self.dag_nodes.push(node);
    }

    /// Parallel efficiency: work / (span * num_threads).
    /// Returns 1.0 when span == 0 (degenerate case).
    pub fn parallel_efficiency(&self) -> f64 {
        let num_threads = (self.dag_nodes.len() as u64).max(1);
        if self.span == 0 {
            1.0
        } else {
            self.work as f64 / (self.span as f64 * num_threads as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(kind: ComplexityKind) -> ComplexityContract {
        ComplexityContract {
            kind,
            expected_complexity: None,
        }
    }

    #[test]
    fn step_count_starts_at_zero() {
        let count = StepCount::new();
        assert_eq!(count.value(), 0);
    }

    #[test]
    fn step_count_increments() {
        let mut count = StepCount::new();
        count.increment();
        count.increment();
        assert_eq!(count.value(), 2);
    }

    #[test]
    fn step_count_weighted_increment() {
        let mut count = StepCount::new();
        count.increment_by(100);
        assert_eq!(count.value(), 100);
    }

    #[test]
    fn constant_complexity_never_violated_for_small_steps() {
        let contract = c(ComplexityKind::Constant);
        assert!(!contract.is_violated(0, 10_000));
        assert!(!contract.is_violated(5000, 10_000));
        assert!(contract.is_violated(10_001, 10_000));
    }

    #[test]
    fn linear_complexity_check() {
        let contract = c(ComplexityKind::Linear { n: 100 });
        assert!(!contract.is_violated(500, 10));
        assert!(!contract.is_violated(1000, 10));
        assert!(contract.is_violated(1001, 10));
    }

    #[test]
    fn quadratic_complexity_check() {
        let contract = c(ComplexityKind::Quadratic { n: 10 });
        assert!(!contract.is_violated(500, 10));
        assert!(!contract.is_violated(1000, 10));
        assert!(contract.is_violated(1001, 10));
    }

    #[test]
    fn cubic_complexity_check() {
        let contract = c(ComplexityKind::Polynomial { n: 10, k: 3 });
        assert!(!contract.is_violated(5000, 10));
        assert!(!contract.is_violated(10000, 10));
        assert!(contract.is_violated(10001, 10));
    }

    #[test]
    fn logarithmic_complexity_empty_input() {
        let contract = c(ComplexityKind::Logarithmic { n: 0 });
        assert!(!contract.is_violated(0, 10_000));
    }

    #[test]
    fn logarithmic_complexity_check() {
        let contract = c(ComplexityKind::Logarithmic { n: 1024 });
        assert!(!contract.is_violated(50, 10));
        assert!(!contract.is_violated(100, 10));
        assert!(contract.is_violated(101, 10));
    }

    #[test]
    fn linearithmic_complexity_check() {
        let contract = c(ComplexityKind::Linearithmic { n: 100 });
        assert!(!contract.is_violated(3000, 10));
        assert!(!contract.is_violated(6000, 10));
        assert!(contract.is_violated(6001, 10));
    }

    #[test]
    fn sum_complexity_check() {
        let contract = c(ComplexityKind::Sum {
            terms: vec![100, 200],
        });
        assert!(!contract.is_violated(1500, 10));
        assert!(!contract.is_violated(3000, 10));
        assert!(contract.is_violated(3001, 10));
    }

    #[test]
    fn mixed_log_linear_complexity_check() {
        let contract = c(ComplexityKind::MixedLogLinear {
            sum: 30,
            log_term: 10,
        });
        assert!(!contract.is_violated(450, 10));
        assert!(!contract.is_violated(900, 10));
        assert!(contract.is_violated(901, 10));
    }

    #[test]
    fn exponential_complexity_check() {
        let contract = c(ComplexityKind::Exponential { n: 5 });
        assert!(!contract.is_violated(160, 10));
        assert!(!contract.is_violated(320, 10));
        assert!(contract.is_violated(321, 10));
    }

    #[test]
    fn factorial_complexity_check() {
        let contract = c(ComplexityKind::Factorial { n: 5 });
        assert!(!contract.is_violated(600, 10));
        assert!(!contract.is_violated(1200, 10));
        assert!(contract.is_violated(1201, 10));
    }

    #[test]
    fn profiler_step_increment() {
        let mut profiler = Profiler::new(ProfilingConfig::default());
        for _ in 0..100 {
            profiler.step().unwrap();
        }
        assert_eq!(profiler.step_count(), 100);
    }

    #[test]
    fn profiler_infinite_loop_detected() {
        let config = ProfilingConfig {
            global_max_steps: 10,
            ..Default::default()
        };
        let mut profiler = Profiler::new(config);
        for i in 0..10 {
            assert!(profiler.step().is_ok(), "step {} should succeed", i);
        }
        assert_eq!(profiler.step().unwrap_err(), ExitCode::InfiniteLoopDetected);
    }

    #[test]
    fn profiler_recursion_depth() {
        let config = ProfilingConfig {
            max_recursion_depth: 2,
            ..Default::default()
        };
        let mut profiler = Profiler::new(config);
        profiler.enter_recursion().unwrap();
        profiler.enter_recursion().unwrap();
        assert_eq!(
            profiler.enter_recursion().unwrap_err(),
            ExitCode::StackOverflow
        );
    }

    #[test]
    fn profiler_verify_complexity_pass() {
        let mut profiler = Profiler::new(ProfilingConfig::default());
        for _ in 0..50 {
            profiler.step().unwrap();
        }
        let contract = c(ComplexityKind::Linear { n: 100 });
        assert!(profiler.verify_complexity(&contract).is_ok());
    }

    #[test]
    fn profiler_verify_complexity_violation() {
        let mut profiler = Profiler::new(ProfilingConfig {
            c_max: 1,
            ..Default::default()
        });
        for _ in 0..200 {
            profiler.step().unwrap();
        }
        let contract = c(ComplexityKind::Linear { n: 100 });
        assert_eq!(
            profiler.verify_complexity(&contract).unwrap_err(),
            ExitCode::ComplexityViolation
        );
    }
    #[test]
    fn exit_recursion_at_zero_no_panic() {
        Profiler::new(ProfilingConfig::default()).exit_recursion();
    }
    #[test]
    fn is_violated_cmax_zero() {
        assert!(c(ComplexityKind::Constant).is_violated(1, 0));
    }
    #[test]
    fn expected_cost_exp_overflow() {
        assert_eq!(
            ComplexityKind::Exponential { n: 64 }.expected_cost(),
            u64::MAX
        );
        assert_eq!(
            ComplexityKind::Exponential { n: 63 }.expected_cost(),
            1u64 << 63
        );
    }
    #[test]
    fn default_profiling_config() {
        let c = ProfilingConfig::default();
        assert_eq!(c.c_max, 10000);
        assert_eq!(c.global_max_steps, 1_000_000_000_000);
        assert_eq!(c.max_recursion_depth, 10000);
        assert!(!c.stream_mode);
    }
    #[test]
    fn expected_complexity_stored() {
        let contract = ComplexityContract {
            kind: ComplexityKind::Constant,
            expected_complexity: Some("O(N log N)".to_string()),
        };
        assert_eq!(contract.expected_complexity, Some("O(N log N)".to_string()));
        assert_eq!(contract.expected_cost(), 1);
    }

    #[test]
    fn stream_mode_can_be_enabled() {
        let mut config = ProfilingConfig::default();
        config.stream_mode = true;
        assert!(config.stream_mode);
    }

    #[test]
    fn default_complexity_contract_has_no_expected() {
        let contract = ComplexityContract {
            kind: ComplexityKind::Linear { n: 1 },
            expected_complexity: None,
        };
        assert!(contract.expected_complexity.is_none());
    }

    #[test]
    fn prng_seed_stored_in_config() {
        let mut config = ProfilingConfig::default();
        config.prng_seed = 42;
        assert_eq!(config.prng_seed, 42);
    }

    #[test]
    fn expected_complexity_can_be_constant() {
        let contract = ComplexityContract {
            kind: ComplexityKind::Constant,
            expected_complexity: Some("O(1)".to_string()),
        };
        assert_eq!(contract.kind, ComplexityKind::Constant);
        assert_eq!(contract.expected_complexity, Some("O(1)".to_string()));
    }

    #[test]
    fn work_starts_at_zero() {
        let profiler = Profiler::new(ProfilingConfig::default());
        assert_eq!(profiler.work(), 0);
    }

    #[test]
    fn span_starts_at_zero() {
        let profiler = Profiler::new(ProfilingConfig::default());
        assert_eq!(profiler.span(), 0);
    }

    #[test]
    fn record_work_increments() {
        let mut profiler = Profiler::new(ProfilingConfig::default());
        profiler.record_work(10);
        profiler.record_work(5);
        assert_eq!(profiler.work(), 15);
    }

    #[test]
    fn record_span_increments() {
        let mut profiler = Profiler::new(ProfilingConfig::default());
        profiler.record_span(3);
        profiler.record_span(7);
        assert_eq!(profiler.span(), 10);
    }

    #[test]
    fn add_dag_node_tracks_node() {
        let mut profiler = Profiler::new(ProfilingConfig::default());
        profiler.add_dag_node(DagNode {
            id: 1,
            work_cost: 5,
            span_cost: 2,
            dependencies: vec![],
        });
        profiler.add_dag_node(DagNode {
            id: 2,
            work_cost: 3,
            span_cost: 4,
            dependencies: vec![1],
        });
        assert_eq!(profiler.dag_nodes.len(), 2);
    }

    #[test]
    fn parallel_efficiency_zero_span_returns_one() {
        let profiler = Profiler::new(ProfilingConfig::default());
        assert_eq!(profiler.parallel_efficiency(), 1.0);
    }

    #[test]
    fn parallel_efficiency_calculation() {
        let mut profiler = Profiler::new(ProfilingConfig::default());
        profiler.record_work(100);
        profiler.record_span(25);
        profiler.add_dag_node(DagNode {
            id: 1,
            work_cost: 50,
            span_cost: 25,
            dependencies: vec![],
        });
        profiler.add_dag_node(DagNode {
            id: 2,
            work_cost: 50,
            span_cost: 0,
            dependencies: vec![],
        });
        assert!((profiler.parallel_efficiency() - 2.0).abs() < 1e-9);
    }
}
