//! Step counter and complexity profiling for the UEAS abstract interpreter.
//!
//! The step counter is the basis for invariant enforcement and complexity
//! validation. Each primitive operation increments a monotonic u64 counter.
//! The kernel enforces `@Complexity` contracts by comparing step counts
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
pub struct StepCount(pub u64);

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

/// Configuration for complexity profiling.
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Maximum constant multiplier before complexity violation is flagged.
    /// Default 10^4 per SPEC.md Section 6.4.
    pub c_max: u64,
    /// Global maximum step count before INFINITE_LOOP_DETECTED trap.
    /// Default 10^12 per SPEC.md Section 6.5.
    pub global_max_steps: u64,
    /// Maximum recursion depth before STACK_OVERFLOW trap.
    /// Default 10^4.
    pub max_recursion_depth: u64,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            c_max: 10_000,
            global_max_steps: 1_000_000_000_000,
            max_recursion_depth: 10_000,
        }
    }
}

/// Complexity contract forms per SPEC.md Section 6.4 and Appendix C.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityContract {
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

impl ComplexityContract {
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
    ///
    /// Returns `true` if the complexity has been breached
    /// (`s > C_max * E`), `false` if the contract is satisfied.
    pub fn is_violated(&self, step_count: u64, c_max: u64) -> bool {
        let expected = self.expected_cost();
        // Guard against overflow in multiplication
        let bound = c_max.saturating_mul(expected);
        // If bound overflowed to u64::MAX, treat as no violation
        if bound == u64::MAX && expected > 0 {
            return false;
        }
        step_count > bound
    }
}

/// Manages step counting and complexity enforcement.
#[derive(Debug, Clone)]
pub struct Profiler {
    step_count: StepCount,
    config: ProfilingConfig,
    recursion_depth: u64,
}

impl Profiler {
    /// Create a new profiler with default configuration.
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            step_count: StepCount::new(),
            config,
            recursion_depth: 0,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let contract = ComplexityContract::Constant;
        assert!(!contract.is_violated(0, 10_000));
        assert!(!contract.is_violated(5000, 10_000));
        assert!(contract.is_violated(10_001, 10_000));
    }

    #[test]
    fn linear_complexity_check() {
        let contract = ComplexityContract::Linear { n: 100 };
        // E = 100, C_max = 10, bound = 1000
        assert!(!contract.is_violated(500, 10));
        assert!(!contract.is_violated(1000, 10));
        assert!(contract.is_violated(1001, 10));
    }

    #[test]
    fn quadratic_complexity_check() {
        let contract = ComplexityContract::Quadratic { n: 10 };
        // E = 100, C_max = 10, bound = 1000
        assert!(!contract.is_violated(500, 10));
        assert!(!contract.is_violated(1000, 10));
        assert!(contract.is_violated(1001, 10));
    }

    #[test]
    fn cubic_complexity_check() {
        let contract = ComplexityContract::Polynomial { n: 10, k: 3 };
        // E = 1000, C_max = 10, bound = 10000
        assert!(!contract.is_violated(5000, 10));
        assert!(!contract.is_violated(10000, 10));
        assert!(contract.is_violated(10001, 10));
    }

    #[test]
    fn logarithmic_complexity_empty_input() {
        let contract = ComplexityContract::Logarithmic { n: 0 };
        assert!(!contract.is_violated(0, 10_000));
    }

    #[test]
    fn logarithmic_complexity_check() {
        let contract = ComplexityContract::Logarithmic { n: 1024 };
        // log2(1024) = 10, C_max = 10, bound = 100
        assert!(!contract.is_violated(50, 10));
        assert!(!contract.is_violated(100, 10));
        assert!(contract.is_violated(101, 10));
    }

    #[test]
    fn linearithmic_complexity_check() {
        let contract = ComplexityContract::Linearithmic { n: 100 };
        // log2(100) = 6 (floor), n*log = 600, C_max = 10, bound = 6000
        assert!(!contract.is_violated(3000, 10));
        assert!(!contract.is_violated(6000, 10));
        assert!(contract.is_violated(6001, 10));
    }

    #[test]
    fn sum_complexity_check() {
        let contract = ComplexityContract::Sum {
            terms: vec![100, 200],
        };
        // E = 300, C_max = 10, bound = 3000
        assert!(!contract.is_violated(1500, 10));
        assert!(!contract.is_violated(3000, 10));
        assert!(contract.is_violated(3001, 10));
    }

    #[test]
    fn mixed_log_linear_complexity_check() {
        // Dijkstra: O((V+E) log V), V=10, E=20 => sum=30, log2(10)=3, E=90, C_max=10, bound=900
        let contract = ComplexityContract::MixedLogLinear {
            sum: 30,
            log_term: 10,
        };
        assert!(!contract.is_violated(450, 10));
        assert!(!contract.is_violated(900, 10));
        assert!(contract.is_violated(901, 10));
    }

    #[test]
    fn exponential_complexity_check() {
        let contract = ComplexityContract::Exponential { n: 5 };
        // E = 2^5 = 32, C_max = 10, bound = 320
        assert!(!contract.is_violated(160, 10));
        assert!(!contract.is_violated(320, 10));
        assert!(contract.is_violated(321, 10));
    }

    #[test]
    fn factorial_complexity_check() {
        let contract = ComplexityContract::Factorial { n: 5 };
        // E = 120, C_max = 10, bound = 1200
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
        let contract = ComplexityContract::Linear { n: 100 };
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
        let contract = ComplexityContract::Linear { n: 100 };
        assert_eq!(
            profiler.verify_complexity(&contract).unwrap_err(),
            ExitCode::ComplexityViolation
        );
    }
}
