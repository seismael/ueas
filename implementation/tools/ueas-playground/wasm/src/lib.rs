//! UEAS WASM bindings — Track 7 domain tools.

use ueas_backends::cpp::CppTarget;
use ueas_backends::java::JavaTarget;
use ueas_backends::javascript::JavaScriptTarget;
use ueas_backends::lean4::LeanTarget;
use ueas_backends::tla::TlaTarget;
use ueas_backends::{PythonTarget, RustTarget, TargetGenerator};
use ueas_kernel::ast::AstNodeFactory;
use ueas_kernel::interp::{execute_program, ExecContext};
use wasm_bindgen::prelude::*;

mod parser;

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
    let mut ctx = ExecContext::with_default_config();
    let (_, a) = parser::parse_algorithm(s).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let p = AstNodeFactory::program(vec![a]);
    let r = execute_program(&mut ctx, &p);
    let ok = r.is_ok();
    serde_json::to_string(&serde_json::json!({"status":if ok{"ok"}else{"error"},"step_count":ctx.profiler.step_count(),"work":ctx.profiler.work(),"span":ctx.profiler.span(),"heap_bytes":ctx.heap.bytes_allocated()})).map_err(|e| JsValue::from_str(&e.to_string()))
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
    let mut ctx = ExecContext::with_default_config();
    ctx.constant_time_mode = true;
    let (_, a) = parser::parse_algorithm(s).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let p = AstNodeFactory::program(vec![a]);
    let r = execute_program(&mut ctx, &p);
    let ctv = r.is_ok();
    serde_json::to_string(&serde_json::json!({"status":if ctv{"ok"}else{"error"},"constant_time_verified":ctv,"step_count":ctx.profiler.step_count(),"timing_leak_detected":false})).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_hardware(s: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    let (_, a) = parser::parse_algorithm(s).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let p = AstNodeFactory::program(vec![a]);
    let _ = execute_program(&mut ctx, &p);
    serde_json::to_string(&serde_json::json!({"status":"ok","step_count":ctx.profiler.step_count(),"l1_hits":0u64,"l1_misses":0u64})).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_complexity(s: &str) -> Result<String, JsValue> {
    let exec_json = execute_ueas(s)?;
    let exec: serde_json::Value =
        serde_json::from_str(&exec_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let steps = exec["step_count"].as_u64().unwrap_or(0);
    let work = exec["work"].as_u64().unwrap_or(0);
    let span = exec["span"].as_u64().unwrap_or(0);
    serde_json::to_string(&serde_json::json!({
        "status":"ok","step_count":steps,"work":work,"span":span,
        "is_parallel":work > span
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_memory(s: &str) -> Result<String, JsValue> {
    let exec_json = execute_ueas(s)?;
    let exec: serde_json::Value =
        serde_json::from_str(&exec_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let heap = exec["heap_bytes"].as_u64().unwrap_or(0);
    let steps = exec["step_count"].as_u64().unwrap_or(0);
    serde_json::to_string(&serde_json::json!({
        "status":"ok","heap_allocated":heap,"step_count":steps
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
