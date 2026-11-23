//! Axiom 1 Purity Auditor: rejects host I/O, network, filesystem, clock, and unsafe constructs.

use serde::{Deserialize, Serialize};

/// Axiom 1 purity violation discovered during extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurityViolation {
    pub line: usize,
    pub symbol: String,
    pub reason: String,
}

const FORBIDDEN_CALLS: &[(&str, &str)] = &[
    (
        "open",
        "Host filesystem I/O violates UEAS Axiom 1 (Pure Algorithmic Substrate)",
    ),
    ("fs", "Host filesystem module access violates UEAS Axiom 1"),
    ("std::fs", "Host filesystem access violates UEAS Axiom 1"),
    ("std::net", "Host network access violates UEAS Axiom 1"),
    ("reqwest", "Host HTTP client violates UEAS Axiom 1"),
    ("fetch", "Host network fetch violates UEAS Axiom 1"),
    ("socket", "Host socket I/O violates UEAS Axiom 1"),
    (
        "SystemTime",
        "Non-deterministic wall clock violates UEAS Axiom 1",
    ),
    (
        "Date.now",
        "Non-deterministic wall clock violates UEAS Axiom 1",
    ),
    (
        "time.time",
        "Non-deterministic wall clock violates UEAS Axiom 1",
    ),
    ("unsafe", "Unsafe memory block violates UEAS Axiom 1"),
];

/// Scan an identifier or call expression text for Axiom 1 violations.
pub fn check_symbol_purity(symbol: &str, line: usize, out: &mut Vec<PurityViolation>) {
    for &(pat, reason) in FORBIDDEN_CALLS {
        if symbol == pat
            || symbol.starts_with(&format!("{pat}."))
            || symbol.starts_with(&format!("{pat}::"))
        {
            out.push(PurityViolation {
                line,
                symbol: symbol.to_string(),
                reason: reason.to_string(),
            });
            return;
        }
    }
}
