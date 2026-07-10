//! UEAS WASM bindings — expose kernel functions to JavaScript.
//!
//! Build: cd tools/ueas-playground/wasm && wasm-pack build --target web
//! Requires: cargo install wasm-pack

use ueas_backends::{PythonTarget, RustTarget, TargetGenerator};
use ueas_kernel::ast::{AstNode, AstNodeFactory, AstNodeKind};
use ueas_kernel::interp::{execute_program, ExecContext};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_ueas(source: &str) -> Result<String, JsValue> {
    // Stub: full parser integration requires ANTLR4 bridge
    Ok(format!("parsed {} bytes", source.len()))
}

#[wasm_bindgen]
pub fn execute_ueas(_source: &str) -> Result<String, JsValue> {
    let mut ctx = ExecContext::with_default_config();
    let algo = AstNode::internal(
        AstNodeKind::Algorithm,
        vec![
            AstNodeFactory::identifier("wasm"),
            AstNodeFactory::string_literal("O(1)"),
            AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("42"))),
        ],
        None,
    );
    let program = AstNodeFactory::program(vec![algo]);
    match execute_program(&mut ctx, &program) {
        Ok(result) => Ok(format!("{:?}", result)),
        Err(e) => Err(JsValue::from_str(&e.name())),
    }
}

#[wasm_bindgen]
pub fn transpile_ueas(_source: &str, target: &str) -> Result<String, JsValue> {
    let algo = AstNode::internal(
        AstNodeKind::Algorithm,
        vec![
            AstNodeFactory::identifier("wasm"),
            AstNodeFactory::string_literal("O(1)"),
            AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("42"))),
        ],
        None,
    );
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
        _ => Err(JsValue::from_str("unsupported target")),
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    // Initialize panic hook for better errors
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
