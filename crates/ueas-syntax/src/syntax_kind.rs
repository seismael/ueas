//! Lossless syntax kinds and Rowan language definition for UEAS v2.0.

use rowan::Language;

/// All lexical tokens and CST composite node kinds for UEAS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[allow(non_camel_case_types, missing_docs)]
pub enum SyntaxKind {
    // ===== Trivia =====
    WHITESPACE = 0,
    NEWLINE,
    LINE_COMMENT,
    BLOCK_COMMENT,

    // ===== Literals & Identifiers =====
    IDENT,
    INT_LIT,
    REAL_LIT,
    STRING_LIT,

    // ===== Keywords =====
    KW_MODULE,
    KW_IMPORT,
    KW_RECORD,
    KW_ALGORITHM,
    KW_FUNCTION,
    KW_PROCEDURE,
    KW_REQUIRE,
    KW_ENSURE,
    KW_COMPLEXITY,
    KW_MEMORY,
    KW_WHERE,
    KW_MUT,
    KW_LET,
    KW_CONST,
    KW_RETURN,
    KW_IF,
    KW_THEN,
    KW_ELSE,
    KW_FOR,
    KW_EACH,
    KW_IN,
    KW_DO,
    KW_WHILE,
    KW_MATCH,
    KW_CASE,
    KW_END,
    KW_BREAK,
    KW_CONTINUE,
    KW_PASS,
    KW_ASSERT,
    KW_INVARIANT,
    KW_DECREASES,
    KW_FORALL,
    KW_EXISTS,
    KW_SPAWN,
    KW_SYNC,
    KW_PARALLEL,
    KW_AND,
    KW_OR,
    KW_NOT,
    KW_MOD,
    KW_AS,
    KW_TRUE,
    KW_FALSE,
    KW_NONE,
    KW_SOME,
    KW_OK,
    KW_ERR,
    KW_INFINITY,
    KW_NAN,
    KW_SCALES,
    KW_GENERATOR,

    // ===== Punctuation & Operators =====
    AT,           // @
    COLON,        // :
    DOUBLE_COLON, // ::
    COMMA,        // ,
    DOT,          // .
    DOT_DOT,      // ..
    DOT_DOT_EQ,   // ..=
    LPAREN,       // (
    RPAREN,       // )
    LBRACKET,     // [
    RBRACKET,     // ]
    LBRACE,       // {
    RBRACE,       // }
    LARROW,       // <- or :=
    RARROW,       // ->
    FAT_ARROW,    // =>
    IMPLIES,      // ==>
    IFF,          // <==>
    EQ,           // ==
    NEQ,          // !=
    LE,           // <=
    GE,           // >=
    LT,           // <
    GT,           // >
    ASSIGN_EQ,    // =
    PLUS,         // +
    MINUS,        // -
    STAR,         // *
    SLASH,        // /
    PERCENT,      // %
    AMP,          // &
    PIPE,         // |
    CARET,        // ^
    TILDE,        // ~
    LSHIFT,       // <<
    RSHIFT,       // >>

    // ===== Special =====
    ERROR_TOKEN,
    EOF,

    // ===== Composite CST Nodes =====
    SOURCE_FILE,
    MODULE_DECL,
    IMPORT_DECL,
    DOTTED_IDENT,
    DECORATOR,
    HW_PARAM,
    RECORD_DECL,
    FIELD_DECL,
    ALGORITHM_DECL,
    TYPE_PARAMS,
    TYPE_PARAM,
    PARAM_LIST,
    PARAM,
    REQUIRE_BLOCK,
    ENSURE_BLOCK,
    COMPLEXITY_BLOCK,
    COMPLEXITY_ENTRY,
    TEST_DECL,
    BENCHMARK_DECL,
    BLOCK,

    // Statements
    LET_STMT,
    CONST_STMT,
    ASSIGN_STMT,
    EXPR_STMT,
    RETURN_STMT,
    IF_STMT,
    ELSE_IF_BRANCH,
    ELSE_BRANCH,
    WHILE_STMT,
    FOR_STMT,
    MATCH_STMT,
    MATCH_ARM,
    ASSERT_STMT,
    INVARIANT_STMT,
    DECREASES_STMT,
    BREAK_STMT,
    CONTINUE_STMT,
    PASS_STMT,
    SYNC_STMT,

    // Expressions
    LITERAL_EXPR,
    IDENT_EXPR,
    PAREN_EXPR,
    LIST_EXPR,
    SET_OR_MAP_EXPR,
    MAP_ENTRY,
    TUPLE_EXPR,
    RECORD_LIT_EXPR,
    RECORD_FIELD_INIT,
    BINARY_EXPR,
    UNARY_EXPR,
    CAST_EXPR,
    CALL_EXPR,
    ARG_LIST,
    INDEX_EXPR,
    SLICE_EXPR,
    FIELD_EXPR,
    METHOD_CALL_EXPR,
    RANGE_EXPR,
    QUANTIFIER_EXPR,
    SPAWN_EXPR,

    // Types
    NAMED_TYPE,
    GENERIC_TYPE,
    TYPE_ARG_LIST,

    // Error node
    ERROR,

    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    /// Returns `true` if this kind is whitespace or a comment.
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::WHITESPACE | Self::NEWLINE | Self::LINE_COMMENT | Self::BLOCK_COMMENT
        )
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// Rowan language tag for UEAS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UeasLanguage {}

impl Language for UeasLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        if raw.0 < SyntaxKind::__LAST as u16 {
            // Safe conversion via explicit discriminant table or match;
            // using match-free bounded check via transmute-free helper:
            syntax_kind_from_u16(raw.0)
        } else {
            SyntaxKind::ERROR
        }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

/// Type alias for a Rowan SyntaxNode in UEAS.
pub type SyntaxNode = rowan::SyntaxNode<UeasLanguage>;
/// Type alias for a Rowan SyntaxToken in UEAS.
pub type SyntaxToken = rowan::SyntaxToken<UeasLanguage>;

fn syntax_kind_from_u16(raw: u16) -> SyntaxKind {
    // Exhaustive safe mapping without unsafe transmute (#![forbid(unsafe_code)]!)
    const ALL: &[SyntaxKind] = &[
        SyntaxKind::WHITESPACE,
        SyntaxKind::NEWLINE,
        SyntaxKind::LINE_COMMENT,
        SyntaxKind::BLOCK_COMMENT,
        SyntaxKind::IDENT,
        SyntaxKind::INT_LIT,
        SyntaxKind::REAL_LIT,
        SyntaxKind::STRING_LIT,
        SyntaxKind::KW_MODULE,
        SyntaxKind::KW_IMPORT,
        SyntaxKind::KW_RECORD,
        SyntaxKind::KW_ALGORITHM,
        SyntaxKind::KW_FUNCTION,
        SyntaxKind::KW_PROCEDURE,
        SyntaxKind::KW_REQUIRE,
        SyntaxKind::KW_ENSURE,
        SyntaxKind::KW_COMPLEXITY,
        SyntaxKind::KW_MEMORY,
        SyntaxKind::KW_WHERE,
        SyntaxKind::KW_MUT,
        SyntaxKind::KW_LET,
        SyntaxKind::KW_CONST,
        SyntaxKind::KW_RETURN,
        SyntaxKind::KW_IF,
        SyntaxKind::KW_THEN,
        SyntaxKind::KW_ELSE,
        SyntaxKind::KW_FOR,
        SyntaxKind::KW_EACH,
        SyntaxKind::KW_IN,
        SyntaxKind::KW_DO,
        SyntaxKind::KW_WHILE,
        SyntaxKind::KW_MATCH,
        SyntaxKind::KW_CASE,
        SyntaxKind::KW_END,
        SyntaxKind::KW_BREAK,
        SyntaxKind::KW_CONTINUE,
        SyntaxKind::KW_PASS,
        SyntaxKind::KW_ASSERT,
        SyntaxKind::KW_INVARIANT,
        SyntaxKind::KW_DECREASES,
        SyntaxKind::KW_FORALL,
        SyntaxKind::KW_EXISTS,
        SyntaxKind::KW_SPAWN,
        SyntaxKind::KW_SYNC,
        SyntaxKind::KW_PARALLEL,
        SyntaxKind::KW_AND,
        SyntaxKind::KW_OR,
        SyntaxKind::KW_NOT,
        SyntaxKind::KW_MOD,
        SyntaxKind::KW_AS,
        SyntaxKind::KW_TRUE,
        SyntaxKind::KW_FALSE,
        SyntaxKind::KW_NONE,
        SyntaxKind::KW_SOME,
        SyntaxKind::KW_OK,
        SyntaxKind::KW_ERR,
        SyntaxKind::KW_INFINITY,
        SyntaxKind::KW_NAN,
        SyntaxKind::KW_SCALES,
        SyntaxKind::KW_GENERATOR,
        SyntaxKind::AT,
        SyntaxKind::COLON,
        SyntaxKind::DOUBLE_COLON,
        SyntaxKind::COMMA,
        SyntaxKind::DOT,
        SyntaxKind::DOT_DOT,
        SyntaxKind::DOT_DOT_EQ,
        SyntaxKind::LPAREN,
        SyntaxKind::RPAREN,
        SyntaxKind::LBRACKET,
        SyntaxKind::RBRACKET,
        SyntaxKind::LBRACE,
        SyntaxKind::RBRACE,
        SyntaxKind::LARROW,
        SyntaxKind::RARROW,
        SyntaxKind::FAT_ARROW,
        SyntaxKind::IMPLIES,
        SyntaxKind::IFF,
        SyntaxKind::EQ,
        SyntaxKind::NEQ,
        SyntaxKind::LE,
        SyntaxKind::GE,
        SyntaxKind::LT,
        SyntaxKind::GT,
        SyntaxKind::ASSIGN_EQ,
        SyntaxKind::PLUS,
        SyntaxKind::MINUS,
        SyntaxKind::STAR,
        SyntaxKind::SLASH,
        SyntaxKind::PERCENT,
        SyntaxKind::AMP,
        SyntaxKind::PIPE,
        SyntaxKind::CARET,
        SyntaxKind::TILDE,
        SyntaxKind::LSHIFT,
        SyntaxKind::RSHIFT,
        SyntaxKind::ERROR_TOKEN,
        SyntaxKind::EOF,
        SyntaxKind::SOURCE_FILE,
        SyntaxKind::MODULE_DECL,
        SyntaxKind::IMPORT_DECL,
        SyntaxKind::DOTTED_IDENT,
        SyntaxKind::DECORATOR,
        SyntaxKind::HW_PARAM,
        SyntaxKind::RECORD_DECL,
        SyntaxKind::FIELD_DECL,
        SyntaxKind::ALGORITHM_DECL,
        SyntaxKind::TYPE_PARAMS,
        SyntaxKind::TYPE_PARAM,
        SyntaxKind::PARAM_LIST,
        SyntaxKind::PARAM,
        SyntaxKind::REQUIRE_BLOCK,
        SyntaxKind::ENSURE_BLOCK,
        SyntaxKind::COMPLEXITY_BLOCK,
        SyntaxKind::COMPLEXITY_ENTRY,
        SyntaxKind::TEST_DECL,
        SyntaxKind::BENCHMARK_DECL,
        SyntaxKind::BLOCK,
        SyntaxKind::LET_STMT,
        SyntaxKind::CONST_STMT,
        SyntaxKind::ASSIGN_STMT,
        SyntaxKind::EXPR_STMT,
        SyntaxKind::RETURN_STMT,
        SyntaxKind::IF_STMT,
        SyntaxKind::ELSE_IF_BRANCH,
        SyntaxKind::ELSE_BRANCH,
        SyntaxKind::WHILE_STMT,
        SyntaxKind::FOR_STMT,
        SyntaxKind::MATCH_STMT,
        SyntaxKind::MATCH_ARM,
        SyntaxKind::ASSERT_STMT,
        SyntaxKind::INVARIANT_STMT,
        SyntaxKind::DECREASES_STMT,
        SyntaxKind::BREAK_STMT,
        SyntaxKind::CONTINUE_STMT,
        SyntaxKind::PASS_STMT,
        SyntaxKind::SYNC_STMT,
        SyntaxKind::LITERAL_EXPR,
        SyntaxKind::IDENT_EXPR,
        SyntaxKind::PAREN_EXPR,
        SyntaxKind::LIST_EXPR,
        SyntaxKind::SET_OR_MAP_EXPR,
        SyntaxKind::MAP_ENTRY,
        SyntaxKind::TUPLE_EXPR,
        SyntaxKind::RECORD_LIT_EXPR,
        SyntaxKind::RECORD_FIELD_INIT,
        SyntaxKind::BINARY_EXPR,
        SyntaxKind::UNARY_EXPR,
        SyntaxKind::CAST_EXPR,
        SyntaxKind::CALL_EXPR,
        SyntaxKind::ARG_LIST,
        SyntaxKind::INDEX_EXPR,
        SyntaxKind::SLICE_EXPR,
        SyntaxKind::FIELD_EXPR,
        SyntaxKind::METHOD_CALL_EXPR,
        SyntaxKind::RANGE_EXPR,
        SyntaxKind::QUANTIFIER_EXPR,
        SyntaxKind::SPAWN_EXPR,
        SyntaxKind::NAMED_TYPE,
        SyntaxKind::GENERIC_TYPE,
        SyntaxKind::TYPE_ARG_LIST,
        SyntaxKind::ERROR,
    ];
    ALL.get(raw as usize).copied().unwrap_or(SyntaxKind::ERROR)
}
