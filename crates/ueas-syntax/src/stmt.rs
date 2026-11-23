//! Statement and block parser for UEAS v2.0.

use crate::expr::{parse_expr, parse_type_expr};
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind;

/// Parse statements until one of `stop_kinds` or `EOF`.
pub(crate) fn parse_block(p: &mut Parser<'_>, stop_kinds: &[SyntaxKind]) {
    p.start_node(SyntaxKind::BLOCK);
    loop {
        let cur = p.current();
        if cur == SyntaxKind::EOF || stop_kinds.contains(&cur) {
            break;
        }
        // Check `else if` / `end ...`
        if cur == SyntaxKind::KW_END || cur == SyntaxKind::KW_ELSE {
            break;
        }
        parse_stmt(p);
    }
    p.finish_node();
}

pub(crate) fn parse_stmt(p: &mut Parser<'_>) {
    match p.current() {
        SyntaxKind::KW_LET => {
            p.start_node(SyntaxKind::LET_STMT);
            p.bump(); // let
            p.eat(SyntaxKind::KW_MUT);
            p.expect_ident("expected variable name after 'let'");
            if p.eat(SyntaxKind::COLON) {
                parse_type_expr(p);
            }
            p.expect(SyntaxKind::LARROW, "expected '<-' in let declaration");
            parse_expr(p);
            p.finish_node();
        }
        SyntaxKind::KW_CONST => {
            p.start_node(SyntaxKind::CONST_STMT);
            p.bump(); // const
            p.expect_ident("expected constant name after 'const'");
            if p.eat(SyntaxKind::COLON) {
                parse_type_expr(p);
            }
            p.expect(SyntaxKind::LARROW, "expected '<-' in const declaration");
            parse_expr(p);
            p.finish_node();
        }
        SyntaxKind::KW_RETURN => {
            p.start_node(SyntaxKind::RETURN_STMT);
            p.bump(); // return
            if !p.at_newline_boundary()
                && !matches!(
                    p.current(),
                    SyntaxKind::KW_END | SyntaxKind::KW_ELSE | SyntaxKind::EOF
                )
            {
                parse_expr(p);
            }
            p.finish_node();
        }
        SyntaxKind::KW_IF => parse_if_stmt(p),
        SyntaxKind::KW_WHILE => parse_while_stmt(p),
        SyntaxKind::KW_FOR => parse_for_stmt(p, false),
        SyntaxKind::KW_PARALLEL => {
            if p.nth(1, true) == SyntaxKind::KW_FOR {
                parse_for_stmt(p, true);
            } else {
                p.error_and_bump("expected 'for' after 'parallel'", None);
            }
        }
        SyntaxKind::KW_MATCH => parse_match_stmt(p),
        SyntaxKind::KW_ASSERT => {
            p.start_node(SyntaxKind::ASSERT_STMT);
            p.bump(); // assert
            parse_cond_msg_stmt(p);
            p.finish_node();
        }
        SyntaxKind::KW_INVARIANT => {
            p.start_node(SyntaxKind::INVARIANT_STMT);
            p.bump(); // invariant
            parse_cond_msg_stmt(p);
            p.finish_node();
        }
        SyntaxKind::KW_DECREASES => {
            p.start_node(SyntaxKind::DECREASES_STMT);
            p.bump(); // decreases
            parse_expr(p);
            p.finish_node();
        }
        SyntaxKind::KW_BREAK => {
            p.start_node(SyntaxKind::BREAK_STMT);
            p.bump();
            p.finish_node();
        }
        SyntaxKind::KW_CONTINUE => {
            p.start_node(SyntaxKind::CONTINUE_STMT);
            p.bump();
            p.finish_node();
        }
        SyntaxKind::KW_PASS => {
            p.start_node(SyntaxKind::PASS_STMT);
            p.bump();
            p.finish_node();
        }
        SyntaxKind::KW_SYNC => {
            p.start_node(SyntaxKind::SYNC_STMT);
            p.bump();
            p.finish_node();
        }
        _ => {
            let cp = p.checkpoint();
            parse_expr(p);
            if !p.at_newline_boundary() && p.eat(SyntaxKind::LARROW) {
                p.start_node_at(cp, SyntaxKind::ASSIGN_STMT);
                parse_expr(p);
                p.finish_node();
            } else {
                p.start_node_at(cp, SyntaxKind::EXPR_STMT);
                p.finish_node();
            }
        }
    }
}

fn parse_cond_msg_stmt(p: &mut Parser<'_>) {
    if p.eat(SyntaxKind::LPAREN) {
        parse_expr(p);
        if p.eat(SyntaxKind::COMMA) {
            if p.at(SyntaxKind::STRING_LIT) {
                p.start_node(SyntaxKind::LITERAL_EXPR);
                p.bump();
                p.finish_node();
            } else {
                p.error("expected string message literal", None);
            }
        }
        p.expect(SyntaxKind::RPAREN, "expected ')'");
    } else {
        parse_expr(p);
    }
}

fn parse_if_stmt(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::IF_STMT);
    p.bump(); // if
    parse_expr(p);
    p.expect(SyntaxKind::KW_THEN, "expected 'then' after if condition");
    parse_block(p, &[SyntaxKind::KW_ELSE, SyntaxKind::KW_END]);

    while p.at(SyntaxKind::KW_ELSE) {
        p.eat_trivia();
        if p.nth(1, false) == SyntaxKind::KW_IF {
            p.start_node(SyntaxKind::ELSE_IF_BRANCH);
            p.bump(); // else
            p.bump(); // if
            parse_expr(p);
            p.expect(
                SyntaxKind::KW_THEN,
                "expected 'then' after else if condition",
            );
            parse_block(p, &[SyntaxKind::KW_ELSE, SyntaxKind::KW_END]);
            p.finish_node();
        } else {
            p.start_node(SyntaxKind::ELSE_BRANCH);
            p.bump(); // else
            parse_block(p, &[SyntaxKind::KW_END]);
            p.finish_node();
            break;
        }
    }

    p.expect(SyntaxKind::KW_END, "expected 'end if'");
    p.eat(SyntaxKind::KW_IF);
    p.finish_node();
}

fn parse_while_stmt(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::WHILE_STMT);
    p.bump(); // while
    parse_expr(p);
    p.expect(SyntaxKind::KW_DO, "expected 'do' after while condition");
    parse_block(p, &[SyntaxKind::KW_END]);
    p.expect(SyntaxKind::KW_END, "expected 'end while'");
    p.eat(SyntaxKind::KW_WHILE);
    p.finish_node();
}

fn parse_for_stmt(p: &mut Parser<'_>, parallel: bool) {
    p.start_node(SyntaxKind::FOR_STMT);
    if parallel {
        p.bump(); // parallel
    }
    p.bump(); // for
    p.eat(SyntaxKind::KW_EACH);
    p.expect_ident("expected loop variable in 'for' loop");
    p.expect(SyntaxKind::KW_IN, "expected 'in' in 'for' loop");
    parse_expr(p);
    p.expect(SyntaxKind::KW_DO, "expected 'do' in 'for' loop");
    parse_block(p, &[SyntaxKind::KW_END]);
    p.expect(SyntaxKind::KW_END, "expected 'end for'");
    p.eat(SyntaxKind::KW_FOR);
    p.finish_node();
}

fn parse_match_stmt(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::MATCH_STMT);
    p.bump(); // match
    parse_expr(p);
    while p.at(SyntaxKind::KW_CASE) {
        p.start_node(SyntaxKind::MATCH_ARM);
        p.bump(); // case
        parse_expr(p); // pattern expression (e.g. Some(x), None, Ok(v), Err(e), literal, _)
        p.expect(SyntaxKind::FAT_ARROW, "expected '=>' after match pattern");
        parse_block(p, &[SyntaxKind::KW_CASE, SyntaxKind::KW_END]);
        p.finish_node();
    }
    p.expect(SyntaxKind::KW_END, "expected 'end match'");
    p.eat(SyntaxKind::KW_MATCH);
    p.finish_node();
}
