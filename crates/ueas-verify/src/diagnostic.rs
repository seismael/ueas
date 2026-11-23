//! Verification levels and machine-readable repair diagnostics.

use serde::{Deserialize, Serialize};
use ueas_syntax::SourceSpan;

/// Progressive 5-level verification hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VerificationLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
}

impl VerificationLevel {
    /// Canonical string label (`"L0"`..`"L4"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L0 => "L0",
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::L4 => "L4",
        }
    }
}

/// Structured, actionable diagnostic emitted when a verification gate fails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairDiagnostic {
    pub code: String,
    pub level: VerificationLevel,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub counterexample: Option<serde_json::Value>,
    pub suggested_fix: Option<String>,
}
