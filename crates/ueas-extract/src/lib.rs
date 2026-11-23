//! UEAS Tree-sitter multi-language algorithm extractors (Python, TypeScript, Rust) & Axiom 1 auditor.
#![allow(missing_docs)]

pub mod purity;
pub mod python;
pub mod rust_lang;
pub mod typescript;

pub use purity::PurityViolation;
pub use python::PythonExtractor;
pub use rust_lang::RustExtractor;
pub use typescript::TypeScriptExtractor;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned by source extractors.
#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("failed to initialize tree-sitter grammar: {0}")]
    ParserInit(String),
    #[error("failed to parse source: {0}")]
    ParseFailed(String),
    #[error("unsupported source language: {0}")]
    UnsupportedLanguage(String),
}

/// Extracted UEAS v2.0 source and Axiom 1 purity audit report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionReport {
    pub ueas_source: String,
    pub extracted_algorithms: Vec<String>,
    pub axiom1_violations: Vec<PurityViolation>,
}

/// Polymorphic extractor trait (`Extractor`) per `AGENTS.md`.
pub trait Extractor: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn extract(&self, source: &str) -> Result<ExtractionReport, ExtractError>;
}

/// Dispatch extraction for `"python"`, `"typescript"`, or `"rust"`.
pub fn extract_source(language: &str, source: &str) -> Result<ExtractionReport, ExtractError> {
    let extractor: Box<dyn Extractor> = match language.to_ascii_lowercase().as_str() {
        "python" | "py" => Box::new(PythonExtractor),
        "typescript" | "ts" => Box::new(TypeScriptExtractor),
        "rust" | "rs" => Box::new(RustExtractor),
        other => return Err(ExtractError::UnsupportedLanguage(other.to_string())),
    };
    extractor.extract(source)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn extracts_pure_python_and_rejects_host_io() {
        let py = r#"
def add_loop(n: int) -> int:
    s = 0
    for i in range(0, n):
        s = s + i
    return s
"#;
        let rep = extract_source("python", py).expect("extract py");
        assert!(rep.axiom1_violations.is_empty());
        assert_eq!(rep.extracted_algorithms, vec!["add_loop"]);
        assert!(ueas_syntax::parse(&rep.ueas_source).is_ok());

        let dirty_py = r#"
def read_secret(path: str) -> int:
    f = open(path)
    return 1
"#;
        let rep2 = extract_source("python", dirty_py).expect("extract dirty py");
        assert!(!rep2.axiom1_violations.is_empty());
    }

    #[test]
    fn extracts_typescript_and_rust_to_parseable_ueas() {
        let ts = "export function doubleVal(x: number): number { return x * 2; }";
        let rep_ts = extract_source("typescript", ts).expect("extract ts");
        assert!(rep_ts.axiom1_violations.is_empty());
        assert!(ueas_syntax::parse(&rep_ts.ueas_source).is_ok());

        let rs = "pub fn triple(x: i64) -> i64 { return x * 3; }";
        let rep_rs = extract_source("rust", rs).expect("extract rs");
        assert!(rep_rs.axiom1_violations.is_empty());
        assert!(ueas_syntax::parse(&rep_rs.ueas_source).is_ok());
    }
}
