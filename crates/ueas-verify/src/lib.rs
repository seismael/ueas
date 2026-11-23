//! UEAS v2.0 5-level progressive verification, property fuzzer, adversarial Big-O fitter & certificates.
#![allow(missing_docs)]

pub mod certificate;
pub mod complexity_fit;
pub mod constant_time;
pub mod diagnostic;
pub mod fuzzer;
pub mod oracle;
pub mod runner;

pub use certificate::VerificationCertificate;
pub use complexity_fit::{
    fit_best_class, verify_complexity, ComplexityFitReport, ScaleMeasurement,
};
pub use constant_time::{verify_constant_time, ConstantTimeReport};
pub use diagnostic::{RepairDiagnostic, VerificationLevel};
pub use fuzzer::{fuzz_algorithm, generate_valid_inputs, FuzzArg, FuzzReport};
pub use oracle::{
    build_oracle_vectors, runtime_to_json, OracleMismatch, OracleReport, OracleVector,
};
pub use runner::{run_tests, TestOutcome};

use ueas_hir::check::linker::ModuleResolver;
use ueas_hir::{compile_to_hir, ENGINE_VERSION, SPEC_VERSION};
use ueas_vm::{compile_program, VmConfig};

/// Options controlling a progressive verification run.
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    pub target_level: VerificationLevel,
    pub fuzz_iterations: usize,
    pub seed: u64,
    pub scales: Vec<usize>,
    pub vm_config: VmConfig,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            target_level: VerificationLevel::L3,
            fuzz_iterations: 64,
            seed: 0x5545_4153_2026_0001,
            scales: vec![8, 16, 32, 64, 128, 256],
            vm_config: VmConfig::default(),
        }
    }
}

/// Run progressive verification up to `options.target_level` and emit a SHA-256 `VerificationCertificate`.
pub fn verify_source(
    source: &str,
    resolver: &dyn ModuleResolver,
    options: &VerifyOptions,
) -> VerificationCertificate {
    let source_sha256 = VerificationCertificate::sha256_hex(source.as_bytes());
    let program = match compile_to_hir(source, resolver) {
        Ok(p) => p,
        Err(errs) => {
            let diagnostics = errs
                .into_iter()
                .map(|msg| RepairDiagnostic {
                    code: "L0_CHECK_ERROR".to_string(),
                    level: VerificationLevel::L0,
                    message: msg,
                    span: None,
                    counterexample: None,
                    suggested_fix: Some(
                        "Fix syntax, type, MVS, or CT static lattice errors".to_string(),
                    ),
                })
                .collect();
            let mut cert = VerificationCertificate {
                schema_version: SPEC_VERSION.to_string(),
                engine_version: ENGINE_VERSION.to_string(),
                module: None,
                source_sha256,
                hir_sha256: String::new(),
                achieved_level: VerificationLevel::L0,
                passed: false,
                tests: Vec::new(),
                fuzz_reports: Vec::new(),
                complexity_reports: Vec::new(),
                constant_time_reports: Vec::new(),
                diagnostics,
                certificate_sha256: String::new(),
            };
            cert.seal();
            return cert;
        }
    };

    let hir_bytes = serde_json::to_vec(&program).unwrap_or_default();
    let hir_sha256 = VerificationCertificate::sha256_hex(&hir_bytes);
    let compiled = match compile_program(&program) {
        Ok(c) => c,
        Err(msg) => {
            let mut cert = VerificationCertificate {
                schema_version: SPEC_VERSION.to_string(),
                engine_version: ENGINE_VERSION.to_string(),
                module: program.module.clone(),
                source_sha256,
                hir_sha256,
                achieved_level: VerificationLevel::L0,
                passed: false,
                tests: Vec::new(),
                fuzz_reports: Vec::new(),
                complexity_reports: Vec::new(),
                constant_time_reports: Vec::new(),
                diagnostics: vec![RepairDiagnostic {
                    code: "L0_COMPILE_ERROR".to_string(),
                    level: VerificationLevel::L0,
                    message: msg,
                    span: None,
                    counterexample: None,
                    suggested_fix: None,
                }],
                certificate_sha256: String::new(),
            };
            cert.seal();
            return cert;
        }
    };

    let mut achieved = VerificationLevel::L0;
    let mut diagnostics = Vec::new();
    let mut tests = Vec::new();
    let mut fuzz_reports = Vec::new();
    let mut complexity_reports = Vec::new();
    let mut constant_time_reports = Vec::new();

    if options.target_level >= VerificationLevel::L1 {
        let (t_out, t_diag) = run_tests(&compiled, &options.vm_config);
        tests = t_out;
        let l1_ok = t_diag.is_empty();
        diagnostics.extend(t_diag);
        if l1_ok {
            achieved = VerificationLevel::L1;
        }
    }

    if options.target_level >= VerificationLevel::L2 && achieved >= VerificationLevel::L1 {
        let mut l2_ok = true;
        for algo in &program.algorithms {
            let (fr, fd) = fuzz_algorithm(&compiled, algo, options.fuzz_iterations, options.seed);
            if !fr.passed {
                l2_ok = false;
            }
            fuzz_reports.push(fr);
            diagnostics.extend(fd);

            let (ct, ctd) = verify_constant_time(&compiled, algo);
            if !ct.passed {
                l2_ok = false;
            }
            constant_time_reports.push(ct);
            diagnostics.extend(ctd);
        }
        if l2_ok {
            achieved = VerificationLevel::L2;
        }
    }

    if options.target_level >= VerificationLevel::L3 && achieved >= VerificationLevel::L2 {
        let mut l3_ok = true;
        for algo in &program.algorithms {
            let (cr, cd) = verify_complexity(&compiled, algo, &options.scales);
            if !cr.passed {
                l3_ok = false;
            }
            complexity_reports.push(cr);
            diagnostics.extend(cd);
        }
        if l3_ok {
            achieved = VerificationLevel::L3;
        }
    }

    let passed = achieved >= options.target_level && diagnostics.is_empty();
    let mut cert = VerificationCertificate {
        schema_version: SPEC_VERSION.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        module: program.module,
        source_sha256,
        hir_sha256,
        achieved_level: achieved,
        passed,
        tests,
        fuzz_reports,
        complexity_reports,
        constant_time_reports,
        diagnostics,
        certificate_sha256: String::new(),
    };
    cert.seal();
    cert
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use ueas_hir::check::linker::NullModuleResolver;
    use ueas_hir::complexity::AsymptoticClass;

    #[test]
    fn binary_search_passes_l3_verification_and_emits_deterministic_cert() {
        let src = r#"
module std.searching

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

@Test "finds middle element"
    let arr <- [1, 3, 5, 7, 9]
    let res <- BinarySearch(arr, 7)
    assert res == Some(3)
end Test
"#;
        let opts = VerifyOptions::default();
        let c1 = verify_source(src, &NullModuleResolver, &opts);
        let c2 = verify_source(src, &NullModuleResolver, &opts);
        assert!(c1.passed, "diagnostics: {:?}", c1.diagnostics);
        assert_eq!(c1.achieved_level, VerificationLevel::L3);
        assert_eq!(c1.certificate_sha256, c2.certificate_sha256);
        assert!(!c1.certificate_sha256.is_empty());
    }

    #[test]
    fn adversarial_fitter_rejects_quadratic_claiming_linearithmic() {
        let src = r#"
module test.bad_complexity

Algorithm FakeFast(data: List<Int>) -> Int
    Complexity:
        Time: O(N log N) where N = data.length,
        Space: O(1)

    let mut sum <- 0
    for i in 0..data.length do
        for j in 0..data.length do
            sum <- sum + data[i] + data[j]
        end for
    end for
    return sum
end Algorithm
"#;
        let opts = VerifyOptions {
            scales: vec![8, 16, 32, 64],
            fuzz_iterations: 8,
            ..VerifyOptions::default()
        };
        let cert = verify_source(src, &NullModuleResolver, &opts);
        assert!(!cert.passed);
        assert!(cert
            .diagnostics
            .iter()
            .any(|d| d.code == "L3_COMPLEXITY_TIME_MISMATCH"));
        assert_eq!(
            cert.complexity_reports[0].fitted_time,
            AsymptoticClass::Quadratic
        );
    }

    #[test]
    fn fuzzer_catches_and_shrinks_postcondition_bug() {
        let src = r#"
module test.fuzz_bug

Algorithm BrokenAbs(x: Int) -> Int
    Ensure(res):
        res >= 0
    Complexity:
        Time: O(1),
        Space: O(1)

    return x
end Algorithm
"#;
        let opts = VerifyOptions {
            target_level: VerificationLevel::L2,
            fuzz_iterations: 32,
            ..VerifyOptions::default()
        };
        let cert = verify_source(src, &NullModuleResolver, &opts);
        assert!(!cert.passed);
        assert!(cert
            .diagnostics
            .iter()
            .any(|d| d.code == "L2_T_POSTCONDITION"));
        assert!(cert.fuzz_reports[0].minimal_counterexample.is_some());
    }

    #[test]
    fn certificate_verify_against_detects_tampering() {
        let src = r#"
module test.cert
Algorithm Id(x: Int) -> Int
    Complexity:
        Time: O(1),
        Space: O(1)
    return x
end Algorithm
"#;
        let cert = verify_source(src, &NullModuleResolver, &VerifyOptions::default());
        assert!(cert.verify_against(src, &NullModuleResolver).is_ok());
        assert!(cert
            .verify_against("module tampered\n", &NullModuleResolver)
            .is_err());
    }

    #[test]
    fn fail_closed_ct_rejects_runtime_trap() {
        let src = r#"
module test.ct_trap
@ConstantTime
Algorithm BadCt(s: Secret<Int>) -> Int
    Complexity:
        Time: O(1),
        Space: O(1)
    return 1 / 0
end Algorithm
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR");
        let compiled = compile_program(&prog).expect("VM");
        let (rep, diags) = verify_constant_time(&compiled, &prog.algorithms[0]);
        assert!(!rep.passed);
        assert!(diags.iter().any(|d| d.code == "CT_DYNAMIC_EXEC_ERROR"));
    }
}
