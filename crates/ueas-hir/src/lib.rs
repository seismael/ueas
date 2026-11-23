//! UEAS typed canonical HIR, module linker, bidirectional type checker, MVS, and CT security lattice.
#![allow(missing_docs)]

pub mod check;
pub mod complexity;
pub mod hir;
pub mod lower;
pub mod pretty;
pub mod stdlib_bundle;
pub mod types;

pub use check::linker::{link_program, ChainedModuleResolver, ModuleResolver, NullModuleResolver};
pub use check::{check_program, CompileError};
pub use complexity::{
    extract_complexity_vars, parse_complexity_bound, AsymptoticClass, ComplexityBound,
    ComplexitySpec,
};
pub use hir::*;
pub use lower::{lower_parse, ENGINE_VERSION, SPEC_VERSION};
pub use pretty::{fmt_expr, fmt_type, pretty_program};
pub use stdlib_bundle::EmbeddedStdlibResolver;
pub use types::{PrimitiveType, SecurityLevel, Type};

use serde::{Deserialize, Serialize};
use ueas_syntax::SourceSpan;

/// Structured diagnostic item preserving exact `SourceSpan` for LSP/IDE/CLI consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub code: String,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub hint: Option<String>,
}

/// Parse, lower, link, and L0-check a UEAS source string, returning span-accurate diagnostics.
pub fn compile_to_hir_detailed(
    source: &str,
    resolver: &dyn ModuleResolver,
) -> Result<Program, Vec<DiagnosticItem>> {
    let parsed = ueas_syntax::parse(source);
    if !parsed.is_ok() {
        return Err(parsed
            .diagnostics
            .into_iter()
            .map(|d| DiagnosticItem {
                code: "E_SYNTAX".to_string(),
                message: d.message,
                span: Some(d.span),
                hint: d.hint,
            })
            .collect());
    }
    let prog = lower_parse(&parsed).map_err(|errs| {
        errs.into_iter()
            .map(|message| DiagnosticItem {
                code: "E_LOWER".to_string(),
                message,
                span: None,
                hint: None,
            })
            .collect::<Vec<_>>()
    })?;
    let mut linked = link_program(prog, resolver).map_err(|errs| {
        errs.into_iter()
            .map(|(sp, message)| DiagnosticItem {
                code: "E_LINK".to_string(),
                message,
                span: Some(sp),
                hint: None,
            })
            .collect::<Vec<_>>()
    })?;
    check_program(&mut linked).map_err(|errs| {
        errs.into_iter()
            .map(|e| DiagnosticItem {
                code: e.code.to_string(),
                message: e.message,
                span: Some(e.span),
                hint: e.hint,
            })
            .collect::<Vec<_>>()
    })?;
    Ok(linked)
}

/// Convenience helper: parse, lower, link (with resolver), and L0-check a UEAS source string.
pub fn compile_to_hir(source: &str, resolver: &dyn ModuleResolver) -> Result<Program, Vec<String>> {
    compile_to_hir_detailed(source, resolver).map_err(|items| {
        items
            .into_iter()
            .map(|d| match d.span {
                Some(sp) => format!(
                    "[{}] line {}:{}: {}",
                    d.code, sp.start_line, sp.start_col, d.message
                ),
                None => format!("[{}] {}", d.code, d.message),
            })
            .collect()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn compile_binary_search_hir_and_json_roundtrip() {
        let src = r#"
Algorithm BinarySearch(data: List<Int>, target: Int) -> Option<Int>
    Require:
        data.length >= 0
    Ensure(res):
        res == None or true
    Complexity:
        Time: O(log N) where N = data.length,
        Space: O(1)

    low <- 0
    high <- data.length - 1
    while low <= high do
        invariant 0 <= low
        decreases high - low + 1
        mid <- low + (high - low) / 2
        if data[mid] == target then
            return Some(mid)
        else if data[mid] < target then
            low <- mid + 1
        else
            high <- mid - 1
        end if
    end while
    return None
end Algorithm
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("valid HIR");
        assert_eq!(prog.spec_version, "2.0.0");
        assert_eq!(prog.algorithms.len(), 1);
        let json = serde_json::to_string_pretty(&prog).unwrap();
        let back: Program = serde_json::from_str(&json).unwrap();
        assert_eq!(prog, back);
    }

    #[test]
    fn rejects_const_reassignment() {
        let src = r#"
Algorithm BadConst() -> Int
    Complexity: "O(1)"
    const PI <- 3
    PI <- 4
    return PI
end Algorithm
"#;
        let err = compile_to_hir(src, &NullModuleResolver).unwrap_err();
        assert!(err.iter().any(|e| e.contains("E_CONST_REASSIGN")));
    }

    #[test]
    fn rejects_recursive_record_cycle() {
        let src = r#"
Record Node
    val: Int
    next: Option<Node>
end Record
"#;
        let err = compile_to_hir(src, &NullModuleResolver).unwrap_err();
        assert!(err.iter().any(|e| e.contains("E_MVS_CYCLE")));
    }

    #[test]
    fn rejects_constant_time_secret_branch_and_index() {
        let src = r#"
@ConstantTime
Algorithm Leak(a: Secret<Int>, arr: List<Int>) -> Int
    Complexity: "O(1)"
    if a > 0 then
        return arr[a]
    end if
    return 0
end Algorithm
"#;
        let err = compile_to_hir(src, &NullModuleResolver).unwrap_err();
        assert!(err.iter().any(|e| e.contains("CT-01")));
        assert!(err.iter().any(|e| e.contains("CT-03")));
    }

    #[test]
    fn rejects_aliased_mut_borrow() {
        let src = r#"
Algorithm Swap(mut a: List<Int>, mut b: List<Int>) -> Void
    Complexity: "O(1)"
end Algorithm

Algorithm Caller(mut x: List<Int>) -> Void
    Complexity: "O(1)"
    Swap(mut x, mut x)
end Algorithm
"#;
        let err = compile_to_hir(src, &NullModuleResolver).unwrap_err();
        assert!(err.iter().any(|e| e.contains("E_MVS_ALIAS")));
    }

    #[test]
    fn detailed_diagnostics_preserve_exact_source_span() {
        let src = "module t\nAlgorithm Bad() -> Int\n    const x <- 1\n    x <- 2\n    return x\nend Algorithm\n";
        let errs = compile_to_hir_detailed(src, &NullModuleResolver).unwrap_err();
        assert!(!errs.is_empty());
        assert!(errs[0].span.is_some());
        assert!(errs[0].span.map_or(0, |s| s.start_line) >= 3);
    }

    #[test]
    fn embedded_stdlib_resolver_links_std_math_gcd() {
        let src = r#"
module app.demo
import std.math.gcd

Algorithm Lcm(a: Int, b: Int) -> Int
    Complexity:
        Time: O(log N) where N = a,
        Space: O(1)
    return (a / GCD(a, b)) * b
end Algorithm
"#;
        let prog = compile_to_hir(src, &EmbeddedStdlibResolver).expect("links std.math.gcd");
        assert!(prog.algorithms.iter().any(|a| a.name == "GCD"));
        assert!(prog.algorithms.iter().any(|a| a.name == "Lcm"));
    }
}
