//! Zero-copy UTF-8 lexer for UEAS v2.0 using `logos`.

use crate::syntax_kind::SyntaxKind;
use logos::Logos;

/// A single lexical token with its slice and byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    /// Lexical kind.
    pub kind: SyntaxKind,
    /// Slice of source text.
    pub text: &'a str,
    /// UTF-8 byte offset in source.
    pub offset: usize,
}

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
enum RawToken {
    #[regex(r"[ \t]+")]
    Whitespace,

    #[regex(r"\r?\n")]
    Newline,

    #[regex(r"#[^\r\n]*")]
    LineComment,

    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    BlockComment,

    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?")]
    RealLit,

    #[regex(r"[0-9]+")]
    IntLit,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[token("@")]
    At,
    #[token("::")]
    DoubleColon,
    #[token(":=")]
    ColonAssign,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("<-")]
    LArrow,
    #[token("->")]
    RArrow,
    #[token("<==>")]
    Iff,
    #[token("==>")]
    Implies,
    #[token("=>")]
    FatArrow,
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("<<")]
    LShift,
    #[token(">>")]
    RShift,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("=")]
    AssignEq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
}

/// Tokenize a UEAS source string into lossless tokens (including trivia).
pub fn tokenize(source: &str) -> Vec<Token<'_>> {
    let mut lexer = RawToken::lexer(source);
    let mut tokens = Vec::new();
    while let Some(res) = lexer.next() {
        let span = lexer.span();
        let text = lexer.slice();
        let kind = match res {
            Ok(raw) => match raw {
                RawToken::Whitespace => SyntaxKind::WHITESPACE,
                RawToken::Newline => SyntaxKind::NEWLINE,
                RawToken::LineComment => SyntaxKind::LINE_COMMENT,
                RawToken::BlockComment => SyntaxKind::BLOCK_COMMENT,
                RawToken::RealLit => SyntaxKind::REAL_LIT,
                RawToken::IntLit => SyntaxKind::INT_LIT,
                RawToken::StringLit => SyntaxKind::STRING_LIT,
                RawToken::Ident => classify_ident(text),
                RawToken::At => SyntaxKind::AT,
                RawToken::DoubleColon => SyntaxKind::DOUBLE_COLON,
                RawToken::ColonAssign | RawToken::LArrow => SyntaxKind::LARROW,
                RawToken::Colon => SyntaxKind::COLON,
                RawToken::Comma => SyntaxKind::COMMA,
                RawToken::DotDotEq => SyntaxKind::DOT_DOT_EQ,
                RawToken::DotDot => SyntaxKind::DOT_DOT,
                RawToken::Dot => SyntaxKind::DOT,
                RawToken::LParen => SyntaxKind::LPAREN,
                RawToken::RParen => SyntaxKind::RPAREN,
                RawToken::LBracket => SyntaxKind::LBRACKET,
                RawToken::RBracket => SyntaxKind::RBRACKET,
                RawToken::LBrace => SyntaxKind::LBRACE,
                RawToken::RBrace => SyntaxKind::RBRACE,
                RawToken::RArrow => SyntaxKind::RARROW,
                RawToken::Iff => SyntaxKind::IFF,
                RawToken::Implies => SyntaxKind::IMPLIES,
                RawToken::FatArrow => SyntaxKind::FAT_ARROW,
                RawToken::Eq => SyntaxKind::EQ,
                RawToken::Neq => SyntaxKind::NEQ,
                RawToken::Le => SyntaxKind::LE,
                RawToken::Ge => SyntaxKind::GE,
                RawToken::LShift => SyntaxKind::LSHIFT,
                RawToken::RShift => SyntaxKind::RSHIFT,
                RawToken::Lt => SyntaxKind::LT,
                RawToken::Gt => SyntaxKind::GT,
                RawToken::AssignEq => SyntaxKind::ASSIGN_EQ,
                RawToken::Plus => SyntaxKind::PLUS,
                RawToken::Minus => SyntaxKind::MINUS,
                RawToken::Star => SyntaxKind::STAR,
                RawToken::Slash => SyntaxKind::SLASH,
                RawToken::Percent => SyntaxKind::PERCENT,
                RawToken::Amp => SyntaxKind::AMP,
                RawToken::Pipe => SyntaxKind::PIPE,
                RawToken::Caret => SyntaxKind::CARET,
                RawToken::Tilde => SyntaxKind::TILDE,
            },
            Err(()) => SyntaxKind::ERROR_TOKEN,
        };
        tokens.push(Token {
            kind,
            text,
            offset: span.start,
        });
    }
    tokens
}

fn classify_ident(text: &str) -> SyntaxKind {
    match text.to_ascii_lowercase().as_str() {
        "module" => SyntaxKind::KW_MODULE,
        "import" => SyntaxKind::KW_IMPORT,
        "record" => SyntaxKind::KW_RECORD,
        "algorithm" => SyntaxKind::KW_ALGORITHM,
        "function" => SyntaxKind::KW_FUNCTION,
        "procedure" => SyntaxKind::KW_PROCEDURE,
        "require" => SyntaxKind::KW_REQUIRE,
        "ensure" => SyntaxKind::KW_ENSURE,
        "complexity" => SyntaxKind::KW_COMPLEXITY,
        "memory" => SyntaxKind::KW_MEMORY,
        "where" => SyntaxKind::KW_WHERE,
        "mut" => SyntaxKind::KW_MUT,
        "let" => SyntaxKind::KW_LET,
        "const" => SyntaxKind::KW_CONST,
        "return" => SyntaxKind::KW_RETURN,
        "if" => SyntaxKind::KW_IF,
        "then" => SyntaxKind::KW_THEN,
        "else" => SyntaxKind::KW_ELSE,
        "for" => SyntaxKind::KW_FOR,
        "each" => SyntaxKind::KW_EACH,
        "in" => SyntaxKind::KW_IN,
        "do" => SyntaxKind::KW_DO,
        "while" => SyntaxKind::KW_WHILE,
        "match" => SyntaxKind::KW_MATCH,
        "case" => SyntaxKind::KW_CASE,
        "end" => SyntaxKind::KW_END,
        "break" => SyntaxKind::KW_BREAK,
        "continue" => SyntaxKind::KW_CONTINUE,
        "pass" => SyntaxKind::KW_PASS,
        "assert" => SyntaxKind::KW_ASSERT,
        "invariant" => SyntaxKind::KW_INVARIANT,
        "decreases" => SyntaxKind::KW_DECREASES,
        "forall" => SyntaxKind::KW_FORALL,
        "exists" => SyntaxKind::KW_EXISTS,
        "spawn" => SyntaxKind::KW_SPAWN,
        "sync" => SyntaxKind::KW_SYNC,
        "parallel" => SyntaxKind::KW_PARALLEL,
        "and" => SyntaxKind::KW_AND,
        "or" => SyntaxKind::KW_OR,
        "not" => SyntaxKind::KW_NOT,
        "mod" => SyntaxKind::KW_MOD,
        "as" => SyntaxKind::KW_AS,
        "true" => SyntaxKind::KW_TRUE,
        "false" => SyntaxKind::KW_FALSE,
        "none" => SyntaxKind::KW_NONE,
        "some" => SyntaxKind::KW_SOME,
        "ok" => SyntaxKind::KW_OK,
        "err" => SyntaxKind::KW_ERR,
        "infinity" => SyntaxKind::KW_INFINITY,
        "nan" => SyntaxKind::KW_NAN,
        "scales" => SyntaxKind::KW_SCALES,
        "generator" => SyntaxKind::KW_GENERATOR,
        _ => SyntaxKind::IDENT,
    }
}
