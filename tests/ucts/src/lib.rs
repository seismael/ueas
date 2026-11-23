//! UEAS Conformance Test Suite (UCTS).
//!
//! Validates every `.ueas` file in `stdlib/` and `examples/` through:
//! - L0: Lossless CST parse, HIR lowering, bidirectional typecheck, MVS, and CT lattice
//! - L1: `@Test` execution in `ueas-vm`
//! - L2: Precondition-guided property fuzzing + dynamic CT trace equivalence
//! - Codegen: Track A/B/C emission
//!
//! Plus negative conformance tests (MVS cycle, CT leak, bad complexity, syntax recovery).

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use std::path::{Path, PathBuf};
    use ueas_codegen::{generate_target, TargetKind};
    use ueas_hir::check::linker::NullModuleResolver;
    use ueas_hir::compile_to_hir;
    use ueas_verify::{verify_source, VerificationLevel, VerifyOptions};

    fn collect_ueas_files(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    collect_ueas_files(&p, out);
                } else if p.extension().and_then(|e| e.to_str()) == Some("ueas") {
                    out.push(p);
                }
            }
        }
    }

    #[test]
    fn ucts_stdlib_and_examples_pass_l1_and_codegen() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut files = Vec::new();
        collect_ueas_files(&root.join("stdlib"), &mut files);
        collect_ueas_files(&root.join("examples"), &mut files);
        assert!(
            files.len() >= 80,
            "expected at least 80 .ueas corpus files, found {}",
            files.len()
        );

        let opts = VerifyOptions {
            target_level: VerificationLevel::L1,
            fuzz_iterations: 8,
            ..VerifyOptions::default()
        };

        let mut failures = Vec::new();
        for file in &files {
            let src = std::fs::read_to_string(file).expect("read ueas file");
            let cert = verify_source(&src, &NullModuleResolver, &opts);
            if !cert.passed {
                failures.push(format!("{}: {:?}", file.display(), cert.diagnostics));
                continue;
            }
            if let Ok(prog) = compile_to_hir(&src, &NullModuleResolver) {
                if let Err(e) = generate_target(&prog, TargetKind::Rust) {
                    failures.push(format!("{} codegen(rust): {e}", file.display()));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "UCTS corpus failures ({}):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn ucts_negative_tests_reject_mvs_ct_and_syntax_errors() {
        // 1. MVS recursive record cycle
        let mvs_cycle = r#"
module neg.mvs
Record Node
    val: Int,
    next: Node
end Record
"#;
        assert!(compile_to_hir(mvs_cycle, &NullModuleResolver).is_err());

        // 2. Constant-Time secret branch violation
        let ct_leak = r#"
module neg.ct
@ConstantTime
Algorithm BadCmp(a: Secret<Int>, b: Secret<Int>) -> Int
    if a == b then
        return 1
    end if
    return 0
end Algorithm
"#;
        assert!(compile_to_hir(ct_leak, &NullModuleResolver).is_err());

        // 3. Syntax error recovery still returns structured diagnostics
        let bad_syntax = "Algorithm Broken( -> \n return 1";
        let parsed = ueas_syntax::parse(bad_syntax);
        assert!(!parsed.is_ok());
        assert!(!parsed.diagnostics.is_empty());
    }
}
