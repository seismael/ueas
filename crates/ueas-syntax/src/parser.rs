//! Core Rowan CST builder and token cursor with trivia skipping and error recovery.

use crate::diagnostics::{LineIndex, SyntaxDiagnostic};
use crate::lexer::{tokenize, Token};
use crate::syntax_kind::{SyntaxKind, SyntaxNode};
use rowan::{Checkpoint, GreenNode, GreenNodeBuilder};

/// Lossless parse output containing the CST root and any syntax diagnostics.
#[derive(Debug, Clone)]
pub struct Parse {
    /// Root Rowan green node.
    pub green: GreenNode,
    /// Syntax diagnostics collected during parsing.
    pub diagnostics: Vec<SyntaxDiagnostic>,
    /// Line index for byte-offset-to-line/col mapping.
    pub line_index: LineIndex,
}

impl Parse {
    /// Obtain a typed Rowan `SyntaxNode` root.
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Returns `true` if parsing succeeded with zero syntax errors.
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Stateful recursive-descent + Pratt parser producing a lossless Rowan CST.
pub(crate) struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
    builder: GreenNodeBuilder<'static>,
    diagnostics: Vec<SyntaxDiagnostic>,
    line_index: LineIndex,
    source_len: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            pos: 0,
            builder: GreenNodeBuilder::new(),
            diagnostics: Vec::new(),
            line_index: LineIndex::new(source),
            source_len: source.len(),
        }
    }

    pub(crate) fn finish(mut self) -> Parse {
        self.eat_trivia();
        self.builder.finish_node();
        Parse {
            green: self.builder.finish(),
            diagnostics: self.diagnostics,
            line_index: self.line_index,
        }
    }

    pub(crate) fn start_root(&mut self) {
        self.builder.start_node(SyntaxKind::SOURCE_FILE.into());
    }

    pub(crate) fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }

    pub(crate) fn start_node(&mut self, kind: SyntaxKind) {
        self.eat_trivia();
        self.builder.start_node(kind.into());
    }

    pub(crate) fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into());
    }

    pub(crate) fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    /// Peek at the next non-trivia token kind (skipping whitespace, newlines, and comments).
    pub(crate) fn current(&self) -> SyntaxKind {
        self.nth(0, true)
    }

    pub(crate) fn nth(&self, mut n: usize, skip_newlines: bool) -> SyntaxKind {
        let mut idx = self.pos;
        while idx < self.tokens.len() {
            let k = self.tokens[idx].kind;
            let is_skip = if skip_newlines {
                k.is_trivia()
            } else {
                matches!(
                    k,
                    SyntaxKind::WHITESPACE | SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT
                )
            };
            if !is_skip {
                if n == 0 {
                    return k;
                }
                n -= 1;
            }
            idx += 1;
        }
        SyntaxKind::EOF
    }

    /// Returns `true` if a newline occurs before the next non-trivia token.
    pub(crate) fn at_newline_boundary(&self) -> bool {
        let mut idx = self.pos;
        while idx < self.tokens.len() {
            let k = self.tokens[idx].kind;
            if k == SyntaxKind::NEWLINE {
                return true;
            }
            if !k.is_trivia() {
                return false;
            }
            idx += 1;
        }
        true
    }

    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.current() == kind
    }

    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(crate) fn bump(&mut self) {
        self.eat_trivia();
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos];
            self.builder.token(tok.kind.into(), tok.text);
            self.pos += 1;
        }
    }

    pub(crate) fn eat_trivia(&mut self) {
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_trivia() {
            let tok = self.tokens[self.pos];
            self.builder.token(tok.kind.into(), tok.text);
            self.pos += 1;
        }
    }

    pub(crate) fn current_offset(&self) -> usize {
        let mut idx = self.pos;
        while idx < self.tokens.len() && self.tokens[idx].kind.is_trivia() {
            idx += 1;
        }
        self.tokens
            .get(idx)
            .map(|t| t.offset)
            .unwrap_or(self.source_len)
    }

    pub(crate) fn expect(&mut self, kind: SyntaxKind, msg: &str) -> bool {
        if self.eat(kind) {
            true
        } else {
            self.error(msg, None);
            false
        }
    }

    pub(crate) fn error(&mut self, msg: &str, hint: Option<&str>) {
        let start = self.current_offset();
        let end = self
            .tokens
            .iter()
            .skip(self.pos)
            .find(|t| !t.kind.is_trivia())
            .map(|t| t.offset + t.text.len())
            .unwrap_or(start);
        self.diagnostics.push(SyntaxDiagnostic {
            span: self.line_index.span(start, end),
            message: msg.to_string(),
            hint: hint.map(str::to_string),
        });
    }

    pub(crate) fn error_and_bump(&mut self, msg: &str, hint: Option<&str>) {
        self.error(msg, hint);
        if !self.at(SyntaxKind::EOF) {
            self.start_node(SyntaxKind::ERROR);
            self.bump();
            self.finish_node();
        }
    }

    /// Check if `kind` can be used as an identifier in non-keyword contexts.
    pub(crate) fn is_ident_like(&self, kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::IDENT | SyntaxKind::KW_SCALES | SyntaxKind::KW_GENERATOR
        )
    }

    /// Consume an identifier (or contextual keyword).
    pub(crate) fn expect_ident(&mut self, msg: &str) -> bool {
        if self.is_ident_like(self.current()) {
            self.bump();
            true
        } else {
            self.error(msg, None);
            false
        }
    }
}
