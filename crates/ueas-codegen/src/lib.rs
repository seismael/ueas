//! UEAS v2.0 dual-track code generation (Track A: Rust/Python/TS/C++/Go, Track B: Dafny/Lean4/Verus/Quint/TLA+, Track C: LaTeX/Typst).
#![allow(missing_docs)]

pub mod common;
pub mod lir;
pub mod smtlib;
pub mod track_a;
pub mod track_b;
pub mod track_c;
pub mod wasm_target;

pub use common::{CodeGenerator, GeneratedArtifact};
pub use lir::{lower_hir_to_lir, LirFunc, LirProgram, LirStmt};
pub use smtlib::SmtLib2Target;
pub use track_a::{CppTarget, GoTarget, PythonTarget, RustTarget, TypeScriptTarget};
pub use track_b::{DafnyTarget, LeanTarget, QuintTarget, TlaTarget, VerusTarget};
pub use track_c::{LatexTarget, TypstTarget};
pub use wasm_target::{encode_wasm_module, WasmTarget, WitTarget};

use serde::{Deserialize, Serialize};
use ueas_hir::hir::Program;

/// All supported UEAS v2.0 code-generation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetKind {
    Rust,
    Python,
    TypeScript,
    Cpp,
    Go,
    Dafny,
    Lean4,
    Verus,
    Quint,
    Tla,
    Latex,
    Typst,
    Wasm,
    Wit,
    SmtLib2,
}

impl TargetKind {
    /// Parse a CLI/API target name string.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "typescript" | "ts" => Some(Self::TypeScript),
            "cpp" | "c++" => Some(Self::Cpp),
            "go" | "golang" => Some(Self::Go),
            "dafny" | "dfy" => Some(Self::Dafny),
            "lean" | "lean4" => Some(Self::Lean4),
            "verus" => Some(Self::Verus),
            "quint" | "qnt" => Some(Self::Quint),
            "tla" | "tla+" => Some(Self::Tla),
            "latex" | "tex" => Some(Self::Latex),
            "typst" | "typ" => Some(Self::Typst),
            "wasm" => Some(Self::Wasm),
            "wit" => Some(Self::Wit),
            "smt2" | "smtlib2" => Some(Self::SmtLib2),
            _ => None,
        }
    }
}

/// Generate code for `target` from a typed UEAS `Program`.
pub fn generate_target(program: &Program, target: TargetKind) -> Result<GeneratedArtifact, String> {
    let gen: Box<dyn CodeGenerator> = match target {
        TargetKind::Rust => Box::new(RustTarget),
        TargetKind::Python => Box::new(PythonTarget),
        TargetKind::TypeScript => Box::new(TypeScriptTarget),
        TargetKind::Cpp => Box::new(CppTarget),
        TargetKind::Go => Box::new(GoTarget),
        TargetKind::Dafny => Box::new(DafnyTarget),
        TargetKind::Lean4 => Box::new(LeanTarget),
        TargetKind::Verus => Box::new(VerusTarget),
        TargetKind::Quint => Box::new(QuintTarget),
        TargetKind::Tla => Box::new(TlaTarget),
        TargetKind::Latex => Box::new(LatexTarget),
        TargetKind::Typst => Box::new(TypstTarget),
        TargetKind::Wasm => Box::new(WasmTarget),
        TargetKind::Wit => Box::new(WitTarget),
        TargetKind::SmtLib2 => Box::new(SmtLib2Target),
    };
    gen.generate(program)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use ueas_hir::check::linker::NullModuleResolver;
    use ueas_hir::compile_to_hir;

    #[test]
    fn generates_all_12_targets_from_binary_search() {
        let src = r#"
module std.searching

Algorithm BinarySearch(data: List<Int>, target: Int) -> Option<Int>
    Require:
        forall i in 0..(data.length - 1) :: data[i] <= data[i + 1]
    Complexity:
        Time: O(log N) where N = data.length,
        Space: O(1)

    let mut low <- 0
    let mut high <- data.length - 1
    while low <= high do
        invariant 0 <= low
        decreases high - low + 1
        let mid <- low + (high - low) / 2
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

@Test "finds element"
    let arr <- [1, 3, 5, 7, 9]
    let res <- BinarySearch(arr, 7)
    assert res == Some(3)
end Test
"#;
        let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR");
        for target in [
            TargetKind::Rust,
            TargetKind::Python,
            TargetKind::TypeScript,
            TargetKind::Cpp,
            TargetKind::Go,
            TargetKind::Dafny,
            TargetKind::Lean4,
            TargetKind::Verus,
            TargetKind::Quint,
            TargetKind::Tla,
            TargetKind::Latex,
            TargetKind::Typst,
            TargetKind::Wit,
            TargetKind::SmtLib2,
        ] {
            let art = generate_target(&prog, target).expect("codegen");
            assert!(!art.code.is_empty());
        }

        let lir = lower_hir_to_lir(&prog);
        assert_eq!(lir.functions.len(), 1);

        let gcd_src = r#"
module std.math
Algorithm Gcd(a: Int, b: Int) -> Int
    Require:
        a >= 0 and b >= 0
    Ensure(res):
        res >= 0
    Complexity:
        Time: O(log N) where N = a,
        Space: O(1)
    let mut x <- a
    let mut y <- b
    while y != 0 do
        let t <- y
        y <- x % y
        x <- t
    end while
    return x
end Algorithm
"#;
        let gcd_prog = compile_to_hir(gcd_src, &NullModuleResolver).expect("gcd HIR");
        let wasm_bytes = encode_wasm_module(&gcd_prog).expect("wasm binary");
        assert_eq!(&wasm_bytes[0..4], b"\0asm");
    }
}
