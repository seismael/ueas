//! Execution trace events for dynamic observational constant-time verification.

use serde::{Deserialize, Serialize};

/// Single observable microarchitectural trace event emitted by the VM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub pc: usize,
    pub cache_line: Option<u64>,
    pub branch_taken: Option<bool>,
}
