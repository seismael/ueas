//! Level 1 `@Test` suite runner.

use crate::diagnostic::{RepairDiagnostic, VerificationLevel};
use serde::{Deserialize, Serialize};
use ueas_vm::{CompiledProgram, VmConfig, VmSession};

/// Result of running a single `@Test` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestOutcome {
    pub name: String,
    pub passed: bool,
    pub steps: u64,
    pub error: Option<String>,
}

/// Run all `@Test` blocks in `compiled`, returning outcomes and any repair diagnostics.
pub fn run_tests(
    compiled: &CompiledProgram,
    config: &VmConfig,
) -> (Vec<TestOutcome>, Vec<RepairDiagnostic>) {
    let mut outcomes = Vec::with_capacity(compiled.tests.len());
    let mut diags = Vec::new();

    for (name, _) in &compiled.tests {
        let chunk_name = format!("test::{name}");
        match VmSession::new(compiled.clone(), &chunk_name, Vec::new(), config.clone()) {
            Ok(mut session) => match session.run() {
                Ok(res) => outcomes.push(TestOutcome {
                    name: name.clone(),
                    passed: true,
                    steps: res.steps,
                    error: None,
                }),
                Err(trap) => {
                    diags.push(RepairDiagnostic {
                        code: format!("L1_{}", trap.trap.code()),
                        level: VerificationLevel::L1,
                        message: format!("@Test '{name}' failed: {}", trap.message),
                        span: trap.span,
                        counterexample: None,
                        suggested_fix: Some(
                            "Inspect failing assertion or contract inside @Test".to_string(),
                        ),
                    });
                    outcomes.push(TestOutcome {
                        name: name.clone(),
                        passed: false,
                        steps: trap.step,
                        error: Some(trap.to_string()),
                    });
                }
            },
            Err(err) => {
                diags.push(RepairDiagnostic {
                    code: "L1_INIT_ERROR".to_string(),
                    level: VerificationLevel::L1,
                    message: format!("@Test '{name}' failed to initialize: {err}"),
                    span: err.span,
                    counterexample: None,
                    suggested_fix: None,
                });
                outcomes.push(TestOutcome {
                    name: name.clone(),
                    passed: false,
                    steps: 0,
                    error: Some(err.to_string()),
                });
            }
        }
    }

    (outcomes, diags)
}
