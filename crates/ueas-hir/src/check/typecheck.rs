//! Bidirectional type checker, `const` immutability, and complexity-binding validator.

use crate::check::CompileError;
use crate::complexity::extract_complexity_vars;
use crate::hir::*;
use crate::types::{PrimitiveType, Type};
use std::collections::{HashMap, HashSet};
use ueas_syntax::SourceSpan;

#[derive(Debug, Clone)]
struct VarBinding {
    ty: Type,
    is_const: bool,
}

/// Run static type checking, `const` enforcement, control-flow checks, and complexity binding validation.
pub fn check_types(program: &mut Program, errors: &mut Vec<CompileError>) {
    let records: HashMap<String, RecordDecl> = program
        .records
        .iter()
        .map(|r| (r.name.clone(), r.clone()))
        .collect();

    let algo_sigs: HashMap<String, (Vec<Type>, Type)> = program
        .algorithms
        .iter()
        .map(|a| {
            (
                a.name.clone(),
                (
                    a.parameters.iter().map(|p| p.ty.clone()).collect(),
                    a.return_type.clone(),
                ),
            )
        })
        .collect();

    for algo in &mut program.algorithms {
        let mut scopes: Vec<HashMap<String, VarBinding>> = vec![HashMap::new()];
        for p in &mut algo.parameters {
            if p.ty == Type::Infer {
                p.ty = Type::int();
            }
            scopes[0].insert(
                p.name.clone(),
                VarBinding {
                    ty: p.ty.clone(),
                    is_const: false,
                },
            );
        }

        // Validate complexity variable bindings
        check_complexity_bindings(algo, &scopes, &records, &algo_sigs, errors);

        // Check preconditions
        for pre in &algo.preconditions {
            let t = infer_expr(pre, &scopes, &records, &algo_sigs, errors);
            if t != Type::Infer && t.peel_secret() != &Type::bool() {
                errors.push(CompileError {
                    code: "E_TYPE_MISMATCH",
                    span: pre.span(),
                    message: "Require precondition must evaluate to Bool".to_string(),
                    hint: None,
                });
            }
        }

        // Check body statements
        check_stmts(
            &algo.body,
            &algo.return_type,
            0,
            &mut scopes,
            &records,
            &algo_sigs,
            errors,
        );

        // Check postconditions with optional result binding
        if !algo.postconditions.is_empty() {
            let mut post_scope = scopes.clone();
            let res_name = algo
                .postcondition_binding
                .clone()
                .unwrap_or_else(|| "result".to_string());
            post_scope[0].insert(
                res_name,
                VarBinding {
                    ty: algo.return_type.clone(),
                    is_const: true,
                },
            );
            for post in &algo.postconditions {
                let t = infer_expr(post, &post_scope, &records, &algo_sigs, errors);
                if t != Type::Infer && t.peel_secret() != &Type::bool() {
                    errors.push(CompileError {
                        code: "E_TYPE_MISMATCH",
                        span: post.span(),
                        message: "Ensure postcondition must evaluate to Bool".to_string(),
                        hint: None,
                    });
                }
            }
        }
    }
}

fn check_complexity_bindings(
    algo: &AlgorithmDecl,
    scopes: &[HashMap<String, VarBinding>],
    records: &HashMap<String, RecordDecl>,
    algo_sigs: &HashMap<String, (Vec<Type>, Type)>,
    errors: &mut Vec<CompileError>,
) {
    let mut needed = HashSet::new();
    if let Some(t) = &algo.complexity.time {
        needed.extend(extract_complexity_vars(&t.raw));
    }
    if let Some(s) = &algo.complexity.space {
        needed.extend(extract_complexity_vars(&s.raw));
    }
    if needed.is_empty() {
        return;
    }

    let bound_names: HashSet<&str> = algo
        .complexity
        .bindings
        .iter()
        .map(|(k, _)| k.as_str())
        .collect();

    for (_, expr) in &algo.complexity.bindings {
        let _ = infer_expr(expr, scopes, records, algo_sigs, errors);
    }

    // If `N` is the sole complexity variable and isn't explicitly bound, allow unambiguous
    // inference only when the algorithm has a single collection/integer parameter.
    let single_param_inferable =
        needed.len() == 1 && needed.contains("N") && algo.parameters.len() == 1;

    if !single_param_inferable {
        for var in needed {
            // Allow `N` when there's a collection/int parameter named `n` or `data` as fallback only if bindings exist?
            // Spec rule: if multiple vars (e.g. V, E) or explicit bindings exist, all must be bound!
            if !bound_names.contains(var.as_str())
                && !algo
                    .parameters
                    .iter()
                    .any(|p| p.name.eq_ignore_ascii_case(&var))
                && !(var == "N"
                    && algo.parameters.iter().any(|p| {
                        matches!(
                            p.ty.peel_secret(),
                            Type::List { .. }
                                | Type::Set { .. }
                                | Type::Map { .. }
                                | Type::Primitive {
                                    prim: PrimitiveType::String
                                }
                        )
                    }))
            {
                errors.push(CompileError {
                    code: "E_COMPLEXITY_BINDING",
                    span: algo.span,
                    message: format!(
                        "complexity variable '{var}' in '{}' is not bound (use `where {var} = ...`)",
                        algo.complexity
                            .time
                            .as_ref()
                            .map(|t| t.raw.as_str())
                            .unwrap_or("O(?)")
                    ),
                    hint: Some(format!("add `where {var} = ...` to Complexity block")),
                });
            }
        }
    }
}

fn check_stmts(
    stmts: &[Stmt],
    ret_ty: &Type,
    loop_depth: usize,
    scopes: &mut Vec<HashMap<String, VarBinding>>,
    records: &HashMap<String, RecordDecl>,
    algo_sigs: &HashMap<String, (Vec<Type>, Type)>,
    errors: &mut Vec<CompileError>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let {
                name,
                ty,
                init,
                span,
                ..
            } => {
                let inf = infer_expr(init, scopes, records, algo_sigs, errors);
                let final_ty = match ty {
                    Some(explicit) => {
                        if inf != Type::Infer && !types_compatible(&inf, explicit) {
                            errors.push(CompileError {
                                code: "E_TYPE_MISMATCH",
                                span: *span,
                                message: format!("let '{name}' type mismatch"),
                                hint: None,
                            });
                        }
                        explicit.clone()
                    }
                    None => inf,
                };
                if let Some(top) = scopes.last_mut() {
                    top.insert(
                        name.clone(),
                        VarBinding {
                            ty: final_ty,
                            is_const: false,
                        },
                    );
                }
            }
            Stmt::Const {
                name,
                ty,
                init,
                span,
            } => {
                let inf = infer_expr(init, scopes, records, algo_sigs, errors);
                let final_ty = match ty {
                    Some(explicit) => {
                        if inf != Type::Infer && !types_compatible(&inf, explicit) {
                            errors.push(CompileError {
                                code: "E_TYPE_MISMATCH",
                                span: *span,
                                message: format!("const '{name}' type mismatch"),
                                hint: None,
                            });
                        }
                        explicit.clone()
                    }
                    None => inf,
                };
                if let Some(top) = scopes.last_mut() {
                    top.insert(
                        name.clone(),
                        VarBinding {
                            ty: final_ty,
                            is_const: true,
                        },
                    );
                }
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                let val_ty = infer_expr(value, scopes, records, algo_sigs, errors);
                if let LValue::Var { name, .. } = target {
                    if let Some(binding) = lookup_var(name, scopes) {
                        if binding.is_const {
                            errors.push(CompileError {
                                code: "E_CONST_REASSIGN",
                                span: *span,
                                message: format!("cannot reassign immutable constant '{name}'"),
                                hint: Some(
                                    "declare with `let` if mutation is intended".to_string(),
                                ),
                            });
                        } else if val_ty != Type::Infer
                            && binding.ty != Type::Infer
                            && !types_compatible(&val_ty, &binding.ty)
                        {
                            errors.push(CompileError {
                                code: "E_TYPE_MISMATCH",
                                span: *span,
                                message: format!("cannot assign incompatible type to '{name}'"),
                                hint: None,
                            });
                        }
                    } else if let Some(top) = scopes.last_mut() {
                        // Implicit local variable declaration on first `<-` assignment
                        top.insert(
                            name.clone(),
                            VarBinding {
                                ty: val_ty,
                                is_const: false,
                            },
                        );
                    }
                } else {
                    check_lvalue_type(target, scopes, records, algo_sigs, errors);
                }
            }
            Stmt::Expr { expr, .. } => {
                let _ = infer_expr(expr, scopes, records, algo_sigs, errors);
            }
            Stmt::Return { value, span } => {
                if let Some(v) = value {
                    let vt = infer_expr(v, scopes, records, algo_sigs, errors);
                    if vt != Type::Infer
                        && *ret_ty != Type::Infer
                        && *ret_ty != Type::void()
                        && !types_compatible(&vt, ret_ty)
                    {
                        errors.push(CompileError {
                            code: "E_RETURN_TYPE",
                            span: *span,
                            message: "return value type does not match Algorithm return type"
                                .to_string(),
                            hint: None,
                        });
                    }
                }
            }
            Stmt::If {
                condition,
                then_body,
                else_if_branches,
                else_body,
                ..
            } => {
                let _ = infer_expr(condition, scopes, records, algo_sigs, errors);
                scopes.push(HashMap::new());
                check_stmts(
                    then_body, ret_ty, loop_depth, scopes, records, algo_sigs, errors,
                );
                scopes.pop();
                for (c, b) in else_if_branches {
                    let _ = infer_expr(c, scopes, records, algo_sigs, errors);
                    scopes.push(HashMap::new());
                    check_stmts(b, ret_ty, loop_depth, scopes, records, algo_sigs, errors);
                    scopes.pop();
                }
                if let Some(eb) = else_body {
                    scopes.push(HashMap::new());
                    check_stmts(eb, ret_ty, loop_depth, scopes, records, algo_sigs, errors);
                    scopes.pop();
                }
            }
            Stmt::While {
                condition,
                invariants,
                decreases,
                body,
                ..
            } => {
                let _ = infer_expr(condition, scopes, records, algo_sigs, errors);
                for inv in invariants {
                    let _ = infer_expr(inv, scopes, records, algo_sigs, errors);
                }
                if let Some(dec) = decreases {
                    let _ = infer_expr(dec, scopes, records, algo_sigs, errors);
                }
                scopes.push(HashMap::new());
                check_stmts(
                    body,
                    ret_ty,
                    loop_depth + 1,
                    scopes,
                    records,
                    algo_sigs,
                    errors,
                );
                scopes.pop();
            }
            Stmt::For {
                var,
                iterable,
                invariants,
                body,
                ..
            } => {
                let it_ty = infer_expr(iterable, scopes, records, algo_sigs, errors);
                let elem_ty = match it_ty.peel_secret() {
                    Type::List { elem } | Type::Set { elem } => (**elem).clone(),
                    Type::Map { key, .. } => (**key).clone(),
                    _ => Type::int(),
                };
                scopes.push(HashMap::new());
                if let Some(top) = scopes.last_mut() {
                    top.insert(
                        var.clone(),
                        VarBinding {
                            ty: elem_ty,
                            is_const: false,
                        },
                    );
                }
                for inv in invariants {
                    let _ = infer_expr(inv, scopes, records, algo_sigs, errors);
                }
                check_stmts(
                    body,
                    ret_ty,
                    loop_depth + 1,
                    scopes,
                    records,
                    algo_sigs,
                    errors,
                );
                scopes.pop();
            }
            Stmt::Match {
                scrutinee,
                arms,
                span,
            } => {
                let st = infer_expr(scrutinee, scopes, records, algo_sigs, errors);
                check_match_exhaustiveness(&st, arms, *span, errors);
                for arm in arms {
                    scopes.push(HashMap::new());
                    bind_pattern(&arm.pattern, &st, scopes);
                    check_stmts(
                        &arm.body, ret_ty, loop_depth, scopes, records, algo_sigs, errors,
                    );
                    scopes.pop();
                }
            }
            Stmt::Assert { condition, .. } | Stmt::Invariant { condition, .. } => {
                let _ = infer_expr(condition, scopes, records, algo_sigs, errors);
            }
            Stmt::Decreases { rank, .. } => {
                let _ = infer_expr(rank, scopes, records, algo_sigs, errors);
            }
            Stmt::Break { span } | Stmt::Continue { span } => {
                if loop_depth == 0 {
                    errors.push(CompileError {
                        code: "E_LOOP_CTRL",
                        span: *span,
                        message: "break/continue used outside of a loop".to_string(),
                        hint: None,
                    });
                }
            }
            Stmt::Sync { .. } => {}
        }
    }
}

fn check_match_exhaustiveness(
    scrutinee_ty: &Type,
    arms: &[MatchArm],
    span: SourceSpan,
    errors: &mut Vec<CompileError>,
) {
    let has_wildcard = arms
        .iter()
        .any(|a| matches!(a.pattern, Pattern::Wildcard | Pattern::Ident { .. }));
    if has_wildcard {
        return;
    }
    match scrutinee_ty.peel_secret() {
        Type::Option { .. } => {
            let has_some = arms
                .iter()
                .any(|a| matches!(a.pattern, Pattern::Some { .. }));
            let has_none = arms.iter().any(|a| matches!(a.pattern, Pattern::None));
            if !(has_some && has_none) {
                errors.push(CompileError {
                    code: "E_NON_EXHAUSTIVE_MATCH",
                    span,
                    message: "non-exhaustive match on Option<T> (must cover Some and None)"
                        .to_string(),
                    hint: None,
                });
            }
        }
        Type::Result { .. } => {
            let has_ok = arms.iter().any(|a| matches!(a.pattern, Pattern::Ok { .. }));
            let has_err = arms
                .iter()
                .any(|a| matches!(a.pattern, Pattern::Err { .. }));
            if !(has_ok && has_err) {
                errors.push(CompileError {
                    code: "E_NON_EXHAUSTIVE_MATCH",
                    span,
                    message: "non-exhaustive match on Result<T, E> (must cover Ok and Err)"
                        .to_string(),
                    hint: None,
                });
            }
        }
        _ => {}
    }
}

fn bind_pattern(pat: &Pattern, ty: &Type, scopes: &mut [HashMap<String, VarBinding>]) {
    match pat {
        Pattern::Ident { name } => {
            if let Some(top) = scopes.last_mut() {
                top.insert(
                    name.clone(),
                    VarBinding {
                        ty: ty.clone(),
                        is_const: false,
                    },
                );
            }
        }
        Pattern::Some { inner } => {
            let inner_ty = match ty.peel_secret() {
                Type::Option { inner } => (**inner).clone(),
                _ => Type::Infer,
            };
            bind_pattern(inner, &inner_ty, scopes);
        }
        Pattern::Ok { inner } => {
            let inner_ty = match ty.peel_secret() {
                Type::Result { ok, .. } => (**ok).clone(),
                _ => Type::Infer,
            };
            bind_pattern(inner, &inner_ty, scopes);
        }
        Pattern::Err { inner } => {
            let inner_ty = match ty.peel_secret() {
                Type::Result { err, .. } => (**err).clone(),
                _ => Type::Infer,
            };
            bind_pattern(inner, &inner_ty, scopes);
        }
        _ => {}
    }
}

fn check_lvalue_type(
    lv: &LValue,
    scopes: &[HashMap<String, VarBinding>],
    records: &HashMap<String, RecordDecl>,
    algo_sigs: &HashMap<String, (Vec<Type>, Type)>,
    errors: &mut Vec<CompileError>,
) {
    match lv {
        LValue::Var { .. } => {}
        LValue::Index { target, index, .. } => {
            check_lvalue_type(target, scopes, records, algo_sigs, errors);
            let _ = infer_expr(index, scopes, records, algo_sigs, errors);
        }
        LValue::Field { target, .. } => {
            check_lvalue_type(target, scopes, records, algo_sigs, errors);
        }
    }
}

fn infer_expr(
    expr: &Expr,
    scopes: &[HashMap<String, VarBinding>],
    records: &HashMap<String, RecordDecl>,
    algo_sigs: &HashMap<String, (Vec<Type>, Type)>,
    errors: &mut Vec<CompileError>,
) -> Type {
    match expr {
        Expr::IntLit { .. } => Type::int(),
        Expr::RealLit { .. } | Expr::InfinityLit { .. } | Expr::NanLit { .. } => Type::real(),
        Expr::BoolLit { .. } => Type::bool(),
        Expr::StringLit { .. } => Type::string(),
        Expr::NoneLit { .. } => Type::Option {
            inner: Box::new(Type::Infer),
        },
        Expr::Ident { name, .. } => lookup_var(name, scopes)
            .map(|b| b.ty)
            .unwrap_or(Type::Infer),
        Expr::ListLit { elements, .. } => {
            let first = elements
                .first()
                .map(|e| infer_expr(e, scopes, records, algo_sigs, errors))
                .unwrap_or(Type::Infer);
            for e in elements.iter().skip(1) {
                let _ = infer_expr(e, scopes, records, algo_sigs, errors);
            }
            Type::List {
                elem: Box::new(first),
            }
        }
        Expr::SetLit { elements, .. } => {
            let first = elements
                .first()
                .map(|e| infer_expr(e, scopes, records, algo_sigs, errors))
                .unwrap_or(Type::Infer);
            Type::Set {
                elem: Box::new(first),
            }
        }
        Expr::MapLit { entries, .. } => {
            let (k, v) = entries
                .first()
                .map(|(ke, ve)| {
                    (
                        infer_expr(ke, scopes, records, algo_sigs, errors),
                        infer_expr(ve, scopes, records, algo_sigs, errors),
                    )
                })
                .unwrap_or((Type::Infer, Type::Infer));
            Type::Map {
                key: Box::new(k),
                val: Box::new(v),
            }
        }
        Expr::TupleLit { elements, .. } => Type::Tuple {
            elems: elements
                .iter()
                .map(|e| infer_expr(e, scopes, records, algo_sigs, errors))
                .collect(),
        },
        Expr::RecordLit { name, fields, span } => {
            if let Some(rec) = records.get(name) {
                for f in &rec.fields {
                    if !fields.iter().any(|(fnm, _)| fnm == &f.name) {
                        errors.push(CompileError {
                            code: "E_MISSING_FIELD",
                            span: *span,
                            message: format!("Record '{name}' missing field '{}'", f.name),
                            hint: None,
                        });
                    }
                }
            }
            for (_, v) in fields {
                let _ = infer_expr(v, scopes, records, algo_sigs, errors);
            }
            Type::Record {
                name: name.clone(),
                type_args: vec![],
            }
        }
        Expr::Binary {
            op,
            left,
            right,
            span,
        } => {
            let lt = infer_expr(left, scopes, records, algo_sigs, errors);
            let rt = infer_expr(right, scopes, records, algo_sigs, errors);
            let sec = lt.is_secret() || rt.is_secret();
            let res = match op {
                BinaryOp::Eq
                | BinaryOp::Neq
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge
                | BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::Implies
                | BinaryOp::Iff
                | BinaryOp::In
                | BinaryOp::NotIn => Type::bool(),
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    if lt != Type::Infer
                        && rt != Type::Infer
                        && lt.is_numeric()
                        && rt.is_numeric()
                        && lt.peel_secret() != rt.peel_secret()
                        && !(lt.is_integer() && rt.is_integer())
                    {
                        errors.push(CompileError {
                            code: "E_TYPE_MISMATCH",
                            span: *span,
                            message: "implicit coercion between Int and Real is forbidden (use `as Real`)".to_string(),
                            hint: None,
                        });
                    }
                    if lt != Type::Infer {
                        lt.peel_secret().clone()
                    } else {
                        rt.peel_secret().clone()
                    }
                }
                BinaryOp::BitAnd
                | BinaryOp::BitOr
                | BinaryOp::BitXor
                | BinaryOp::Shl
                | BinaryOp::Shr => lt.peel_secret().clone(),
            };
            if sec {
                Type::Secret {
                    inner: Box::new(res),
                }
            } else {
                res
            }
        }
        Expr::Unary { op, operand, .. } => {
            let ot = infer_expr(operand, scopes, records, algo_sigs, errors);
            match op {
                UnaryOp::Not => Type::bool(),
                _ => ot,
            }
        }
        Expr::Cast {
            expr,
            target_type,
            span,
        } => {
            let src = infer_expr(expr, scopes, records, algo_sigs, errors);
            if src != Type::Infer && !src.can_cast_to(target_type) {
                errors.push(CompileError {
                    code: "E_INVALID_CAST",
                    span: *span,
                    message: "invalid explicit type cast".to_string(),
                    hint: None,
                });
            }
            target_type.clone()
        }
        Expr::Call { callee, args, .. } => {
            for a in args {
                let _ = infer_expr(&a.value, scopes, records, algo_sigs, errors);
            }
            match callee.as_str() {
                "Some" => Type::Option {
                    inner: Box::new(
                        args.first()
                            .map(|a| infer_expr(&a.value, scopes, records, algo_sigs, errors))
                            .unwrap_or(Type::Infer),
                    ),
                },
                "sqrt" | "log2" | "sin" | "cos" | "randReal" => Type::real(),
                "length" | "cardinality" | "strlen" | "floor" | "ceil" | "randInt" => Type::int(),
                "range" => Type::List {
                    elem: Box::new(Type::int()),
                },
                "select" => args
                    .get(1)
                    .map(|a| infer_expr(&a.value, scopes, records, algo_sigs, errors))
                    .unwrap_or(Type::Infer),
                _ => algo_sigs
                    .get(callee)
                    .map(|(_, r)| r.clone())
                    .unwrap_or(Type::Infer),
            }
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            let rt = infer_expr(receiver, scopes, records, algo_sigs, errors);
            for a in args {
                let _ = infer_expr(&a.value, scopes, records, algo_sigs, errors);
            }
            match method.as_str() {
                "length" | "cardinality" | "degree" => Type::int(),
                "contains" | "containsKey" | "hasEdge" => Type::bool(),
                "pop" => match rt.peel_secret() {
                    Type::List { elem } => (**elem).clone(),
                    _ => Type::Infer,
                },
                _ => Type::Infer,
            }
        }
        Expr::Index { target, index, .. } => {
            let tt = infer_expr(target, scopes, records, algo_sigs, errors);
            let _ = infer_expr(index, scopes, records, algo_sigs, errors);
            match tt.peel_secret() {
                Type::List { elem } | Type::Matrix { elem } => (**elem).clone(),
                Type::Map { val, .. } => (**val).clone(),
                Type::Primitive {
                    prim: PrimitiveType::String,
                } => Type::string(),
                _ => Type::Infer,
            }
        }
        Expr::Slice {
            target, start, end, ..
        } => {
            let tt = infer_expr(target, scopes, records, algo_sigs, errors);
            if let Some(s) = start {
                let _ = infer_expr(s, scopes, records, algo_sigs, errors);
            }
            if let Some(e) = end {
                let _ = infer_expr(e, scopes, records, algo_sigs, errors);
            }
            tt
        }
        Expr::Field { target, field, .. } => {
            let tt = infer_expr(target, scopes, records, algo_sigs, errors);
            match (tt.peel_secret(), field.as_str()) {
                (_, "length" | "cardinality" | "rows" | "cols") => Type::int(),
                (Type::Record { name, .. }, f) => records
                    .get(name)
                    .and_then(|r| r.fields.iter().find(|rf| rf.name == f))
                    .map(|rf| rf.ty.clone())
                    .unwrap_or(Type::Infer),
                _ => Type::Infer,
            }
        }
        Expr::Range { start, end, .. } => {
            let _ = infer_expr(start, scopes, records, algo_sigs, errors);
            let _ = infer_expr(end, scopes, records, algo_sigs, errors);
            Type::List {
                elem: Box::new(Type::int()),
            }
        }
        Expr::Quantifier {
            var,
            iterable,
            body,
            ..
        } => {
            let it = infer_expr(iterable, scopes, records, algo_sigs, errors);
            let elem_ty = match it.peel_secret() {
                Type::List { elem } | Type::Set { elem } => (**elem).clone(),
                _ => Type::int(),
            };
            let mut inner_scopes = scopes.to_vec();
            let mut q_scope = HashMap::new();
            q_scope.insert(
                var.clone(),
                VarBinding {
                    ty: elem_ty,
                    is_const: true,
                },
            );
            inner_scopes.push(q_scope);
            let _ = infer_expr(body, &inner_scopes, records, algo_sigs, errors);
            Type::bool()
        }
        Expr::Spawn { expr, .. } => infer_expr(expr, scopes, records, algo_sigs, errors),
    }
}

fn lookup_var(name: &str, scopes: &[HashMap<String, VarBinding>]) -> Option<VarBinding> {
    for sc in scopes.iter().rev() {
        if let Some(b) = sc.get(name) {
            return Some(b.clone());
        }
    }
    None
}

fn types_compatible(a: &Type, b: &Type) -> bool {
    let ap = a.peel_secret();
    let bp = b.peel_secret();
    if *ap == Type::Infer || *bp == Type::Infer || ap == bp {
        return true;
    }
    // Generic TypeVar matches concrete types at static level before instantiation
    if matches!(ap, Type::TypeVar { .. }) || matches!(bp, Type::TypeVar { .. }) {
        return true;
    }
    match (ap, bp) {
        (Type::List { elem: e1 }, Type::List { elem: e2 })
        | (Type::Set { elem: e1 }, Type::Set { elem: e2 })
        | (Type::Option { inner: e1 }, Type::Option { inner: e2 })
        | (Type::Matrix { elem: e1 }, Type::Matrix { elem: e2 }) => types_compatible(e1, e2),
        (Type::Map { key: k1, val: v1 }, Type::Map { key: k2, val: v2 }) => {
            types_compatible(k1, k2) && types_compatible(v1, v2)
        }
        (Type::Result { ok: o1, err: e1 }, Type::Result { ok: o2, err: e2 }) => {
            types_compatible(o1, o2) && types_compatible(e1, e2)
        }
        _ => false,
    }
}
