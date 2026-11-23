//! Top-level declarations (`module`, `import`, `Record`, `Algorithm`, `@Test`, `@Benchmark`).

use crate::expr::{parse_expr, parse_type_expr};
use crate::parser::Parser;
use crate::stmt::parse_block;
use crate::syntax_kind::SyntaxKind;

pub(crate) fn parse_source_file(p: &mut Parser<'_>) {
    p.start_root();
    while !p.at(SyntaxKind::EOF) {
        match p.current() {
            SyntaxKind::KW_MODULE => parse_module_decl(p),
            SyntaxKind::KW_IMPORT => parse_import_decl(p),
            SyntaxKind::KW_RECORD => parse_record_decl(p),
            SyntaxKind::AT => parse_decorated_or_test(p),
            SyntaxKind::KW_ALGORITHM | SyntaxKind::KW_FUNCTION | SyntaxKind::KW_PROCEDURE => {
                parse_algorithm_decl(p, 0);
            }
            _ => {
                p.error_and_bump(
                    "expected top-level declaration (module, import, Record, Algorithm, @Test, @Benchmark)",
                    None,
                );
            }
        }
    }
}

fn parse_module_decl(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::MODULE_DECL);
    p.bump(); // module
    parse_dotted_ident(p);
    p.finish_node();
}

fn parse_import_decl(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::IMPORT_DECL);
    p.bump(); // import
    p.eat(SyntaxKind::COLON);
    parse_dotted_ident(p);
    p.finish_node();
}

fn parse_dotted_ident(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::DOTTED_IDENT);
    p.expect_ident("expected module/import identifier");
    while !p.at_newline_boundary() && p.eat(SyntaxKind::DOT) {
        p.expect_ident("expected identifier after '.'");
    }
    p.finish_node();
}

fn parse_decorated_or_test(p: &mut Parser<'_>) {
    // Peek ahead to check @Test or @Benchmark vs @ConstantTime / @HardwareProfile
    if p.nth(1, true) == SyntaxKind::IDENT {
        // We can parse decorator first; if it's @Test or @Benchmark, emit TEST_DECL/BENCHMARK_DECL
        let cp = p.checkpoint();
        parse_decorator(p);
        // Check what follows:
        let mut dec_count = 1usize;
        while p.at(SyntaxKind::AT) {
            parse_decorator(p);
            dec_count += 1;
        }
        match p.current() {
            SyntaxKind::KW_ALGORITHM | SyntaxKind::KW_FUNCTION | SyntaxKind::KW_PROCEDURE => {
                p.start_node_at(cp, SyntaxKind::ALGORITHM_DECL);
                parse_algorithm_inner(p);
                p.finish_node();
            }
            SyntaxKind::KW_RECORD => {
                let _ = dec_count;
                parse_record_decl(p);
            }
            SyntaxKind::STRING_LIT => {
                // @Test "name" ... end Test OR @Benchmark "name" ... end Benchmark
                // Wrap at checkpoint!
                // Note: inspect whether decorator was Test or Benchmark by checking block tokens
                parse_test_or_benchmark_tail(p, cp);
            }
            _ => {
                p.error(
                    "expected Algorithm, Record, or test name after decorator",
                    None,
                );
            }
        }
    } else {
        p.error_and_bump("expected decorator name after '@'", None);
    }
}

fn parse_decorator(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::DECORATOR);
    p.bump(); // @
    p.expect_ident("expected decorator identifier");
    if !p.at_newline_boundary() && p.eat(SyntaxKind::LPAREN) {
        while !p.at(SyntaxKind::RPAREN) && !p.at(SyntaxKind::EOF) {
            p.start_node(SyntaxKind::HW_PARAM);
            p.expect_ident("expected hardware parameter name");
            p.expect(SyntaxKind::ASSIGN_EQ, "expected '=' in decorator parameter");
            if p.at(SyntaxKind::INT_LIT) {
                p.bump();
            }
            if p.is_ident_like(p.current()) {
                p.bump(); // KB / MB / B
            }
            p.finish_node();
            if !p.eat(SyntaxKind::COMMA) {
                break;
            }
        }
        p.expect(SyntaxKind::RPAREN, "expected ')' closing decorator");
    }
    p.finish_node();
}

fn parse_test_or_benchmark_tail(p: &mut Parser<'_>, cp: rowan::Checkpoint) {
    // Check if body contains `scales` / `generator` -> Benchmark, else Test
    // Better yet: scan lookahead for KW_SCALES / KW_GENERATOR before KW_END
    let mut is_bench = false;
    let mut i = 0;
    loop {
        let k = p.nth(i, true);
        if matches!(k, SyntaxKind::EOF | SyntaxKind::KW_END) {
            break;
        }
        if matches!(k, SyntaxKind::KW_SCALES | SyntaxKind::KW_GENERATOR) {
            is_bench = true;
            break;
        }
        i += 1;
    }

    if is_bench {
        p.start_node_at(cp, SyntaxKind::BENCHMARK_DECL);
        p.bump(); // STRING_LIT
        if p.eat(SyntaxKind::KW_SCALES) {
            p.expect(SyntaxKind::LBRACKET, "expected '[' after scales");
            while !p.at(SyntaxKind::RBRACKET) && !p.at(SyntaxKind::EOF) {
                if p.at(SyntaxKind::INT_LIT) {
                    p.start_node(SyntaxKind::LITERAL_EXPR);
                    p.bump();
                    p.finish_node();
                } else {
                    p.error_and_bump("expected integer scale", None);
                }
                if !p.eat(SyntaxKind::COMMA) {
                    break;
                }
            }
            p.expect(SyntaxKind::RBRACKET, "expected ']'");
        }
        if p.eat(SyntaxKind::KW_GENERATOR) {
            p.expect(SyntaxKind::LPAREN, "expected '(' after generator");
            p.expect_ident("expected scale parameter name");
            p.expect(SyntaxKind::RPAREN, "expected ')'");
            p.expect(SyntaxKind::FAT_ARROW, "expected '=>' before generator body");
            parse_expr(p);
        }
        p.expect(SyntaxKind::KW_END, "expected 'end Benchmark'");
        if p.is_ident_like(p.current()) {
            p.bump(); // Benchmark
        }
        p.finish_node();
    } else {
        p.start_node_at(cp, SyntaxKind::TEST_DECL);
        p.bump(); // STRING_LIT
        parse_block(p, &[SyntaxKind::KW_END]);
        p.expect(SyntaxKind::KW_END, "expected 'end Test'");
        if p.is_ident_like(p.current()) {
            p.bump(); // Test
        }
        p.finish_node();
    }
}

fn parse_record_decl(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::RECORD_DECL);
    p.bump(); // Record
    p.expect_ident("expected Record name");
    if !p.at_newline_boundary() && p.at(SyntaxKind::LT) {
        parse_type_params(p);
    }
    while !p.at(SyntaxKind::KW_END) && !p.at(SyntaxKind::EOF) {
        if !p.is_ident_like(p.current()) {
            p.error_and_bump("expected field name in Record", None);
            continue;
        }
        p.start_node(SyntaxKind::FIELD_DECL);
        p.expect_ident("expected field name in Record");
        p.expect(SyntaxKind::COLON, "expected ':' after field name");
        parse_type_expr(p);
        p.eat(SyntaxKind::COMMA);
        p.finish_node();
    }
    p.expect(SyntaxKind::KW_END, "expected 'end Record'");
    p.eat(SyntaxKind::KW_RECORD);
    p.finish_node();
}

fn parse_algorithm_decl(p: &mut Parser<'_>, _dec_count: usize) {
    p.start_node(SyntaxKind::ALGORITHM_DECL);
    parse_algorithm_inner(p);
    p.finish_node();
}

fn parse_algorithm_inner(p: &mut Parser<'_>) {
    p.bump(); // Algorithm / Function / Procedure
    p.expect_ident("expected Algorithm name");
    if !p.at_newline_boundary() && p.at(SyntaxKind::LT) {
        parse_type_params(p);
    }

    // Parameter list
    p.start_node(SyntaxKind::PARAM_LIST);
    p.expect(SyntaxKind::LPAREN, "expected '(' after Algorithm name");
    while !p.at(SyntaxKind::RPAREN) && !p.at(SyntaxKind::EOF) {
        p.start_node(SyntaxKind::PARAM);
        p.eat(SyntaxKind::KW_MUT);
        p.expect_ident("expected parameter name");
        if p.eat(SyntaxKind::COLON) {
            parse_type_expr(p);
        }
        p.finish_node();
        if !p.eat(SyntaxKind::COMMA) {
            break;
        }
    }
    p.expect(SyntaxKind::RPAREN, "expected ')' closing parameters");
    p.finish_node();

    // Optional `-> ReturnType`
    if !p.at_newline_boundary() && p.eat(SyntaxKind::RARROW) {
        parse_type_expr(p);
    }

    // Optional preamble blocks (Require, Ensure, Complexity, Memory — in any standard order)
    loop {
        match p.current() {
            SyntaxKind::KW_REQUIRE => parse_require_block(p),
            SyntaxKind::KW_ENSURE => parse_ensure_block(p),
            SyntaxKind::KW_COMPLEXITY => parse_complexity_block(p),
            SyntaxKind::KW_MEMORY => parse_memory_line(p),
            _ => break,
        }
    }

    // Algorithm body until `end Algorithm` or next top-level `Algorithm` (graceful compat for legacy files)
    parse_block(
        p,
        &[
            SyntaxKind::KW_END,
            SyntaxKind::KW_ALGORITHM,
            SyntaxKind::KW_RECORD,
            SyntaxKind::AT,
        ],
    );
    if p.eat(SyntaxKind::KW_END) {
        p.eat(SyntaxKind::KW_ALGORITHM);
    }
}

fn parse_type_params(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::TYPE_PARAMS);
    p.bump(); // <
    while !p.at(SyntaxKind::GT) && !p.at(SyntaxKind::EOF) {
        p.start_node(SyntaxKind::TYPE_PARAM);
        p.expect_ident("expected type parameter name");
        if p.eat(SyntaxKind::COLON) {
            p.expect_ident("expected trait bound (Eq, Ord, Hash, Numeric)");
            while p.eat(SyntaxKind::PLUS) {
                p.expect_ident("expected trait bound after '+'");
            }
        }
        p.finish_node();
        if !p.eat(SyntaxKind::COMMA) {
            break;
        }
    }
    p.expect(SyntaxKind::GT, "expected '>' closing type parameters");
    p.finish_node();
}

fn parse_require_block(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::REQUIRE_BLOCK);
    p.bump(); // Require
    p.eat(SyntaxKind::COLON);
    // Support both inline `Require: a: Int, b: Int` AND multi-line predicates
    while !matches!(
        p.current(),
        SyntaxKind::KW_ENSURE
            | SyntaxKind::KW_COMPLEXITY
            | SyntaxKind::KW_MEMORY
            | SyntaxKind::KW_END
            | SyntaxKind::EOF
    ) {
        // Stop if we've reached body statements
        if is_definitely_body_stmt(p) {
            break;
        }
        if p.is_ident_like(p.current()) && p.nth(1, true) == SyntaxKind::COLON {
            p.start_node(SyntaxKind::PARAM);
            p.eat(SyntaxKind::KW_MUT);
            p.bump(); // param name
            p.bump(); // :
            parse_type_expr(p);
            p.finish_node();
            p.eat(SyntaxKind::COMMA);
        } else {
            parse_expr(p);
            p.eat(SyntaxKind::COMMA);
        }
    }
    p.finish_node();
}

fn parse_ensure_block(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::ENSURE_BLOCK);
    p.bump(); // Ensure
    if !p.at_newline_boundary() && p.eat(SyntaxKind::LPAREN) {
        p.expect_ident("expected result binding name in Ensure(...)");
        p.expect(SyntaxKind::RPAREN, "expected ')'");
    }
    p.eat(SyntaxKind::COLON);

    while !matches!(
        p.current(),
        SyntaxKind::KW_COMPLEXITY
            | SyntaxKind::KW_MEMORY
            | SyntaxKind::KW_REQUIRE
            | SyntaxKind::KW_END
            | SyntaxKind::EOF
    ) {
        if is_definitely_body_stmt(p) {
            break;
        }
        // Check if this is a standalone Return Type (e.g. `Ensure: Integer` or `Ensure: List<Int>`)
        if is_type_only_line(p) {
            parse_type_expr(p);
        } else {
            parse_expr(p);
        }
        p.eat(SyntaxKind::COMMA);
    }
    p.finish_node();
}

fn is_type_only_line(p: &Parser<'_>) -> bool {
    if !p.is_ident_like(p.current()) {
        return false;
    }
    // Look ahead: either `<ident>` followed by newline/EOF, or `<ident> < ... >` followed by newline/EOF
    let mut i = 1;
    if p.nth(1, false) == SyntaxKind::LT {
        // Could be `Type<...>` OR `x < 5`. Only treat as type if balanced `<...>` ends at newline/EOF
        // and starts with capitalized identifier (UEAS types are PascalCase: List, Map, Set, Option, Result, etc.)
        let mut depth = 0i32;
        loop {
            let k = p.nth(i, false);
            match k {
                SyntaxKind::LT => depth += 1,
                SyntaxKind::GT => {
                    depth -= 1;
                    if depth == 0 {
                        let after = p.nth(i + 1, false);
                        return matches!(
                            after,
                            SyntaxKind::NEWLINE | SyntaxKind::EOF | SyntaxKind::COMMA
                        );
                    }
                }
                SyntaxKind::NEWLINE | SyntaxKind::EOF => return false,
                _ => {}
            }
            i += 1;
        }
    }
    let next_raw = p.nth(1, false);
    matches!(next_raw, SyntaxKind::NEWLINE | SyntaxKind::EOF)
}

fn is_definitely_body_stmt(p: &Parser<'_>) -> bool {
    match p.current() {
        SyntaxKind::KW_LET
        | SyntaxKind::KW_CONST
        | SyntaxKind::KW_RETURN
        | SyntaxKind::KW_IF
        | SyntaxKind::KW_WHILE
        | SyntaxKind::KW_FOR
        | SyntaxKind::KW_PARALLEL
        | SyntaxKind::KW_MATCH
        | SyntaxKind::KW_ASSERT
        | SyntaxKind::KW_INVARIANT
        | SyntaxKind::KW_DECREASES
        | SyntaxKind::KW_BREAK
        | SyntaxKind::KW_CONTINUE
        | SyntaxKind::KW_PASS
        | SyntaxKind::KW_SYNC
        | SyntaxKind::KW_SPAWN => true,
        _ => {
            // Check if this line contains `<-` assignment
            let mut i = 0;
            loop {
                let k = p.nth(i, false);
                if matches!(k, SyntaxKind::NEWLINE | SyntaxKind::EOF) {
                    return false;
                }
                if k == SyntaxKind::LARROW {
                    return true;
                }
                i += 1;
            }
        }
    }
}

fn parse_complexity_block(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::COMPLEXITY_BLOCK);
    p.bump(); // Complexity
    p.eat(SyntaxKind::COLON);
    while !matches!(
        p.current(),
        SyntaxKind::KW_MEMORY
            | SyntaxKind::KW_REQUIRE
            | SyntaxKind::KW_ENSURE
            | SyntaxKind::KW_END
            | SyntaxKind::EOF
    ) {
        if is_definitely_body_stmt(p) {
            break;
        }
        p.start_node(SyntaxKind::COMPLEXITY_ENTRY);
        if p.at(SyntaxKind::STRING_LIT) {
            p.start_node(SyntaxKind::LITERAL_EXPR);
            p.bump();
            p.finish_node();
        } else if p.is_ident_like(p.current())
            && matches!(p.nth(1, true), SyntaxKind::COLON | SyntaxKind::ASSIGN_EQ)
        {
            p.bump(); // key (Time, Space, Work, Span, Expected, N, V, E)
            p.bump(); // : or =
            if p.at(SyntaxKind::STRING_LIT) {
                p.start_node(SyntaxKind::LITERAL_EXPR);
                p.bump();
                p.finish_node();
            } else if p.is_ident_like(p.current()) && p.nth(1, false) == SyntaxKind::LPAREN {
                parse_big_o_formula(p);
                if p.eat(SyntaxKind::KW_WHERE) {
                    loop {
                        p.expect_ident("expected binding variable after 'where'");
                        p.expect(SyntaxKind::ASSIGN_EQ, "expected '=' in complexity binding");
                        parse_expr(p);
                        if p.at(SyntaxKind::COMMA) && p.nth(2, true) == SyntaxKind::ASSIGN_EQ {
                            p.bump();
                        } else {
                            break;
                        }
                    }
                }
            } else {
                parse_expr(p);
            }
        } else if p.is_ident_like(p.current()) {
            if p.nth(1, false) == SyntaxKind::LPAREN {
                parse_big_o_formula(p);
            } else {
                parse_expr(p);
            }
            if p.eat(SyntaxKind::KW_WHERE) {
                loop {
                    p.expect_ident("expected binding variable after 'where'");
                    p.expect(SyntaxKind::ASSIGN_EQ, "expected '=' in complexity binding");
                    parse_expr(p);
                    if !p.eat(SyntaxKind::COMMA) {
                        break;
                    }
                }
            }
        } else {
            p.finish_node();
            break;
        }
        p.finish_node();
        if !p.eat(SyntaxKind::COMMA) && p.at_newline_boundary() {
            // Continue only if next line is another Complexity entry (Time/Space/Work/Span)
            let next = p.current();
            if !(p.is_ident_like(next)
                && matches!(p.nth(1, true), SyntaxKind::COLON | SyntaxKind::ASSIGN_EQ))
            {
                break;
            }
        }
    }
    p.finish_node();
}

fn parse_big_o_formula(p: &mut Parser<'_>) {
    p.bump(); // O
    if p.eat(SyntaxKind::LPAREN) {
        let mut depth = 1i32;
        while depth > 0 && !p.at(SyntaxKind::EOF) {
            if p.at(SyntaxKind::LPAREN) {
                depth += 1;
            } else if p.at(SyntaxKind::RPAREN) {
                depth -= 1;
            }
            p.bump();
        }
    }
}

fn parse_memory_line(p: &mut Parser<'_>) {
    p.start_node(SyntaxKind::COMPLEXITY_BLOCK);
    p.bump(); // Memory
    p.eat(SyntaxKind::COLON);
    p.start_node(SyntaxKind::COMPLEXITY_ENTRY);
    parse_expr(p);
    p.finish_node();
    p.finish_node();
}
