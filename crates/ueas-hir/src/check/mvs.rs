//! Mutable Value Semantics (MVS) & Non-Recursive Record Cycle Checker.

use crate::check::CompileError;
use crate::hir::*;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Verify that no `Record` contains direct or mutual recursive references (enforcing Arena Pattern)
/// and that all `mut` parameters / call arguments obey non-aliasing Mutable Value Semantics.
pub fn check_mvs(program: &Program, errors: &mut Vec<CompileError>) {
    check_record_cycles(program, errors);

    let sigs: HashMap<&str, &AlgorithmDecl> = program
        .algorithms
        .iter()
        .map(|a| (a.name.as_str(), a))
        .collect();

    for algo in &program.algorithms {
        let mut mut_vars: HashSet<String> = algo
            .parameters
            .iter()
            .filter(|p| p.is_mut)
            .map(|p| p.name.clone())
            .collect();
        check_stmts_mvs(&algo.body, &sigs, &mut mut_vars, errors);
    }
}

fn check_record_cycles(program: &Program, errors: &mut Vec<CompileError>) {
    let mut deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for rec in &program.records {
        let mut d = Vec::new();
        for f in &rec.fields {
            collect_record_refs(&f.ty, &mut d);
        }
        deps.insert(rec.name.as_str(), d);
    }

    for rec in &program.records {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        if has_cycle(rec.name.as_str(), &deps, &mut visiting, &mut visited) {
            errors.push(CompileError {
                code: "E_MVS_CYCLE",
                span: rec.span,
                message: format!(
                    "recursive Record '{}' violates value-semantic acyclic memory model",
                    rec.name
                ),
                hint: Some(
                    "use the Arena Pattern: store nodes in a List<Record> and reference them via Option<Int> indices".to_string(),
                ),
            });
        }
    }
}

fn collect_record_refs<'a>(ty: &'a Type, out: &mut Vec<&'a str>) {
    match ty {
        Type::Record { name, type_args } => {
            out.push(name.as_str());
            for t in type_args {
                collect_record_refs(t, out);
            }
        }
        Type::List { elem } | Type::Set { elem } | Type::Matrix { elem } => {
            collect_record_refs(elem, out);
        }
        Type::Map { key, val } => {
            collect_record_refs(key, out);
            collect_record_refs(val, out);
        }
        Type::Option { inner } | Type::Secret { inner } => collect_record_refs(inner, out),
        Type::Result { ok, err } => {
            collect_record_refs(ok, out);
            collect_record_refs(err, out);
        }
        Type::Tuple { elems } => {
            for e in elems {
                collect_record_refs(e, out);
            }
        }
        _ => {}
    }
}

fn has_cycle<'a>(
    cur: &'a str,
    deps: &HashMap<&'a str, Vec<&'a str>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
) -> bool {
    if visiting.contains(cur) {
        return true;
    }
    if visited.contains(cur) {
        return false;
    }
    visiting.insert(cur);
    if let Some(nexts) = deps.get(cur) {
        for &n in nexts {
            if has_cycle(n, deps, visiting, visited) {
                return true;
            }
        }
    }
    visiting.remove(cur);
    visited.insert(cur);
    false
}

fn check_stmts_mvs(
    stmts: &[Stmt],
    sigs: &HashMap<&str, &AlgorithmDecl>,
    mut_vars: &mut HashSet<String>,
    errors: &mut Vec<CompileError>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let {
                name, is_mut, init, ..
            } => {
                check_expr_mvs(init, sigs, errors);
                if *is_mut {
                    mut_vars.insert(name.clone());
                }
            }
            Stmt::Const { init, .. } => check_expr_mvs(init, sigs, errors),
            Stmt::Assign { target, value, .. } => {
                check_expr_mvs(value, sigs, errors);
                // Track implicit local variables as mutable in local scope
                if let LValue::Var { name, .. } = target {
                    mut_vars.insert(name.clone());
                }
            }
            Stmt::Expr { expr, .. } => check_expr_mvs(expr, sigs, errors),
            Stmt::Return { value: Some(v), .. } => check_expr_mvs(v, sigs, errors),
            Stmt::If {
                condition,
                then_body,
                else_if_branches,
                else_body,
                ..
            } => {
                check_expr_mvs(condition, sigs, errors);
                check_stmts_mvs(then_body, sigs, mut_vars, errors);
                for (c, b) in else_if_branches {
                    check_expr_mvs(c, sigs, errors);
                    check_stmts_mvs(b, sigs, mut_vars, errors);
                }
                if let Some(eb) = else_body {
                    check_stmts_mvs(eb, sigs, mut_vars, errors);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                check_expr_mvs(condition, sigs, errors);
                check_stmts_mvs(body, sigs, mut_vars, errors);
            }
            Stmt::For { iterable, body, .. } => {
                check_expr_mvs(iterable, sigs, errors);
                check_stmts_mvs(body, sigs, mut_vars, errors);
            }
            Stmt::Match {
                scrutinee, arms, ..
            } => {
                check_expr_mvs(scrutinee, sigs, errors);
                for arm in arms {
                    check_stmts_mvs(&arm.body, sigs, mut_vars, errors);
                }
            }
            Stmt::Assert { condition, .. } | Stmt::Invariant { condition, .. } => {
                check_expr_mvs(condition, sigs, errors);
            }
            _ => {}
        }
    }
}

fn check_expr_mvs(
    expr: &Expr,
    sigs: &HashMap<&str, &AlgorithmDecl>,
    errors: &mut Vec<CompileError>,
) {
    match expr {
        Expr::Call { callee, args, span } => {
            // Reject aliased `mut` root variables passed twice to the same call (`Swap(mut a, mut a)`)
            let mut mut_roots = HashSet::new();
            for (idx, arg) in args.iter().enumerate() {
                check_expr_mvs(&arg.value, sigs, errors);
                let expects_mut = sigs
                    .get(callee.as_str())
                    .and_then(|s| s.parameters.get(idx))
                    .is_some_and(|p| p.is_mut);
                if arg.is_mut || expects_mut {
                    if let Some(root) = expr_root_var(&arg.value) {
                        if !mut_roots.insert(root.to_string()) {
                            errors.push(CompileError {
                                code: "E_MVS_ALIAS",
                                span: *span,
                                message: format!(
                                    "cannot pass '{root}' mutably more than once in call to '{callee}'"
                                ),
                                hint: Some(
                                    "MVS forbids overlapping mutable borrows (aliasing)".to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }
        Expr::Binary { left, right, .. } => {
            check_expr_mvs(left, sigs, errors);
            check_expr_mvs(right, sigs, errors);
        }
        Expr::Unary { operand, .. } => check_expr_mvs(operand, sigs, errors),
        Expr::MethodCall { receiver, args, .. } => {
            check_expr_mvs(receiver, sigs, errors);
            for a in args {
                check_expr_mvs(&a.value, sigs, errors);
            }
        }
        Expr::Index { target, index, .. } => {
            check_expr_mvs(target, sigs, errors);
            check_expr_mvs(index, sigs, errors);
        }
        _ => {}
    }
}

fn expr_root_var(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident { name, .. } => Some(name.as_str()),
        Expr::Index { target, .. } | Expr::Field { target, .. } => expr_root_var(target),
        _ => None,
    }
}
