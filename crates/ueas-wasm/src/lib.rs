//! UEAS v2.0 WebAssembly bindings for Browser Web Worker and Edge MCP.
#![allow(missing_docs)]

use serde::Serialize;
use ueas_codegen::{generate_target, TargetKind};
use ueas_hir::{compile_to_hir, compile_to_hir_detailed, DiagnosticItem, EmbeddedStdlibResolver};
use ueas_syntax::{format_source, parse};
use ueas_verify::{verify_source, VerificationLevel, VerifyOptions};
use ueas_vm::{compile_program, run_hir, VmConfig};
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct CheckResponse {
    ok: bool,
    errors: Vec<String>,
    diagnostics: Vec<DiagnosticItem>,
    algorithms: Vec<String>,
}

#[derive(Serialize)]
struct FormatResponse {
    ok: bool,
    formatted: Option<String>,
    errors: Vec<String>,
}

/// Parse, lower, typecheck, MVS-check, and CT-check a UEAS v2.1 source string.
#[wasm_bindgen]
pub fn ueas_check(source: &str) -> String {
    let resp = match compile_to_hir_detailed(source, &EmbeddedStdlibResolver) {
        Ok(prog) => CheckResponse {
            ok: true,
            errors: Vec::new(),
            diagnostics: Vec::new(),
            algorithms: prog.algorithms.into_iter().map(|a| a.name).collect(),
        },
        Err(diagnostics) => CheckResponse {
            ok: false,
            errors: diagnostics.iter().map(|d| d.message.clone()).collect(),
            diagnostics,
            algorithms: Vec::new(),
        },
    };
    serde_json::to_string(&resp).unwrap_or_else(|_| "{\"ok\":false}".to_string())
}

/// Format UEAS v2.0 source code using the lossless CST formatter.
#[wasm_bindgen]
pub fn ueas_format(source: &str) -> String {
    let resp = match format_source(source) {
        Ok(formatted) => FormatResponse {
            ok: true,
            formatted: Some(formatted),
            errors: Vec::new(),
        },
        Err(diags) => FormatResponse {
            ok: false,
            formatted: None,
            errors: diags
                .into_iter()
                .map(|d| {
                    format!(
                        "line {}:{}: {}",
                        d.span.start_line, d.span.start_col, d.message
                    )
                })
                .collect(),
        },
    };
    serde_json::to_string(&resp).unwrap_or_else(|_| "{\"ok\":false}".to_string())
}

/// Run progressive verification (`"L0"`, `"L1"`, `"L2"`, `"L3"`) and return a SHA-256 certificate JSON.
#[wasm_bindgen]
pub fn ueas_verify(source: &str, level: &str, fuzz_iterations: usize) -> String {
    let target_level = match level.to_ascii_uppercase().as_str() {
        "L0" => VerificationLevel::L0,
        "L1" => VerificationLevel::L1,
        "L2" => VerificationLevel::L2,
        _ => VerificationLevel::L3,
    };
    let opts = VerifyOptions {
        target_level,
        fuzz_iterations: fuzz_iterations.clamp(1, 256),
        ..VerifyOptions::default()
    };
    let cert = verify_source(source, &EmbeddedStdlibResolver, &opts);
    serde_json::to_string(&cert).unwrap_or_else(|_| "{\"passed\":false}".to_string())
}

/// Transpile UEAS v2.0 source to a Track A, B, or C target (`rust`, `python`, `typescript`, `cpp`, `go`, `dafny`, `lean4`, `verus`, `quint`, `tla`, `latex`, `typst`).
#[wasm_bindgen]
pub fn ueas_transpile(source: &str, target: &str) -> String {
    let Some(tk) = TargetKind::parse(target) else {
        return serde_json::json!({ "ok": false, "error": format!("unknown target '{target}'") })
            .to_string();
    };
    match compile_to_hir(source, &EmbeddedStdlibResolver) {
        Ok(prog) => match generate_target(&prog, tk) {
            Ok(art) => serde_json::json!({ "ok": true, "artifact": art }).to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
        },
        Err(errors) => serde_json::json!({ "ok": false, "errors": errors }).to_string(),
    }
}

/// Execute `@Test` blocks or a specific zero-arg `entry` in `ueas-vm` and return real execution metrics.
#[wasm_bindgen]
pub fn ueas_run(source: &str, entry: &str, step_budget: u64) -> String {
    let prog = match compile_to_hir(source, &EmbeddedStdlibResolver) {
        Ok(p) => p,
        Err(errors) => {
            return serde_json::json!({ "ok": false, "errors": errors }).to_string();
        }
    };
    let cfg = VmConfig {
        step_budget: step_budget.clamp(1_000, 10_000_000),
        ..VmConfig::default()
    };
    if entry.is_empty() {
        let compiled = match compile_program(&prog) {
            Ok(c) => c,
            Err(e) => return serde_json::json!({ "ok": false, "error": e }).to_string(),
        };
        let mut results = Vec::new();
        for (name, _) in &compiled.tests {
            let chunk_name = format!("test::{name}");
            match ueas_vm::VmSession::new(compiled.clone(), &chunk_name, Vec::new(), cfg.clone())
                .and_then(|mut s| s.run())
            {
                Ok(res) => results.push(serde_json::json!({
                    "name": name,
                    "passed": true,
                    "steps": res.steps,
                    "peak_live_bytes": res.heap.peak_live_bytes,
                    "cache": res.cache,
                    "work_span": res.work_span,
                })),
                Err(trap) => results.push(serde_json::json!({
                    "name": name,
                    "passed": false,
                    "error": trap.to_string(),
                })),
            }
        }
        serde_json::json!({ "ok": true, "tests": results }).to_string()
    } else {
        match run_hir(&prog, entry, Vec::new(), cfg) {
            Ok(res) => serde_json::json!({
                "ok": true,
                "value": res.value.to_string(),
                "steps": res.steps,
                "heap": res.heap,
                "cache": res.cache,
                "work_span": res.work_span,
            })
            .to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
        }
    }
}

/// Emit canonical HIR JSON for a UEAS v2.0 source string.
#[wasm_bindgen]
pub fn ueas_hir_json(source: &str) -> String {
    let parsed = parse(source);
    if !parsed.is_ok() {
        return serde_json::json!({ "ok": false, "diagnostics": parsed.diagnostics }).to_string();
    }
    match compile_to_hir(source, &EmbeddedStdlibResolver) {
        Ok(prog) => serde_json::json!({ "ok": true, "hir": prog }).to_string(),
        Err(errors) => serde_json::json!({ "ok": false, "errors": errors }).to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn wasm_api_roundtrip_check_verify_transpile() {
        let src = r#"
module wasm.demo
Algorithm Id(x: Int) -> Int
    Complexity:
        Time: O(1),
        Space: O(1)
    return x
end Algorithm
@Test "id"
    assert Id(42) == 42
end Test
"#;
        let chk: serde_json::Value = serde_json::from_str(&ueas_check(src)).unwrap();
        assert_eq!(chk["ok"], true);
        let ver: serde_json::Value = serde_json::from_str(&ueas_verify(src, "L3", 8)).unwrap();
        assert_eq!(ver["passed"], true);
        let tr: serde_json::Value = serde_json::from_str(&ueas_transpile(src, "rust")).unwrap();
        assert_eq!(tr["ok"], true);
    }
}
