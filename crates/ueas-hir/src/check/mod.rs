//! Static verification pass (Level L0): module linking, type checking, MVS, and CT security lattice.

pub mod constant_time;
pub mod linker;
pub mod mvs;
pub mod typecheck;

use crate::hir::Program;
use serde::{Deserialize, Serialize};
use ueas_syntax::SourceSpan;

/// Structured static compilation error emitted by `ueas-hir`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileError {
    pub code: &'static str,
    pub span: SourceSpan,
    pub message: String,
    pub hint: Option<String>,
}

/// Run the full Level L0 static checker pipeline on a linked `Program`.
pub fn check_program(program: &mut Program) -> Result<(), Vec<CompileError>> {
    let mut errors = Vec::new();
    typecheck::check_types(program, &mut errors);
    mvs::check_mvs(program, &mut errors);
    constant_time::check_constant_time(program, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
