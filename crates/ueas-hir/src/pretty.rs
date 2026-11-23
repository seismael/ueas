//! Pretty-printer turning any `ueas-hir::Program` into clean canonical `.ueas` v2.0 source.

use crate::hir::*;
use crate::types::{PrimitiveType, Type};

/// Format a `Program` HIR back into canonical UEAS v2.0 pseudocode.
pub fn pretty_program(prog: &Program) -> String {
    let mut out = String::new();
    if let Some(m) = &prog.module {
        out.push_str(&format!("module {m}\n\n"));
    }
    for imp in &prog.imports {
        out.push_str(&format!("import {}\n", imp.path));
    }
    if !prog.imports.is_empty() {
        out.push('\n');
    }
    for rec in &prog.records {
        out.push_str(&format!("Record {}\n", rec.name));
        for f in &rec.fields {
            out.push_str(&format!("    {}: {}\n", f.name, fmt_type(&f.ty)));
        }
        out.push_str("end Record\n\n");
    }
    for algo in &prog.algorithms {
        pretty_algorithm(algo, &mut out);
        out.push('\n');
    }
    for t in &prog.tests {
        out.push_str(&format!("@Test \"{}\"\n", t.name));
        for s in &t.body {
            pretty_stmt(s, 1, &mut out);
        }
        out.push_str("end Test\n\n");
    }
    out
}

fn pretty_algorithm(algo: &AlgorithmDecl, out: &mut String) {
    if algo.constant_time {
        out.push_str("@ConstantTime\n");
    }
    let params: Vec<String> = algo
        .parameters
        .iter()
        .map(|p| {
            let m = if p.is_mut { "mut " } else { "" };
            format!("{m}{}: {}", p.name, fmt_type(&p.ty))
        })
        .collect();
    out.push_str(&format!(
        "Algorithm {}({}) -> {}\n",
        algo.name,
        params.join(", "),
        fmt_type(&algo.return_type)
    ));
    if !algo.preconditions.is_empty() {
        out.push_str("    Require:\n");
        for pre in &algo.preconditions {
            out.push_str(&format!("        {}\n", fmt_expr(pre)));
        }
    }
    if !algo.postconditions.is_empty() {
        let b = algo.postcondition_binding.as_deref().unwrap_or("res");
        out.push_str(&format!("    Ensure({b}):\n"));
        for post in &algo.postconditions {
            out.push_str(&format!("        {}\n", fmt_expr(post)));
        }
    }
    out.push_str("    Complexity:\n");
    if let Some(t) = &algo.complexity.time {
        out.push_str(&format!("        Time: {}", t.raw));
        if !algo.complexity.bindings.is_empty() {
            let b: Vec<String> = algo
                .complexity
                .bindings
                .iter()
                .map(|(k, v)| format!("{k} = {}", fmt_expr(v)))
                .collect();
            out.push_str(&format!(" where {}", b.join(", ")));
        }
        out.push('\n');
    }
    if let Some(s) = &algo.complexity.space {
        out.push_str(&format!("        Space: {}\n", s.raw));
    }
    out.push('\n');
    for s in &algo.body {
        pretty_stmt(s, 1, out);
    }
    out.push_str("end Algorithm\n");
}

fn pretty_stmt(stmt: &Stmt, indent: usize, out: &mut String) {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::Let {
            name, is_mut, init, ..
        } => {
            let m = if *is_mut { "mut " } else { "" };
            out.push_str(&format!("{pad}let {m}{name} <- {}\n", fmt_expr(init)));
        }
        Stmt::Const { name, init, .. } => {
            out.push_str(&format!("{pad}const {name} <- {}\n", fmt_expr(init)));
        }
        Stmt::Assign { target, value, .. } => {
            out.push_str(&format!(
                "{pad}{} <- {}\n",
                fmt_lvalue(target),
                fmt_expr(value)
            ));
        }
        Stmt::Expr { expr, .. } => {
            out.push_str(&format!("{pad}{}\n", fmt_expr(expr)));
        }
        Stmt::Return { value, .. } => match value {
            Some(v) => out.push_str(&format!("{pad}return {}\n", fmt_expr(v))),
            None => out.push_str(&format!("{pad}return\n")),
        },
        Stmt::If {
            condition,
            then_body,
            else_if_branches,
            else_body,
            ..
        } => {
            out.push_str(&format!("{pad}if {} then\n", fmt_expr(condition)));
            for s in then_body {
                pretty_stmt(s, indent + 1, out);
            }
            for (c, b) in else_if_branches {
                out.push_str(&format!("{pad}else if {} then\n", fmt_expr(c)));
                for s in b {
                    pretty_stmt(s, indent + 1, out);
                }
            }
            if let Some(eb) = else_body {
                out.push_str(&format!("{pad}else\n"));
                for s in eb {
                    pretty_stmt(s, indent + 1, out);
                }
            }
            out.push_str(&format!("{pad}end if\n"));
        }
        Stmt::While {
            condition,
            invariants,
            decreases,
            body,
            ..
        } => {
            out.push_str(&format!("{pad}while {} do\n", fmt_expr(condition)));
            for inv in invariants {
                out.push_str(&format!("{pad}    invariant {}\n", fmt_expr(inv)));
            }
            if let Some(dec) = decreases {
                out.push_str(&format!("{pad}    decreases {}\n", fmt_expr(dec)));
            }
            for s in body {
                pretty_stmt(s, indent + 1, out);
            }
            out.push_str(&format!("{pad}end while\n"));
        }
        Stmt::For {
            var,
            iterable,
            is_parallel,
            invariants,
            body,
            ..
        } => {
            let par = if *is_parallel { "parallel " } else { "" };
            out.push_str(&format!(
                "{pad}{par}for each {var} in {} do\n",
                fmt_expr(iterable)
            ));
            for inv in invariants {
                out.push_str(&format!("{pad}    invariant {}\n", fmt_expr(inv)));
            }
            for s in body {
                pretty_stmt(s, indent + 1, out);
            }
            out.push_str(&format!("{pad}end for\n"));
        }
        Stmt::Match {
            scrutinee, arms, ..
        } => {
            out.push_str(&format!("{pad}match {}\n", fmt_expr(scrutinee)));
            for arm in arms {
                out.push_str(&format!("{pad}    case {} =>\n", fmt_pattern(&arm.pattern)));
                for s in &arm.body {
                    pretty_stmt(s, indent + 2, out);
                }
            }
            out.push_str(&format!("{pad}end match\n"));
        }
        Stmt::Assert { condition, .. } => {
            out.push_str(&format!("{pad}assert {}\n", fmt_expr(condition)));
        }
        Stmt::Invariant { condition, .. } => {
            out.push_str(&format!("{pad}invariant {}\n", fmt_expr(condition)));
        }
        Stmt::Decreases { rank, .. } => {
            out.push_str(&format!("{pad}decreases {}\n", fmt_expr(rank)));
        }
        Stmt::Break { .. } => out.push_str(&format!("{pad}break\n")),
        Stmt::Continue { .. } => out.push_str(&format!("{pad}continue\n")),
        Stmt::Sync { .. } => out.push_str(&format!("{pad}sync\n")),
    }
}

fn fmt_lvalue(lv: &LValue) -> String {
    match lv {
        LValue::Var { name, .. } => name.clone(),
        LValue::Index { target, index, .. } => {
            format!("{}[{}]", fmt_lvalue(target), fmt_expr(index))
        }
        LValue::Field { target, field, .. } => format!("{}.{field}", fmt_lvalue(target)),
    }
}

fn fmt_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Ident { name } => name.clone(),
        Pattern::Some { inner } => format!("Some({})", fmt_pattern(inner)),
        Pattern::None => "None".to_string(),
        Pattern::Ok { inner } => format!("Ok({})", fmt_pattern(inner)),
        Pattern::Err { inner } => format!("Err({})", fmt_pattern(inner)),
        Pattern::IntLit { value } => value.clone(),
        Pattern::BoolLit { value } => value.to_string(),
        Pattern::StringLit { value } => format!("\"{value}\""),
    }
}

/// Format an expression into canonical UEAS source.
pub fn fmt_expr(expr: &Expr) -> String {
    match expr {
        Expr::IntLit { value, .. } => value.clone(),
        Expr::RealLit { value, .. } => format!("{value}"),
        Expr::BoolLit { value, .. } => value.to_string(),
        Expr::StringLit { value, .. } => format!("\"{value}\""),
        Expr::NoneLit { .. } => "None".to_string(),
        Expr::InfinityLit { .. } => "Infinity".to_string(),
        Expr::NanLit { .. } => "NaN".to_string(),
        Expr::Ident { name, .. } => name.clone(),
        Expr::ListLit { elements, .. } => {
            let elems: Vec<String> = elements.iter().map(fmt_expr).collect();
            format!("[{}]", elems.join(", "))
        }
        Expr::SetLit { elements, .. } => {
            let elems: Vec<String> = elements.iter().map(fmt_expr).collect();
            format!("{{{}}}", elems.join(", "))
        }
        Expr::MapLit { entries, .. } => {
            let es: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("{}: {}", fmt_expr(k), fmt_expr(v)))
                .collect();
            format!("{{{}}}", es.join(", "))
        }
        Expr::TupleLit { elements, .. } => {
            let elems: Vec<String> = elements.iter().map(fmt_expr).collect();
            format!("({})", elems.join(", "))
        }
        Expr::RecordLit { name, fields, .. } => {
            let fs: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", fmt_expr(v)))
                .collect();
            format!("{name} {{ {} }}", fs.join(", "))
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let s = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Mod => "mod",
                BinaryOp::Eq => "==",
                BinaryOp::Neq => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "and",
                BinaryOp::Or => "or",
                BinaryOp::Implies => "==>",
                BinaryOp::Iff => "<==>",
                BinaryOp::In => "in",
                BinaryOp::NotIn => "not in",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
            };
            format!("({} {s} {})", fmt_expr(left), fmt_expr(right))
        }
        Expr::Unary { op, operand, .. } => match op {
            UnaryOp::Neg => format!("-{}", fmt_expr(operand)),
            UnaryOp::Not => format!("not {}", fmt_expr(operand)),
            UnaryOp::BitNot => format!("~{}", fmt_expr(operand)),
        },
        Expr::Cast {
            expr, target_type, ..
        } => format!("({} as {})", fmt_expr(expr), fmt_type(target_type)),
        Expr::Call { callee, args, .. } => {
            let a: Vec<String> = args
                .iter()
                .map(|x| {
                    let m = if x.is_mut { "mut " } else { "" };
                    format!("{m}{}", fmt_expr(&x.value))
                })
                .collect();
            format!("{callee}({})", a.join(", "))
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            let a: Vec<String> = args.iter().map(|x| fmt_expr(&x.value)).collect();
            format!("{}.{method}({})", fmt_expr(receiver), a.join(", "))
        }
        Expr::Index { target, index, .. } => format!("{}[{}]", fmt_expr(target), fmt_expr(index)),
        Expr::Slice {
            target, start, end, ..
        } => {
            let s = start.as_ref().map(|e| fmt_expr(e)).unwrap_or_default();
            let e = end.as_ref().map(|e| fmt_expr(e)).unwrap_or_default();
            format!("{}[{s}:{e}]", fmt_expr(target))
        }
        Expr::Field { target, field, .. } => format!("{}.{field}", fmt_expr(target)),
        Expr::Range {
            start,
            end,
            inclusive,
            ..
        } => {
            let sep = if *inclusive { "..=" } else { ".." };
            format!("{}{sep}{}", fmt_expr(start), fmt_expr(end))
        }
        Expr::Quantifier {
            is_forall,
            var,
            iterable,
            body,
            ..
        } => {
            let q = if *is_forall { "forall" } else { "exists" };
            format!("{q} {var} in {} :: {}", fmt_expr(iterable), fmt_expr(body))
        }
        Expr::Spawn { expr, .. } => format!("spawn {}", fmt_expr(expr)),
    }
}

/// Format a UEAS type into canonical source string.
pub fn fmt_type(ty: &Type) -> String {
    match ty {
        Type::Primitive { prim } => match prim {
            PrimitiveType::Int => "Int".to_string(),
            PrimitiveType::Int64 => "Int64".to_string(),
            PrimitiveType::UInt64 => "UInt64".to_string(),
            PrimitiveType::UInt32 => "UInt32".to_string(),
            PrimitiveType::Byte => "Byte".to_string(),
            PrimitiveType::Real => "Real".to_string(),
            PrimitiveType::Bool => "Bool".to_string(),
            PrimitiveType::String => "String".to_string(),
            PrimitiveType::Void => "Void".to_string(),
        },
        Type::List { elem } => format!("List<{}>", fmt_type(elem)),
        Type::Set { elem } => format!("Set<{}>", fmt_type(elem)),
        Type::Map { key, val } => format!("Map<{}, {}>", fmt_type(key), fmt_type(val)),
        Type::Graph {
            node,
            weight,
            directed,
        } => {
            let name = if *directed { "DirectedGraph" } else { "Graph" };
            format!("{name}<{}, {}>", fmt_type(node), fmt_type(weight))
        }
        Type::Matrix { elem } => format!("Matrix<{}>", fmt_type(elem)),
        Type::Option { inner } => format!("Option<{}>", fmt_type(inner)),
        Type::Result { ok, err } => format!("Result<{}, {}>", fmt_type(ok), fmt_type(err)),
        Type::Tuple { elems } => {
            let es: Vec<String> = elems.iter().map(fmt_type).collect();
            format!("Tuple<{}>", es.join(", "))
        }
        Type::Secret { inner } => format!("Secret<{}>", fmt_type(inner)),
        Type::Record { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let ts: Vec<String> = type_args.iter().map(fmt_type).collect();
                format!("{name}<{}>", ts.join(", "))
            }
        }
        Type::TypeVar { name } => name.clone(),
        Type::Infer => "Int".to_string(),
    }
}
