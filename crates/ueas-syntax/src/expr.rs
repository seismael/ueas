//! 13-level Pratt expression parser and type expression parser for UEAS v2.0.

use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind;

/// Parse a full expression (starting at lowest precedence: `<==>` / `==>`).
pub(crate) fn parse_expr(p: &mut Parser<'_>) {
    parse_expr_bp(p, 1);
}

/// Returns `(left_bp, right_bp)` for binary / range / cast operators.
fn infix_bp(kind: SyntaxKind, next: SyntaxKind) -> Option<(u8, u8)> {
    match kind {
        SyntaxKind::IFF => Some((2, 1)),     // right-associative
        SyntaxKind::IMPLIES => Some((4, 3)), // right-associative
        SyntaxKind::KW_OR => Some((5, 6)),
        SyntaxKind::KW_AND => Some((7, 8)),
        SyntaxKind::EQ
        | SyntaxKind::NEQ
        | SyntaxKind::LT
        | SyntaxKind::LE
        | SyntaxKind::GT
        | SyntaxKind::GE
        | SyntaxKind::KW_IN => Some((9, 10)),
        SyntaxKind::KW_NOT if next == SyntaxKind::KW_IN => Some((9, 10)),
        SyntaxKind::DOT_DOT | SyntaxKind::DOT_DOT_EQ => Some((11, 12)),
        SyntaxKind::PIPE => Some((13, 14)),
        SyntaxKind::CARET => Some((15, 16)),
        SyntaxKind::AMP => Some((17, 18)),
        SyntaxKind::LSHIFT | SyntaxKind::RSHIFT => Some((19, 20)),
        SyntaxKind::PLUS | SyntaxKind::MINUS => Some((21, 22)),
        SyntaxKind::STAR | SyntaxKind::SLASH | SyntaxKind::KW_MOD | SyntaxKind::PERCENT => {
            Some((23, 24))
        }
        SyntaxKind::KW_AS => Some((27, 28)),
        _ => None,
    }
}

fn parse_expr_bp(p: &mut Parser<'_>, min_bp: u8) {
    p.eat_trivia();
    let lhs_cp = p.checkpoint();

    // Prefix / Primary
    match p.current() {
        SyntaxKind::MINUS | SyntaxKind::KW_NOT | SyntaxKind::TILDE => {
            p.start_node(SyntaxKind::UNARY_EXPR);
            p.bump();
            parse_expr_bp(p, 25);
            p.finish_node();
        }
        SyntaxKind::KW_SPAWN => {
            p.start_node(SyntaxKind::SPAWN_EXPR);
            p.bump();
            parse_expr_bp(p, 25);
            p.finish_node();
        }
        SyntaxKind::KW_FORALL | SyntaxKind::KW_EXISTS => {
            parse_quantifier_expr(p);
        }
        _ => {
            parse_primary(p);
        }
    }

    // Postfix & Infix loop
    loop {
        // Postfix operators must NOT cross a newline unless inside parens/brackets,
        // so `x\n[1]` or `foo\n(a)` doesn't accidentally attach across statements!
        if !p.at_newline_boundary() {
            match p.current() {
                SyntaxKind::LPAREN => {
                    p.start_node_at(lhs_cp, SyntaxKind::CALL_EXPR);
                    parse_arg_list(p);
                    p.finish_node();
                    continue;
                }
                SyntaxKind::LBRACKET => {
                    parse_index_or_slice(p, lhs_cp);
                    continue;
                }
                SyntaxKind::DOT => {
                    p.bump(); // .
                    p.expect_ident("expected field or method name after '.'");
                    if !p.at_newline_boundary() && p.at(SyntaxKind::LPAREN) {
                        p.start_node_at(lhs_cp, SyntaxKind::METHOD_CALL_EXPR);
                        parse_arg_list(p);
                        p.finish_node();
                    } else {
                        p.start_node_at(lhs_cp, SyntaxKind::FIELD_EXPR);
                        p.finish_node();
                    }
                    continue;
                }
                _ => {}
            }
        }

        // Do not cross a newline for infix operators either unless already inside an expression
        // where newline is explicitly handled, or wait: in UEAS, statements are newline-separated
        // without semicolons, so an infix operator on the SAME line continues, whereas starting a
        // new line starts a new statement!
        if p.at_newline_boundary() {
            break;
        }

        let op = p.current();
        let next = p.nth(1, true);
        let Some((l_bp, r_bp)) = infix_bp(op, next) else {
            break;
        };
        if l_bp < min_bp {
            break;
        }

        if op == SyntaxKind::KW_AS {
            p.start_node_at(lhs_cp, SyntaxKind::CAST_EXPR);
            p.bump(); // as
            parse_type_expr(p);
            p.finish_node();
            continue;
        }

        if matches!(op, SyntaxKind::DOT_DOT | SyntaxKind::DOT_DOT_EQ) {
            p.start_node_at(lhs_cp, SyntaxKind::RANGE_EXPR);
            p.bump();
            parse_expr_bp(p, r_bp);
            p.finish_node();
            continue;
        }

        p.start_node_at(lhs_cp, SyntaxKind::BINARY_EXPR);
        p.bump();
        if op == SyntaxKind::KW_NOT && next == SyntaxKind::KW_IN {
            p.bump(); // in
        }
        parse_expr_bp(p, r_bp);
        p.finish_node();
    }
}

fn parse_primary(p: &mut Parser<'_>) {
    match p.current() {
        SyntaxKind::INT_LIT
        | SyntaxKind::REAL_LIT
        | SyntaxKind::STRING_LIT
        | SyntaxKind::KW_TRUE
        | SyntaxKind::KW_FALSE
        | SyntaxKind::KW_NONE
        | SyntaxKind::KW_INFINITY
        | SyntaxKind::KW_NAN => {
            p.start_node(SyntaxKind::LITERAL_EXPR);
            p.bump();
            p.finish_node();
        }
        SyntaxKind::KW_SOME | SyntaxKind::KW_OK | SyntaxKind::KW_ERR => {
            p.start_node(SyntaxKind::CALL_EXPR);
            p.start_node(SyntaxKind::IDENT_EXPR);
            p.bump();
            p.finish_node();
            if p.at(SyntaxKind::LPAREN) {
                parse_arg_list(p);
            } else {
                p.error("expected '(' after Some/Ok/Err", None);
            }
            p.finish_node();
        }
        k if p.is_ident_like(k) => {
            let cp = p.checkpoint();
            p.start_node(SyntaxKind::IDENT_EXPR);
            p.bump();
            p.finish_node();

            // Record literal: `RecordName { field: expr, ... }` on same line
            if !p.at_newline_boundary() && p.at(SyntaxKind::LBRACE) {
                // Disambiguate: RecordName { ident: expr } vs block
                if is_record_lit_lookahead(p) {
                    p.start_node_at(cp, SyntaxKind::RECORD_LIT_EXPR);
                    parse_record_lit_fields(p);
                    p.finish_node();
                }
            }
        }
        SyntaxKind::LPAREN => {
            let cp = p.checkpoint();
            p.bump(); // (
            if p.at(SyntaxKind::RPAREN) {
                p.start_node_at(cp, SyntaxKind::TUPLE_EXPR);
                p.bump();
                p.finish_node();
                return;
            }
            parse_expr(p);
            if p.eat(SyntaxKind::COMMA) {
                p.start_node_at(cp, SyntaxKind::TUPLE_EXPR);
                while !p.at(SyntaxKind::RPAREN) && !p.at(SyntaxKind::EOF) {
                    parse_expr(p);
                    if !p.eat(SyntaxKind::COMMA) {
                        break;
                    }
                }
                p.expect(SyntaxKind::RPAREN, "expected ')' closing tuple");
                p.finish_node();
            } else {
                p.start_node_at(cp, SyntaxKind::PAREN_EXPR);
                p.expect(SyntaxKind::RPAREN, "expected ')'");
                p.finish_node();
            }
        }
        SyntaxKind::LBRACKET => {
            p.start_node(SyntaxKind::LIST_EXPR);
            p.bump(); // [
            while !p.at(SyntaxKind::RBRACKET) && !p.at(SyntaxKind::EOF) {
                parse_expr(p);
                if !p.eat(SyntaxKind::COMMA) {
                    break;
                }
            }
            p.expect(SyntaxKind::RBRACKET, "expected ']' closing list literal");
            p.finish_node();
        }
        SyntaxKind::LBRACE => {
            p.start_node(SyntaxKind::SET_OR_MAP_EXPR);
            p.bump(); // {
            while !p.at(SyntaxKind::RBRACE) && !p.at(SyntaxKind::EOF) {
                let entry_cp = p.checkpoint();
                parse_expr(p);
                if p.eat(SyntaxKind::COLON) {
                    p.start_node_at(entry_cp, SyntaxKind::MAP_ENTRY);
                    parse_expr(p);
                    p.finish_node();
                }
                if !p.eat(SyntaxKind::COMMA) {
                    break;
                }
            }
            p.expect(SyntaxKind::RBRACE, "expected '}' closing set/map literal");
            p.finish_node();
        }
        _ => {
            p.error_and_bump("expected expression", None);
        }
    }
}

fn is_record_lit_lookahead(p: &Parser<'_>) -> bool {
    // `{ }` or `{ ident : ... }`
    let t1 = p.nth(1, true);
    let t2 = p.nth(2, true);
    t1 == SyntaxKind::RBRACE || (p.is_ident_like(t1) && t2 == SyntaxKind::COLON)
}

fn parse_record_lit_fields(p: &mut Parser<'_>) {
    p.bump(); // {
    while !p.at(SyntaxKind::RBRACE) && !p.at(SyntaxKind::EOF) {
        p.start_node(SyntaxKind::RECORD_FIELD_INIT);
        p.expect_ident("expected field name in record literal");
        p.expect(SyntaxKind::COLON, "expected ':' after field name");
        parse_expr(p);
        p.finish_node();
        if !p.eat(SyntaxKind::COMMA) {
            break;
        }
    }
    p.expect(SyntaxKind::RBRACE, "expected '}' closing record literal");
}

fn parse_arg_list(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::ARG_LIST);
    p.bump(); // (
    while !p.at(SyntaxKind::RPAREN) && !p.at(SyntaxKind::EOF) {
        p.eat(SyntaxKind::KW_MUT);
        parse_expr(p);
        if !p.eat(SyntaxKind::COMMA) {
            break;
        }
    }
    p.expect(SyntaxKind::RPAREN, "expected ')' closing argument list");
    p.finish_node();
}

fn parse_index_or_slice(p: &mut Parser<'_>, lhs_cp: rowan::Checkpoint) {
    p.bump(); // [
    if p.eat(SyntaxKind::COLON) {
        p.start_node_at(lhs_cp, SyntaxKind::SLICE_EXPR);
        if !p.at(SyntaxKind::RBRACKET) {
            parse_expr(p);
        }
        p.expect(SyntaxKind::RBRACKET, "expected ']' closing slice");
        p.finish_node();
        return;
    }

    parse_expr(p);
    if p.eat(SyntaxKind::COLON) {
        p.start_node_at(lhs_cp, SyntaxKind::SLICE_EXPR);
        if !p.at(SyntaxKind::RBRACKET) {
            parse_expr(p);
        }
        p.expect(SyntaxKind::RBRACKET, "expected ']' closing slice");
        p.finish_node();
    } else {
        p.start_node_at(lhs_cp, SyntaxKind::INDEX_EXPR);
        p.expect(SyntaxKind::RBRACKET, "expected ']' closing index");
        p.finish_node();
    }
}

fn parse_quantifier_expr(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::QUANTIFIER_EXPR);
    p.bump(); // forall / exists
    p.expect_ident("expected bound variable in quantifier");
    p.expect(SyntaxKind::KW_IN, "expected 'in' in quantifier");
    parse_expr(p);
    p.expect(
        SyntaxKind::DOUBLE_COLON,
        "expected '::' before quantifier body",
    );
    parse_expr(p);
    p.finish_node();
}

/// Parse a UEAS type expression, e.g. `List<Int>`, `Map<String, Option<Real>>`.
pub(crate) fn parse_type_expr(p: &mut Parser<'_>) {
    let mut rshift_pending = 0usize;
    parse_type_expr_inner(p, &mut rshift_pending);
}

fn parse_type_expr_inner(p: &mut Parser<'_>, rshift_pending: &mut usize) {
    let cp = p.checkpoint();
    p.start_node(SyntaxKind::NAMED_TYPE);
    p.expect_ident("expected type name");
    p.finish_node();

    if !p.at_newline_boundary() && p.at(SyntaxKind::LT) {
        p.start_node_at(cp, SyntaxKind::GENERIC_TYPE);
        p.start_node(SyntaxKind::TYPE_ARG_LIST);
        p.bump(); // <
        while !p.at(SyntaxKind::GT) && !p.at(SyntaxKind::RSHIFT) && !p.at(SyntaxKind::EOF) {
            if p.at(SyntaxKind::INT_LIT) {
                p.start_node(SyntaxKind::LITERAL_EXPR);
                p.bump();
                p.finish_node();
            } else {
                parse_type_expr_inner(p, rshift_pending);
            }
            if *rshift_pending > 0 || !p.eat(SyntaxKind::COMMA) {
                break;
            }
        }
        if p.eat(SyntaxKind::GT) {
            // consumed '>'
        } else if p.at(SyntaxKind::RSHIFT) {
            if *rshift_pending > 0 {
                *rshift_pending -= 1;
                p.bump(); // consume '>>' now that both levels closed
            } else {
                *rshift_pending += 1;
            }
        } else {
            p.expect(SyntaxKind::GT, "expected '>' closing type arguments");
        }
        p.finish_node();
        p.finish_node();
    }
}
