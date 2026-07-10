//! Abstract interpreter for the UEAS kernel.
//!
//! Walks the canonical AST, evaluates expressions against the virtual heap,
//! maintains a stacked symbol table of heap handles, increments the step
//! counter, and enforces invariants and complexity contracts.

use crate::ast::{AstNode, AstNodeKind, AstValue};
use crate::heap::{HeapHandle, TypeTag, VirtualHeap};
use crate::profiling::{ComplexityContract, ComplexityKind, Profiler, ProfilingConfig};
use crate::traps::{ExitCode, TrapRegister};
use std::collections::{HashMap, HashSet};

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
    pub fn variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for scope in &self.scopes {
            for name in scope.symbols.keys() {
                names.push(name.clone());
            }
        }
        names
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
    pub fn lookup(&self, name: &str, heap: &mut VirtualHeap) -> Option<AstValue> {
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
        AstValue::Pointer(id) => (id.to_be_bytes().to_vec(), TypeTag::HeapHandle),
    }
}

fn read_value_from_heap(heap: &mut VirtualHeap, handle: HeapHandle) -> Result<AstValue, ExitCode> {
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
        TypeTag::HeapHandle
        | TypeTag::Unknown
        | TypeTag::Set
        | TypeTag::List
        | TypeTag::Map
        | TypeTag::Graph
        | TypeTag::DirectedGraph
        | TypeTag::UndirectedGraph
        | TypeTag::Matrix
        | TypeTag::Option
        | TypeTag::Result
        | TypeTag::Tuple => AstValue::Pointer(handle.as_u64()),
    })
}

#[derive(Debug)]
pub struct ExecContext {
    pub heap: VirtualHeap,
    pub symbols: SymbolTable,
    pub profiler: Profiler,
    pub trap: TrapRegister,
    /// When true, all if-statements execute both branches and compare step
    /// counts for timing leak detection (ADR 0016 / @ConstantTime).
    pub constant_time_mode: bool,
    /// Variable names declared with Secret<T> type (RFC 0011).
    pub secret_variables: HashSet<String>,
    /// PRNG state for stochastic algorithm execution (RFC 0009).
    pub prng_state: u64,
}

/// Default PRNG seed for reproducible stochastic execution (RFC 0009).
pub const PRNG_DEFAULT_SEED: u64 = 0xCAFE_F00D_D15C_0001;

/// Advance an LCG PRNG (m = 2^64, a = 6364136223846793005, c = 1442695040888963407).
/// Returns the new state.
pub fn prng_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

/// Generate a pseudo-random integer in [min, max] (inclusive).
pub fn rand_range(prng: &mut u64, min: i64, max: i64) -> i64 {
    let r = prng_next(prng);
    let range = max.saturating_sub(min).saturating_add(1) as u64;
    if range == 0 {
        return min;
    }
    let v = (r % range) as i64;
    min + v
}

impl ExecContext {
    pub fn new(config: ProfilingConfig) -> Self {
        let seed = config.prng_seed;
        Self {
            heap: VirtualHeap::with_default_config(),
            symbols: SymbolTable::new(),
            profiler: Profiler::new(config),
            trap: TrapRegister::new(),
            constant_time_mode: false,
            secret_variables: HashSet::new(),
            prng_state: seed,
        }
    }
    pub fn with_default_config() -> Self {
        Self::new(ProfilingConfig::default())
    }
    /// Generate a pseudo-random integer in [min, max] (inclusive) (RFC 0009).
    pub fn rand_int(&mut self, min: i64, max: i64) -> i64 {
        rand_range(&mut self.prng_state, min, max)
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
        | AstNodeKind::BooleanLiteral => node.value.clone().ok_or(ExitCode::InvalidOperation),
        AstNodeKind::NoneLiteral => Ok(AstValue::None),
        AstNodeKind::InfinityLiteral => Ok(AstValue::Real(f64::INFINITY)),
        AstNodeKind::NanLiteral => Ok(AstValue::Real(f64::NAN)),
        AstNodeKind::Identifier => {
            let name = match &node.value {
                Some(AstValue::String(s)) => s.as_str(),
                _ => return Err(ExitCode::InvalidOperation),
            };
            ctx.symbols
                .lookup(name, &mut ctx.heap)
                .ok_or(ExitCode::NullDereference)
        }
        AstNodeKind::BinaryExpression => eval_binary(ctx, node),
        AstNodeKind::UnaryExpression => eval_unary(ctx, node),
        AstNodeKind::FunctionCall => eval_function_call(ctx, node),
        AstNodeKind::SetLiteral | AstNodeKind::ListLiteral | AstNodeKind::MapLiteral => {
            Ok(AstValue::Pointer(
                ctx.heap
                    .allocate(node.children.len() * 8, TypeTag::Unknown)
                    .map_err(|_| ExitCode::HeapExhaustion)?
                    .as_u64(),
            ))
        }
        _ => Err(ExitCode::InvalidOperation),
    }
}

fn eval_binary(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 3 {
        return Err(ExitCode::InvalidOperation);
    }
    let op = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let left = evaluate(ctx, &node.children[1])?;
    let right = evaluate(ctx, &node.children[2])?;
    match op.as_str() {
        "+" => op2(&left, &right, |a, b| a + b, |a, b| a + b),
        "-" => op2(&left, &right, |a, b| a - b, |a, b| a - b),
        "*" => op2(&left, &right, |a, b| a * b, |a, b| a * b),
        "in" => eval_in_operator(&left, &right),
        "!in" | "notin" => {
            let inner = eval_in_operator(&left, &right)?;
            Ok(AstValue::Boolean(!matches!(inner, AstValue::Boolean(true))))
        }
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
            _ => Err(ExitCode::InvalidOperation),
        },
        "mod" => match (&left, &right) {
            (AstValue::Integer(a), AstValue::Integer(b)) => {
                if *b == 0 {
                    Err(ExitCode::DivisionByZero)
                } else {
                    Ok(AstValue::Integer(a % b))
                }
            }
            _ => Err(ExitCode::InvalidOperation),
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
        "|" => bitwise_op(&left, &right, |a, b| a | b),
        "&" => bitwise_op(&left, &right, |a, b| a & b),
        "^" => bitwise_op(&left, &right, |a, b| a ^ b),
        "<<" => bitwise_op(&left, &right, |a, b| a << b),
        ">>" => bitwise_op(&left, &right, |a, b| a >> b),
        _ => Err(ExitCode::InvalidOperation),
    }
}

fn eval_in_operator(left: &AstValue, right: &AstValue) -> Result<AstValue, ExitCode> {
    Ok(AstValue::Boolean(left == right))
}

fn bitwise_op<F>(left: &AstValue, right: &AstValue, op: F) -> Result<AstValue, ExitCode>
where
    F: Fn(i64, i64) -> i64,
{
    match (left, right) {
        (AstValue::Integer(a), AstValue::Integer(b)) => Ok(AstValue::Integer(op(*a, *b))),
        _ => Err(ExitCode::InvalidOperation),
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
        _ => Err(ExitCode::InvalidOperation),
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
        _ => Err(ExitCode::InvalidOperation),
    }
}

fn eval_unary(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 2 {
        return Err(ExitCode::InvalidOperation);
    }
    let op = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let operand = evaluate(ctx, &node.children[1])?;
    match op.as_str() {
        "-" => match &operand {
            AstValue::Integer(x) => {
                let v = x.checked_neg().ok_or(ExitCode::InvalidOperation)?;
                Ok(AstValue::Integer(v))
            }
            AstValue::Real(x) => Ok(AstValue::Real(-x)),
            _ => Err(ExitCode::InvalidOperation),
        },
        "not" => Ok(AstValue::Boolean(!is_truthy(&operand))),
        _ => Err(ExitCode::InvalidOperation),
    }
}

fn eval_function_call(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    let name = match node.children.first().and_then(|n| n.value.as_ref()) {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let mut args = Vec::new();
    for child in node.children.iter().skip(1) {
        args.push(evaluate(ctx, child)?);
    }
    ctx.profiler.step()?;
    let (result, weight) = dispatch_builtin(&name, &args, &ctx.heap, &mut ctx.prng_state)?;
    ctx.profiler.step_weighted(weight)?;
    Ok(result)
}

fn dispatch_builtin(
    name: &str,
    args: &[AstValue],
    heap: &VirtualHeap,
    prng: &mut u64,
) -> Result<(AstValue, u64), ExitCode> {
    match name {
        "sqrt" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            let x: f64 = match &args[0] {
                AstValue::Integer(i) => *i as f64,
                AstValue::Real(r) => *r,
                _ => return Err(ExitCode::InvalidOperation),
            };
            if x < 0.0 {
                return Err(ExitCode::InvalidOperation);
            }
            Ok((AstValue::Real(x.sqrt()), 1))
        }
        "length" | "cardinality" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            match &args[0] {
                AstValue::Integer(x) => Ok((AstValue::Integer(*x), 1)),
                AstValue::Real(x) => Ok((AstValue::Integer(*x as i64), 1)),
                AstValue::Pointer(id) => {
                    let handle = HeapHandle::from_u64(*id);
                    let size = heap.allocation_size(handle).unwrap_or(0) as i64;
                    Ok((AstValue::Integer(size), 1))
                }
                _ => Ok((AstValue::Integer(0), 1)),
            }
        }
        "contains" => {
            if args.len() < 2 {
                return Err(ExitCode::InvalidOperation);
            }
            Ok((
                AstValue::Boolean(args[1..].contains(&args[0])),
                args.len() as u64,
            ))
        }
        "append" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            Ok((args[args.len() - 1].clone(), 1))
        }
        "pop" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            Ok((args[0].clone(), 1))
        }
        "union" | "intersection" | "difference" => {
            if args.len() < 2 {
                return Err(ExitCode::InvalidOperation);
            }
            let w = args.len() as u64;
            Ok((AstValue::Pointer(0), w))
        }
        "get" | "put" | "containsKey" => {
            if args.len() < 2 {
                return Err(ExitCode::InvalidOperation);
            }
            Ok((args.last().cloned().unwrap_or(AstValue::None), 1))
        }
        "keys" | "values" | "nodes" | "edges" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            Err(ExitCode::InvalidOperation)
        }
        "transpose" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            let cost = match &args[0] {
                AstValue::Pointer(id) => {
                    heap.allocation_size(HeapHandle::from_u64(*id)).unwrap_or(1) as u64
                }
                _ => 1,
            };
            Ok((AstValue::Pointer(0), cost.max(1)))
        }
        "substring" => {
            if args.len() < 3 {
                return Err(ExitCode::InvalidOperation);
            }
            let s = match &args[0] {
                AstValue::String(s) => s.clone(),
                _ => return Err(ExitCode::InvalidOperation),
            };
            let start: usize = match &args[1] {
                AstValue::Integer(x) => *x as usize,
                _ => 0,
            };
            let end: usize = match &args[2] {
                AstValue::Integer(x) => *x as usize,
                _ => s.len(),
            };
            let r: String = s
                .chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .collect();
            Ok((AstValue::String(r), 1))
        }
        "concat" => {
            let r: String = args
                .iter()
                .map(|v| match v {
                    AstValue::String(s) => s.clone(),
                    AstValue::Integer(x) => x.to_string(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("");
            Ok((AstValue::String(r), args.len() as u64))
        }
        "strlen" => {
            if args.is_empty() {
                return Err(ExitCode::InvalidOperation);
            }
            let n = match &args[0] {
                AstValue::String(s) => s.len() as i64,
                _ => 0,
            };
            Ok((AstValue::Integer(n), 1))
        }
        "prepend" | "slice" | "adjacent" | "neighbors" | "addNode" | "addEdge" | "removeNode"
        | "extractMin" | "weight" | "multiply" | "determinant" | "range" | "emptyList"
        | "emptySet" | "emptyMap" | "zeroMatrix" | "add" => Err(ExitCode::InvalidOperation),
        "random" | "randInt" => {
            let min: i64 = match args.first() {
                Some(AstValue::Integer(x)) => *x,
                _ => 0,
            };
            let max: i64 = match args.get(1) {
                Some(AstValue::Integer(x)) => *x,
                _ => min,
            };
            let r = rand_range(prng, min, max);
            Ok((AstValue::Integer(r), 1))
        }
        "randReal" => {
            let r = prng_next(prng);
            let v = (r as f64) / (u64::MAX as f64);
            Ok((AstValue::Real(v), 1))
        }
        _ => Err(ExitCode::InvalidOperation),
    }
}

pub fn is_truthy(value: &AstValue) -> bool {
    match value {
        AstValue::Boolean(b) => *b,
        AstValue::Integer(x) => *x != 0,
        AstValue::String(s) => s != "0" && !s.is_empty(),
        AstValue::Real(x) => *x != 0.0,
        AstValue::None => false,
        AstValue::Pointer(id) => *id != 0,
    }
}

pub fn execute_program(ctx: &mut ExecContext, program: &AstNode) -> Result<AstValue, ExitCode> {
    if program.kind != AstNodeKind::Program {
        return Err(ExitCode::InvalidOperation);
    }
    let mut last = AstValue::None;
    for algo in &program.children {
        last = execute_algorithm(ctx, algo)?;
    }
    Ok(last)
}

fn execute_algorithm(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    ctx.symbols.push_scope();
    ctx.profiler.enter_recursion()?;
    let mut last = AstValue::None;
    for (i, child) in node.children.iter().enumerate() {
        if i < 2 {
            continue;
        }
        match child.kind {
            AstNodeKind::VariableDeclaration => {
                execute_var_decl(ctx, child)?;
            }
            AstNodeKind::ConstDeclaration => {
                execute_const_decl(ctx, child)?;
            }
            AstNodeKind::Assignment => {
                execute_assignment(ctx, child)?;
            }
            AstNodeKind::Return => {
                if !child.children.is_empty() {
                    last = evaluate(ctx, &child.children[0])?;
                }
                ctx.profiler.exit_recursion();
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
            AstNodeKind::Yield => {
                last = exec_yield(ctx, child)?;
            }
            AstNodeKind::Await => {
                last = exec_await(ctx, child)?;
            }
            AstNodeKind::Spawn => {
                last = exec_spawn(ctx, child)?;
            }
            AstNodeKind::Sync => {
                last = exec_sync(ctx, child)?;
            }
            AstNodeKind::ParallelFor => {
                last = exec_parallel_for(ctx, child)?;
            }
            _ => {
                if let Ok(v) = evaluate(ctx, child) {
                    last = v;
                }
            }
        }
    }
    ctx.profiler.exit_recursion();
    ctx.symbols.pop_scope();
    enforce_complexity(ctx, node)?;
    enforce_memory(ctx, node)?;
    Ok(last)
}

/// Enforce Memory: annotation by checking heap bytes allocated.
fn enforce_memory(ctx: &ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    // Memory annotation, if present, follows complexity string in algorithm children.
    // v3.0 format: "Memory" annotation string (without @ prefix).
    for child in node.children.iter().skip(1) {
        match &child.value {
            Some(AstValue::String(s)) if s.starts_with("Memory") || s.contains("@Memory") => {
                let bytes = ctx.heap.bytes_allocated() as u64;
                let max_bytes = 256 * 1024 * 1024; // default 256MB
                if bytes > max_bytes {
                    return Err(ExitCode::HeapExhaustion);
                }
                return Ok(());
            }
            _ => continue,
        }
    }
    Ok(())
}

fn enforce_complexity(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.len() < 2 {
        return Ok(());
    }
    let complexity_str = match &node.children[1].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Ok(()),
    };

    // Evaluate variableBinding expressions to get concrete input sizes
    let mut n_val: u64 = 0;
    let mut v_val: u64 = 0;
    let mut e_val: u64 = 0;
    let mut expected_str: Option<String> = None;

    for child in &node.children {
        if child.kind == AstNodeKind::VariableBinding {
            if child.children.len() < 2 {
                continue;
            }
            let var_name = match &child.children[0].value {
                Some(AstValue::String(s)) => s.clone(),
                _ => continue,
            };
            if var_name == "Expected" {
                expected_str = match &child.children[1].value {
                    Some(AstValue::String(s)) => Some(s.clone()),
                    _ => None,
                };
                continue;
            }
            if let Ok(value) = evaluate(ctx, &child.children[1]) {
                let concrete = match value {
                    AstValue::Integer(x) => x.max(0) as u64,
                    _ => 0,
                };
                match var_name.as_str() {
                    "N" => n_val = concrete,
                    "V" => v_val = concrete,
                    "E" => e_val = concrete,
                    _ => {}
                }
            }
        }
    }

    if n_val == 0 && v_val == 0 && e_val == 0 {
        n_val = 1;
    }

    let s = complexity_str.trim();
    let kind = match () {
        _ if s.contains("O(1)") => ComplexityKind::Constant,
        _ if s.contains("O((V+E) log V)") => {
            let sum = v_val.saturating_add(e_val).max(1);
            ComplexityKind::MixedLogLinear {
                sum,
                log_term: v_val.max(1),
            }
        }
        _ if s.contains("O(V+E)") || s.contains("O(V + E)") => ComplexityKind::Sum {
            terms: vec![v_val.max(1), e_val.max(1)],
        },
        _ if s.contains("O(N^3)") => ComplexityKind::Polynomial { n: n_val, k: 3 },
        _ if s.contains("O(N^2)") => ComplexityKind::Quadratic { n: n_val },
        _ if s.contains("O(N log N)") => ComplexityKind::Linearithmic { n: n_val },
        _ if s.contains("O(log N)") => ComplexityKind::Logarithmic { n: n_val },
        _ if s.contains("O(2^N)") => ComplexityKind::Exponential {
            n: n_val.clamp(1, 20),
        },
        _ if s.contains("O(N!)") => ComplexityKind::Factorial {
            n: n_val.clamp(1, 10),
        },
        _ if s.contains("O(N)")
            && !s.contains("O(N^")
            && !s.contains("O(N ")
            && !s.contains("O(N!") =>
        {
            ComplexityKind::Linear { n: n_val }
        }
        _ => ComplexityKind::Linear { n: n_val },
    };
    let contract = ComplexityContract {
        kind,
        expected_complexity: expected_str,
    };
    ctx.profiler.verify_complexity(&contract)
}

pub fn execute_var_decl(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    if node.children.len() > 1 && node.children[1].kind == AstNodeKind::SecretType {
        ctx.secret_variables.insert(name.clone());
    }
    let init = if node.children.len() > 2 {
        Some(evaluate(ctx, &node.children[2])?)
    } else {
        None
    };
    ctx.symbols
        .declare(&name, &init.unwrap_or(AstValue::None), &mut ctx.heap)?;
    Ok(())
}

pub fn execute_const_decl(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    if node.children.len() < 3 {
        return Err(ExitCode::InvalidOperation);
    }
    let init = evaluate(ctx, &node.children[2])?;
    ctx.symbols.declare(&name, &init, &mut ctx.heap)?;
    Ok(())
}

pub fn execute_assignment(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.len() < 2 {
        return Err(ExitCode::InvalidOperation);
    }
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let value = evaluate(ctx, &node.children[1])?;
    match ctx.symbols.assign(&name, &value, &mut ctx.heap) {
        Ok(()) => Ok(()),
        Err(ExitCode::NullDereference) => {
            ctx.symbols.declare(&name, &value, &mut ctx.heap)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn condition_uses_secret(ctx: &ExecContext, cond: &AstNode) -> bool {
    if cond.kind == AstNodeKind::Identifier {
        if let Some(AstValue::String(name)) = &cond.value {
            return ctx.secret_variables.contains(name);
        }
    }
    for child in &cond.children {
        if condition_uses_secret(ctx, child) {
            return true;
        }
    }
    false
}

pub fn execute_if(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }

    // Timing leak detection (ADR 0016 / RFC 0011): in @ConstantTime mode,
    // if the condition depends on a Secret variable, execute both branches
    // and compare step counts. A divergence means non-constant execution time.
    if ctx.constant_time_mode
        && node.children.len() >= 2
        && condition_uses_secret(ctx, &node.children[0])
    {
        let cond_val = evaluate(ctx, &node.children[0])?;
        let steps_before = ctx.profiler.step_count();

        // Execute the taken branch
        let result = if is_truthy(&cond_val) {
            exec_body(ctx, &node.children[1].children)?
        } else if node.children.len() > 2 {
            exec_body(ctx, &node.children[2].children)?
        } else {
            AstValue::None
        };

        let branch_steps = ctx.profiler.step_count() - steps_before;

        // Execute the OPPOSITE branch to compare step counts
        let steps_before_alt = ctx.profiler.step_count();
        let _alt_result = if is_truthy(&cond_val) {
            if node.children.len() > 2 {
                exec_body(ctx, &node.children[2].children)?
            } else {
                AstValue::None
            }
        } else {
            exec_body(ctx, &node.children[1].children)?
        };

        let alt_steps = ctx.profiler.step_count() - steps_before_alt;

        if branch_steps != alt_steps {
            ctx.trap.set(ExitCode::TimingLeak);
            return Err(ExitCode::TimingLeak);
        }

        return Ok(result);
    }

    if is_truthy(&evaluate(ctx, &node.children[0])?) && node.children.len() > 1 {
        exec_body(ctx, &node.children[1].children)
    } else if node.children.len() > 2 {
        exec_body(ctx, &node.children[2].children)
    } else {
        Ok(AstValue::None)
    }
}

pub fn execute_while(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
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

pub fn execute_for(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 2 {
        return Ok(AstValue::None);
    }
    let iter_name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let n: i64 = match evaluate(ctx, &node.children[1])? {
        AstValue::Integer(x) => x,
        _ => 1,
    };
    ctx.symbols.push_scope();
    ctx.profiler.enter_recursion()?;
    let mut last = AstValue::None;
    for i in 0..n.max(1) {
        ctx.profiler.step()?;
        ctx.symbols
            .declare(&iter_name, &AstValue::Integer(i), &mut ctx.heap)?;
        for body_node in node.children.iter().skip(2) {
            last = exec_stmt(ctx, body_node)?;
        }
    }
    ctx.profiler.exit_recursion();
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

pub fn exec_body(ctx: &mut ExecContext, body: &[AstNode]) -> Result<AstValue, ExitCode> {
    let mut last = AstValue::None;
    for stmt in body {
        last = exec_stmt(ctx, stmt)?;
    }
    Ok(last)
}

pub fn exec_stmt(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    match node.kind {
        AstNodeKind::VariableDeclaration => {
            execute_var_decl(ctx, node)?;
            Ok(AstValue::None)
        }
        AstNodeKind::ConstDeclaration => {
            execute_const_decl(ctx, node)?;
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
        AstNodeKind::Yield => exec_yield(ctx, node),
        AstNodeKind::Await => exec_await(ctx, node),
        AstNodeKind::Spawn => exec_spawn(ctx, node),
        AstNodeKind::Sync => exec_sync(ctx, node),
        AstNodeKind::ParallelFor => exec_parallel_for(ctx, node),
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

pub fn exec_yield(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }
    evaluate(ctx, &node.children[0])
}

pub fn exec_await(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    let name = match node.children.first().and_then(|n| n.value.as_ref()) {
        Some(AstValue::String(s)) => s.as_str(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    ctx.symbols
        .lookup(name, &mut ctx.heap)
        .ok_or(ExitCode::NullDereference)
}

pub fn exec_spawn(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 2 {
        return Err(ExitCode::InvalidOperation);
    }
    let name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let value = evaluate(ctx, &node.children[1])?;
    ctx.symbols.declare(&name, &value, &mut ctx.heap)?;
    ctx.profiler.record_work(1);
    ctx.profiler.record_span(1);
    Ok(value)
}

pub fn exec_sync(ctx: &mut ExecContext, _node: &AstNode) -> Result<AstValue, ExitCode> {
    ctx.profiler.record_work(1);
    ctx.profiler.record_span(1);
    Ok(AstValue::None)
}

pub fn exec_parallel_for(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 3 {
        return Ok(AstValue::None);
    }
    let iter_name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::InvalidOperation),
    };
    let n: i64 = match evaluate(ctx, &node.children[1])? {
        AstValue::Integer(x) => x,
        _ => 1,
    };
    ctx.symbols.push_scope();
    ctx.profiler.enter_recursion()?;
    let mut last = AstValue::None;
    let body = &node.children[2];
    for i in 0..n.max(1) {
        ctx.profiler.step()?;
        ctx.profiler.record_work(1);
        ctx.symbols
            .declare(&iter_name, &AstValue::Integer(i), &mut ctx.heap)?;
        last = exec_stmt(ctx, body)?;
    }
    ctx.profiler.record_span(n.max(1) as u64);
    ctx.profiler.exit_recursion();
    ctx.symbols.pop_scope();
    Ok(last)
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
            c.symbols.lookup("count", &mut c.heap).unwrap(),
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
            c.symbols.lookup("x", &mut c.heap).unwrap(),
            AstValue::Integer(8)
        );
    }
    #[test]
    fn builtin_substring() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        let (v, _) = dispatch_builtin(
            "substring",
            &[
                AstValue::String("hello".to_string()),
                AstValue::Integer(1),
                AstValue::Integer(4),
            ],
            &heap,
            &mut prng,
        )
        .unwrap();
        assert_eq!(v, AstValue::String("ell".to_string()));
    }
    #[test]
    fn builtin_concat() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        let (v, _) = dispatch_builtin(
            "concat",
            &[
                AstValue::String("a".to_string()),
                AstValue::String("b".to_string()),
            ],
            &heap,
            &mut prng,
        )
        .unwrap();
        assert_eq!(v, AstValue::String("ab".to_string()));
    }
    #[test]
    fn builtin_strlen() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        let (v, _) = dispatch_builtin(
            "strlen",
            &[AstValue::String("hello".to_string())],
            &heap,
            &mut prng,
        )
        .unwrap();
        assert_eq!(v, AstValue::Integer(5));
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
            c.symbols.lookup("a", &mut c.heap).unwrap(),
            AstValue::Integer(10)
        );
        assert_eq!(
            c.symbols.lookup("b", &mut c.heap).unwrap(),
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
        let _body = AstNode::internal(AstNodeKind::WhileLoop, vec![], None);
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
            c.symbols.lookup("x", &mut c.heap).unwrap(),
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
        let steps_before = c.profiler.step_count();
        execute_for(&mut c, &for_node).ok();
        assert!(
            c.profiler.step_count() > steps_before,
            "for-loop should increment step counter"
        );
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
            c.symbols.lookup("x", &mut c.heap).unwrap(),
            AstValue::Integer(42)
        );
    }
    #[test]
    fn builtin_length() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        assert_eq!(
            dispatch_builtin("length", &[AstValue::Integer(5)], &heap, &mut prng)
                .unwrap()
                .0,
            AstValue::Integer(5)
        );
    }
    #[test]
    fn builtin_contains_true() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        assert_eq!(
            dispatch_builtin(
                "contains",
                &[AstValue::Integer(1), AstValue::Integer(1)],
                &heap,
                &mut prng
            )
            .unwrap()
            .0,
            AstValue::Boolean(true)
        );
    }
    #[test]
    fn builtin_contains_false() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        assert_eq!(
            dispatch_builtin(
                "contains",
                &[AstValue::Integer(1), AstValue::Integer(2)],
                &heap,
                &mut prng
            )
            .unwrap()
            .0,
            AstValue::Boolean(false)
        );
    }
    #[test]
    fn builtin_append() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        assert_eq!(
            dispatch_builtin(
                "append",
                &[AstValue::Integer(1), AstValue::Integer(42)],
                &heap,
                &mut prng
            )
            .unwrap()
            .0,
            AstValue::Integer(42)
        );
    }
    #[test]
    fn builtin_pop() {
        let heap = VirtualHeap::with_default_config();
        let mut prng = PRNG_DEFAULT_SEED;
        assert_eq!(
            dispatch_builtin("pop", &[AstValue::Integer(99)], &heap, &mut prng)
                .unwrap()
                .0,
            AstValue::Integer(99)
        );
    }
    #[test]
    fn eval_bitwise_and() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "&",
            AstNodeFactory::integer_literal("6"),
            AstNodeFactory::integer_literal("3"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Integer(2));
    }
    #[test]
    fn eval_bitwise_or() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "|",
            AstNodeFactory::integer_literal("4"),
            AstNodeFactory::integer_literal("1"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Integer(5));
    }
    #[test]
    fn eval_bitwise_xor() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "^",
            AstNodeFactory::integer_literal("5"),
            AstNodeFactory::integer_literal("3"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Integer(6));
    }
    #[test]
    fn while_false_never_executes() {
        let mut c = ctx();
        declare_int(&mut c, "x", 0);
        let w = AstNode::internal(
            AstNodeKind::WhileLoop,
            vec![
                AstNodeFactory::boolean_literal(false),
                AstNode::internal(
                    AstNodeKind::WhileLoop,
                    vec![AstNodeFactory::assignment(
                        AstNodeFactory::identifier("x"),
                        AstNodeFactory::integer_literal("99"),
                    )],
                    None,
                ),
            ],
            None,
        );
        execute_while(&mut c, &w).ok();
        assert_eq!(
            c.symbols.lookup("x", &mut c.heap).unwrap(),
            AstValue::Integer(0)
        );
    }
    #[test]
    fn eval_shift_left() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "<<",
            AstNodeFactory::integer_literal("1"),
            AstNodeFactory::integer_literal("3"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Integer(8));
    }
    // Critical tests
    #[test]
    fn eval_infinity_literal() {
        let mut c = ctx();
        let v = evaluate(&mut c, &AstNodeFactory::infinity_literal()).unwrap();
        assert_eq!(v, AstValue::Real(f64::INFINITY));
    }
    #[test]
    fn eval_nan_literal() {
        let mut c = ctx();
        let v = evaluate(&mut c, &AstNodeFactory::nan_literal()).unwrap();
        assert!(matches!(v, AstValue::Real(x) if x.is_nan()));
    }
    #[test]
    fn eval_le_true() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "<=",
            AstNodeFactory::integer_literal("5"),
            AstNodeFactory::integer_literal("5"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Boolean(true));
    }
    #[test]
    fn eval_ge_false() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            ">=",
            AstNodeFactory::integer_literal("3"),
            AstNodeFactory::integer_literal("5"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Boolean(false));
    }
    #[test]
    fn eval_neg_i64_min_traps() {
        let mut c = ctx();
        let n = AstNodeFactory::unary_expression(
            "-",
            AstNodeFactory::integer_literal(&i64::MIN.to_string()),
        );
        assert_eq!(
            evaluate(&mut c, &n).unwrap_err(),
            ExitCode::InvalidOperation
        );
    }
    #[test]
    fn eval_real_div_zero() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "/",
            AstNodeFactory::real_literal(1.0),
            AstNodeFactory::real_literal(0.0),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap_err(), ExitCode::DivisionByZero);
    }
    #[test]
    fn eval_and_with_non_bool() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "and",
            AstNodeFactory::integer_literal("1"),
            AstNodeFactory::integer_literal("0"),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Boolean(false));
    }
    #[test]
    fn eval_bitwise_real_traps() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "&",
            AstNodeFactory::real_literal(1.0),
            AstNodeFactory::integer_literal("2"),
        );
        assert_eq!(
            evaluate(&mut c, &n).unwrap_err(),
            ExitCode::InvalidOperation
        );
    }
    #[test]
    fn exec_var_undeclared_assignment_auto_declares() {
        let mut c = ctx();
        execute_assignment(
            &mut c,
            &AstNodeFactory::assignment(
                AstNodeFactory::identifier("novar"),
                AstNodeFactory::integer_literal("1"),
            ),
        )
        .unwrap();
        assert_eq!(
            c.symbols.lookup("novar", &mut c.heap).unwrap(),
            AstValue::Integer(1)
        );
    }
    #[test]
    fn exec_while_empty_children() {
        let mut c = ctx();
        let n = AstNode::internal(AstNodeKind::WhileLoop, vec![], None);
        let v = execute_while(&mut c, &n).unwrap();
        assert_eq!(v, AstValue::None);
    }
    #[test]
    fn exec_if_false_no_else() {
        let mut c = ctx();
        let n = AstNode::internal(
            AstNodeKind::If,
            vec![AstNodeFactory::boolean_literal(false)],
            None,
        );
        let v = execute_if(&mut c, &n).unwrap();
        assert_eq!(v, AstValue::None);
    }
    #[test]
    fn exec_if_empty_children() {
        let mut c = ctx();
        let n = AstNode::internal(AstNodeKind::If, vec![], None);
        let v = execute_if(&mut c, &n).unwrap();
        assert_eq!(v, AstValue::None);
    }
    #[test]
    fn cmp_bool_vs_bool() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "<",
            AstNodeFactory::boolean_literal(false),
            AstNodeFactory::boolean_literal(true),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Boolean(true));
    }
    #[test]
    fn eval_trapped_state() {
        let mut c = ctx();
        c.trap.set(ExitCode::DivisionByZero);
        assert_eq!(
            evaluate(&mut c, &AstNodeFactory::integer_literal("1")).unwrap_err(),
            ExitCode::DivisionByZero
        );
    }
    #[test]
    fn exec_for_non_integer() {
        let mut c = ctx();
        let f = AstNode::internal(
            AstNodeKind::ForLoop,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String("i".to_string())),
                ),
                AstNodeFactory::real_literal(3.14),
                AstNodeFactory::assignment(
                    AstNodeFactory::identifier("i"),
                    AstNodeFactory::integer_literal("0"),
                ),
            ],
            None,
        );
        execute_for(&mut c, &f).ok();
    }
    #[test]
    fn exec_program_non_program_traps() {
        let mut c = ctx();
        assert_eq!(
            execute_program(&mut c, &AstNodeFactory::integer_literal("1")).unwrap_err(),
            ExitCode::InvalidOperation
        );
    }
    #[test]
    fn exec_assert_empty_children() {
        let n = AstNode::internal(AstNodeKind::Assert, vec![], None);
        assert!(execute_assert(&mut ctx(), &n).is_ok());
    }
    #[test]
    fn exec_invariant_empty_children() {
        let n = AstNode::internal(AstNodeKind::Invariant, vec![], None);
        assert!(execute_invariant(&mut ctx(), &n).is_ok());
    }
    #[test]
    fn eval_or_with_non_bool() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            "or",
            AstNodeFactory::string_literal(""),
            AstNodeFactory::boolean_literal(true),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Boolean(true));
    }
    #[test]
    fn complexity_n3_path() {
        let mut c = ctx();
        let algo = AstNodeFactory::algorithm("T", vec![], None, "O(N^3)", vec![], vec![]);
        assert!(execute_algorithm(&mut c, &algo).is_ok());
    }

    #[test]
    fn constant_time_secret_branch_equal_steps_ok() {
        let mut c = ctx();
        c.constant_time_mode = true;
        c.secret_variables.insert("secret_key".to_string());
        declare_int(&mut c, "secret_key", 1);
        declare_int(&mut c, "result", 0);
        let if_node = AstNode::internal(
            AstNodeKind::If,
            vec![
                AstNodeFactory::binary_expression(
                    ">",
                    AstNodeFactory::identifier("secret_key"),
                    AstNodeFactory::integer_literal("0"),
                ),
                AstNode::internal(
                    AstNodeKind::If,
                    vec![AstNodeFactory::assignment(
                        AstNodeFactory::identifier("result"),
                        AstNodeFactory::integer_literal("42"),
                    )],
                    None,
                ),
                AstNode::internal(
                    AstNodeKind::If,
                    vec![AstNodeFactory::assignment(
                        AstNodeFactory::identifier("result"),
                        AstNodeFactory::integer_literal("99"),
                    )],
                    None,
                ),
            ],
            None,
        );
        let result = execute_if(&mut c, &if_node);
        assert!(result.is_ok(), "Expected ok, got {:?}", result.err());
        assert_eq!(c.trap.code(), ExitCode::NoError);
    }

    #[test]
    fn constant_time_secret_branch_divergent_traps() {
        let mut c = ctx();
        c.constant_time_mode = true;
        c.secret_variables.insert("secret_key".to_string());
        declare_int(&mut c, "secret_key", 1);
        let if_node = AstNode::internal(
            AstNodeKind::If,
            vec![
                AstNodeFactory::binary_expression(
                    ">",
                    AstNodeFactory::identifier("secret_key"),
                    AstNodeFactory::integer_literal("0"),
                ),
                AstNode::internal(
                    AstNodeKind::If,
                    vec![AstNodeFactory::assignment(
                        AstNodeFactory::identifier("x"),
                        AstNodeFactory::integer_literal("1"),
                    )],
                    None,
                ),
                AstNode::internal(
                    AstNodeKind::If,
                    vec![
                        AstNodeFactory::assignment(
                            AstNodeFactory::identifier("x"),
                            AstNodeFactory::integer_literal("2"),
                        ),
                        AstNodeFactory::assignment(
                            AstNodeFactory::identifier("x"),
                            AstNodeFactory::integer_literal("3"),
                        ),
                    ],
                    None,
                ),
            ],
            None,
        );
        let result = execute_if(&mut c, &if_node);
        assert_eq!(result.unwrap_err(), ExitCode::TimingLeak);
        assert!(c.trap.is_trapped());
    }
    #[test]
    fn eval_random_returns_in_range() {
        let mut c = ctx();
        let call = AstNodeFactory::function_call(
            "random",
            vec![
                AstNodeFactory::integer_literal("1"),
                AstNodeFactory::integer_literal("10"),
            ],
        );
        for _ in 0..100 {
            match evaluate(&mut c, &call) {
                Ok(AstValue::Integer(v)) => {
                    assert!(v >= 1 && v <= 10, "random value {} out of [1,10]", v);
                }
                other => panic!("unexpected random result: {:?}", other),
            }
        }
    }
    #[test]
    fn eval_random_is_deterministic_with_seed() {
        let config = ProfilingConfig {
            prng_seed: 42,
            ..ProfilingConfig::default()
        };
        let mut c1 = ExecContext::new(config.clone());
        let mut c2 = ExecContext::new(config);
        let call = AstNodeFactory::function_call(
            "random",
            vec![
                AstNodeFactory::integer_literal("0"),
                AstNodeFactory::integer_literal("100"),
            ],
        );
        for _ in 0..20 {
            let v1 = evaluate(&mut c1, &call).unwrap();
            let v2 = evaluate(&mut c2, &call).unwrap();
            assert_eq!(v1, v2, "same seed should produce same sequence");
        }
    }
    #[test]
    fn prng_state_updates() {
        let mut c = ctx();
        let s0 = c.prng_state;
        c.rand_int(0, 1000);
        assert_ne!(c.prng_state, s0, "PRNG state should advance");
        let s1 = c.prng_state;
        c.rand_int(-5, 5);
        assert_ne!(c.prng_state, s1, "PRNG state should advance again");
    }
    #[test]
    fn eval_random_single_value_range() {
        let mut c = ctx();
        let call = AstNodeFactory::function_call(
            "random",
            vec![
                AstNodeFactory::integer_literal("7"),
                AstNodeFactory::integer_literal("7"),
            ],
        );
        for _ in 0..10 {
            assert_eq!(evaluate(&mut c, &call).unwrap(), AstValue::Integer(7));
        }
    }
    #[test]
    fn expected_complexity_parsed_from_binding() {
        let _c = ctx();
        // Construct an algorithm AST manually with Expected binding
        let expected_binding = AstNode::internal(
            AstNodeKind::VariableBinding,
            vec![
                AstNode::leaf(
                    AstNodeKind::Identifier,
                    Some(AstValue::String("Expected".to_string())),
                ),
                AstNode::leaf(
                    AstNodeKind::StringLiteral,
                    Some(AstValue::String("O(N log N)".to_string())),
                ),
            ],
            None,
        );
        let algo = AstNodeFactory::algorithm(
            "ExpectedTest",
            vec![],
            None,
            "O(N^2)",
            vec![expected_binding],
            vec![],
        );
        // The algorithm has "Complexity: \"O(N^2)\", Expected=\"O(N log N)\""
        // We can verify complexity_str and binding are present in the AST
        let complexity_val = algo.children.get(1).and_then(|n| n.value.clone());
        assert!(
            complexity_val.is_some(),
            "complexity string should be present"
        );
        let has_expected = algo.children.iter().any(|child| {
            child.kind == AstNodeKind::VariableBinding
                && child.children.first().and_then(|n| n.value.clone())
                    == Some(AstValue::String("Expected".to_string()))
        });
        assert!(has_expected, "Expected binding should be present");
    }

    #[test]
    fn exec_yield_returns_expression_value() {
        let mut c = ctx();
        let y = AstNode::internal(
            AstNodeKind::Yield,
            vec![AstNodeFactory::integer_literal("42")],
            None,
        );
        let result = exec_yield(&mut c, &y).unwrap();
        assert_eq!(result, AstValue::Integer(42));
    }

    #[test]
    fn exec_await_not_found_returns_declared() {
        let mut c = ctx();
        declare_int(&mut c, "x", 99);
        let a = AstNode::internal(
            AstNodeKind::Await,
            vec![AstNodeFactory::identifier("x")],
            None,
        );
        let result = exec_await(&mut c, &a).unwrap();
        assert_eq!(result, AstValue::Integer(99));
    }

    #[test]
    fn secret_variable_tracks_across_scopes() {
        let mut c = ctx();
        c.constant_time_mode = true;
        let secret_i = AstNodeFactory::type_node("Integer", vec![]);
        let secret_t = AstNode::internal(AstNodeKind::SecretType, vec![secret_i], None);
        let decl = AstNode::internal(
            AstNodeKind::VariableDeclaration,
            vec![
                AstNodeFactory::identifier("key"),
                secret_t.clone(),
                AstNodeFactory::integer_literal("0"),
            ],
            None,
        );
        execute_var_decl(&mut c, &decl).unwrap();
        assert!(c.secret_variables.contains("key"));
    }

    #[test]
    fn condition_uses_secret_finds_nested() {
        let mut c = ctx();
        c.constant_time_mode = true;
        c.secret_variables.insert("key".to_string());
        let cond = AstNodeFactory::binary_expression(
            "==",
            AstNodeFactory::identifier("key"),
            AstNodeFactory::integer_literal("1"),
        );
        assert!(condition_uses_secret(&c, &cond));
    }

    #[test]
    fn condition_uses_secret_returns_false_for_non_secret() {
        let mut c = ctx();
        c.constant_time_mode = true;
        let cond = AstNodeFactory::binary_expression(
            "==",
            AstNodeFactory::identifier("x"),
            AstNodeFactory::integer_literal("1"),
        );
        assert!(!condition_uses_secret(&c, &cond));
    }

    #[test]
    fn random_integer_only_range_produces_same_value() {
        let mut c1 = ctx();
        let mut c2 = ctx();
        c1.prng_state = 42;
        c2.prng_state = 42;
        for _ in 0..10 {
            assert_eq!(c1.rand_int(7, 7), 7);
            assert_eq!(c2.rand_int(7, 7), 7);
        }
    }

    #[test]
    fn random_different_seeds_produce_different() {
        let mut c1 = ctx();
        let mut c2 = ctx();
        c1.prng_state = 1;
        c2.prng_state = 2;
        let v1 = c1.rand_int(0, 1000);
        let v2 = c2.rand_int(0, 1000);
        assert_ne!(v1, v2);
    }

    #[test]
    fn eval_binary_gte_real_literals() {
        let mut c = ctx();
        let n = AstNodeFactory::binary_expression(
            ">=",
            AstNodeFactory::real_literal(0.0),
            AstNodeFactory::real_literal(0.0),
        );
        assert_eq!(evaluate(&mut c, &n).unwrap(), AstValue::Boolean(true));
    }

    #[test]
    fn exec_const_decl_immutable() {
        let mut c = ctx();
        let typ = AstNodeFactory::type_node("Integer", vec![]);
        let d = AstNode::internal(
            AstNodeKind::ConstDeclaration,
            vec![
                AstNodeFactory::identifier("C"),
                typ,
                AstNodeFactory::integer_literal("42"),
            ],
            None,
        );
        execute_const_decl(&mut c, &d).unwrap();
        assert_eq!(
            c.symbols.lookup("C", &mut c.heap).unwrap(),
            AstValue::Integer(42)
        );
    }

    #[test]
    fn exec_while_with_complex_cond() {
        let mut c = ctx();
        declare_int(&mut c, "x", 5);
        let w = AstNode::internal(
            AstNodeKind::WhileLoop,
            vec![
                AstNodeFactory::binary_expression(
                    ">",
                    AstNodeFactory::identifier("x"),
                    AstNodeFactory::integer_literal("0"),
                ),
                AstNode::internal(
                    AstNodeKind::WhileLoop,
                    vec![AstNodeFactory::assignment(
                        AstNodeFactory::identifier("x"),
                        AstNodeFactory::binary_expression(
                            "-",
                            AstNodeFactory::identifier("x"),
                            AstNodeFactory::integer_literal("1"),
                        ),
                    )],
                    None,
                ),
            ],
            None,
        );
        execute_while(&mut c, &w).ok();
        assert_eq!(
            c.symbols.lookup("x", &mut c.heap).unwrap(),
            AstValue::Integer(0)
        );
    }

    #[test]
    fn exec_for_zero_iterations() {
        let mut c = ctx();
        let f = AstNode::internal(
            AstNodeKind::ForLoop,
            vec![
                AstNodeFactory::identifier("i"),
                AstNodeFactory::integer_literal("0"),
            ],
            None,
        );
        let result = execute_for(&mut c, &f).unwrap();
        assert_eq!(result, AstValue::None);
    }

    #[test]
    fn for_loop_inner_assignment_visible_after() {
        let mut c = ctx();
        declare_int(&mut c, "sum", 0);
        let f = AstNode::internal(
            AstNodeKind::ForLoop,
            vec![
                AstNodeFactory::identifier("i"),
                AstNodeFactory::integer_literal("3"),
                AstNodeFactory::assignment(
                    AstNodeFactory::identifier("sum"),
                    AstNodeFactory::binary_expression(
                        "+",
                        AstNodeFactory::identifier("sum"),
                        AstNodeFactory::identifier("i"),
                    ),
                ),
            ],
            None,
        );
        execute_for(&mut c, &f).ok();
        let v = c.symbols.lookup("sum", &mut c.heap).unwrap();
        assert_eq!(v, AstValue::Integer(3));
    }

    #[test]
    fn spawn_sync_records_work_span() {
        let mut c = ctx();
        let s = AstNode::internal(
            AstNodeKind::Spawn,
            vec![
                AstNodeFactory::identifier("t"),
                AstNodeFactory::integer_literal("42"),
            ],
            None,
        );
        exec_spawn(&mut c, &s).unwrap();
        assert!(c.profiler.work() > 0);
        assert!(c.profiler.span() > 0);
    }

    #[test]
    fn sync_increments_work() {
        let mut c = ctx();
        let w_before = c.profiler.work();
        exec_sync(&mut c, &AstNode::internal(AstNodeKind::Sync, vec![], None)).unwrap();
        assert!(c.profiler.work() > w_before);
    }

    #[test]
    fn parallel_for_increments_work() {
        let mut c = ctx();
        let body = AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("1")));
        let pf = AstNode::internal(
            AstNodeKind::ParallelFor,
            vec![
                AstNodeFactory::identifier("i"),
                AstNodeFactory::integer_literal("4"),
                body,
            ],
            None,
        );
        exec_parallel_for(&mut c, &pf).unwrap();
        assert!(c.profiler.work() > 0);
        assert!(c.profiler.span() > 0);
    }
}
