//! Canonical Lowered Imperative IR (`LIR`) desugaring high-level HIR constructs once.

use serde::{Deserialize, Serialize};
use ueas_hir::hir::{
    AlgorithmDecl, BinaryOp, Expr, LValue, Param, Program, RecordDecl, Stmt, UnaryOp,
};
use ueas_hir::types::Type;
use ueas_syntax::SourceSpan;

/// Lowered UEAS program ready for source, WASM, or plugin emission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LirProgram {
    pub module: Option<String>,
    pub records: Vec<RecordDecl>,
    pub functions: Vec<LirFunc>,
}

/// Lowered imperative function with desugared contracts and quantifiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LirFunc {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub preconditions: Vec<Expr>,
    pub postconditions: Vec<Expr>,
    pub postcondition_binding: Option<String>,
    pub constant_time: bool,
    pub body: Vec<LirStmt>,
}

/// Lowered statement (no `Quantifier` expressions left inside conditions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LirStmt {
    Let {
        name: String,
        is_mut: bool,
        ty: Option<Type>,
        init: Expr,
    },
    Assign {
        target: LValue,
        value: Expr,
    },
    Expr {
        expr: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    Assert {
        condition: Expr,
        message: String,
    },
    If {
        condition: Expr,
        then_body: Vec<LirStmt>,
        else_body: Vec<LirStmt>,
    },
    While {
        condition: Expr,
        body: Vec<LirStmt>,
    },
    Break,
    Continue,
}

/// Lower an HIR `Program` into a canonical `LirProgram`, desugaring `For` loops and quantifiers.
pub fn lower_hir_to_lir(program: &Program) -> LirProgram {
    let mut counter = 0usize;
    let functions = program
        .algorithms
        .iter()
        .map(|a| lower_algo(a, &mut counter))
        .collect();
    LirProgram {
        module: program.module.clone(),
        records: program.records.clone(),
        functions,
    }
}

fn lower_algo(algo: &AlgorithmDecl, counter: &mut usize) -> LirFunc {
    let mut body = Vec::new();
    for req in &algo.preconditions {
        let desugared = desugar_expr(req, &mut body, counter);
        body.push(LirStmt::Assert {
            condition: desugared,
            message: format!("Require precondition failed in {}", algo.name),
        });
    }
    lower_stmts(
        &algo.body,
        &algo.postconditions,
        algo.postcondition_binding.as_deref(),
        &mut body,
        counter,
    );
    LirFunc {
        name: algo.name.clone(),
        params: algo.parameters.clone(),
        return_type: algo.return_type.clone(),
        preconditions: algo.preconditions.clone(),
        postconditions: algo.postconditions.clone(),
        postcondition_binding: algo.postcondition_binding.clone(),
        constant_time: algo.constant_time,
        body,
    }
}

fn lower_stmts(
    stmts: &[Stmt],
    posts: &[Expr],
    post_bind: Option<&str>,
    out: &mut Vec<LirStmt>,
    counter: &mut usize,
) {
    for s in stmts {
        match s {
            Stmt::Let { name, ty, init, .. } => {
                let d = desugar_expr(init, out, counter);
                out.push(LirStmt::Let {
                    name: name.clone(),
                    is_mut: true,
                    ty: ty.clone(),
                    init: d,
                });
            }
            Stmt::Const { name, ty, init, .. } => {
                let d = desugar_expr(init, out, counter);
                out.push(LirStmt::Let {
                    name: name.clone(),
                    is_mut: false,
                    ty: ty.clone(),
                    init: d,
                });
            }
            Stmt::Assign { target, value, .. } => {
                let d = desugar_expr(value, out, counter);
                out.push(LirStmt::Assign {
                    target: target.clone(),
                    value: d,
                });
            }
            Stmt::Expr { expr, .. } => {
                let d = desugar_expr(expr, out, counter);
                out.push(LirStmt::Expr { expr: d });
            }
            Stmt::Return { value, span } => {
                if let Some(v) = value {
                    let d = desugar_expr(v, out, counter);
                    if posts.is_empty() {
                        out.push(LirStmt::Return { value: Some(d) });
                    } else {
                        let rname = post_bind.unwrap_or("result").to_string();
                        out.push(LirStmt::Let {
                            name: rname.clone(),
                            is_mut: false,
                            ty: None,
                            init: d,
                        });
                        for p in posts {
                            let dp = desugar_expr(p, out, counter);
                            out.push(LirStmt::Assert {
                                condition: dp,
                                message: "Ensure postcondition failed".to_string(),
                            });
                        }
                        out.push(LirStmt::Return {
                            value: Some(Expr::Ident {
                                name: rname,
                                span: *span,
                            }),
                        });
                    }
                } else {
                    out.push(LirStmt::Return { value: None });
                }
            }
            Stmt::Assert {
                condition, message, ..
            } => {
                let d = desugar_expr(condition, out, counter);
                out.push(LirStmt::Assert {
                    condition: d,
                    message: message
                        .clone()
                        .unwrap_or_else(|| "assertion failed".to_string()),
                });
            }
            Stmt::If {
                condition,
                then_body,
                else_if_branches,
                else_body,
                ..
            } => {
                let cond = desugar_expr(condition, out, counter);
                let mut then_l = Vec::new();
                lower_stmts(then_body, posts, post_bind, &mut then_l, counter);
                let mut tail = Vec::new();
                if let Some(eb) = else_body {
                    lower_stmts(eb, posts, post_bind, &mut tail, counter);
                }
                for (ec, eb) in else_if_branches.iter().rev() {
                    let mut branch_then = Vec::new();
                    lower_stmts(eb, posts, post_bind, &mut branch_then, counter);
                    let mut cond_pre = Vec::new();
                    let dec = desugar_expr(ec, &mut cond_pre, counter);
                    cond_pre.push(LirStmt::If {
                        condition: dec,
                        then_body: branch_then,
                        else_body: tail,
                    });
                    tail = cond_pre;
                }
                out.push(LirStmt::If {
                    condition: cond,
                    then_body: then_l,
                    else_body: tail,
                });
            }
            Stmt::While {
                condition, body, ..
            } => {
                let cond = desugar_expr(condition, out, counter);
                let mut b = Vec::new();
                lower_stmts(body, posts, post_bind, &mut b, counter);
                out.push(LirStmt::While {
                    condition: cond,
                    body: b,
                });
            }
            Stmt::For {
                var,
                iterable:
                    Expr::Range {
                        start,
                        end,
                        inclusive,
                        ..
                    },
                body,
                span,
                ..
            } => {
                let ds = desugar_expr(start, out, counter);
                let de = desugar_expr(end, out, counter);
                *counter += 1;
                let end_var = format!("__lir_end_{counter}");
                out.push(LirStmt::Let {
                    name: var.clone(),
                    is_mut: true,
                    ty: None,
                    init: ds,
                });
                out.push(LirStmt::Let {
                    name: end_var.clone(),
                    is_mut: false,
                    ty: None,
                    init: de,
                });
                let mut lbody = Vec::new();
                lower_stmts(body, posts, post_bind, &mut lbody, counter);
                lbody.push(LirStmt::Assign {
                    target: LValue::Var {
                        name: var.clone(),
                        span: *span,
                    },
                    value: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Ident {
                            name: var.clone(),
                            span: *span,
                        }),
                        right: Box::new(Expr::IntLit {
                            value: "1".to_string(),
                            span: *span,
                        }),
                        span: *span,
                    },
                });
                out.push(LirStmt::While {
                    condition: Expr::Binary {
                        op: if *inclusive {
                            BinaryOp::Le
                        } else {
                            BinaryOp::Lt
                        },
                        left: Box::new(Expr::Ident {
                            name: var.clone(),
                            span: *span,
                        }),
                        right: Box::new(Expr::Ident {
                            name: end_var,
                            span: *span,
                        }),
                        span: *span,
                    },
                    body: lbody,
                });
            }
            Stmt::Break { .. } => out.push(LirStmt::Break),
            Stmt::Continue { .. } => out.push(LirStmt::Continue),
            _ => {}
        }
    }
}

fn desugar_expr(expr: &Expr, stmts: &mut Vec<LirStmt>, counter: &mut usize) -> Expr {
    match expr {
        Expr::Quantifier {
            is_forall,
            var,
            iterable,
            body,
            span,
        } => {
            *counter += 1;
            let res_var = format!("__lir_quant_{counter}");
            stmts.push(LirStmt::Let {
                name: res_var.clone(),
                is_mut: true,
                ty: None,
                init: Expr::BoolLit {
                    value: *is_forall,
                    span: *span,
                },
            });
            if let Expr::Range {
                start,
                end,
                inclusive,
                ..
            } = iterable.as_ref()
            {
                let ds = desugar_expr(start, stmts, counter);
                let de = desugar_expr(end, stmts, counter);
                *counter += 1;
                let end_var = format!("__lir_qend_{counter}");
                stmts.push(LirStmt::Let {
                    name: var.clone(),
                    is_mut: true,
                    ty: None,
                    init: ds,
                });
                stmts.push(LirStmt::Let {
                    name: end_var.clone(),
                    is_mut: false,
                    ty: None,
                    init: de,
                });
                let mut lbody = Vec::new();
                let db = desugar_expr(body, &mut lbody, counter);
                let trigger = if *is_forall {
                    Expr::Unary {
                        op: UnaryOp::Not,
                        operand: Box::new(db),
                        span: *span,
                    }
                } else {
                    db
                };
                lbody.push(LirStmt::If {
                    condition: trigger,
                    then_body: vec![
                        LirStmt::Assign {
                            target: LValue::Var {
                                name: res_var.clone(),
                                span: *span,
                            },
                            value: Expr::BoolLit {
                                value: !*is_forall,
                                span: *span,
                            },
                        },
                        LirStmt::Break,
                    ],
                    else_body: Vec::new(),
                });
                lbody.push(LirStmt::Assign {
                    target: LValue::Var {
                        name: var.clone(),
                        span: *span,
                    },
                    value: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Ident {
                            name: var.clone(),
                            span: *span,
                        }),
                        right: Box::new(Expr::IntLit {
                            value: "1".to_string(),
                            span: *span,
                        }),
                        span: *span,
                    },
                });
                stmts.push(LirStmt::While {
                    condition: Expr::Binary {
                        op: if *inclusive {
                            BinaryOp::Le
                        } else {
                            BinaryOp::Lt
                        },
                        left: Box::new(Expr::Ident {
                            name: var.clone(),
                            span: *span,
                        }),
                        right: Box::new(Expr::Ident {
                            name: end_var,
                            span: *span,
                        }),
                        span: *span,
                    },
                    body: lbody,
                });
            }
            Expr::Ident {
                name: res_var,
                span: *span,
            }
        }
        other => other.clone(),
    }
}

/// Utility to get a dummy `SourceSpan` if needed.
pub fn empty_span() -> SourceSpan {
    SourceSpan {
        start_byte: 0,
        end_byte: 0,
        start_line: 1,
        start_col: 1,
        end_line: 1,
        end_col: 1,
    }
}
