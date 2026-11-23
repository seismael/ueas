//! Shared `CodeGenerator` trait and artifact types for UEAS dual-track code generation.

use serde::{Deserialize, Serialize};
use ueas_hir::hir::Program;

/// Generated source artifact produced by a `CodeGenerator`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub target: String,
    pub extension: String,
    pub code: String,
    pub test_code: Option<String>,
}

/// Polymorphic code-generation trait implemented by every Track A, B, and C target.
pub trait CodeGenerator: Send + Sync {
    fn target_id(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
    fn generate(&self, program: &Program) -> Result<GeneratedArtifact, String>;
}

/// Helper to convert PascalCase or CamelCase identifiers to snake_case when needed.
pub fn to_snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Indent helper for clean generated formatting.
pub fn indent(level: usize) -> String {
    "    ".repeat(level)
}
