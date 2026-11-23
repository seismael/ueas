/// <reference types="tree-sitter-cli/dsl" />
// Tree-sitter grammar for UEAS v2.0 (Universal Executable Algorithm Standard)

module.exports = grammar({
  name: 'ueas',

  extras: $ => [
    /[ \t\r\n]+/,
    $.line_comment,
    $.block_comment,
  ],

  word: $ => $.identifier,

  rules: {
    source_file: $ => seq(
      optional($.module_decl),
      repeat($.import_decl),
      repeat(choice($.record_decl, $.algorithm_decl, $.test_decl, $.benchmark_decl))
    ),

    line_comment: _ => token(seq('#', /[^\r\n]*/)),
    block_comment: _ => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),

    module_decl: $ => seq(/module/i, $.dotted_ident),
    import_decl: $ => seq(/import/i, optional(':'), $.dotted_ident),
    dotted_ident: $ => seq($.identifier, repeat(seq('.', $.identifier))),

    record_decl: $ => seq(
      /record/i,
      $.identifier,
      optional($.type_params),
      repeat($.field_decl),
      /end/i,
      optional(/record/i)
    ),

    field_decl: $ => seq($.identifier, ':', $.type_expr),

    algorithm_decl: $ => seq(
      repeat($.decorator),
      choice(/algorithm/i, /function/i, /procedure/i),
      $.identifier,
      optional($.type_params),
      '(',
      optional(sep1(',', $.param)),
      ')',
      optional(seq('->', $.type_expr)),
      repeat(choice($.require_block, $.ensure_block, $.complexity_block)),
      repeat($._statement),
      optional(seq(/end/i, optional(/algorithm/i)))
    ),

    decorator: $ => seq(
      '@',
      $.identifier,
      optional(seq('(', sep1(',', $.hw_param), ')'))
    ),

    hw_param: $ => seq($.identifier, '=', $.int_literal, optional($.identifier)),

    type_params: $ => seq('<', sep1(',', $.type_param), '>'),
    type_param: $ => seq($.identifier, optional(seq(':', sep1('+', $.identifier)))),

    param: $ => seq(optional(/mut/i), $.identifier, optional(seq(':', $.type_expr))),

    require_block: $ => seq(/require/i, optional(':'), repeat(choice($.param, $._expr))),
    ensure_block: $ => seq(
      /ensure/i,
      optional(seq('(', $.identifier, ')')),
      optional(':'),
      repeat(choice($.type_expr, $._expr))
    ),
    complexity_block: $ => seq(
      choice(/complexity/i, /memory/i),
      optional(':'),
      sep1(',', $.complexity_entry)
    ),
    complexity_entry: $ => choice(
      $.string_literal,
      seq($.identifier, choice(':', '='), choice($.string_literal, $._expr))
    ),

    test_decl: $ => seq('@', /test/i, $.string_literal, repeat($._statement), /end/i, optional(/test/i)),
    benchmark_decl: $ => seq(
      '@', /benchmark/i, $.string_literal,
      optional(seq(/scales/i, '[', sep1(',', $.int_literal), ']')),
      optional(seq(/generator/i, '(', $.identifier, ')', '=>', $._expr)),
      /end/i, optional(/benchmark/i)
    ),

    _statement: $ => choice(
      $.let_stmt,
      $.const_stmt,
      $.assign_stmt,
      $.return_stmt,
      $.if_stmt,
      $.while_stmt,
      $.for_stmt,
      $.match_stmt,
      $.assert_stmt,
      $.invariant_stmt,
      $.decreases_stmt,
      $.break_stmt,
      $.continue_stmt,
      $.pass_stmt,
      $.sync_stmt,
      $.expr_stmt
    ),

    let_stmt: $ => seq(/let/i, optional(/mut/i), $.identifier, optional(seq(':', $.type_expr)), choice('<-', ':='), $._expr),
    const_stmt: $ => seq(/const/i, $.identifier, optional(seq(':', $.type_expr)), choice('<-', ':='), $._expr),
    assign_stmt: $ => prec.right(1, seq($._expr, choice('<-', ':='), $._expr)),
    return_stmt: $ => prec.right(1, seq(/return/i, optional($._expr))),
    assert_stmt: $ => seq(/assert/i, $._expr),
    invariant_stmt: $ => seq(/invariant/i, $._expr),
    decreases_stmt: $ => seq(/decreases/i, $._expr),
    break_stmt: _ => /break/i,
    continue_stmt: _ => /continue/i,
    pass_stmt: _ => /pass/i,
    sync_stmt: _ => /sync/i,
    expr_stmt: $ => $._expr,

    if_stmt: $ => seq(
      /if/i, $._expr, /then/i,
      repeat($._statement),
      repeat($.else_if_branch),
      optional($.else_branch),
      /end/i, optional(/if/i)
    ),
    else_if_branch: $ => seq(/else/i, /if/i, $._expr, /then/i, repeat($._statement)),
    else_branch: $ => seq(/else/i, repeat($._statement)),

    while_stmt: $ => seq(/while/i, $._expr, /do/i, repeat($._statement), /end/i, optional(/while/i)),
    for_stmt: $ => seq(optional(/parallel/i), /for/i, optional(/each/i), $.identifier, /in/i, $._expr, /do/i, repeat($._statement), /end/i, optional(/for/i)),
    match_stmt: $ => seq(/match/i, $._expr, repeat($.match_arm), /end/i, optional(/match/i)),
    match_arm: $ => seq(/case/i, $._expr, '=>', repeat($._statement)),

    _expr: $ => choice(
      $.binary_expr,
      $.unary_expr,
      $.quantifier_expr,
      $.call_expr,
      $.index_expr,
      $.field_expr,
      $.list_expr,
      $.set_or_map_expr,
      $.paren_expr,
      $.identifier,
      $.int_literal,
      $.real_literal,
      $.string_literal,
      $.bool_literal,
      $.none_literal
    ),

    binary_expr: $ => choice(
      prec.right(1, seq($._expr, choice('==>', '<==>'), $._expr)),
      prec.left(2, seq($._expr, /or/i, $._expr)),
      prec.left(3, seq($._expr, /and/i, $._expr)),
      prec.left(4, seq($._expr, choice('==', '!=', '<', '<=', '>', '>=', /in/i), $._expr)),
      prec.left(5, seq($._expr, choice('..', '..='), $._expr)),
      prec.left(6, seq($._expr, '|', $._expr)),
      prec.left(7, seq($._expr, '^', $._expr)),
      prec.left(8, seq($._expr, '&', $._expr)),
      prec.left(9, seq($._expr, choice('<<', '>>'), $._expr)),
      prec.left(10, seq($._expr, choice('+', '-'), $._expr)),
      prec.left(11, seq($._expr, choice('*', '/', /mod/i, '%'), $._expr)),
    ),

    unary_expr: $ => prec.right(12, seq(choice('-', /not/i, '~', /spawn/i), $._expr)),
    quantifier_expr: $ => prec.right(1, seq(choice(/forall/i, /exists/i), $.identifier, /in/i, $._expr, '::', $._expr)),
    call_expr: $ => prec(14, seq($._expr, '(', optional(sep1(',', seq(optional(/mut/i), $._expr))), ')')),
    index_expr: $ => prec(14, seq($._expr, '[', optional($._expr), optional(seq(':', optional($._expr))), ']')),
    field_expr: $ => prec(14, seq($._expr, '.', $.identifier)),
    list_expr: $ => seq('[', optional(sep1(',', $._expr)), ']'),
    set_or_map_expr: $ => seq('{', optional(sep1(',', choice($._expr, seq($._expr, ':', $._expr)))), '}'),
    paren_expr: $ => seq('(', $._expr, ')'),

    type_expr: $ => prec.right(seq($.identifier, optional(seq('<', sep1(',', choice($.type_expr, $.int_literal)), '>')))),

    identifier: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    int_literal: _ => /[0-9]+/,
    real_literal: _ => /[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?/,
    string_literal: _ => /"([^"\\]|\\.)*"/,
    bool_literal: _ => choice(/true/i, /false/i),
    none_literal: _ => /none/i,
  }
});

function sep1(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)));
}
