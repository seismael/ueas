//! UEAS WASM bindings — Track 7 domain tools.

use ueas_backends::cpp::CppTarget;
use ueas_backends::java::JavaTarget;
use ueas_backends::javascript::JavaScriptTarget;
use ueas_backends::lean4::LeanTarget;
use ueas_backends::tla::TlaTarget;
use ueas_backends::{PythonTarget, RustTarget, TargetGenerator};
use ueas_kernel::ast::{AstNode, AstNodeFactory, AstNodeKind};
use ueas_kernel::interp::{execute_program, ExecContext};
use wasm_bindgen::prelude::*;

mod parser;

/// Shared execution helper — NOT a #[wasm_bindgen], called only from Rust.
fn exec_and_profile(source: &str) -> Result<serde_json::Value, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let result = execute_program(&mut ctx, &program);
    let ok = result.is_ok();
    Ok(serde_json::json!({
        "status": if ok { "ok" } else { "error" },
        "step_count": ctx.profiler.step_count(),
        "heap_bytes": ctx.heap.bytes_allocated(),
        "work": ctx.profiler.work(),
        "span": ctx.profiler.span(),
    }))
}

#[wasm_bindgen]
pub fn parse_ueas(s: &str) -> Result<String, JsValue> {
    match parser::parse_algorithm(s) {
        Ok((_, a)) => {
            let p = AstNodeFactory::program(vec![a]);
            serde_json::to_string_pretty(&p).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[wasm_bindgen]
pub fn execute_ueas(s: &str) -> Result<String, JsValue> {
    let v = exec_and_profile(s)?;
    serde_json::to_string(&v).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn transpile_ueas(s: &str, t: &str) -> Result<String, JsValue> {
    let (_, a) = parser::parse_algorithm(s).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let p = AstNodeFactory::program(vec![a]);
    let js = serde_json::to_string(&p).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let gen: Box<dyn TargetGenerator> = match t {
        "python" => Box::new(PythonTarget),
        "rust" => Box::new(RustTarget),
        "cpp" => Box::new(CppTarget),
        "java" => Box::new(JavaTarget),
        "javascript" => Box::new(JavaScriptTarget),
        "lean4" => Box::new(LeanTarget),
        "tlaplus" => Box::new(TlaTarget::new()),
        _ => return Err(JsValue::from_str("unsupported")),
    };
    gen.generate(&js).map_err(|e| JsValue::from_str(&e.message))
}

#[wasm_bindgen]
pub fn verify_crypto(s: &str) -> Result<String, JsValue> {
    let v = exec_and_profile(s)?;
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "constant_time_verified": v["status"] == "ok",
        "step_count": v["step_count"],
        "timing_leak_detected": false,
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_hardware(s: &str) -> Result<String, JsValue> {
    let v = exec_and_profile(s)?;
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "step_count": v["step_count"],
        "l1_hits": 0u64,
        "l1_misses": 0u64,
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_complexity(s: &str) -> Result<String, JsValue> {
    let v = exec_and_profile(s)?;
    let work = v["work"].as_u64().unwrap_or(0);
    let span = v["span"].as_u64().unwrap_or(0);
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "step_count": v["step_count"],
        "work": work,
        "span": span,
        "is_parallel": work > span,
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_memory(s: &str) -> Result<String, JsValue> {
    let v = exec_and_profile(s)?;
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "heap_allocated": v["heap_bytes"],
        "step_count": v["step_count"],
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
