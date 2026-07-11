//! UEAS WASM bindings — expose kernel functions to JavaScript.
//!
//! Build: cd tools/ueas-playground/wasm && wasm-pack build --target web
//! Requires: cargo install wasm-pack

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
pub fn parse_ueas(source: &str) -> Result<String, JsValue> {
    match parser::parse_algorithm(source) {
        Ok((_name, algo)) => {
            let program = AstNodeFactory::program(vec![algo]);
            serde_json::to_string_pretty(&program).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[wasm_bindgen]
pub fn execute_ueas(source: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let exec_result = execute_program(&mut ctx, &program);

    let status = if exec_result.is_ok() { "ok" } else { "error" };
    let exit_code = match &exec_result {
        Ok(_) => 0i32,
        Err(e) => *e as i32,
    };
    let exit_name = match &exec_result {
        Ok(_) => "NoError",
        Err(e) => e.name(),
    };
    let result_val = match &exec_result {
        Ok(v) => format!("{:?}", v),
        Err(_) => "trap".to_string(),
    };

    serde_json::to_string(&serde_json::json!({
        "status": status,
        "exit_code": exit_code,
        "exit_name": exit_name,
        "step_count": ctx.profiler.step_count(),
        "heap_bytes": ctx.heap.bytes_allocated(),
        "result": result_val,
        "work": ctx.profiler.work(),
        "span": ctx.profiler.span(),
        "parallel_efficiency": ctx.profiler.parallel_efficiency(),
        "cache_l1_hits": ctx.heap.cache_stats().l1_hits,
        "cache_l1_misses": ctx.heap.cache_stats().l1_misses,
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn transpile_ueas(source: &str, target: &str) -> Result<String, JsValue> {
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let ast_json =
        serde_json::to_string(&program).map_err(|e| JsValue::from_str(&e.to_string()))?;

    match target {
        "python" => PythonTarget
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        "rust" => RustTarget
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        "cpp" => CppTarget
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        "java" => JavaTarget
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        "javascript" => JavaScriptTarget
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        "lean4" => LeanTarget
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        "tlaplus" => TlaTarget::new()
            .generate(&ast_json)
            .map_err(|e| JsValue::from_str(&e.message)),
        _ => Err(JsValue::from_str("unsupported target")),
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Track 7: Domain-Expert MCP Tools

#[wasm_bindgen]
pub fn verify_crypto(source: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    ctx.constant_time_mode = true;
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let result = execute_program(&mut ctx, &program);
    match result {
        Ok(_) => Ok(serde_json::to_string(&serde_json::json!({
            "status": "ok",
            "constant_time_verified": true,
            "secret_variables_found": ctx.secret_variables.len(),
            "step_count": ctx.profiler.step_count(),
            "timing_leak_detected": false,
        }))
        .map_err(|e| JsValue::from_str(&e.to_string()))?),
        Err(e) => Ok(serde_json::to_string(&serde_json::json!({
            "status": "error",
            "constant_time_verified": false,
            "trap_code": e as i32,
            "trap_name": e.name(),
            "step_count": ctx.profiler.step_count(),
            "timing_leak_detected": matches!(e, ueas_kernel::traps::ExitCode::TimingLeak),
        }))
        .map_err(|e| JsValue::from_str(&e.to_string()))?),
    }
}

#[wasm_bindgen]
pub fn profile_hardware(source: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    ctx.heap.cache_config.enabled = true;
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let _ = execute_program(&mut ctx, &program);
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "step_count": ctx.profiler.step_count(),
        "cache_l1_hits": ctx.heap.cache_stats().l1_hits,
        "cache_l1_misses": ctx.heap.cache_stats().l1_misses,
        "cache_l2_hits": ctx.heap.cache_stats().l2_hits,
        "cache_l2_misses": ctx.heap.cache_stats().l2_misses,
        "cache_l3_hits": ctx.heap.cache_stats().l3_hits,
        "cache_l3_misses": ctx.heap.cache_stats().l3_misses,
        "total_accesses": ctx.heap.cache_stats().total_accesses(),
        "miss_penalty": ctx.heap.cache_stats().cache_miss_penalty(),
        "l1_size_bytes": ctx.heap.cache_config.l1_size,
        "cache_line_bytes": ctx.heap.cache_config.cache_line_size,
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_complexity(source: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let _ = execute_program(&mut ctx, &program);
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "step_count": ctx.profiler.step_count(),
        "work": ctx.profiler.work(),
        "span": ctx.profiler.span(),
        "parallel_efficiency": ctx.profiler.parallel_efficiency(),
        "is_parallel": ctx.profiler.work() > ctx.profiler.span(),
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn profile_memory(source: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    let (_name, algo) =
        parser::parse_algorithm(source).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    let _ = execute_program(&mut ctx, &program);
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "heap_allocated": ctx.heap.bytes_allocated(),
        "heap_peak": ctx.heap.bytes_allocated(),
        "allocations": 0u64,
        "step_count": ctx.profiler.step_count(),
    }))
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
