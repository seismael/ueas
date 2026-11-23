//! Comment-preserving lossless CST formatter for UEAS v2.0.

use crate::diagnostics::SyntaxDiagnostic;
use crate::parse;
use crate::syntax_kind::{SyntaxKind, SyntaxNode};
use rowan::NodeOrToken;

/// Format UEAS source text into canonical 4-space indented layout while preserving all comments.
pub fn format_source(source: &str) -> Result<String, Vec<SyntaxDiagnostic>> {
    let parsed = parse(source);
    if !parsed.is_ok() {
        return Err(parsed.diagnostics);
    }
    let root = parsed.syntax();
    let mut out = String::with_capacity(source.len());
    format_node(&root, 0, &mut out);
    let trimmed = out.trim_end().to_string();
    Ok(if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    })
}

fn format_node(node: &SyntaxNode, indent: usize, out: &mut String) {
    let child_indent = match node.kind() {
        SyntaxKind::BLOCK
        | SyntaxKind::REQUIRE_BLOCK
        | SyntaxKind::ENSURE_BLOCK
        | SyntaxKind::COMPLEXITY_BLOCK
        | SyntaxKind::RECORD_DECL
        | SyntaxKind::TEST_DECL
        | SyntaxKind::BENCHMARK_DECL => indent + 1,
        _ => indent,
    };

    for child in node.children_with_tokens() {
        match child {
            NodeOrToken::Node(n) => {
                let is_stmt_or_decl = matches!(
                    n.kind(),
                    SyntaxKind::MODULE_DECL
                        | SyntaxKind::IMPORT_DECL
                        | SyntaxKind::RECORD_DECL
                        | SyntaxKind::FIELD_DECL
                        | SyntaxKind::ALGORITHM_DECL
                        | SyntaxKind::DECORATOR
                        | SyntaxKind::TEST_DECL
                        | SyntaxKind::BENCHMARK_DECL
                        | SyntaxKind::REQUIRE_BLOCK
                        | SyntaxKind::ENSURE_BLOCK
                        | SyntaxKind::COMPLEXITY_BLOCK
                        | SyntaxKind::LET_STMT
                        | SyntaxKind::CONST_STMT
                        | SyntaxKind::ASSIGN_STMT
                        | SyntaxKind::EXPR_STMT
                        | SyntaxKind::RETURN_STMT
                        | SyntaxKind::IF_STMT
                        | SyntaxKind::ELSE_IF_BRANCH
                        | SyntaxKind::ELSE_BRANCH
                        | SyntaxKind::WHILE_STMT
                        | SyntaxKind::FOR_STMT
                        | SyntaxKind::MATCH_STMT
                        | SyntaxKind::MATCH_ARM
                        | SyntaxKind::ASSERT_STMT
                        | SyntaxKind::INVARIANT_STMT
                        | SyntaxKind::DECREASES_STMT
                        | SyntaxKind::BREAK_STMT
                        | SyntaxKind::CONTINUE_STMT
                        | SyntaxKind::PASS_STMT
                        | SyntaxKind::SYNC_STMT
                );
                let node_indent = if matches!(
                    n.kind(),
                    SyntaxKind::ELSE_IF_BRANCH | SyntaxKind::ELSE_BRANCH | SyntaxKind::MATCH_ARM
                ) {
                    indent
                } else {
                    child_indent
                };
                if is_stmt_or_decl {
                    ensure_newline_and_indent(out, node_indent);
                }
                format_node(&n, node_indent, out);
            }
            NodeOrToken::Token(t) => match t.kind() {
                SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE => {}
                SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT => {
                    if !out.ends_with('\n') && !out.is_empty() {
                        out.push(' ');
                    } else if out.ends_with('\n') || out.is_empty() {
                        push_indent(out, indent);
                    }
                    out.push_str(t.text());
                    out.push('\n');
                }
                SyntaxKind::KW_END => {
                    let end_indent = indent.saturating_sub(1);
                    ensure_newline_and_indent(out, end_indent);
                    out.push_str(t.text());
                }
                SyntaxKind::COMMA => {
                    out.push(',');
                    out.push(' ');
                }
                SyntaxKind::COLON => {
                    out.push(':');
                    out.push(' ');
                }
                SyntaxKind::DOUBLE_COLON => {
                    out.push_str(" :: ");
                }
                SyntaxKind::LARROW => out.push_str(" <- "),
                SyntaxKind::RARROW => out.push_str(" -> "),
                SyntaxKind::FAT_ARROW => out.push_str(" => "),
                SyntaxKind::IMPLIES => out.push_str(" ==> "),
                SyntaxKind::IFF => out.push_str(" <==> "),
                SyntaxKind::EQ
                | SyntaxKind::NEQ
                | SyntaxKind::LE
                | SyntaxKind::GE
                | SyntaxKind::PLUS
                | SyntaxKind::STAR
                | SyntaxKind::SLASH
                | SyntaxKind::PERCENT
                | SyntaxKind::AMP
                | SyntaxKind::PIPE
                | SyntaxKind::CARET
                | SyntaxKind::LSHIFT
                | SyntaxKind::RSHIFT
                | SyntaxKind::ASSIGN_EQ
                | SyntaxKind::KW_AND
                | SyntaxKind::KW_OR
                | SyntaxKind::KW_MOD
                | SyntaxKind::KW_AS
                | SyntaxKind::KW_IN => {
                    if !out.ends_with(' ') && !out.ends_with('\n') {
                        out.push(' ');
                    }
                    out.push_str(t.text());
                    out.push(' ');
                }
                SyntaxKind::LT | SyntaxKind::GT => {
                    let in_type = matches!(
                        node.kind(),
                        SyntaxKind::TYPE_ARG_LIST | SyntaxKind::TYPE_PARAMS
                    );
                    if in_type {
                        out.push_str(t.text());
                    } else {
                        if !out.ends_with(' ') && !out.ends_with('\n') {
                            out.push(' ');
                        }
                        out.push_str(t.text());
                        out.push(' ');
                    }
                }
                SyntaxKind::MINUS => {
                    if node.kind() == SyntaxKind::UNARY_EXPR {
                        out.push('-');
                    } else {
                        if !out.ends_with(' ') && !out.ends_with('\n') {
                            out.push(' ');
                        }
                        out.push('-');
                        out.push(' ');
                    }
                }
                SyntaxKind::LPAREN
                | SyntaxKind::RPAREN
                | SyntaxKind::LBRACKET
                | SyntaxKind::RBRACKET
                | SyntaxKind::DOT
                | SyntaxKind::DOT_DOT
                | SyntaxKind::DOT_DOT_EQ
                | SyntaxKind::AT
                | SyntaxKind::TILDE => {
                    out.push_str(t.text());
                }
                _ => {
                    let prev = out.chars().last();
                    if let Some(c) = prev {
                        if c.is_alphanumeric() || c == '_' || c == ')' || c == ']' || c == '"' {
                            out.push(' ');
                        }
                    }
                    out.push_str(t.text());
                }
            },
        }
    }
}

fn ensure_newline_and_indent(out: &mut String, indent: usize) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    push_indent(out, indent);
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("    ");
    }
}
