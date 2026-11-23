//! Static `@ConstantTime` `Public ⊑ Secret` Information-Flow Security Lattice (`CT-01`..`CT-04`).

use crate::check::CompileError;
use crate::hir::*;
use crate::types::SecurityLevel;
use std::collections::HashMap;
use ueas_syntax::SourceSpan;

/// Verify static constant-time security guarantees for all `@ConstantTime` algorithms.
pub fn check_constant_time(program: &mut Program, errors: &mut Vec<CompileError>) {
    for algo in &mut program.algorithms {
        if !algo.constant_time {
            continue;
        }
        let mut env: HashMap<String, SecurityLevel> = HashMap::new();
        for p in &algo.parameters {
            let lvl = if p.ty.is_secret() {
                SecurityLevel::Secret
            } else {
                SecurityLevel::Public
            };
            env.insert(p.name.clone(), lvl);
        }
        let mut declass = Vec::new();
        check_stmts(&algo.body, &mut env, &mut declass, errors);
        algo.declassifications = declass;
    }
}

fn check_stmts(
    stmts: &[Stmt],
    env: &mut HashMap<String, SecurityLevel>,
    declass: &mut Vec<SourceSpan>,
    errors: &mut Vec<CompileError>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { name, ty, init, .. } => {
                let mut lvl = expr_level(init, env, declass, errors);
                if ty.as_ref().is_some_and(|t| t.is_secret()) {
                    lvl = SecurityLevel::Secret;
                }
                env.insert(name.clone(), lvl);
            }
            Stmt::Const { name, ty, init, .. } => {
                let mut lvl = expr_level(init, env, declass, errors);
                if ty.as_ref().is_some_and(|t| t.is_secret()) {
                    lvl = SecurityLevel::Secret;
                }
                env.insert(name.clone(), lvl);
            }
            Stmt::Assign { target, value, .. } => {
                let lvl = expr_level(value, env, declass, errors);
                check_lvalue(target, env, declass, errors);
                if let LValue::Var { name, .. } = target {
                    env.insert(name.clone(), lvl);
                }
            }
            Stmt::Expr { expr, .. } => {
                expr_level(expr, env, declass, errors);
            }
            Stmt::Return { value: Some(v), .. } => {
                expr_level(v, env, declass, errors);
            }
            Stmt::If {
                condition,
                then_body,
                else_if_branches,
                else_body,
                span,
            } => {
                if expr_level(condition, env, declass, errors) == SecurityLevel::Secret {
                    errors.push(CompileError {
                        code: "CT-01",
                        span: *span,
                        message: "secret-dependent 'if' branch violates @ConstantTime".to_string(),
                        hint: Some(
                            "replace branching with branchless `select(cond, a, b)`".to_string(),
                        ),
                    });
                }
                check_stmts(then_body, env, declass, errors);
                for (c, b) in else_if_branches {
                    if expr_level(c, env, declass, errors) == SecurityLevel::Secret {
                        errors.push(CompileError {
                            code: "CT-01",
                            span: c.span(),
                            message: "secret-dependent 'else if' branch violates @ConstantTime"
                                .to_string(),
                            hint: Some("use branchless `select(cond, a, b)`".to_string()),
                        });
                    }
                    check_stmts(b, env, declass, errors);
                }
                if let Some(eb) = else_body {
                    check_stmts(eb, env, declass, errors);
                }
            }
            Stmt::While {
                condition,
                body,
                span,
                ..
            } => {
                if expr_level(condition, env, declass, errors) == SecurityLevel::Secret {
                    errors.push(CompileError {
                        code: "CT-02",
                        span: *span,
                        message: "secret-dependent 'while' loop bound violates @ConstantTime"
                            .to_string(),
                        hint: Some("loop bounds must depend only on Public values".to_string()),
                    });
                }
                check_stmts(body, env, declass, errors);
            }
            Stmt::For {
                iterable,
                body,
                span,
                ..
            } => {
                if expr_level(iterable, env, declass, errors) == SecurityLevel::Secret {
                    errors.push(CompileError {
                        code: "CT-02",
                        span: *span,
                        message: "secret-dependent 'for' loop range violates @ConstantTime"
                            .to_string(),
                        hint: Some("loop bounds must depend only on Public values".to_string()),
                    });
                }
                check_stmts(body, env, declass, errors);
            }
            Stmt::Match {
                scrutinee,
                arms,
                span,
            } => {
                if expr_level(scrutinee, env, declass, errors) == SecurityLevel::Secret {
                    errors.push(CompileError {
                        code: "CT-01",
                        span: *span,
                        message: "secret-dependent 'match' violates @ConstantTime".to_string(),
                        hint: Some("use branchless `select`".to_string()),
                    });
                }
                for arm in arms {
                    check_stmts(&arm.body, env, declass, errors);
                }
            }
            _ => {}
        }
    }
}

fn check_lvalue(
    lv: &LValue,
    env: &HashMap<String, SecurityLevel>,
    declass: &mut Vec<SourceSpan>,
    errors: &mut Vec<CompileError>,
) {
    match lv {
        LValue::Var { .. } => {}
        LValue::Index {
            target,
            index,
            span,
        } => {
            check_lvalue(target, env, declass, errors);
            if expr_level(index, env, declass, errors) == SecurityLevel::Secret {
                errors.push(CompileError {
                    code: "CT-03",
                    span: *span,
                    message: "secret-dependent memory index write leaks cache timing".to_string(),
                    hint: Some("index arrays/maps only with Public indices".to_string()),
                });
            }
        }
        LValue::Field { target, .. } => check_lvalue(target, env, declass, errors),
    }
}

fn expr_level(
    expr: &Expr,
    env: &HashMap<String, SecurityLevel>,
    declass: &mut Vec<SourceSpan>,
    errors: &mut Vec<CompileError>,
) -> SecurityLevel {
    match expr {
        Expr::IntLit { .. }
        | Expr::RealLit { .. }
        | Expr::BoolLit { .. }
        | Expr::StringLit { .. }
        | Expr::NoneLit { .. }
        | Expr::InfinityLit { .. }
        | Expr::NanLit { .. } => SecurityLevel::Public,
        Expr::Ident { name, .. } => env.get(name).copied().unwrap_or(SecurityLevel::Public),
        Expr::Unary { operand, .. } => expr_level(operand, env, declass, errors),
        Expr::Cast { expr, .. } => expr_level(expr, env, declass, errors),
        Expr::Binary {
            op,
            left,
            right,
            span,
        } => {
            let l = expr_level(left, env, declass, errors);
            let r = expr_level(right, env, declass, errors);
            let combined = l.max(r);
            if combined == SecurityLevel::Secret && matches!(op, BinaryOp::Div | BinaryOp::Mod) {
                errors.push(CompileError {
                    code: "CT-04",
                    span: *span,
                    message: "division/modulo on Secret operand has variable hardware latency"
                        .to_string(),
                    hint: Some("use constant-time bitwise/Barrett reduction".to_string()),
                });
            }
            combined
        }
        Expr::Index {
            target,
            index,
            span,
        } => {
            let tl = expr_level(target, env, declass, errors);
            let il = expr_level(index, env, declass, errors);
            if il == SecurityLevel::Secret {
                errors.push(CompileError {
                    code: "CT-03",
                    span: *span,
                    message: "secret-dependent array/map lookup leaks cache timing".to_string(),
                    hint: Some(
                        "scan linearly with `select(...)` instead of `arr[secret]`".to_string(),
                    ),
                });
            }
            tl
        }
        Expr::Call { callee, args, span } => {
            if callee == "declassify" {
                for a in args {
                    expr_level(&a.value, env, declass, errors);
                }
                declass.push(*span);
                return SecurityLevel::Public;
            }
            let mut lvl = SecurityLevel::Public;
            for a in args {
                lvl = lvl.max(expr_level(&a.value, env, declass, errors));
            }
            lvl
        }
        Expr::MethodCall { receiver, args, .. } => {
            let mut lvl = expr_level(receiver, env, declass, errors);
            for a in args {
                lvl = lvl.max(expr_level(&a.value, env, declass, errors));
            }
            lvl
        }
        Expr::ListLit { elements, .. }
        | Expr::SetLit { elements, .. }
        | Expr::TupleLit { elements, .. } => {
            elements.iter().fold(SecurityLevel::Public, |acc, e| {
                acc.max(expr_level(e, env, declass, errors))
            })
        }
        _ => SecurityLevel::Public,
    }
}
