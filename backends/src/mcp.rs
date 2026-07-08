//! MCP (Model Context Protocol) endpoint for the UEAS transpilation backend.
//!
//! Per SPEC.md Section 7.5, the transpilation back-end MUST expose a
//! standard MCP API endpoint to allow autonomous AI agents to ingest
//! a canonical AST and produce target-language code.
//!
//! # Endpoint
//! POST /mcp/v1/transpile
//!
//! # Request Body
//! { "ast": <Program>, "target": <language>, "options": {...} }
//!
//! # Response Body
//! { "source": <string>, "warnings": [...], "target_version": <string> }

use crate::TargetGenerator;
use serde::{Deserialize, Serialize};

/// MCP transpilation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTranspileRequest {
    /// The canonical UEAS AST (JSON Program node).
    pub ast: serde_json::Value,
    /// Target language identifier (e.g., "python", "rust").
    pub target: String,
    /// Optional transpilation options (e.g., optimization level).
    #[serde(default)]
    pub options: McpTranspileOptions,
}

/// Options for MCP transpilation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpTranspileOptions {
    /// Optimization level (0 = none, 1 = basic, 2 = aggressive).
    #[serde(default)]
    pub optimization_level: u8,
    /// Whether to include source comments in generated code.
    #[serde(default)]
    pub include_comments: bool,
}

/// MCP transpilation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTranspileResponse {
    /// The generated target-language source code.
    pub source: String,
    /// Any warnings produced during transpilation.
    #[serde(default)]
    pub warnings: Vec<McpWarning>,
    /// The target language version used.
    pub target_version: String,
}

/// A warning produced during transpilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWarning {
    /// Warning message.
    pub message: String,
    /// AST node kind that triggered the warning (if applicable).
    pub node_kind: Option<String>,
    /// Source location that triggered the warning (if known).
    pub location: Option<McpSourceLocation>,
}

/// Source location in the original UEAS source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSourceLocation {
    pub line: usize,
    pub column: usize,
}

/// MCP transpilation error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpErrorResponse {
    /// Error code.
    pub error_code: String,
    /// Human-readable error message.
    pub message: String,
    /// AST node kind that caused the error (if applicable).
    pub node_kind: Option<String>,
}

/// Handle an MCP transpilation request.
///
/// Routes to the appropriate `TargetGenerator` based on the target language.
/// Returns the generated source code or an error response.
pub fn handle_transpile(
    request: &McpTranspileRequest,
) -> Result<McpTranspileResponse, McpErrorResponse> {
    let ast_json = serde_json::to_string(&request.ast).map_err(|e| McpErrorResponse {
        error_code: "INVALID_AST".to_string(),
        message: format!("Failed to serialize AST: {}", e),
        node_kind: None,
    })?;

    match request.target.as_str() {
        "python" => {
            let target = crate::PythonTarget;
            let source = target.generate(&ast_json).map_err(|e| McpErrorResponse {
                error_code: "TRANSPILATION_FAILED".to_string(),
                message: e.message,
                node_kind: e.node_kind,
            })?;
            Ok(McpTranspileResponse {
                source,
                warnings: vec![],
                target_version: target.version().to_string(),
            })
        }
        "rust" => {
            let target = crate::RustTarget;
            let source = target.generate(&ast_json).map_err(|e| McpErrorResponse {
                error_code: "TRANSPILATION_FAILED".to_string(),
                message: e.message,
                node_kind: e.node_kind,
            })?;
            Ok(McpTranspileResponse {
                source,
                warnings: vec![],
                target_version: target.version().to_string(),
            })
        }
        _ => Err(McpErrorResponse {
            error_code: "UNSUPPORTED_TARGET".to_string(),
            message: format!("Target '{}' is not supported", request.target),
            node_kind: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_transpile_python_expression() {
        let request = McpTranspileRequest {
            ast: serde_json::json!({
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "+"},
                    {"kind": "IntegerLiteral", "value": "1"},
                    {"kind": "IntegerLiteral", "value": "2"}
                ]
            }),
            target: "python".to_string(),
            options: McpTranspileOptions::default(),
        };
        let response = handle_transpile(&request).unwrap();
        assert_eq!(response.source, "(1 + 2)");
        assert_eq!(response.target_version, "3.11");
    }

    #[test]
    fn mcp_transpile_rust_expression() {
        let request = McpTranspileRequest {
            ast: serde_json::json!({
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "*"},
                    {"kind": "IntegerLiteral", "value": "3"},
                    {"kind": "IntegerLiteral", "value": "4"}
                ]
            }),
            target: "rust".to_string(),
            options: McpTranspileOptions::default(),
        };
        let response = handle_transpile(&request).unwrap();
        assert_eq!(response.source, "(3_i64 * 4_i64)");
        assert_eq!(response.target_version, "2021");
    }

    #[test]
    fn mcp_transpile_unsupported_target() {
        let request = McpTranspileRequest {
            ast: serde_json::json!({"kind": "IntegerLiteral", "value": "1"}),
            target: "javascript".to_string(),
            options: McpTranspileOptions::default(),
        };
        let err = handle_transpile(&request).unwrap_err();
        assert_eq!(err.error_code, "UNSUPPORTED_TARGET");
    }

    #[test]
    fn mcp_request_serialization_round_trip() {
        let request = McpTranspileRequest {
            ast: serde_json::json!({"kind": "IntegerLiteral", "value": "42"}),
            target: "python".to_string(),
            options: McpTranspileOptions {
                optimization_level: 1,
                include_comments: true,
            },
        };
        let json = serde_json::to_string(&request).unwrap();
        let restored: McpTranspileRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.target, "python");
    }
}
