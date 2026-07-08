//! Abstract interpreter for the UEAS kernel.
//!
//! Walks the canonical AST, evaluates expressions against the virtual heap,
//! maintains a stacked symbol table, increments the step counter, and
//! enforces invariants and complexity contracts.
//!
//! # Architecture
//! - `ExecContext` — holds the heap, symbol table, profiler, and trap register
//! - `evaluate(node)` — recursively evaluates an AST node to an `AstValue`
//! - `execute(program)` — executes a full program node

use crate::ast::{AstNode, AstNodeKind, AstValue};
use crate::heap::VirtualHeap;
use crate::profiling::{Profiler, ProfilingConfig};
use crate::traps::{ExitCode, TrapRegister};
use std::collections::HashMap;

/// A value stored in the symbol table.
#[derive(Debug, Clone)]
pub enum SymbolValue {
    Value(AstValue),
    HeapHandle(crate::heap::HeapHandle),
}

/// A stacked lexical scope.
#[derive(Debug, Clone, Default)]
struct Scope {
    symbols: HashMap<String, SymbolValue>,
}

/// Stack of lexical scopes for variable resolution.
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

    /// Push a new scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// Pop the topmost scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Declare a variable in the current (innermost) scope.
    pub fn declare(&mut self, name: &str, value: SymbolValue) -> Result<(), ExitCode> {
        let scope = self.scopes.last_mut().ok_or(ExitCode::StackOverflow)?;
        scope.symbols.insert(name.to_string(), value);
        Ok(())
    }

    /// Look up a variable through all scopes, innermost first.
    pub fn lookup(&self, name: &str) -> Option<&SymbolValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.symbols.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Mutably look up a variable for assignment.
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolValue> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.symbols.contains_key(name) {
                return scope.symbols.get_mut(name);
            }
        }
        None
    }
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
///
/// Recursively walks the AST. Increments the step counter per operation.
/// Returns `Err(ExitCode)` if a trap condition is detected.
pub fn evaluate(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if ctx.trap.is_trapped() {
        return Err(ctx.trap.code());
    }

    ctx.profiler.step()?;

    match node.kind {
        AstNodeKind::IntegerLiteral => {
            if let Some(AstValue::Integer(s)) = &node.value {
                Ok(AstValue::Integer(s.clone()))
            } else if let Some(AstValue::String(s)) = &node.value {
                Ok(AstValue::Integer(s.clone()))
            } else {
                Err(ExitCode::HeapExhaustion)
            }
        }
        AstNodeKind::RealLiteral => {
            if let Some(AstValue::Real(_)) = node.value {
                Ok(node.value.clone().unwrap())
            } else {
                Err(ExitCode::HeapExhaustion)
            }
        }
        AstNodeKind::StringLiteral => {
            if let Some(AstValue::String(_)) = node.value {
                Ok(node.value.clone().unwrap())
            } else {
                Err(ExitCode::HeapExhaustion)
            }
        }
        AstNodeKind::BooleanLiteral => {
            if let Some(AstValue::Boolean(_)) = node.value {
                Ok(node.value.clone().unwrap())
            } else {
                Err(ExitCode::HeapExhaustion)
            }
        }
        AstNodeKind::NoneLiteral => Ok(AstValue::None),
        AstNodeKind::Identifier => {
            if let Some(AstValue::String(ref name)) = node.value {
                match ctx.symbols.lookup(name) {
                    Some(SymbolValue::Value(v)) => Ok(v.clone()),
                    Some(SymbolValue::HeapHandle(_)) => Err(ExitCode::NullDereference),
                    None => Err(ExitCode::NullDereference),
                }
            } else {
                Err(ExitCode::HeapExhaustion)
            }
        }
        AstNodeKind::BinaryExpression => eval_binary(ctx, node),
        AstNodeKind::UnaryExpression => eval_unary(ctx, node),
        AstNodeKind::FunctionCall => eval_function_call(ctx, node),
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Evaluate a binary expression node.
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
        // Arithmetic
        "+" => binary_arithmetic(&left, &right, |a, b| a + b, |a, b| a + b),
        "-" => binary_arithmetic(&left, &right, |a, b| a - b, |a, b| a - b),
        "*" => binary_arithmetic(&left, &right, |a, b| a * b, |a, b| a * b),
        "/" => match (&left, &right) {
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
        },
        "mod" => match (&left, &right) {
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
        },
        // Comparison
        "==" => Ok(AstValue::Boolean(left == right)),
        "!=" => Ok(AstValue::Boolean(left != right)),
        "<" => binary_cmp(&left, &right, |a, b| a < b),
        "<=" => binary_cmp(&left, &right, |a, b| a <= b),
        ">" => binary_cmp(&left, &right, |a, b| a > b),
        ">=" => binary_cmp(&left, &right, |a, b| a >= b),
        // Logical
        "and" => {
            let l = is_truthy(&left);
            if !l {
                return Ok(AstValue::Boolean(false));
            }
            Ok(AstValue::Boolean(is_truthy(&right)))
        }
        "or" => {
            let l = is_truthy(&left);
            if l {
                return Ok(AstValue::Boolean(true));
            }
            Ok(AstValue::Boolean(is_truthy(&right)))
        }
        _ => Err(ExitCode::HeapExhaustion),
    }
}

/// Evaluate a unary expression node.
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
            AstValue::Integer(s) => {
                let x: i64 = s.parse().map_err(|_| ExitCode::HeapExhaustion)?;
                Ok(AstValue::Integer((-x).to_string()))
            }
            AstValue::String(s) => {
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

/// Evaluate a function call.
fn eval_function_call(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    let name = match node.children.first().and_then(|n| n.value.as_ref()) {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };

    let mut args = Vec::new();
    for child in node.children.iter().skip(1) {
        args.push(evaluate(ctx, child)?);
    }

    // Step cost: function call itself
    ctx.profiler.step()?;

    match name.as_str() {
        "sqrt" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            match &args[0] {
                AstValue::String(s) => {
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
        "length" => {
            // length of a collection — approximated here
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            let count = count_collection_items(&args[0]);
            Ok(AstValue::String(count.to_string()))
        }
        "cardinality" => {
            if args.is_empty() {
                return Err(ExitCode::HeapExhaustion);
            }
            let count = count_collection_items(&args[0]);
            Ok(AstValue::String(count.to_string()))
        }
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

/// Determine if a value is truthy.
fn is_truthy(value: &AstValue) -> bool {
    match value {
        AstValue::Boolean(b) => *b,
        AstValue::String(s) => s != "0" && !s.is_empty(),
        AstValue::Real(x) => *x != 0.0,
        AstValue::Integer(s) => s != "0",
        AstValue::None => false,
    }
}

/// Count items in a collection for length/cardinality builtins.
fn count_collection_items(_value: &AstValue) -> usize {
    // Simplified: return 10 as placeholder for collection size
    10
}

/// Execute a full program AST.
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

/// Execute an algorithm node.
fn execute_algorithm(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    ctx.symbols.push_scope();
    let mut last_value = AstValue::None;

    // Process body statements
    for (i, child) in node.children.iter().enumerate() {
        // Skip metadata nodes (name identifier, complexity string)
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
                // Expression statement
                if let Ok(v) = evaluate(ctx, child) {
                    last_value = v;
                }
            }
        }
    }
    ctx.symbols.pop_scope();

    // Enforce complexity contract
    enforce_complexity(ctx, node)?;

    Ok(last_value)
}

/// Parse complexity string from algorithm node and verify the step count.
fn enforce_complexity(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    use crate::profiling::ComplexityContract;
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
    ctx.symbols.declare(&name, SymbolValue::Value(value))?;
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

    match ctx.symbols.lookup_mut(&target_name) {
        Some(slot) => {
            *slot = SymbolValue::Value(value);
            Ok(())
        }
        None => Err(ExitCode::NullDereference),
    }
}

fn execute_if(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }
    let condition = evaluate(ctx, &node.children[0])?;
    if is_truthy(&condition) {
        // Consequent
        if node.children.len() > 1 {
            return execute_algorithm_body(ctx, &node.children[1]);
        }
    } else if node.children.len() > 2 {
        // Alternate
        return execute_algorithm_body(ctx, &node.children[2]);
    }
    Ok(AstValue::None)
}

fn execute_while(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.is_empty() {
        return Ok(AstValue::None);
    }
    let mut last_value = AstValue::None;
    while is_truthy(&evaluate(ctx, &node.children[0])?) {
        ctx.profiler.step()?;
        if node.children.len() > 1 {
            last_value = execute_algorithm_body(ctx, &node.children[1])?;
        }
    }
    Ok(last_value)
}

fn execute_for(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
    if node.children.len() < 2 {
        return Ok(AstValue::None);
    }
    let iterator_name = match &node.children[0].value {
        Some(AstValue::String(s)) => s.clone(),
        _ => return Err(ExitCode::HeapExhaustion),
    };
    // Evaluate the collection expression (simplified — assume it yields a count)
    let count = evaluate(ctx, &node.children[1])?;
    ctx.symbols.push_scope();
    let mut last_value = AstValue::None;

    // Simulate iterating `count` times
    let n: usize = match &count {
        AstValue::String(s) => s.parse().unwrap_or(1),
        _ => 1,
    };
    for i in 0..n {
        ctx.symbols.declare(
            &iterator_name,
            SymbolValue::Value(AstValue::Integer(i.to_string())),
        )?;
        for body_node in node.children.iter().skip(2) {
            last_value = execute_algorithm_body(ctx, body_node)?;
        }
    }
    ctx.symbols.pop_scope();
    Ok(last_value)
}

fn execute_assert(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.is_empty() {
        return Ok(());
    }
    let condition = evaluate(ctx, &node.children[0])?;
    if !is_truthy(&condition) {
        ctx.trap.set(ExitCode::AssertionFailure);
        return Err(ExitCode::AssertionFailure);
    }
    Ok(())
}

fn execute_invariant(ctx: &mut ExecContext, node: &AstNode) -> Result<(), ExitCode> {
    if node.children.is_empty() {
        return Ok(());
    }
    let condition = evaluate(ctx, &node.children[0])?;
    if !is_truthy(&condition) {
        ctx.trap.set(ExitCode::InvariantViolation);
        return Err(ExitCode::InvariantViolation);
    }
    Ok(())
}

fn execute_algorithm_body(ctx: &mut ExecContext, node: &AstNode) -> Result<AstValue, ExitCode> {
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
    use crate::profiling::ProfilingConfig;

    fn make_ctx() -> ExecContext {
        ExecContext::with_default_config()
    }

    fn declare_var(ctx: &mut ExecContext, name: &str, value: AstValue) {
        ctx.symbols
            .declare(name, SymbolValue::Value(value))
            .unwrap();
    }

    // ===== Expression Evaluation Tests =====

    #[test]
    fn eval_integer_literal() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::integer_literal("42");
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("42".to_string()));
    }

    #[test]
    fn eval_addition_integers() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "+",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("5"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("15".to_string()));
    }

    #[test]
    fn eval_subtraction_integers() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "-",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("3"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("7".to_string()));
    }

    #[test]
    fn eval_multiplication_integers() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "*",
            AstNodeFactory::integer_literal("6"),
            AstNodeFactory::integer_literal("7"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("42".to_string()));
    }

    #[test]
    fn eval_division_integers() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "/",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("3"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("3".to_string()));
    }

    #[test]
    fn eval_division_by_zero_traps() {
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
    fn eval_modulo_by_zero_traps() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "mod",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("0"),
        );
        assert_eq!(
            evaluate(&mut ctx, &node).unwrap_err(),
            ExitCode::DivisionByZero
        );
    }

    #[test]
    fn eval_equality_true() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "==",
            AstNodeFactory::integer_literal("5"),
            AstNodeFactory::integer_literal("5"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(true));
    }

    #[test]
    fn eval_equality_false() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "==",
            AstNodeFactory::integer_literal("5"),
            AstNodeFactory::integer_literal("3"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(false));
    }

    #[test]
    fn eval_not_equal_true() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "!=",
            AstNodeFactory::integer_literal("5"),
            AstNodeFactory::integer_literal("3"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(true));
    }

    #[test]
    fn eval_less_than() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "<",
            AstNodeFactory::integer_literal("3"),
            AstNodeFactory::integer_literal("5"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(true));
    }

    #[test]
    fn eval_greater_than() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            ">",
            AstNodeFactory::integer_literal("10"),
            AstNodeFactory::integer_literal("5"),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(true));
    }

    #[test]
    fn eval_logical_and() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "and",
            AstNodeFactory::boolean_literal(true),
            AstNodeFactory::boolean_literal(true),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(true));
    }

    #[test]
    fn eval_logical_and_short_circuit() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "and",
            AstNodeFactory::boolean_literal(false),
            AstNodeFactory::boolean_literal(true),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(false));
    }

    #[test]
    fn eval_logical_or() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::binary_expression(
            "or",
            AstNodeFactory::boolean_literal(false),
            AstNodeFactory::boolean_literal(true),
        );
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(true));
    }

    #[test]
    fn eval_unary_negation() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::unary_expression("-", AstNodeFactory::integer_literal("42"));
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("-42".to_string()));
    }

    #[test]
    fn eval_unary_not_true() {
        let mut ctx = make_ctx();
        let node = AstNodeFactory::unary_expression("not", AstNodeFactory::boolean_literal(true));
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Boolean(false));
    }

    #[test]
    fn eval_nested_expression() {
        let mut ctx = make_ctx();
        let inner = AstNodeFactory::binary_expression(
            "*",
            AstNodeFactory::integer_literal("3"),
            AstNodeFactory::integer_literal("4"),
        );
        let outer =
            AstNodeFactory::binary_expression("+", inner, AstNodeFactory::integer_literal("2"));
        let result = evaluate(&mut ctx, &outer).unwrap();
        assert_eq!(result, AstValue::Integer("14".to_string()));
    }

    #[test]
    fn eval_identifier_lookup() {
        let mut ctx = make_ctx();
        declare_var(&mut ctx, "x", AstValue::Integer("42".to_string()));
        let node = AstNodeFactory::identifier("x");
        let result = evaluate(&mut ctx, &node).unwrap();
        assert_eq!(result, AstValue::Integer("42".to_string()));
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

    // ===== Statement Execution Tests =====

    #[test]
    fn exec_variable_declaration() {
        let mut ctx = make_ctx();
        let typ = AstNodeFactory::type_node("Integer", vec![]);
        let init = AstNodeFactory::integer_literal("100");
        let decl = AstNodeFactory::variable_declaration("count", typ, Some(init));
        execute_var_decl(&mut ctx, &decl).unwrap();
        assert!(ctx.symbols.lookup("count").is_some());
    }

    #[test]
    fn exec_assignment() {
        let mut ctx = make_ctx();
        declare_var(&mut ctx, "x", AstValue::Integer("0".to_string()));
        let target = AstNodeFactory::identifier("x");
        let value = AstNodeFactory::integer_literal("42");
        let assign = AstNodeFactory::assignment(target, value);
        execute_assignment(&mut ctx, &assign).unwrap();
        if let Some(SymbolValue::Value(AstValue::Integer(s))) = ctx.symbols.lookup("x") {
            assert_eq!(s, "42");
        } else {
            panic!("x not found");
        }
    }

    #[test]
    fn exec_assertion_pass() {
        let mut ctx = make_ctx();
        let condition = AstNodeFactory::boolean_literal(true);
        let assert_node = AstNodeFactory::assert_stmt(condition, None);
        execute_assert(&mut ctx, &assert_node).unwrap();
        assert!(ctx.trap.is_ok());
    }

    #[test]
    fn exec_assertion_fail() {
        let mut ctx = make_ctx();
        let condition = AstNodeFactory::boolean_literal(false);
        let assert_node = AstNodeFactory::assert_stmt(condition, None);
        assert_eq!(
            execute_assert(&mut ctx, &assert_node).unwrap_err(),
            ExitCode::AssertionFailure
        );
    }

    #[test]
    fn exec_invariant_pass() {
        let mut ctx = make_ctx();
        let condition = AstNodeFactory::boolean_literal(true);
        let inv_node = AstNodeFactory::invariant_stmt(condition, None);
        execute_invariant(&mut ctx, &inv_node).unwrap();
    }

    #[test]
    fn exec_invariant_fail() {
        let mut ctx = make_ctx();
        let condition = AstNodeFactory::boolean_literal(false);
        let inv_node = AstNodeFactory::invariant_stmt(condition, None);
        assert_eq!(
            execute_invariant(&mut ctx, &inv_node).unwrap_err(),
            ExitCode::InvariantViolation
        );
    }

    #[test]
    fn step_counter_increments_on_eval() {
        let mut ctx = make_ctx();
        let before = ctx.profiler.step_count();
        let node = AstNodeFactory::integer_literal("1");
        evaluate(&mut ctx, &node).unwrap();
        assert!(ctx.profiler.step_count() > before);
    }
}
