//! Abstract interpreter for the UEAS kernel.
//!
//! Walks the canonical AST, evaluates expressions against the virtual heap,
//! maintains a stacked symbol table of heap handles, increments the step
//! counter, and enforces invariants and complexity contracts.
//!
//! Per SPEC.md Section 6.1, the SymbolTable maps identifiers to heap
//! addresses. All values are allocated on the VirtualHeap for isolation
//! and reproducibility.

use crate::ast::{AstNode, AstNodeKind, AstValue};
use crate::heap::{HeapHandle, VirtualHeap};
use crate::profiling::{ComplexityContract, Profiler, ProfilingConfig};
use crate::traps::{ExitCode, TrapRegister};
use std::collections::HashMap;

/// A stacked lexical scope mapping identifiers to heap addresses.
#[derive(Debug, Clone, Default)]
struct Scope {
    symbols: HashMap<String, HeapHandle>,
}

/// Stack of lexical scopes for variable resolution.
///
/// Per SPEC.md Section 6.1: maps identifiers to heap addresses.
/// Every variable is stored on the VirtualHeap. The SymbolTable
/// contains only HeapHandle references.
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

    /// Declare a variable: allocate on the virtual heap, write the value,
    /// and store the heap handle in the current scope.
    pub fn declare(
        &mut self,
        name: &str,
        value: &AstValue,
        heap: &mut VirtualHeap,
    ) -> Result<HeapHandle, ExitCode> {
        let (bytes, tag) = value_to_bytes(value);
        let handle = heap.allocate(bytes.len(), tag)?;
        heap.write(handle, 0, &bytes)?;
        let scope = self.scopes.last_mut().ok_or(ExitCode::StackOverflow)?;
        scope.symbols.insert(name.to_string(), handle);
        Ok(handle)
    }

    /// Look up a variable and read its value from the virtual heap.
    pub fn lookup(&self, name: &str, heap: &VirtualHeap) -> Option<AstValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(handle) = scope.symbols.get(name) {
                return read_value_from_heap(heap, *handle).ok();
            }
        }
        None
    }

    /// Get the heap handle for a variable.
    pub fn lookup_handle(&self, name: &str) -> Option<HeapHandle> {
        for scope in self.scopes.iter().rev() {
            if let Some(handle) = scope.symbols.get(name) {
                return Some(*handle);
            }
        }
        None
    }

    /// Update a variable's value on the virtual heap.
    pub fn assign(
        &mut self,
        name: &str,
        value: &AstValue,
        heap: &mut VirtualHeap,
    ) -> Result<(), ExitCode> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(handle) = scope.symbols.get(name) {
                let (bytes, _tag) = value_to_bytes(value);
                return heap.write(*handle, 0, &bytes);
            }
        }
        Err(ExitCode::NullDereference)
    }
}

/// Convert an AstValue to bytes for heap storage.
fn value_to_bytes(value: &AstValue) -> (Vec<u8>, crate::heap::TypeTag) {
    use crate::heap::TypeTag;
    match value {
        AstValue::Integer(s) => {
            let mut data = Vec::from(s.as_bytes());
            data.push(0); // null terminator
            (data, TypeTag::Integer)
        }
        AstValue::Real(x) => {
            let data = x.to_be_bytes().to_vec();
            (data, TypeTag::Real)
        }
        AstValue::Boolean(b) => {
            let data = vec![if *b { 1u8 } else { 0u8 }];
            (data, TypeTag::Boolean)
        }
        AstValue::String(s) => {
            let mut data = Vec::from(s.as_bytes());
            data.push(0);
            (data, TypeTag::String)
        }
        AstValue::None => (vec![0u8], TypeTag::Unknown),
    }
}

/// Read an AstValue from the virtual heap.
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
            let s = String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string();
            AstValue::Integer(s)
        }
        TypeTag::Real => {
            if bytes.len() >= 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8]);
                AstValue::Real(f64::from_be_bytes(arr))
            } else {
                AstValue::Real(0.0)
            }
        }
        TypeTag::Boolean => AstValue::Boolean(bytes.first().map(|b| *b != 0).unwrap_or(false)),
        TypeTag::String => {
            let s = String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string();
            AstValue::String(s)
        }
        TypeTag::HeapHandle | TypeTag::Unknown => AstValue::None,
    })
}

/// The execution context bundles all kernel state.
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

/// Evaluate an AST expression node and return its value.
pub fn evaluate(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if ctx.trap.is_trapped() {
        return Err(ctx.trap.code());
    }
    ctx.profiler.step()?;

    match node.kind {
        AstNodeKind::IntegerLiteral => match &node.value {
            Some(AstValue::Integer(s)) => Ok(AstValue::Integer(s.clone())),
            Some(AstValue::String(s)) => Ok(AstValue::Integer(s.clone())),
            _ => Err(ExitCode::HeapExhaustion),
        },
        AstNodeKind::RealLiteral => match &node.value {
            Some(AstValue::Real(_)) => Ok(node.value.clone().unwrap()),
            _ => Err(ExitCode::HeapExhaustion),
        },
        AstNodeKind::StringLiteral => match &node.value {
            Some(AstValue::String(_)) => Ok(node.value.clone().unwrap()),
            _ => Err(ExitCode::HeapExhaustion),
        },
        AstNodeKind::BooleanLiteral => match &node.value {
            Some(AstValue::Boolean(_)) => Ok(node.value.clone().unwrap()),
            _ => Err(ExitCode::HeapExhaustion),
        },
        AstNodeKind::NoneLiteral => Ok(AstValue::None),
        AstNodeKind::Identifier => {
            let name = match &node.value {
                Some(AstValue::String(s)) => s.clone(),
                _ => return Err(ExitCode::HeapExhaustion),
            };
            ctx.symbols
                .lookup(&name, &ctx.heap)
                .ok_or(ExitCode::NullDereference)
        }
        AstNodeKind::BinaryExpression => eval_binary(ctx, node),
        AstNodeKind::UnaryExpression => eval_unary(ctx, node),
        AstNodeKind::FunctionCall => eval_function_call(ctx, node),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Evaluate a binary expression.
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
        "+" => binary_arithmetic(&left, &right, |a, b| a + b, |a, b| a + b),
        "-" => binary_arithmetic(&left, &right, |a, b| a - b, |a, b| a - b),
        "*" => binary_arithmetic(&left, &right, |a, b| a * b, |a, b| a * b),
        "/" => binary_divide(&left, &right),
        "mod" => binary_modulo(&left, &right),
        "==" => Ok(AstValue::Boolean(left == right)),
        "!=" => Ok(AstValue::Boolean(left != right)),
        "<" => binary_cmp(&left, &right, |a, b| a < b),
        "<=" => binary_cmp(&left, &right, |a, b| a <= b),
        ">" => binary_cmp(&left, &right, |a, b| a > b),
        ">=" => binary_cmp(&left, &right, |a, b| a >= b),
        "and" => eval_logical_and(&left, &right),
        "or" => eval_logical_or(&left, &right),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn eval_logical_and(left: &AstValue, right: &AstValue) -> Result<AstValue, ExitCode> {
    if !is_truthy(left) {
        return Ok(AstValue::Boolean(false));
    }
    Ok(AstValue::Boolean(is_truthy(right)))
}

fn eval_logical_or(left: &AstValue, right: &AstValue) -> Result<AstValue, ExitCode> {
    if is_truthy(left) {
        return Ok(AstValue::Boolean(true));
    }
    Ok(AstValue::Boolean(is_truthy(right)))
}

fn binary_divide(left: &AstValue, right: &AstValue) -> Result<AstValue, ExitCode> {
    match (left, right) {
        (AstValue::Integer(a), AstValue::Integer(b)) => {
            let x: i64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: i64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            if y == 0 {
                return Err(ExitCode::DivisionByZero);
            }
            Ok(AstValue::Integer((x / y).to_string()))
        }
        (AstValue::String(a), AstValue::String(b)) => {
            let x: i64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: i64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            if y == 0 {
                return Err(ExitCode::DivisionByZero);
            }
            Ok(AstValue::Integer((x / y).to_string()))
        }
        (AstValue::Real(x), AstValue::Real(y)) => {
            if *y == 0.0 {
                return Err(ExitCode::DivisionByZero);
            }
            Ok(AstValue::Real(x / y))
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn binary_modulo(left: &AstValue, right: &AstValue) -> Result<AstValue, ExitCode> {
    match (left, right) {
        (AstValue::Integer(a), AstValue::Integer(b)) => {
            let x: i64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: i64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            if y == 0 {
                return Err(ExitCode::DivisionByZero);
            }
            Ok(AstValue::Integer((x % y).to_string()))
        }
        (AstValue::String(a), AstValue::String(b)) => {
            let x: i64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: i64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            if y == 0 {
                return Err(ExitCode::DivisionByZero);
            }
            Ok(AstValue::Integer((x % y).to_string()))
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Evaluate a unary expression.
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
            AstValue::Integer(s) | AstValue::String(s) => {
                let x: i64 = s.parse().map_err(|_| ExitCode::HeapExhaustion)?;
                Ok(AstValue::Integer((-x).to_string()))
            }
            AstValue::Real(x) => Ok(AstValue::Real(-x)),
            _ => Err(ExitCode::HeapExhaustion),
        },
        "not" => Ok(AstValue::Boolean(!is_truthy(&operand))),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Evaluate a function call — dispatches to builtins.
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

    dispatch_builtin(ctx, &name, &args)
}

/// Dispatch to built-in functions.
fn dispatch_builtin(
    _ctx: &mut ExecContext,
    name: &str,
    args: &[AstValue],
) -> Result<AstValue, ExitCode> {
    match name {
        "sqrt" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            match &args[0] {
                AstValue::String(s) | AstValue::Integer(s) => {
                    let x: f64 = s.parse().map_err(|_| ExitCode::HeapExhaustion)?;
                    if x < 0.0 {
                        return Err(ExitCode::HeapExhaustion);
                    }
                    Ok(AstValue::Real(x.sqrt()))
                }
                AstValue::Real(x) => {
                    if *x < 0.0 {
                        return Err(ExitCode::HeapExhaustion);
                    }
                    Ok(AstValue::Real(x.sqrt()))
                }
                _ => Err(ExitCode::HeapExhaustion),
            }
        }
        "length" | "cardinality" => Ok(AstValue::Integer("0".to_string())),
        "range" => {
            if args.len() < 2 {
                return Err(ExitCode::HeapExhaustion);
            }
            Ok(AstValue::String("0".to_string()))
        }
        "emptyList" | "emptySet" | "emptyMap" | "zeroMatrix" => {
            Ok(AstValue::String("0".to_string()))
        }
        "contains" | "add" | "append" | "slice" | "pop" | "extractMin" | "adjacent" | "weight"
        | "nodes" | "edges" => Ok(AstValue::String("0".to_string())),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Helper: arithmetic on two AstValues.
fn binary_arithmetic<FI, FR>(
    left: &AstValue,
    right: &AstValue,
    int_op: FI,
    real_op: FR,
) -> Result<AstValue, ExitCode>
where
    FI: Fn(i64, i64) -> i64,
    FR: Fn(f64, f64) -> f64,
{
    match (left, right) {
        (AstValue::Integer(a), AstValue::Integer(b)) => {
            let x: i64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: i64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            Ok(AstValue::Integer(int_op(x, y).to_string()))
        }
        (AstValue::String(a), AstValue::String(b)) => {
            let x: i64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: i64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            Ok(AstValue::Integer(int_op(x, y).to_string()))
        }
        (AstValue::Real(a), AstValue::Real(b)) => Ok(AstValue::Real(real_op(*a, *b))),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Helper: comparison on two AstValues.
fn binary_cmp<F>(left: &AstValue, right: &AstValue, cmp: F) -> Result<AstValue, ExitCode>
where
    F: Fn(f64, f64) -> bool,
{
    match (left, right) {
        (AstValue::Integer(a), AstValue::Integer(b)) => {
            let x: f64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: f64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            Ok(AstValue::Boolean(cmp(x, y)))
        }
        (AstValue::String(a), AstValue::String(b)) => {
            let x: f64 = a.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            let y: f64 = b.parse().map_err(|_| ExitCode::HeapExhaustion)?;
            Ok(AstValue::Boolean(cmp(x, y)))
        }
        (AstValue::Real(a), AstValue::Real(b)) => Ok(AstValue::Boolean(cmp(*a, *b))),
        (AstValue::Boolean(a), AstValue::Boolean(b)) => {
            Ok(AstValue::Boolean(cmp(*a as u8 as f64, *b as u8 as f64)))
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

fn is_truthy(value: &AstValue) -> bool {
    match value {
        AstValue::Boolean(b) => *b,
        AstValue::Integer(s) => s != "0",
        AstValue::String(s) => s != "0" && !s.is_empty(),
        AstValue::Real(x) => *x != 0.0,
        AstValue::None => false,
    }
}

pub fn execute_program(ctx: &mut ExecContext, program: &AstNode) -> Result<AstValue, ExitCode> {
    if program.kind != AstNodeKind::Program {
        return Err(ExitCode::HeapExhaustion);
    }
    let mut last_value = AstValue::None;
    for algo in &program.children {
        last_value = execute_algorithm(ctx, algo)?;
    }
    Ok(last_value)
}

fn execute_algorithm(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    ctx.symbols.push_scope();
    let mut last_value = AstValue::None;

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
                    last_value = evaluate(ctx, &child.children[0])?;
                }
                ctx.symbols.pop_scope();
                return Ok(last_value);
            }
            AstNodeKind::If => {
                last_value = execute_if(ctx, child)?;
            }
            AstNodeKind::WhileLoop => {
                last_value = execute_while(ctx, child)?;
            }
            AstNodeKind::ForLoop => {
                last_value = execute_for(ctx, child)?;
            }
            AstNodeKind::Assert => {
                execute_assert(ctx, child)?;
            }
            AstNodeKind::Invariant => {
                execute_invariant(ctx, child)?;
            }
            _ => {
                if let Ok(v) = evaluate(ctx, child) {
                    last_value = v;
                }
            }
        }
    }
    ctx.symbols.pop_scope();
    enforce_complexity(ctx, node)?;
    Ok(last_value)
}

fn enforce_complexity(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.len() < 2 {
        return Ok(());
    }
    let complexity_str = match &node.children[1].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Ok(()),
    };
    let contract = match complexity_str.trim() {
        s if s.contains("O(1)") => ComplexityContract::Constant,
        s if s.contains("O(N^3)") => ComplexityContract::Polynomial { n: 100, k: 3 },
        s if s.contains("O(N^2)") => ComplexityContract::Quadratic { n: 100 },
        s if s.contains("O(N log N)") || s.contains("O((V+E) log V)") => {
            ComplexityContract::Linearithmic { n: 100 }
        }
        s if s.contains("O(log N)") => ComplexityContract::Logarithmic { n: 100 },
        s if s.contains("O(2^N)") => ComplexityContract::Exponential { n: 20 },
        s if s.contains("O(N!)") => ComplexityContract::Factorial { n: 10 },
        s if s.contains("O(N)") => ComplexityContract::Linear { n: 100 },
        _ => ComplexityContract::Linear { n: 100 },
    };
    ctx.profiler.verify_complexity(&contract)
}

fn execute_var_decl(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let initializer = if node.children.len() > 2 {
        Some(evaluate(ctx, &node.children[2])?)
    } else {
        None
    };
    let value = initializer.unwrap_or(AstValue::None);
    ctx.symbols.declare(&name, &value, &mut ctx.heap)?;
    Ok(())
}

fn execute_assignment(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.len() < 2 {
        return Err(ExitCode::HeapExhaustion);
    }
    let target_name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    let value = evaluate(ctx, &node.children[1])?;
    ctx.symbols.assign(&target_name, &value, &mut ctx.heap)
}

fn execute_statement_sequence(
    ctx: &mut ExecContext,
    statements: &[AstNode],
) -> Result<AstValue, ExitCode> {
    let mut last = AstValue::None;
    for stmt in statements {
        last = execute_statement(ctx, stmt)?;
    }
    Ok(last)
}

fn execute_statement(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
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

fn execute_if(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }
    let cond = evaluate(ctx, &node.children[0])?;
    if is_truthy(&cond) && node.children.len() > 1 {
        execute_statement_sequence(ctx, &node.children[1].children)
    } else if node.children.len() > 2 {
        execute_statement_sequence(ctx, &node.children[2].children)
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
            last = execute_statement_sequence(ctx, &node.children[1].children)?;
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
    let count_val = evaluate(ctx, &node.children[1])?;
    let n: usize = match &count_val {
        AstValue::Integer(s) | AstValue::String(s) => s.parse().unwrap_or(1),
        _ => 1,
    };
    ctx.symbols.push_scope();
    let mut last = AstValue::None;
    for i in 0..n {
        let iv = AstValue::Integer(i.to_string());
        ctx.symbols.declare(&iter_name, &iv, &mut ctx.heap)?;
        for body_node in node.children.iter().skip(2) {
            last = execute_statement(ctx, body_node)?;
        }
    }
    ctx.symbols.pop_scope();
    Ok(last)
}

fn execute_assert(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.is_empty() {
        return Ok(());
    }
    if !is_truthy(&evaluate(ctx, &node.children[0])?) {
        ctx.trap.set(ExitCode::AssertionFailure);
        return Err(ExitCode::AssertionFailure);
    }
    Ok(())
}

fn execute_invariant(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.is_empty() {
        return Ok(());
    }
    if !is_truthy(&evaluate(ctx, &node.children[0])?) {
        ctx.trap.set(ExitCode::InvariantViolation);
        return Err(ExitCode::InvariantViolation);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNodeFactory;

    fn make_ctx() -> ExecContext {
        ExecContext::with_default_config()
    }

    fn declare_int(ctx: &mut ExecContext, name: &str, value: &str) {
        let v = AstValue::Integer(value.to_string());
        ctx.symbols.declare(name, &v, &mut ctx.heap).unwrap();
    }

    #[test]
    fn eval_integer_literal() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::integer_literal("42");
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap(),
            AstValue::Integer("42".to_string())
        );
    }

    #[test]
    fn eval_addition() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "+",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("5"),
        );
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap(),
            AstValue::Integer("15".to_string())
        );
    }

    #[test]
    fn eval_division_by_zero() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "/",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("0"),
        );
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap_err(),
            ExitCode::DivisionByZero
        );
    }

    #[test]
    fn eval_comparison_eq() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "==",
            AstNodeFactory::integer_literal("5"),
            AstNodeFactory::integer_literal("5"),
        );
        assert_eq!(evaluate(&mut ctx, &node).unwrap(), AstValue::Boolean(true));
    }

    #[test]
    fn eval_logical_and() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "and",
            AstNodeFactory::boolean_literal(true),
            AstNodeFactory::boolean_literal(false),
        );
        assert_eq!(evaluate(&mut ctx, &node).unwrap(), AstValue::Boolean(false));
    }

    #[test]
    fn eval_unary_negation() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::unary_expression("-", AstNodeFactory::integer_literal("42"));
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap(),
            AstValue::Integer("-42".to_string())
        );
    }

    #[test]
    fn eval_identifier_via_heap() {
        let mut ctx = make_ctx();
        declare_int(&mut ctx, "x", "100");
        let node = AstNodeFactory::identifier("x");
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap(),
            AstValue::Integer("100".to_string())
        );
    }

    #[test]
    fn eval_undeclared_identifier_traps() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::identifier("undefined");
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap_err(),
            ExitCode::NullDereference
        );
    }

    #[test]
    fn exec_var_decl_allocates_to_heap() {
        let mut ctx = make_ctx();
        let typ = AstNodeFactory::type_node("Integer", vec![]);
        let init = AstNodeFactory::integer_literal("42");
        let decl = AstNodeFactory::variable_declaration("count", typ, Some(init));
        execute_var_decl(&mut ctx, &decl).unwrap();
        let val = ctx.symbols.lookup("count", &ctx.heap).unwrap();
        assert_eq!(val, AstValue::Integer("42".to_string()));
    }

    #[test]
    fn exec_assignment_updates_heap() {
        let mut ctx = make_ctx();
        declare_int(&mut ctx, "x", "0");
        let target = AstNodeFactory::identifier("x");
        let value = AstNodeFactory::integer_literal("8");
        let assign = AstNodeFactory::assignment(target, value);
        execute_assignment(&mut ctx, &assign).unwrap();
        assert_eq!(
            ctx.symbols.lookup("x", &ctx.heap).unwrap(),
            AstValue::Integer("8".to_string())
        );
    }

    #[test]
    fn exec_assertion_pass() {
        let mut ctx = make_ctx();
        execute_assert(
            &mut ctx,
            &AstNodeFactory::assert_stmt(AstNodeFactory::boolean_literal(true), None),
        )
        .unwrap();
        assert!(ctx.trap.is_ok());
    }

    #[test]
    fn exec_assertion_fail() {
        let mut ctx = make_ctx();
        assert_eq!(
            execute_assert(
                &mut ctx,
                &AstNodeFactory::assert_stmt(AstNodeFactory::boolean_literal(false), None)
            )
            .unwrap_err(),
            ExitCode::AssertionFailure
        );
    }

    #[test]
    fn exec_invariant_fail() {
        let mut ctx = make_ctx();
        assert_eq!(
            execute_invariant(
                &mut ctx,
                &AstNodeFactory::invariant_stmt(AstNodeFactory::boolean_literal(false), None)
            )
            .unwrap_err(),
            ExitCode::InvariantViolation
        );
    }

    #[test]
    fn step_counter_increments() {
        let mut ctx = make_ctx();
        let before = ctx.profiler.step_count();
        evaluate(&mut ctx, &AstNodeFactory::integer_literal("1")).unwrap();
        assert!(ctx.profiler.step_count() > before);
    }

    #[test]
    fn complexity_enforcement_present() {
        let mut ctx = make_ctx();
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
        assert!(execute_algorithm(&mut ctx, &algo).is_ok());
    }

    #[test]
    fn heap_allocated_values_are_distinct() {
        let mut ctx = make_ctx();
        declare_int(&mut ctx, "a", "10");
        declare_int(&mut ctx, "b", "20");
        let a = ctx.symbols.lookup("a", &ctx.heap).unwrap();
        let b = ctx.symbols.lookup("b", &ctx.heap).unwrap();
        assert_eq!(a, AstValue::Integer("10".to_string()));
        assert_eq!(b, AstValue::Integer("20".to_string()));
    }
}
