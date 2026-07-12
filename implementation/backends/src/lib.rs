//! UEAS Transpilation Backends — plugin system for TargetGenerator implementations.
//!
//! Dafny is the single imperative transpilation target, replacing the deprecated
//! Python, Rust, C++, Java, and JavaScript direct transpilers. Dafny provides
//! Z3 theorem proving for mathematical correctness, then generates production
//! code in C++, Python, Java, Go, C#, and JavaScript via `dafny build`.
//!
//! Lean 4, TLA+, and LaTeX targets are retained for formal verification and
//! academic publishing — distinct concerns from imperative code generation.

pub mod dafny;
pub mod latex;
pub mod lean4;
pub mod tla;

pub use dafny::DafnyTarget;
pub use latex::LatexTarget;
pub use lean4::LeanTarget;
pub use tla::TlaTarget;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Error returned when transpilation fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranspilationError {
    pub message: String,
    pub node_kind: Option<String>,
}

impl TranspilationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            node_kind: None,
        }
    }

    pub fn with_node_kind(message: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            node_kind: Some(kind.into()),
        }
    }
}

/// GoF Strategy — every transpilation target implements this trait.
///
/// Each implementation translates the canonical UEAS AST into idiomatic
/// source code for a specific target language. The kernel selects the
/// appropriate strategy at transpile time.
///
/// # Semantic Equivalence Guarantee (SPEC.md Section 7.3)
///
/// Two transpiled programs generated from the same AST for different
/// targets must produce mathematically identical outputs on identical
/// inputs. The cross-target equivalence test suite verifies this.
pub trait TargetGenerator {
    /// Returns the target language identifier (e.g., "python", "rust", "cpp").
    fn language(&self) -> &str;

    /// Returns the target language version string.
    fn version(&self) -> &str;

    /// Transpile a validated UEAS AST into target source code.
    ///
    /// The input AST MUST have passed kernel validation.
    fn generate(&self, ast_json: &str) -> Result<String, TranspilationError>;

    /// Transpile a full program AST into a complete source file.
    fn generate_program(&self, ast_json: &str) -> Result<String, TranspilationError> {
        self.generate(ast_json)
    }

    /// Returns the set of UEAS AST node kinds this target supports.
    fn supported_kinds(&self) -> Vec<&str>;

    /// Returns a type mapping from UEAS primitive types to target-language
    /// native types.
    fn type_map(&self) -> Vec<(&str, &str)>;
}

// Imperative transpilation replaced by DafnyTarget (backends/src/dafny.rs).
// See docs/adr/ for the architectural decision record.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dafny_target_available() {
        let t = DafnyTarget;
        assert_eq!(t.language(), "dafny");
    }
    #[test]
    fn lean_target_available() {
        let t = LeanTarget;
        assert_eq!(t.language(), "lean4");
    }
    #[test]
    fn tla_target_available() {
        let t = TlaTarget::default();
        assert_eq!(t.language(), "tlaplus");
    }
    #[test]
    fn latex_target_available() {
        let t = LatexTarget;
        assert_eq!(t.language(), "latex");
    }
}
