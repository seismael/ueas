//! UEAS WASM bindings — expose kernel functions to JavaScript.
//!
//! Build: cd tools/ueas-playground/wasm && wasm-pack build --target web
//! Requires: cargo install wasm-pack

use ueas_backends::{PythonTarget, RustTarget, TargetGenerator};
use ueas_backends::cpp::CppTarget;
use ueas_backends::java::JavaTarget;
use ueas_backends::javascript::JavaScriptTarget;
use ueas_backends::lean4::LeanTarget;
use ueas_backends::tla::TlaTarget;
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
    let (_name, algo) = parser::parse_algorithm(source)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let program = AstNodeFactory::program(vec![algo]);
    match execute_program(&mut ctx, &program) {
        Ok(result) => Ok(format!("{:?}", result)),
        Err(e) => Err(JsValue::from_str(&e.name())),
    }
}

#[wasm_bindgen]
pub fn transpile_ueas(source: &str, target: &str) -> Result<String, JsValue> {
    let (_name, algo) = parser::parse_algorithm(source)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
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
    // Initialize panic hook for better errors
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
