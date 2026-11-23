//! Lossless CST parser, Pratt expression parser, and formatter for UEAS v2.0.

pub mod decl;
pub mod diagnostics;
pub mod expr;
pub mod format;
pub mod lexer;
pub mod parser;
pub mod stmt;
pub mod syntax_kind;

pub use diagnostics::{LineIndex, SourceSpan, SyntaxDiagnostic};
pub use format::format_source;
pub use parser::Parse;
pub use syntax_kind::{SyntaxKind, SyntaxNode, SyntaxToken, UeasLanguage};

/// Parse a UEAS source file into a lossless Rowan CST + structured diagnostics.
pub fn parse(source: &str) -> Parse {
    let mut p = parser::Parser::new(source);
    decl::parse_source_file(&mut p);
    p.finish()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_binary_search_with_else_if_and_contracts() {
        let src = r#"
module std.searching

Algorithm BinarySearch<T: Ord>(data: List<T>, target: T) -> Option<Int>
    Require:
        forall i in 0..(data.length - 1) :: data[i] <= data[i + 1]
    Ensure(res):
        res != None ==> data[0] <= target
    Complexity:
        Time: O(log N) where N = data.length,
        Space: O(1)

    low <- 0
    high <- data.length - 1

    while low <= high do
        invariant 0 <= low and high < data.length
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

@Test "BinarySearch standard"
    assert BinarySearch([1, 3, 5], 3) == Some(1)
end Test
"#;
        let p = parse(src);
        assert!(p.is_ok(), "diagnostics: {:?}", p.diagnostics);
        let root = p.syntax();
        assert_eq!(root.kind(), SyntaxKind::SOURCE_FILE);
        assert_eq!(root.text().to_string(), src); // Lossless round-trip!
    }

    #[test]
    fn parse_precedence_and_left_associativity() {
        let src = r#"
Algorithm Prec(a: Int, b: Int, c: Int) -> Bool
    Complexity: "O(1)"
    x <- 10 - 3 - 2
    return a < b + 1 and not (c == 0)
end Algorithm
"#;
        let p = parse(src);
        assert!(p.is_ok(), "diagnostics: {:?}", p.diagnostics);
        let root = p.syntax();
        // Verify `10 - 3 - 2` has `(10 - 3)` as left child of outer `-`
        let bins: Vec<_> = root
            .descendants()
            .filter(|n| n.kind() == SyntaxKind::BINARY_EXPR)
            .collect();
        let outer_sub = bins
            .iter()
            .find(|n| n.text() == "10 - 3 - 2")
            .expect("outer subtraction");
        let first_child = outer_sub.first_child().expect("left child");
        assert_eq!(first_child.kind(), SyntaxKind::BINARY_EXPR);
        assert_eq!(first_child.text().to_string(), "10 - 3");
    }

    #[test]
    fn parse_recovers_from_syntax_errors() {
        let src = r#"
Algorithm Broken(x: Int) -> Int
    Complexity: "O(1)"
    y <- +
    return x
end Algorithm
"#;
        let p = parse(src);
        assert!(!p.is_ok());
        assert_eq!(p.syntax().text().to_string(), src);
    }

    #[test]
    fn formatter_is_idempotent_and_preserves_comments() {
        let src = "Algorithm Foo(x: Int) -> Int\nComplexity: \"O(1)\"\n# keep comment\nif x>0 then\nreturn x\nelse\nreturn -x\nend if\nend Algorithm\n";
        let f1 = format_source(src).expect("format 1");
        let f2 = format_source(&f1).expect("format 2");
        assert_eq!(f1, f2);
        assert!(f1.contains("# keep comment"));
        assert!(f1.contains("    if x > 0 then"));
        assert!(f1.contains("    else"));
    }
}
