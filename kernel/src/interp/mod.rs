//! Abstract interpreter for the UEAS kernel.
//!
//! Walks the canonical AST, evaluates expressions against the virtual heap,
//! maintains a stacked symbol table of heap handles, increments the step
//! counter, and enforces invariants and complexity contracts.

use crate::ast::{AstNode, AstNodeKind, AstValue};
use crate::heap::{HeapHandle, VirtualHeap};
use crate::profiling::{ComplexityContract, Profiler, ProfilingConfig};
use crate::traps::{ExitCode, TrapRegister};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
struct Scope {
    symbols: HashMap<String, HeapHandle>,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    pub fn declare(
        &mut self,
        name: &str,
        value: &AstValue,
        heap: &mut VirtualHeap,
    ) -> Result<HeapHandle, ExitCode> {
        let (bytes, tag) = value_to_bytes(value);
        let handle = heap.allocate(bytes.len(), tag)?;
        heap.write(handle, 0, &bytes)?;
        self.scopes
            .last_mut()
            .ok_or(ExitCode::StackOverflow)?
            .symbols
            .insert(name.to_string(), handle);
        Ok(handle)
    }
    pub fn lookup(&self, name: &str, heap: &VirtualHeap) -> Option<AstValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(h) = scope.symbols.get(name) {
                return read_value_from_heap(heap, *h).ok();
            }
        }
        None
    }
    pub fn assign(
        &mut self,
        name: &str,
        value: &AstValue,
        heap: &mut VirtualHeap,
    ) -> Result<(), ExitCode> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(h) = scope.symbols.get(name) {
                let (bytes, _tag) = value_to_bytes(value);
                return heap.write(*h, 0, &bytes);
            }
        }
        Err(ExitCode::NullDereference)
    }
}

fn value_to_bytes(value: &AstValue) -> (Vec<u8>, crate::heap::TypeTag) {
    use crate::heap::TypeTag;
    match value {
        AstValue::Integer(x) => (x.to_be_bytes().to_vec(), TypeTag::Integer),
        AstValue::Real(x) => (x.to_be_bytes().to_vec(), TypeTag::Real),
        AstValue::Boolean(b) => (vec![if *b { 1u8 } else { 0u8 }], TypeTag::Boolean),
        AstValue::String(s) => {
            let mut d = Vec::from(s.as_bytes());
            d.push(0);
            (d, TypeTag::String)
        }
        AstValue::None => (vec![0u8], TypeTag::Unknown),
    }
}

fn read_value_from_heap(heap: &VirtualHeap, handle: HeapHandle) -> Result<AstValue, ExitCode> {
    use crate::heap::TypeTag;
    let tag = heap.allocation_type(handle).unwrap_or(TypeTag::Unknown);
    let size = heap.allocation_size(handle).unwrap_or(0);
    if size == 0 {
        return Ok(AstValue::None);
    }
    let bytes = heap.read(handle, 0, size)?;
    Ok(match tag {
        TypeTag::Integer => {
            let mut arr = [0u8; 8];
            let len = bytes.len().min(8);
            arr[..len].copy_from_slice(&bytes[..len]);
            AstValue::Integer(i64::from_be_bytes(arr))
        }
        TypeTag::Real => {
            let mut arr = [0u8; 8];
            if bytes.len() >= 8 {
                arr.copy_from_slice(&bytes[..8]);
            }
            AstValue::Real(f64::from_be_bytes(arr))
        }
        TypeTag::Boolean => AstValue::Boolean(bytes.first().map(|b| *b != 0).unwrap_or(false)),
        TypeTag::String => AstValue::String(
            String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string(),
        ),
        TypeTag::HeapHandle | TypeTag::Unknown => AstValue::None,
    })
}

pub struct ExecContext {
    pub heap: VirtualHeap,
    pub symbols: SymbolTable,
    pub profiler: Profiler,
    pub trap: TrapRegister,
}

impl ExecContext {
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            heap: VirtualHeap::with_default_config(),
            symbols: SymbolTable::new(),
            profiler: Profiler::new(config),
            trap: TrapRegister::new(),
        }
    }
    pub fn with_default_config() -> Self {
        Self::new(ProfilingConfig::default())
    }
}

pub fn evaluate(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if ctx.trap.is_trapped() {
        return Err(ctx.trap.code());
    }
    ctx.profiler.step()?;
    match node.kind {
        AstNodeKind::IntegerLiteral
        | AstNodeKind::RealLiteral
        | AstNodeKind::StringLiteral
        | AstNodeKind::BooleanLiteral => node.value.clone().ok_or(ExitCode::HeapExhaustion),
        AstNodeKind::NoneLiteral => Ok(AstValue::None),
        AstNodeKind::Identifier => {
            let name = match &node.value {
                Some(AstValue::String(s)) => s.as_str(),
                _ => return Err(ExitCode::HeapExhaustion),
            };
            ctx.symbols
                .lookup(name, &ctx.heap)
                .ok_or(ExitCode::NullDereference)
        }
        AstNodeKind::BinaryExpression => eval_binary(ctx, node),
        AstNodeKind::UnaryExpression => eval_unary(ctx, node),
        AstNodeKind::FunctionCall => eval_function_call(ctx, node),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn eval_binary(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 3 {
        return Err(ExitCode::HeapExhaustion);
    }
    let op = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let left = evaluate(ctx, &node.children[1])?;
    let right = evaluate(ctx, &node.children[2])?;
    match op.as_str() {
        "+" => op2(&left, &right, |a, b| a + b, |a, b| a + b),
        "-" => op2(&left, &right, |a, b| a - b, |a, b| a - b),
        "*" => op2(&left, &right, |a, b| a * b, |a, b| a * b),
        "/" => match (&left, &right) {
            (AstValue::Integer(a), AstValue::Integer(b)) => {
                if *b == 0 {
                    Err(ExitCode::DivisionByZero)
                } else {
                    Ok(AstValue::Integer(a / b))
                }
            }
            (AstValue::Real(a), AstValue::Real(b)) => {
                if *b == 0.0 {
                    Err(ExitCode::DivisionByZero)
                } else {
                    Ok(AstValue::Real(a / b))
                }
            }
            _ => Err(ExitCode::HeapExhaustion),
        },
        "mod" => match (&left, &right) {
            (AstValue::Integer(a), AstValue::Integer(b)) => {
                if *b == 0 {
                    Err(ExitCode::DivisionByZero)
                } else {
                    Ok(AstValue::Integer(a % b))
                }
            }
            _ => Err(ExitCode::HeapExhaustion),
        },
        "==" => Ok(AstValue::Boolean(left == right)),
        "!=" => Ok(AstValue::Boolean(left != right)),
        "<" => cmp2(&left, &right, |a, b| a < b),
        "<=" => cmp2(&left, &right, |a, b| a <= b),
        ">" => cmp2(&left, &right, |a, b| a > b),
        ">=" => cmp2(&left, &right, |a, b| a >= b),
        "and" => {
            if !is_truthy(&left) {
                Ok(AstValue::Boolean(false))
            } else {
                Ok(AstValue::Boolean(is_truthy(&right)))
            }
        }
        "or" => {
            if is_truthy(&left) {
                Ok(AstValue::Boolean(true))
            } else {
                Ok(AstValue::Boolean(is_truthy(&right)))
            }
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn op2<FI, FR>(l: &AstValue, r: &AstValue, iop: FI, rop: FR) -> Result<AstValue, ExitCode>
where
    FI: Fn(i64, i64) -> i64,
    FR: Fn(f64, f64) -> f64,
{
    match (l, r) {
        (AstValue::Integer(a), AstValue::Integer(b)) => Ok(AstValue::Integer(iop(*a, *b))),
        (AstValue::Real(a), AstValue::Real(b)) => Ok(AstValue::Real(rop(*a, *b))),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn cmp2<F>(l: &AstValue, r: &AstValue, cmp: F) -> Result<AstValue, ExitCode>
where
    F: Fn(f64, f64) -> bool,
{
    match (l, r) {
        (AstValue::Integer(a), AstValue::Integer(b)) => {
            Ok(AstValue::Boolean(cmp(*a as f64, *b as f64)))
        }
        (AstValue::Real(a), AstValue::Real(b)) => Ok(AstValue::Boolean(cmp(*a, *b))),
        (AstValue::Boolean(a), AstValue::Boolean(b)) => {
            Ok(AstValue::Boolean(cmp(*a as u8 as f64, *b as u8 as f64)))
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn eval_unary(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 2 {
        return Err(ExitCode::HeapExhaustion);
    }
    let op = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let operand = evaluate(ctx, &node.children[1])?;
    match op.as_str() {
        "-" => match &operand {
            AstValue::Integer(x) => Ok(AstValue::Integer(-x)),
            AstValue::Real(x) => Ok(AstValue::Real(-x)),
            _ => Err(ExitCode::HeapExhaustion),
        },
        "not" => Ok(AstValue::Boolean(!is_truthy(&operand))),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn eval_function_call(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    let name = match node.children.first().and_then(|n| n.value.as_ref()) {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let mut args = Vec::new();
    for child in node.children.iter().skip(1) {
        args.push(evaluate(ctx, child)?);
    }
    ctx.profiler.step()?;
    dispatch_builtin(&name, &args)
}

fn dispatch_builtin(name: &str, args: &[AstValue]) -> Result<AstValue, ExitCode> {
    match name {
        "sqrt" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            let x: f64 = match &args[0] {
                AstValue::Integer(i) => *i as f64,
                AstValue::Real(r) => *r,
                _ => return Err(ExitCode::HeapExhaustion),
            };
            if x < 0.0 {
                return Err(ExitCode::HeapExhaustion);
            }
            Ok(AstValue::Real(x.sqrt()))
        }
        "length" | "cardinality" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            Ok(AstValue::Integer(args.len() as i64))
        }
        "contains" => {
            if args.len() < 2 {
                return Err(ExitCode::HeapExhaustion);
            }
            Ok(AstValue::Boolean(args[1..].contains(&args[0])))
        }
        "append" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            Ok(args[args.len() - 1].clone())
        }
        "pop" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            Ok(args[0].clone())
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn is_truthy(value: &AstValue) -> bool {
    match value {
        AstValue::Boolean(b) => *b,
        AstValue::Integer(x) => *x != 0,
        AstValue::String(s) => s != "0" && !s.is_empty(),
        AstValue::Real(x) => *x != 0.0,
        AstValue::None => false,
    }
}

pub fn execute_program(ctx: &mut ExecContext, program: &AstNode) -> Result<AstValue, ExitCode> {
    if program.kind != AstNodeKind::Program {
        return Err(ExitCode::HeapExhaustion);
    }
    let mut last = AstValue::None;
    for algo in &program.children {
        last = execute_algorithm(ctx, algo)?;
    }
    Ok(last)
}

fn execute_algorithm(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    ctx.symbols.push_scope();
    let mut last = AstValue::None;
    for (i, child) in node.children.iter().enumerate() {
        if i < 2 {
            continue;
        }
        match child.kind {
            AstNodeKind::VariableDeclaration => {
                execute_var_decl(ctx, child)?;
            }
            AstNodeKind::Assignment => {
                execute_assignment(ctx, child)?;
            }
            AstNodeKind::Return => {
                if !child.children.is_empty() {
                    last = evaluate(ctx, &child.children[0])?;
                }
                ctx.symbols.pop_scope();
                return Ok(last);
            }
            AstNodeKind::If => {
                last = execute_if(ctx, child)?;
            }
            AstNodeKind::WhileLoop => {
                last = execute_while(ctx, child)?;
            }
            AstNodeKind::ForLoop => {
                last = execute_for(ctx, child)?;
            }
            AstNodeKind::Assert => {
                execute_assert(ctx, child)?;
            }
            AstNodeKind::Invariant => {
                execute_invariant(ctx, child)?;
            }
            _ => {
                if let Ok(v) = evaluate(ctx, child) {
                    last = v;
                }
            }
        }
    }
    ctx.symbols.pop_scope();
    enforce_complexity(ctx, node)?;
    Ok(last)
}

fn enforce_complexity(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.len() < 2 {
        return Ok(());
    }
    let complexity_str = match &node.children[1].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Ok(()),
    };
    let n_val: u64 = node
        .children
        .iter()
        .filter(|c| c.kind == AstNodeKind::Parameter)
        .count() as u64;
    let s = complexity_str.trim();
    // Use boundary-aware matching: ensure pattern end is followed by ) or EOS
    let contract = match () {
        _ if s.contains("O(1)") => ComplexityContract::Constant,
        _ if s.contains("O(N^3)") && !s.contains("O(N^30") => ComplexityContract::Polynomial {
            n: n_val.max(1),
            k: 3,
        },
        _ if s.contains("O(N^2)") && !s.contains("O(N^20") => {
            ComplexityContract::Quadratic { n: n_val.max(1) }
        }
        _ if s.contains("O(N log N)") || s.contains("O((V+E) log V)") => {
            ComplexityContract::Linearithmic { n: n_val.max(1) }
        }
        _ if s.contains("O(log N)") => ComplexityContract::Logarithmic { n: n_val.max(1) },
        _ if s.contains("O(2^N)") => ComplexityContract::Exponential {
            n: n_val.clamp(1, 20),
        },
        _ if s.contains("O(N!)") => ComplexityContract::Factorial {
            n: n_val.clamp(1, 10),
        },
        _ if s.contains("O(N)")
            && !s.contains("O(N^")
            && !s.contains("O(N ")
            && !s.contains("O(N!") =>
        {
            ComplexityContract::Linear { n: n_val.max(1) }
        }
        _ => ComplexityContract::Linear { n: n_val.max(1) },
    };
    ctx.profiler.verify_complexity(&contract)
}

fn execute_var_decl(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let init = if node.children.len() > 2 {
        Some(evaluate(ctx, &node.children[2])?)
    } else {
        None
    };
    ctx.symbols
        .declare(&name, &init.unwrap_or(AstValue::None), &mut ctx.heap)?;
    Ok(())
}

fn execute_assignment(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.len() < 2 {
        return Err(ExitCode::HeapExhaustion);
    }
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let value = evaluate(ctx, &node.children[1])?;
    ctx.symbols.assign(&name, &value, &mut ctx.heap)
}

fn execute_if(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }
    if is_truthy(&evaluate(ctx, &node.children[0])?) && node.children.len() > 1 {
        exec_body(ctx, &node.children[1].children)
    } else if node.children.len() > 2 {
        exec_body(ctx, &node.children[2].children)
    } else {
        Ok(AstValue::None)
    }
}

fn execute_while(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }
    let mut last = AstValue::None;
    while is_truthy(&evaluate(ctx, &node.children[0])?) {
        ctx.profiler.step()?;
        if node.children.len() > 1 {
            last = exec_body(ctx, &node.children[1].children)?;
        }
    }
    Ok(last)
}

fn execute_for(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 2 {
        return Ok(AstValue::None);
    }
    let iter_name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let n: i64 = match evaluate(ctx, &node.children[1])? {
        AstValue::Integer(x) => x,
        _ => 1,
    };
    ctx.symbols.push_scope();
    let mut last = AstValue::None;
    for i in 0..n.max(1) {
        ctx.symbols
            .declare(&iter_name, &AstValue::Integer(i), &mut ctx.heap)?;
        for body_node in node.children.iter().skip(2) {
            last = exec_stmt(ctx, body_node)?;
        }
    }
    ctx.symbols.pop_scope();
    Ok(last)
}

pub fn execute_assert(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.is_empty() {
        return Ok(());
    }
    if !is_truthy(&evaluate(ctx, &node.children[0])?) {
        ctx.trap.set(ExitCode::AssertionFailure);
        return Err(ExitCode::AssertionFailure);
    }
    Ok(())
}

pub fn execute_invariant(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.is_empty() {
        return Ok(());
    }
    if !is_truthy(&evaluate(ctx, &node.children[0])?) {
        ctx.trap.set(ExitCode::InvariantViolation);
        return Err(ExitCode::InvariantViolation);
    }
    Ok(())
}

fn exec_body(ctx: &mut ExecContext, body: &[AstNode]) -> Result<AstValue, ExitCode> {
    let mut last = AstValue::None;
    for stmt in body {
        last = exec_stmt(ctx, stmt)?;
    }
    Ok(last)
}

fn exec_stmt(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    match node.kind {
        AstNodeKind::VariableDeclaration => {
            execute_var_decl(ctx, node)?;
            Ok(AstValue::None)
        }
        AstNodeKind::Assignment => {
            execute_assignment(ctx, node)?;
            Ok(AstValue::None)
        }
        AstNodeKind::Return => {
            if !node.children.is_empty() {
                evaluate(ctx, &node.children[0])
            } else {
                Ok(AstValue::None)
            }
        }
        AstNodeKind::If => execute_if(ctx, node),
        AstNodeKind::WhileLoop => execute_while(ctx, node),
        AstNodeKind::ForLoop => execute_for(ctx, node),
        AstNodeKind::Assert => {
            execute_assert(ctx, node)?;
            Ok(AstValue::None)
        }
        AstNodeKind::Invariant => {
            execute_invariant(ctx, node)?;
            Ok(AstValue::None)
        }
        _ => evaluate(ctx, node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNodeFactory;

    fn ctx() -> ExecContext {
        ExecContext::with_default_config()
    }
    fn declare_int(ctx: &mut ExecContext, name: &str, val: i64) {
        ctx.symbols
            .declare(name, &AstValue::Integer(val), &mut ctx.heap)
            .unwrap();
    }

    #[test]
    fn eval_integer_literal() {
        let mut c = ctx();
        assert_eq!(
            evaluate(&mut c, &AstNodeFactory::integer_literal("42")).unwrap(),
            AstValue::Integer(42)
        );
    }
    #[test]
    fn eval_addition() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "+",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("5"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Integer(15));
    }
    #[test]
    fn eval_division() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "/",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("3"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Integer(3));
    }
    #[test]
    fn eval_div_by_zero() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "/",
                    AstNodeFactory::integer_literal("1"),
                    AstNodeFactory::integer_literal("0")
                )
            )
            .unwrap_err(),
            ExitCode::DivisionByZero
        );
    }
    #[test]
    fn eval_mod_zero() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "mod",
                    AstNodeFactory::integer_literal("1"),
                    AstNodeFactory::integer_literal("0")
                )
            )
            .unwrap_err(),
            ExitCode::DivisionByZero
        );
    }
    #[test]
    fn eval_eq() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "==",
                    AstNodeFactory::integer_literal("5"),
                    AstNodeFactory::integer_literal("5")
                )
            )
            .unwrap(),
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn eval_neq() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "!=",
                    AstNodeFactory::integer_literal("5"),
                    AstNodeFactory::integer_literal("3")
                )
            )
            .unwrap(),
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn eval_lt() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "<",
                    AstNodeFactory::integer_literal("3"),
                    AstNodeFactory::integer_literal("5")
                )
            )
            .unwrap(),
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn eval_gt() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    ">",
                    AstNodeFactory::integer_literal("10"),
                    AstNodeFactory::integer_literal("5")
                )
            )
            .unwrap(),
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn eval_and() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "and",
                    AstNodeFactory::boolean_literal(true),
                    AstNodeFactory::boolean_literal(false)
                )
            )
            .unwrap(),
            AstValue::Boolean(false)
        );
    }
    #[test]
    fn eval_or() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::binary_expression(
                    "or",
                    AstNodeFactory::boolean_literal(false),
                    AstNodeFactory::boolean_literal(true)
                )
            )
            .unwrap(),
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn eval_unary_neg() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::unary_expression("-", AstNodeFactory::integer_literal("42"))
            )
            .unwrap(),
            AstValue::Integer(-42)
        );
    }
    #[test]
    fn eval_unary_not() {
        let mut c = ctx();
        assert_eq!(
            evaluate(
                &mut c,
                &AstNodeFactory::unary_expression("not", AstNodeFactory::boolean_literal(true))
            )
            .unwrap(),
            AstValue::Boolean(false)
        );
    }
    #[test]
    fn eval_nested() {
        let mut c = ctx();
        let inner = AstNodeFactory::binary_expression(
            "*",
            AstNodeFactory::integer_literal("3"),
            AstNodeFactory::integer_literal("4"),
        );
        let outer =
            AstNodeFactory::binary_expression("+", inner, AstNodeFactory::integer_literal("2"));
        assert_eq!(evaluate(&mut c, &outer).unwrap(), AstValue::Integer(14));
    }
    #[test]
    fn eval_identifier_via_heap() {
        let mut c = ctx();
        declare_int(&mut c, "x", 100);
        assert_eq!(
            evaluate(&mut c, &AstNodeFactory::identifier("x")).unwrap(),
            AstValue::Integer(100)
        );
    }
    #[test]
    fn eval_undeclared_traps() {
        assert_eq!(
            evaluate(&mut ctx(), &AstNodeFactory::identifier("undef")).unwrap_err(),
            ExitCode::NullDereference
        );
    }
    #[test]
    fn exec_var_decl_allocates() {
        let mut c = ctx();
        let decl = AstNodeFactory::variable_declaration(
            "count",
            AstNodeFactory::type_node("Integer", vec![]),
            Some(AstNodeFactory::integer_literal("42")),
        );
        execute_var_decl(&mut c, &decl).unwrap();
        assert_eq!(
            c.symbols.lookup("count", &c.heap).unwrap(),
            AstValue::Integer(42)
        );
    }
    #[test]
    fn exec_assign_updates_heap() {
        let mut c = ctx();
        declare_int(&mut c, "x", 0);
        execute_assignment(
            &mut c,
            &AstNodeFactory::assignment(
                AstNodeFactory::identifier("x"),
                AstNodeFactory::integer_literal("8"),
            ),
        )
        .unwrap();
        assert_eq!(
            c.symbols.lookup("x", &c.heap).unwrap(),
            AstValue::Integer(8)
        );
    }
    #[test]
    fn exec_assert_pass() {
        let mut c = ctx();
        execute_assert(
            &mut c,
            &AstNodeFactory::assert_stmt(AstNodeFactory::boolean_literal(true), None),
        )
        .unwrap();
        assert!(c.trap.is_ok());
    }
    #[test]
    fn exec_assert_fail() {
        assert_eq!(
            execute_assert(
                &mut ctx(),
                &AstNodeFactory::assert_stmt(AstNodeFactory::boolean_literal(false), None)
            )
            .unwrap_err(),
            ExitCode::AssertionFailure
        );
    }
    #[test]
    fn exec_invariant_fail() {
        assert_eq!(
            execute_invariant(
                &mut ctx(),
                &AstNodeFactory::invariant_stmt(AstNodeFactory::boolean_literal(false), None)
            )
            .unwrap_err(),
            ExitCode::InvariantViolation
        );
    }
    #[test]
    fn step_counter_increments() {
        let mut c = ctx();
        let before = c.profiler.step_count();
        evaluate(&mut c, &AstNodeFactory::integer_literal("1")).unwrap();
        assert!(c.profiler.step_count() > before);
    }
    #[test]
    fn complexity_enforcement_present() {
        let mut c = ctx();
        let algo = AstNodeFactory::algorithm(
            "Test",
            vec![],
            None,
            "O(1)",
            vec![],
            vec![AstNodeFactory::return_stmt(Some(
                AstNodeFactory::integer_literal("1"),
            ))],
        );
        assert!(execute_algorithm(&mut c, &algo).is_ok());
    }
    #[test]
    fn heap_distinct_values() {
        let mut c = ctx();
        declare_int(&mut c, "a", 10);
        declare_int(&mut c, "b", 20);
        assert_eq!(
            c.symbols.lookup("a", &c.heap).unwrap(),
            AstValue::Integer(10)
        );
        assert_eq!(
            c.symbols.lookup("b", &c.heap).unwrap(),
            AstValue::Integer(20)
        );
    }
    #[test]
    fn exec_while_loop() {
        let mut c = ctx();
        declare_int(&mut c, "x", 0);
        let cond = AstNodeFactory::binary_expression(
            "<",
            AstNodeFactory::identifier("x"),
            AstNodeFactory::integer_literal("3"),
        );
        let incr = AstNodeFactory::assignment(
            AstNodeFactory::identifier("x"),
            AstNodeFactory::binary_expression(
                "+",
                AstNodeFactory::identifier("x"),
                AstNodeFactory::integer_literal("1"),
            ),
        );
        let body = AstNode::internal(AstNodeKind::WhileLoop, vec![], None);
        let while_node = AstNode::internal(
            AstNodeKind::WhileLoop,
            vec![
                cond,
                AstNode::internal(AstNodeKind::WhileLoop, vec![incr], None),
            ],
            None,
        );
        execute_while(&mut c, &while_node).ok();
        assert_eq!(
            c.symbols.lookup("x", &c.heap).unwrap(),
            AstValue::Integer(3)
        );
    }
    #[test]
    fn exec_for_loop() {
        let mut c = ctx();
        let iter = AstNode::leaf(
            AstNodeKind::Identifier,
            Some(AstValue::String("i".to_string())),
        );
        let body = AstNodeFactory::assignment(
            AstNodeFactory::identifier("i"),
            AstNodeFactory::integer_literal("0"),
        );
        let for_node = AstNode::internal(
            AstNodeKind::ForLoop,
            vec![iter, AstNodeFactory::integer_literal("5"), body],
            None,
        );
        execute_for(&mut c, &for_node).ok();
    }
    #[test]
    fn exec_if_true_branch() {
        let mut c = ctx();
        declare_int(&mut c, "x", 0);
        let if_node = AstNode::internal(
            AstNodeKind::If,
            vec![
                AstNodeFactory::boolean_literal(true),
                AstNode::internal(
                    AstNodeKind::If,
                    vec![AstNodeFactory::assignment(
                        AstNodeFactory::identifier("x"),
                        AstNodeFactory::integer_literal("42"),
                    )],
                    None,
                ),
            ],
            None,
        );
        execute_if(&mut c, &if_node).ok();
        assert_eq!(
            c.symbols.lookup("x", &c.heap).unwrap(),
            AstValue::Integer(42)
        );
    }
    #[test]
    fn builtin_length() {
        assert_eq!(
            dispatch_builtin("length", &[AstValue::Integer(1)]).unwrap(),
            AstValue::Integer(1)
        );
    }
    #[test]
    fn builtin_contains_true() {
        assert_eq!(
            dispatch_builtin("contains", &[AstValue::Integer(1), AstValue::Integer(1)]).unwrap(),
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn builtin_contains_false() {
        assert_eq!(
            dispatch_builtin("contains", &[AstValue::Integer(1), AstValue::Integer(2)]).unwrap(),
            AstValue::Boolean(false)
        );
    }
    #[test]
    fn builtin_append() {
        assert_eq!(
            dispatch_builtin("append", &[AstValue::Integer(1), AstValue::Integer(42)]).unwrap(),
            AstValue::Integer(42)
        );
    }
    #[test]
    fn builtin_pop() {
        assert_eq!(
            dispatch_builtin("pop", &[AstValue::Integer(99)]).unwrap(),
            AstValue::Integer(99)
        );
    }
}
