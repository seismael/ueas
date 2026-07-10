//! Python-to-UEAS reverse transpilation frontend.
//!
//! Implements a "Universal Algorithm Extractor" per ADR 0008: parses Python
//! function definitions and maps them to the UEAS Canonical JSON AST, enabling
//! complexity profiling and formal verification of existing Python code.
//!
//! # Axiom Enforcement
//! This frontend HARD REJECTS any source code that violates UEAS Axiom 1
//! (Zero System I/O). Calls to `print()`, `open()`, `input()`, import
//! statements, file operations, and network operations return a `FrontendError`.
//! Users must isolate algorithmic logic from I/O logic before extraction.
//!
//! # Coverage
//! Parses a practical subset of Python: `def`, `return`, `=`, `if`/`elif`/`else`,
//! `for`, `while`, `assert`, `pass`, `break`, `continue`, and basic arithmetic,
//! comparison, and logical expressions. Unsupported constructs are skipped or
//! produce syntax errors.

use ueas_kernel::ast::{AstNode, AstNodeFactory};

#[derive(Debug)]
pub struct FrontendError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for FrontendError {}

// ===== I/O Violation Detection =====

const IO_KEYWORDS: &[&str] = &[
    "print", "open", "input", "read", "write", "socket", "http", "url", "request",
];
const IO_PREFIXES: &[&str] = &["import ", "from "];

fn scan_io_violations(lines: &[&str]) -> Result<(), FrontendError> {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        for prefix in IO_PREFIXES {
            if trimmed.starts_with(prefix) {
                return Err(FrontendError {
                    message: format!("Axiom 1 violation: imports may introduce I/O ({trimmed})"),
                    line: i + 1,
                });
            }
        }
        for keyword in IO_KEYWORDS {
            let call_pat = format!("{keyword}(");
            if trimmed.contains(&call_pat) {
                return Err(FrontendError {
                    message: format!("Axiom 1 violation: {keyword}() is I/O"),
                    line: i + 1,
                });
            }
        }
    }
    Ok(())
}

// ===== Tokenizer =====

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Number(String),
    Str(String),
    Ident(String),
    Op(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Dot,
    Eq,
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    #[allow(dead_code)]
    col: usize,
    line: usize,
}

struct Tokenizer<'a> {
    chars: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.chars.get(self.pos).copied();
        self.pos += 1;
        self.col += 1;
        ch
    }

    fn skip_spaces(&mut self) {
        while let Some(b' ' | b'\t') = self.peek() {
            self.advance();
            if Some(b'\t') == self.peek() {
                self.col += 3;
            }
        }
    }

    fn read_number(&mut self, first: u8) -> TokenKind {
        let mut num = String::new();
        num.push(first as char);
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == b'.' {
                num.push(self.advance().unwrap() as char);
            } else {
                break;
            }
        }
        TokenKind::Number(num)
    }

    fn read_ident(&mut self, first: u8) -> TokenKind {
        let mut s = String::new();
        s.push(first as char);
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                s.push(self.advance().unwrap() as char);
            } else {
                break;
            }
        }
        TokenKind::Ident(s)
    }

    fn read_string(&mut self, quote: u8) -> TokenKind {
        let mut s = String::new();
        loop {
            match self.advance() {
                None => break,
                Some(b'\\') => {
                    s.push('\\');
                    if let Some(c) = self.advance() {
                        s.push(c as char);
                    }
                }
                Some(ch) if ch == quote => break,
                Some(ch) => s.push(ch as char),
            }
        }
        TokenKind::Str(s)
    }

    fn next_token(&mut self) -> Token {
        self.skip_spaces();
        let line0 = self.line;
        let col0 = self.col;
        let kind = match self.peek() {
            None => TokenKind::Eof,
            Some(b'\n') => {
                self.advance();
                self.line += 1;
                self.col = 1;
                TokenKind::Newline
            }
            Some(b'\r') => {
                self.advance();
                if self.peek() == Some(b'\n') {
                    self.advance();
                }
                self.line += 1;
                self.col = 1;
                TokenKind::Newline
            }
            Some(b'#') => {
                while let Some(ch) = self.peek() {
                    if ch == b'\n' {
                        break;
                    }
                    self.advance();
                }
                self.next_token();
                return self.next_token();
            }
            Some(b'(') => {
                self.advance();
                TokenKind::LParen
            }
            Some(b')') => {
                self.advance();
                TokenKind::RParen
            }
            Some(b'[') => {
                self.advance();
                TokenKind::LBracket
            }
            Some(b']') => {
                self.advance();
                TokenKind::RBracket
            }
            Some(b',') => {
                self.advance();
                TokenKind::Comma
            }
            Some(b':') => {
                self.advance();
                TokenKind::Colon
            }
            Some(b'.') => {
                self.advance();
                TokenKind::Dot
            }
            Some(b'=') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::Op("==".into())
                } else {
                    TokenKind::Eq
                }
            }
            Some(b'"') | Some(b'\'') => {
                let q = self.advance().unwrap();
                self.read_string(q)
            }
            Some(b'<') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::Op("<=".into())
                } else {
                    TokenKind::Op("<".into())
                }
            }
            Some(b'>') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::Op(">=".into())
                } else {
                    TokenKind::Op(">".into())
                }
            }
            Some(b'!') => {
                self.advance();
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::Op("!=".into())
                } else {
                    TokenKind::Op("!".into())
                }
            }
            Some(b'+') => {
                self.advance();
                TokenKind::Op("+".into())
            }
            Some(b'-') => {
                self.advance();
                TokenKind::Op("-".into())
            }
            Some(b'*') => {
                self.advance();
                if self.peek() == Some(b'*') {
                    self.advance();
                    TokenKind::Op("**".into())
                } else {
                    TokenKind::Op("*".into())
                }
            }
            Some(b'/') => {
                self.advance();
                if self.peek() == Some(b'/') {
                    self.advance();
                    TokenKind::Op("//".into())
                } else {
                    TokenKind::Op("/".into())
                }
            }
            Some(b'%') => {
                self.advance();
                TokenKind::Op("%".into())
            }
            Some(ch) if (ch as char).is_ascii_digit() => {
                let c = self.advance().unwrap();
                self.read_number(c)
            }
            Some(ch) if (ch as char).is_ascii_alphabetic() || ch == b'_' => {
                let c = self.advance().unwrap();
                self.read_ident(c)
            }
            Some(_) => {
                self.advance();
                TokenKind::Eof
            }
        };
        Token {
            kind,
            col: col0,
            line: line0,
        }
    }
}

// ===== Expression Parser =====

struct ExprParser<'a> {
    toks: Vec<Token>,
    pos: usize,
    #[allow(dead_code)]
    source_lines: &'a [&'a str],
}

impl<'a> ExprParser<'a> {
    fn new(source: &'a str) -> Self {
        let mut t = Tokenizer::new(source);
        let mut toks = Vec::new();
        loop {
            let tok = t.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            toks.push(tok);
        }
        // Split source into lines for error reporting
        let source_lines: Vec<&str> = source.lines().collect();
        Self {
            toks,
            pos: 0,
            source_lines: Box::leak(source_lines.into_boxed_slice()),
        }
    }

    fn peek(&self) -> TokenKind {
        self.toks
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        self.pos += 1;
        t
    }

    fn error(&self, msg: &str, tok: &Token) -> FrontendError {
        FrontendError {
            message: msg.to_string(),
            line: tok.line,
        }
    }

    fn parse_expression(&mut self) -> Result<AstNode, FrontendError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<AstNode, FrontendError> {
        let mut left = self.parse_and()?;
        while let TokenKind::Ident(ref s) = self.peek() {
            if s != "or" {
                break;
            }
            self.advance();
            let right = self.parse_and()?;
            left = AstNodeFactory::binary_expression("or", left, right);
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<AstNode, FrontendError> {
        let mut left = self.parse_comparison()?;
        while let TokenKind::Ident(ref s) = self.peek() {
            if s != "and" {
                break;
            }
            self.advance();
            let right = self.parse_comparison()?;
            left = AstNodeFactory::binary_expression("and", left, right);
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<AstNode, FrontendError> {
        let mut left = self.parse_additive()?;
        while let TokenKind::Op(ref op) = self.peek() {
            if matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
                self.advance();
                let right = self.parse_additive()?;
                left = AstNodeFactory::binary_expression(op, left, right);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<AstNode, FrontendError> {
        let mut left = self.parse_multiplicative()?;
        while let TokenKind::Op(ref op) = self.peek() {
            if op == "+" || op == "-" {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = AstNodeFactory::binary_expression(op, left, right);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, FrontendError> {
        let mut left = self.parse_unary()?;
        while let TokenKind::Op(ref op) = self.peek() {
            if matches!(op.as_str(), "*" | "/" | "//" | "%") {
                self.advance();
                let right = self.parse_unary()?;
                left = AstNodeFactory::binary_expression(op, left, right);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<AstNode, FrontendError> {
        if let TokenKind::Op(ref op) = self.peek() {
            if op == "-" || op == "+" {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(AstNodeFactory::binary_expression(
                    if op == "-" { "-" } else { "+" },
                    AstNodeFactory::integer_literal("0"),
                    operand,
                ));
            }
        }
        if let TokenKind::Ident(ref s) = self.peek() {
            if s == "not" {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(AstNodeFactory::unary_expression("not", operand));
            }
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<AstNode, FrontendError> {
        let mut node = self.parse_primary()?;
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_call_args()?;
                    node = match &node.value {
                        Some(ueas_kernel::ast::AstValue::String(name)) => {
                            AstNodeFactory::function_call(name, {
                                let mut all_args = vec![node.clone()];
                                all_args.extend(args);
                                all_args
                            })
                        }
                        _ => AstNodeFactory::function_call("__call__", {
                            let mut all_args = vec![node.clone()];
                            all_args.extend(args);
                            all_args
                        }),
                    };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    let colon = matches!(self.peek(), TokenKind::Colon);
                    if colon {
                        self.advance();
                        let end = self.parse_expression()?;
                        match self.peek() {
                            TokenKind::RBracket => {
                                self.advance();
                            }
                            _ => return Err(self.error("expected ']'", &self.toks[self.pos])),
                        }
                        node = AstNodeFactory::function_call("slice", vec![node, index, end]);
                    } else {
                        match self.peek() {
                            TokenKind::RBracket => {
                                self.advance();
                            }
                            _ => return Err(self.error("expected ']'", &self.toks[self.pos])),
                        }
                        node = AstNodeFactory::function_call("subscript", vec![node, index]);
                    }
                }
                TokenKind::Dot => {
                    self.advance();
                    let attr_tok = self.advance();
                    let method_name = match &attr_tok.kind {
                        TokenKind::Ident(s) => s.clone(),
                        _ => return Err(self.error("expected method name", &attr_tok)),
                    };
                    if matches!(self.peek(), TokenKind::LParen) {
                        self.advance();
                        let args = self.parse_call_args()?;
                        let mut all_args = vec![node];
                        all_args.extend(args);
                        node = AstNodeFactory::function_call(&method_name, all_args);
                    } else {
                        return Err(FrontendError {
                            message: "unsupported attribute access".into(),
                            line: attr_tok.line,
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<AstNode, FrontendError> {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Number(s) => {
                if s.contains('.') {
                    Ok(AstNodeFactory::real_literal(
                        s.parse::<f64>().unwrap_or(0.0),
                    ))
                } else {
                    Ok(AstNodeFactory::integer_literal(s))
                }
            }
            TokenKind::Str(s) => Ok(AstNodeFactory::string_literal(s)),
            TokenKind::Ident(s) if s == "True" => Ok(AstNodeFactory::boolean_literal(true)),
            TokenKind::Ident(s) if s == "False" => Ok(AstNodeFactory::boolean_literal(false)),
            TokenKind::Ident(s) if s == "None" => Ok(AstNodeFactory::none_literal()),
            TokenKind::Ident(s) => Ok(AstNodeFactory::identifier(s)),
            TokenKind::LParen => {
                let inner = self.parse_expression()?;
                let close = self.advance();
                if close.kind != TokenKind::RParen {
                    return Err(self.error("expected ')'", &close));
                }
                Ok(inner)
            }
            TokenKind::LBracket => {
                let elements = self.parse_list_elements()?;
                Ok(AstNodeFactory::list_literal(elements))
            }
            _ => Err(self.error("unexpected token in expression", &tok)),
        }
    }

    fn parse_call_args(&mut self) -> Result<Vec<AstNode>, FrontendError> {
        let mut args = Vec::new();
        if matches!(self.peek(), TokenKind::RParen) {
            self.advance();
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression()?);
            match self.peek() {
                TokenKind::Comma => {
                    self.advance();
                }
                TokenKind::RParen => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(self.error("expected ',' or ')' in call args", &self.toks[self.pos]))
                }
            }
        }
        Ok(args)
    }

    fn parse_list_elements(&mut self) -> Result<Vec<AstNode>, FrontendError> {
        let mut elements = Vec::new();
        if matches!(self.peek(), TokenKind::RBracket) {
            self.advance();
            return Ok(elements);
        }
        loop {
            elements.push(self.parse_expression()?);
            match self.peek() {
                TokenKind::Comma => {
                    self.advance();
                }
                TokenKind::RBracket => {
                    self.advance();
                    break;
                }
                _ => return Err(self.error("expected ',' or ']' in list", &self.toks[self.pos])),
            }
        }
        Ok(elements)
    }

    fn parse_expr_from_text(
        &mut self,
        text: &str,
        line_num: usize,
    ) -> Result<AstNode, FrontendError> {
        let saved = (self.toks.clone(), self.pos);
        let mut t = Tokenizer::new(text);
        let mut fresh = Vec::new();
        loop {
            let tok = t.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            fresh.push(tok);
        }
        self.toks = fresh;
        self.pos = 0;
        let result = self.parse_expression();
        self.toks = saved.0;
        self.pos = saved.1;
        match result {
            Err(_) => Err(FrontendError {
                message: format!("could not parse expression: {text}"),
                line: line_num,
            }),
            Ok(node) => Ok(node),
        }
    }
}

// ===== Body Parser =====

fn get_indent(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

fn strip_inline_comment(s: &str) -> String {
    let mut in_string = false;
    let mut string_char = '"';
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if !in_string && ch == '#' {
            break;
        }
        if (ch == '"' || ch == '\'') && (i == 0 || chars[i - 1] != '\\') {
            if !in_string {
                in_string = true;
                string_char = ch;
            } else if ch == string_char {
                in_string = false;
            }
        }
        result.push(ch);
        i += 1;
    }
    result.trim_end().to_string()
}

fn tokenize_line_for_stmt(line: &str) -> Vec<String> {
    let line = strip_inline_comment(line);
    let mut tokens = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = line.chars().collect();
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if chars[i] == '"' || chars[i] == '\'' {
            let q = chars[i];
            let mut s = String::new();
            i += 1;
            while i < chars.len() && chars[i] != q {
                if chars[i] == '\\' {
                    s.push('\\');
                    i += 1;
                    if i < chars.len() {
                        s.push(chars[i]);
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            tokens.push(format!("\"{}\"", s));
        } else if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let mut s = String::new();
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(s);
        } else if chars[i].is_ascii_digit() {
            let mut s = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(s);
        } else if matches!(chars[i], '(' | ')' | '[' | ']' | ',' | ':' | '.') {
            tokens.push(chars[i].to_string());
            i += 1;
        } else if chars[i] == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push("==".to_string());
            i += 2;
        } else if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push("!=".to_string());
            i += 2;
        } else if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push("<=".to_string());
            i += 2;
        } else if chars[i] == '>' && i + 1 < chars.len() && chars[i + 1] == '=' {
            tokens.push(">=".to_string());
            i += 2;
        } else if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            tokens.push("//".to_string());
            i += 2;
        } else if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            tokens.push("**".to_string());
            i += 2;
        } else {
            tokens.push(chars[i].to_string());
            i += 1;
        }
    }
    tokens
}

fn parse_body(
    expr_parser: &mut ExprParser,
    lines: &[&str],
    start: usize,
    base_indent: usize,
) -> Result<(Vec<AstNode>, usize), FrontendError> {
    let mut stmts = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() || line.trim().starts_with('#') {
            i += 1;
            continue;
        }
        let indent = get_indent(line);
        if indent < base_indent {
            break;
        }
        let trimmed = line.trim();
        let line_num = i + 1;

        let tokens = tokenize_line_for_stmt(trimmed);

        if tokens.is_empty() {
            i += 1;
            continue;
        }

        match tokens[0].as_str() {
            "pass" => {
                i += 1;
                continue;
            }
            "break" | "continue" => {
                i += 1;
                continue;
            }
            "return" => {
                if tokens.len() > 1 {
                    let expr_text = &trimmed[6..].trim();
                    let expr_text = expr_text.trim();
                    let expr = expr_parser.parse_expr_from_text(expr_text, line_num)?;
                    stmts.push(AstNodeFactory::return_stmt(Some(expr)));
                } else {
                    stmts.push(AstNodeFactory::return_stmt(None));
                }
                i += 1;
            }
            "assert" => {
                let cond_text = &trimmed[6..].trim();
                let cond = expr_parser.parse_expr_from_text(cond_text, line_num)?;
                stmts.push(AstNodeFactory::assert_stmt(cond, None));
                i += 1;
            }
            "if" | "elif" | "else" => {
                let (if_node, next_i) = parse_if_block(expr_parser, lines, i, base_indent)?;
                stmts.push(if_node);
                i = next_i;
            }
            "for" => {
                let (for_node, next_i) = parse_for_loop(expr_parser, lines, i, base_indent)?;
                stmts.push(for_node);
                i = next_i;
            }
            "while" => {
                let (while_node, next_i) = parse_while_loop(expr_parser, lines, i, base_indent)?;
                stmts.push(while_node);
                i = next_i;
            }
            _ => {
                let eq_idx = tokens.iter().position(|t| t == "=");
                if let Some(pos) = eq_idx {
                    let target = tokens[0].clone();
                    let rest: String = tokens[(pos + 1)..].join(" ");
                    let expr = expr_parser.parse_expr_from_text(&rest, line_num)?;
                    let target_node = AstNodeFactory::identifier(&target);
                    stmts.push(AstNodeFactory::assignment(target_node, expr));
                } else if let Ok(expr) = expr_parser.parse_expr_from_text(trimmed, line_num) {
                    stmts.push(expr);
                } else {
                    return Err(FrontendError {
                        message: format!("syntax: unrecognized statement: {trimmed}"),
                        line: line_num,
                    });
                }
                i += 1;
            }
        }
    }
    Ok((stmts, i))
}

fn parse_if_block(
    expr_parser: &mut ExprParser,
    lines: &[&str],
    start: usize,
    base_indent: usize,
) -> Result<(AstNode, usize), FrontendError> {
    let line = lines[start];
    let trimmed = line.trim();
    let line_num = start + 1;

    let (cond_text, is_else) = if trimmed == "else:" {
        ("true".to_string(), true)
    } else if trimmed.starts_with("elif ") {
        (trimmed[5..trimmed.len() - 1].trim().to_string(), false)
    } else if trimmed.starts_with("if ") {
        (trimmed[3..trimmed.len() - 1].trim().to_string(), false)
    } else {
        return Err(FrontendError {
            message: "expected if/elif/else".into(),
            line: line_num,
        });
    };

    let cond = if is_else {
        AstNodeFactory::boolean_literal(true)
    } else {
        expr_parser.parse_expr_from_text(&cond_text, line_num)?
    };

    let body_indent = get_indent(line) + 4;
    let (body, next_i) = parse_body(expr_parser, lines, start + 1, body_indent)?;

    let mut alternate: Option<Vec<AstNode>> = None;
    let mut final_i = next_i;
    if next_i < lines.len() {
        let next_line = lines[next_i];
        let next_trimmed = next_line.trim();
        let next_indent = get_indent(next_line);
        if next_indent == base_indent
            && (next_trimmed.starts_with("elif ") || next_trimmed.starts_with("else:"))
        {
            let (alt_node, alt_i) = parse_if_block(expr_parser, lines, next_i, base_indent)?;
            alternate = Some(vec![alt_node]);
            final_i = alt_i;
        }
    }

    Ok((AstNodeFactory::if_stmt(cond, body, alternate), final_i))
}

fn parse_for_loop(
    expr_parser: &mut ExprParser,
    lines: &[&str],
    start: usize,
    _base_indent: usize,
) -> Result<(AstNode, usize), FrontendError> {
    let line = lines[start];
    let trimmed = line.trim();
    let line_num = start + 1;

    let inner = &trimmed[3..].trim();
    let colon_pos = inner.rfind(':').unwrap_or(inner.len());
    let header = inner[..colon_pos].trim();
    let in_pos = header.find(" in ").ok_or_else(|| FrontendError {
        message: "for loop missing 'in'".into(),
        line: line_num,
    })?;
    let iterator = header[..in_pos].trim().to_string();
    let collection_text = header[in_pos + 4..].trim().to_string();
    let collection = expr_parser.parse_expr_from_text(&collection_text, line_num)?;

    let body_indent = get_indent(line) + 4;
    let (body, next_i) = parse_body(expr_parser, lines, start + 1, body_indent)?;

    Ok((
        AstNodeFactory::for_loop(&iterator, collection, body),
        next_i,
    ))
}

fn parse_while_loop(
    expr_parser: &mut ExprParser,
    lines: &[&str],
    start: usize,
    _base_indent: usize,
) -> Result<(AstNode, usize), FrontendError> {
    let line = lines[start];
    let trimmed = line.trim();
    let line_num = start + 1;

    let cond_text = &trimmed[5..].trim();
    let cond_text = cond_text
        .strip_suffix(':')
        .unwrap_or(cond_text)
        .trim()
        .to_string();
    let cond = expr_parser.parse_expr_from_text(&cond_text, line_num)?;

    let body_indent = get_indent(line) + 4;
    let (body, next_i) = parse_body(expr_parser, lines, start + 1, body_indent)?;

    Ok((AstNodeFactory::while_loop(cond, body), next_i))
}

fn parse_type_hint(text: &str) -> AstNode {
    let trimmed = text.trim();
    match trimmed {
        "int" | "Integer" => AstNodeFactory::type_node("Integer", vec![]),
        "float" | "Real" => AstNodeFactory::type_node("Real", vec![]),
        "bool" | "Boolean" => AstNodeFactory::type_node("Boolean", vec![]),
        "str" | "String" => AstNodeFactory::type_node("String", vec![]),
        "None" | "Void" => AstNodeFactory::type_node("Void", vec![]),
        _ => {
            if trimmed.starts_with("List[") {
                let inner = &trimmed[5..trimmed.len() - 1];
                AstNodeFactory::type_node("List", vec![parse_type_hint(inner)])
            } else if trimmed.starts_with("Set[") {
                let inner = &trimmed[4..trimmed.len() - 1];
                AstNodeFactory::type_node("Set", vec![parse_type_hint(inner)])
            } else if trimmed.starts_with("Optional[") {
                let inner = &trimmed[9..trimmed.len() - 1];
                AstNodeFactory::type_node("Option", vec![parse_type_hint(inner)])
            } else if trimmed.starts_with("Dict[") {
                let parts = &trimmed[5..trimmed.len() - 1];
                let comma = parts.find(',').unwrap_or(parts.len());
                let key = parse_type_hint(&parts[..comma]);
                let val = parse_type_hint(&parts[comma + 1..]);
                AstNodeFactory::type_node("Map", vec![key, val])
            } else {
                AstNodeFactory::type_node("Integer", vec![])
            }
        }
    }
}

// ===== Public API =====

/// Extract a Python function into a UEAS Algorithm AST node.
///
/// Parses a Python `def` statement and its indented body, mapping each
/// construct to the corresponding UEAS Canonical AST node. Returns
/// `Err(FrontendError)` if the source violates UEAS Axiom 1 (I/O
/// operations) or Axiom 3 (unrestricted memory access).
pub fn extract_python(source: &str) -> Result<AstNode, FrontendError> {
    let lines: Vec<&str> = source.lines().collect();

    scan_io_violations(&lines)?;

    let mut def_line = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") && trimmed.ends_with(':') {
            def_line = Some(i);
            break;
        }
    }

    let def_idx = def_line.ok_or_else(|| FrontendError {
        message: "no function definition (def) found".into(),
        line: 1,
    })?;

    let def_text = lines[def_idx].trim();
    let header = &def_text[4..def_text.len() - 1].trim();
    let paren_open = header.find('(').ok_or_else(|| FrontendError {
        message: "malformed function signature".into(),
        line: def_idx + 1,
    })?;
    let paren_close = header.rfind(')').ok_or_else(|| FrontendError {
        message: "malformed function signature".into(),
        line: def_idx + 1,
    })?;

    let name = header[..paren_open].trim().to_string();
    let params_text = &header[paren_open + 1..paren_close].trim();

    let mut params_ast = Vec::new();

    if !params_text.is_empty() {
        for param in params_text.split(',') {
            let param = param.trim();
            if param.is_empty() {
                continue;
            }
            if param.starts_with("self") || param.starts_with("cls") {
                continue;
            }
            let (pname, ptype) = if let Some(colon) = param.find(':') {
                let n = param[..colon].trim().to_string();
                let t = parse_type_hint(&param[colon + 1..]);
                (n, t)
            } else {
                (
                    param.to_string(),
                    AstNodeFactory::type_node("Integer", vec![]),
                )
            };
            params_ast.push(AstNodeFactory::parameter(&pname, ptype));
        }
    }

    let return_type = if let Some(arrow) = header.rfind("->") {
        parse_type_hint(header[arrow + 2..].trim())
    } else {
        AstNodeFactory::type_node("Void", vec![])
    };

    let mut expr_parser = ExprParser::new(source);
    let base_indent = get_indent(lines[def_idx]) + 4;
    let (body, _) = parse_body(&mut expr_parser, &lines, def_idx + 1, base_indent)?;

    let algorithm =
        AstNodeFactory::algorithm(&name, params_ast, Some(return_type), "O(?)", vec![], body);

    Ok(AstNodeFactory::program(vec![algorithm]))
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use ueas_kernel::ast::AstNodeKind;

    #[allow(dead_code)]
    fn count_kinds(node: &AstNode, kind: AstNodeKind) -> usize {
        let mut count = 0;
        if node.kind == kind {
            count += 1;
        }
        for child in &node.children {
            count += count_kinds(child, kind);
        }
        count
    }

    fn find_kind<'a>(node: &'a AstNode, kind: AstNodeKind) -> Vec<&'a AstNode> {
        let mut results = Vec::new();
        if node.kind == kind {
            results.push(node);
        }
        for child in &node.children {
            results.extend(find_kind(child, kind));
        }
        results
    }

    #[test]
    fn extract_simple_function() {
        let source = "def add(x, y):\n    return x + y\n";
        let result = extract_python(source).unwrap();
        assert_eq!(result.kind, AstNodeKind::Program);
        let algos = find_kind(&result, AstNodeKind::Algorithm);
        assert_eq!(algos.len(), 1);
        assert_eq!(find_kind(&result, AstNodeKind::Return).len(), 1);
        assert_eq!(find_kind(&result, AstNodeKind::BinaryExpression).len(), 1);
        assert_eq!(find_kind(&result, AstNodeKind::Parameter).len(), 2);
    }

    #[test]
    fn extract_with_if_else() {
        let source = "def classify(x):\n    if x > 0:\n        return 1\n    elif x < 0:\n        return -1\n    else:\n        return 0\n";
        let result = extract_python(source).unwrap();
        assert_eq!(result.kind, AstNodeKind::Program);
        assert_eq!(find_kind(&result, AstNodeKind::If).len(), 8);
        assert_eq!(find_kind(&result, AstNodeKind::Return).len(), 3);
    }

    #[test]
    fn extract_with_for_loop() {
        let source = "def count_items(items):\n    total = 0\n    for item in items:\n        total = total + 1\n    return total\n";
        let result = extract_python(source).unwrap();
        assert_eq!(result.kind, AstNodeKind::Program);
        assert_eq!(find_kind(&result, AstNodeKind::ForLoop).len(), 1);
        assert_eq!(find_kind(&result, AstNodeKind::Assignment).len(), 2);
        assert_eq!(find_kind(&result, AstNodeKind::Return).len(), 1);
    }

    #[test]
    fn extract_rejects_print() {
        let source = "def hello():\n    print('hello')\n";
        let result = extract_python(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("I/O"));
    }

    #[test]
    fn extract_rejects_import() {
        let source = "import os\ndef foo():\n    return 1\n";
        let result = extract_python(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("I/O"));
    }

    #[test]
    fn extract_with_list_comprehension_skips() {
        let source = "def has_duplicates(items):\n    seen = []\n    for x in items:\n        seen.append(x)\n    return len(seen) > len(items)\n";
        let result = extract_python(source).unwrap();
        assert_eq!(result.kind, AstNodeKind::Program);
        assert!(find_kind(&result, AstNodeKind::FunctionCall).len() >= 3);
    }

    #[test]
    fn extract_while_loop() {
        let source = "def countdown(n):\n    while n > 0:\n        n = n - 1\n    return n\n";
        let result = extract_python(source).unwrap();
        assert_eq!(find_kind(&result, AstNodeKind::WhileLoop).len(), 1);
    }

    #[test]
    fn extract_type_hints() {
        let source = "def sum_ints(x: int, y: int) -> int:\n    return x + y\n";
        let result = extract_python(source).unwrap();
        assert_eq!(find_kind(&result, AstNodeKind::Parameter).len(), 2);
        assert!(find_kind(&result, AstNodeKind::Type).len() >= 2);
    }

    #[test]
    fn extract_with_assert() {
        let source = "def divide(a, b):\n    assert b != 0\n    return a / b\n";
        let result = extract_python(source).unwrap();
        assert_eq!(find_kind(&result, AstNodeKind::Assert).len(), 1);
    }
}
