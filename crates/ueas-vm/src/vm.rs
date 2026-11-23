//! Resumable, deterministic, cost-metered UEAS bytecode VM (`VmSession`).

use crate::builtins::{call_builtin, call_method, cmp_values};
use crate::cache::{CacheConfig, CacheHierarchy, CacheStats};
use crate::chunk::{CompiledProgram, Constant};
use crate::heap::{HeapObject, HeapStats, VirtualHeap};
use crate::op::Op;
use crate::prng::DeterministicPrng;
use crate::profiler::{WorkSpanProfiler, WorkSpanReport};
use crate::trace::TraceEvent;
use crate::trap::{TrapCode, TrapError};
use crate::value::{CanonicalF64, RuntimeValue};
use indexmap::{IndexMap, IndexSet};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use ueas_syntax::SourceSpan;

/// Configuration for a `VmSession`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub step_budget: u64,
    pub heap_budget_bytes: usize,
    pub seed: u64,
    pub cache_config: CacheConfig,
    pub max_snapshots: usize,
    pub record_trace: bool,
    pub enforce_contracts: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            step_budget: 2_000_000,
            heap_budget_bytes: 64 * 1024 * 1024,
            seed: 0x5545_4153_2026_0001,
            cache_config: CacheConfig::default(),
            max_snapshots: 0,
            record_trace: false,
            enforce_contracts: true,
        }
    }
}

/// Active call frame on the VM call stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    pub chunk_idx: usize,
    pub pc: usize,
    pub locals: Vec<RuntimeValue>,
    pub old_locals: Vec<RuntimeValue>,
    pub stack_base: usize,
    pub mut_writebacks: Vec<(u16, u16)>,
}

/// Outcome of executing a single VM step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepOutcome {
    Continue,
    BreakpointHit { line: u32, pc: usize },
    Halted(RuntimeValue),
}

/// Full execution result returned when a session finishes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub value: RuntimeValue,
    pub steps: u64,
    pub heap: HeapStats,
    pub cache: CacheStats,
    pub work_span: WorkSpanReport,
    pub trace: Vec<TraceEvent>,
}

#[derive(Debug, Clone)]
struct VmSnapshot {
    frames: Vec<CallFrame>,
    stack: Vec<RuntimeValue>,
    heap: VirtualHeap,
    cache: CacheHierarchy,
    profiler: WorkSpanProfiler,
    prng: DeterministicPrng,
    steps: u64,
}

/// Resumable, step-driven deterministic VM session.
pub struct VmSession {
    program: CompiledProgram,
    config: VmConfig,
    frames: Vec<CallFrame>,
    stack: Vec<RuntimeValue>,
    heap: VirtualHeap,
    cache: CacheHierarchy,
    profiler: WorkSpanProfiler,
    prng: DeterministicPrng,
    steps: u64,
    trace: Vec<TraceEvent>,
    snapshots: VecDeque<VmSnapshot>,
    breakpoints: HashSet<(String, u32)>,
    halted_value: Option<RuntimeValue>,
}

impl VmSession {
    /// Initialize a `VmSession` at `entry` with `args`.
    pub fn new(
        program: CompiledProgram,
        entry: &str,
        args: Vec<RuntimeValue>,
        config: VmConfig,
    ) -> Result<Self, TrapError> {
        let &chunk_idx = program.chunk_index.get(entry).ok_or_else(|| {
            TrapError::new(
                TrapCode::UndefinedSymbol,
                format!("entry algorithm '{entry}' not found"),
                0,
                0,
                None,
            )
        })?;
        let chunk = &program.chunks[chunk_idx];
        if args.len() != chunk.params.len() {
            return Err(TrapError::new(
                TrapCode::ArityMismatch,
                format!(
                    "algorithm '{entry}' expects {} args, got {}",
                    chunk.params.len(),
                    args.len()
                ),
                0,
                0,
                None,
            ));
        }

        let mut heap = VirtualHeap::new(config.heap_budget_bytes);
        let mut locals = vec![RuntimeValue::Unit; chunk.local_names.len()];
        let mut old_locals = if chunk.uses_old {
            vec![RuntimeValue::Unit; chunk.local_names.len()]
        } else {
            Vec::new()
        };
        for (i, arg) in args.into_iter().enumerate() {
            if chunk.uses_old {
                heap.retain_value(&arg);
                old_locals[i] = arg.clone();
            }
            locals[i] = arg;
        }

        let frame = CallFrame {
            chunk_idx,
            pc: if config.enforce_contracts {
                0
            } else {
                chunk.body_start_pc
            },
            locals,
            old_locals,
            stack_base: 0,
            mut_writebacks: Vec::new(),
        };

        Ok(Self {
            program,
            cache: CacheHierarchy::new(config.cache_config),
            prng: DeterministicPrng::new(config.seed),
            profiler: WorkSpanProfiler::new(),
            heap,
            config,
            frames: vec![frame],
            stack: Vec::new(),
            steps: 0,
            trace: Vec::new(),
            snapshots: VecDeque::new(),
            breakpoints: HashSet::new(),
            halted_value: None,
        })
    }

    /// Access the `VirtualHeap` (e.g. to allocate inputs before starting).
    pub fn heap_mut(&mut self) -> &mut VirtualHeap {
        &mut self.heap
    }

    /// Borrow the `VirtualHeap` immutably.
    pub fn heap(&self) -> &VirtualHeap {
        &self.heap
    }

    /// Replace an argument in the initial frame (e.g. after allocating a heap collection).
    pub fn set_arg(&mut self, idx: usize, val: RuntimeValue) {
        if let Some(frame) = self.frames.first_mut() {
            if idx < frame.locals.len() {
                frame.locals[idx] = val.clone();
            }
            if idx < frame.old_locals.len() {
                frame.old_locals[idx] = val;
            }
        }
    }

    /// Configure source line breakpoints `(chunk_name, 1-based line)`.
    pub fn set_breakpoints(&mut self, bps: HashSet<(String, u32)>) {
        self.breakpoints = bps;
    }

    /// Total VM steps executed so far.
    pub fn steps(&self) -> u64 {
        self.steps
    }

    /// Current top call frame, if any.
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    /// All active call frames (for DAP stackTrace).
    pub fn frames(&self) -> &[CallFrame] {
        &self.frames
    }

    /// Compiled program reference (for DAP local name lookup).
    pub fn program(&self) -> &CompiledProgram {
        &self.program
    }

    /// Step backward one instruction using the snapshot ring buffer (DAP `stepBack`).
    pub fn step_back(&mut self) -> bool {
        if let Some(snap) = self.snapshots.pop_back() {
            self.frames = snap.frames;
            self.stack = snap.stack;
            self.heap = snap.heap;
            self.cache = snap.cache;
            self.profiler = snap.profiler;
            self.prng = snap.prng;
            self.steps = snap.steps;
            self.halted_value = None;
            true
        } else {
            false
        }
    }

    /// Run until halt or trap.
    pub fn run(&mut self) -> Result<ExecutionResult, TrapError> {
        loop {
            match self.step()? {
                StepOutcome::Continue | StepOutcome::BreakpointHit { .. } => {}
                StepOutcome::Halted(val) => {
                    return Ok(ExecutionResult {
                        value: val,
                        steps: self.steps,
                        heap: self.heap.stats(),
                        cache: self.cache.stats(),
                        work_span: self.profiler.report(),
                        trace: self.trace.clone(),
                    });
                }
            }
        }
    }

    /// Execute a single bytecode instruction.
    pub fn step(&mut self) -> Result<StepOutcome, TrapError> {
        if let Some(val) = &self.halted_value {
            return Ok(StepOutcome::Halted(val.clone()));
        }

        if self.config.max_snapshots > 0 {
            if self.snapshots.len() >= self.config.max_snapshots {
                self.snapshots.pop_front();
            }
            self.snapshots.push_back(VmSnapshot {
                frames: self.frames.clone(),
                stack: self.stack.clone(),
                heap: self.heap.clone(),
                cache: self.cache.clone(),
                profiler: self.profiler.clone(),
                prng: self.prng.clone(),
                steps: self.steps,
            });
        }

        let (chunk_idx, pc) = match self.frames.last() {
            Some(f) => (f.chunk_idx, f.pc),
            None => {
                let val = self.stack.pop().unwrap_or(RuntimeValue::Unit);
                self.halted_value = Some(val.clone());
                return Ok(StepOutcome::Halted(val));
            }
        };

        let chunk = &self.program.chunks[chunk_idx];
        let op = match chunk.ops.get(pc) {
            Some(op) => op.clone(),
            None => {
                let val = self.stack.pop().unwrap_or(RuntimeValue::Unit);
                self.halted_value = Some(val.clone());
                return Ok(StepOutcome::Halted(val));
            }
        };
        let span = chunk.spans.get(pc).copied().flatten();

        self.steps += 1;
        if self.steps > self.config.step_budget {
            return Err(TrapError::new(
                TrapCode::StepBudgetExceeded,
                format!("step budget {} exceeded", self.config.step_budget),
                pc,
                self.steps,
                span,
            ));
        }
        self.profiler.record_step(1);

        if let Some(frame) = self.frames.last_mut() {
            frame.pc += 1;
        }

        let mut cache_line = None;
        let mut branch_taken = None;

        match op {
            Op::LoadConst(idx) => {
                let c = self.program.chunks[chunk_idx]
                    .constants
                    .get(idx as usize)
                    .ok_or_else(|| self.trap(TrapCode::InvalidBytecode, "bad const idx", pc, span))?
                    .clone();
                let val = match c {
                    Constant::Unit => RuntimeValue::Unit,
                    Constant::Bool(b) => RuntimeValue::Bool(b),
                    Constant::Int(s) => {
                        let bi = s.parse::<BigInt>().map_err(|_| {
                            self.trap(TrapCode::InvalidBytecode, "bad integer literal", pc, span)
                        })?;
                        RuntimeValue::Int(bi)
                    }
                    Constant::Real(r) => RuntimeValue::Real(CanonicalF64::new(r)?),
                    Constant::Char(ch) => RuntimeValue::Char(ch),
                    Constant::Str(s) => RuntimeValue::Str(s),
                };
                self.stack.push(val);
            }
            Op::LoadLocal(slot) => {
                let frame = self
                    .frames
                    .last()
                    .ok_or_else(|| self.trap(TrapCode::StackUnderflow, "no frame", pc, span))?;
                let val = frame
                    .locals
                    .get(slot as usize)
                    .cloned()
                    .unwrap_or(RuntimeValue::Unit);
                self.heap.retain_value(&val);
                self.stack.push(val);
            }
            Op::LoadOldLocal(slot) => {
                let frame = self
                    .frames
                    .last()
                    .ok_or_else(|| self.trap(TrapCode::StackUnderflow, "no frame", pc, span))?;
                let val = frame
                    .old_locals
                    .get(slot as usize)
                    .cloned()
                    .unwrap_or(RuntimeValue::Unit);
                self.heap.retain_value(&val);
                self.stack.push(val);
            }
            Op::StoreLocal(slot) => {
                let val = self.pop(pc, span)?;
                if let Some(frame) = self.frames.last_mut() {
                    if let Some(cell) = frame.locals.get_mut(slot as usize) {
                        let old = std::mem::replace(cell, val);
                        self.heap.release_value(&old);
                    } else {
                        self.heap.release_value(&val);
                    }
                }
            }
            Op::InitLocal(slot) => {
                let val = self.pop(pc, span)?;
                if let Some(frame) = self.frames.last_mut() {
                    if (slot as usize) >= frame.locals.len() {
                        frame.locals.resize(slot as usize + 1, RuntimeValue::Unit);
                    }
                    let old = std::mem::replace(&mut frame.locals[slot as usize], val);
                    self.heap.release_value(&old);
                }
            }
            Op::DropLocal(slot) => {
                if let Some(frame) = self.frames.last_mut() {
                    if let Some(cell) = frame.locals.get_mut(slot as usize) {
                        let old = std::mem::replace(cell, RuntimeValue::Unit);
                        self.heap.release_value(&old);
                    }
                }
            }
            Op::Pop => {
                let val = self.pop(pc, span)?;
                self.heap.release_value(&val);
            }
            Op::Dup => {
                let top = self.stack.last().cloned().ok_or_else(|| {
                    self.trap(TrapCode::StackUnderflow, "dup on empty stack", pc, span)
                })?;
                self.heap.retain_value(&top);
                self.stack.push(top);
            }
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod | Op::Pow => {
                let b = self.pop(pc, span)?;
                let a = self.pop(pc, span)?;
                let res = self.eval_arith(op, &a, &b, pc, span)?;
                self.stack.push(res);
            }
            Op::Neg => {
                let a = self.pop(pc, span)?;
                let res = match a {
                    RuntimeValue::Int(b) => RuntimeValue::Int(-b),
                    RuntimeValue::Int64(i) => {
                        RuntimeValue::Int64(i.checked_neg().ok_or_else(|| {
                            self.trap(TrapCode::IntegerOverflow, "i64 neg overflow", pc, span)
                        })?)
                    }
                    RuntimeValue::Real(r) => RuntimeValue::Real(CanonicalF64::new(-r.get())?),
                    _ => {
                        return Err(self.trap(
                            TrapCode::TypeMismatch,
                            "cannot negate value",
                            pc,
                            span,
                        ))
                    }
                };
                self.stack.push(res);
            }
            Op::Not => {
                let a = self.pop(pc, span)?;
                match a {
                    RuntimeValue::Bool(b) => self.stack.push(RuntimeValue::Bool(!b)),
                    RuntimeValue::Int(i) => self.stack.push(RuntimeValue::Int(!i)),
                    _ => {
                        return Err(self.trap(TrapCode::TypeMismatch, "cannot apply not", pc, span))
                    }
                }
            }
            Op::BitAnd | Op::BitOr | Op::BitXor | Op::Shl | Op::Shr => {
                let b = self.pop(pc, span)?;
                let a = self.pop(pc, span)?;
                let ia = a.as_bigint().ok_or_else(|| {
                    self.trap(TrapCode::TypeMismatch, "bitwise op requires Int", pc, span)
                })?;
                let ib = b.as_bigint().ok_or_else(|| {
                    self.trap(TrapCode::TypeMismatch, "bitwise op requires Int", pc, span)
                })?;
                let res = match op {
                    Op::BitAnd => ia & ib,
                    Op::BitOr => ia | ib,
                    Op::BitXor => ia ^ ib,
                    Op::Shl => {
                        let s = b.as_usize().ok_or_else(|| {
                            self.trap(TrapCode::IntegerOverflow, "invalid shift count", pc, span)
                        })?;
                        ia << s
                    }
                    Op::Shr => {
                        let s = b.as_usize().ok_or_else(|| {
                            self.trap(TrapCode::IntegerOverflow, "invalid shift count", pc, span)
                        })?;
                        ia >> s
                    }
                    _ => unreachable!(),
                };
                self.stack.push(RuntimeValue::Int(res));
            }
            Op::Eq | Op::Ne => {
                let b = self.pop(pc, span)?;
                let a = self.pop(pc, span)?;
                let eq = self.values_equal(&a, &b)?;
                self.stack
                    .push(RuntimeValue::Bool(if op == Op::Eq { eq } else { !eq }));
            }
            Op::Lt | Op::Le | Op::Gt | Op::Ge => {
                let b = self.pop(pc, span)?;
                let a = self.pop(pc, span)?;
                let ord = cmp_values(&a, &b)?;
                let ok = match op {
                    Op::Lt => ord == std::cmp::Ordering::Less,
                    Op::Le => ord != std::cmp::Ordering::Greater,
                    Op::Gt => ord == std::cmp::Ordering::Greater,
                    Op::Ge => ord != std::cmp::Ordering::Less,
                    _ => unreachable!(),
                };
                self.stack.push(RuntimeValue::Bool(ok));
            }
            Op::Jump(target) => {
                if let Some(frame) = self.frames.last_mut() {
                    frame.pc = target;
                }
            }
            Op::JumpIfFalse(target) => {
                let cond = self.pop_bool(pc, span)?;
                branch_taken = Some(cond);
                if !cond {
                    if let Some(frame) = self.frames.last_mut() {
                        frame.pc = target;
                    }
                }
            }
            Op::JumpIfTrue(target) => {
                let cond = self.pop_bool(pc, span)?;
                branch_taken = Some(cond);
                if cond {
                    if let Some(frame) = self.frames.last_mut() {
                        frame.pc = target;
                    }
                }
            }
            Op::CallNamed {
                name_idx,
                arg_count,
                writeback_id,
            } => {
                let caller_chunk = &self.program.chunks[chunk_idx];
                let name = caller_chunk
                    .strings
                    .get(name_idx as usize)
                    .cloned()
                    .ok_or_else(|| {
                        self.trap(TrapCode::InvalidBytecode, "bad string idx", pc, span)
                    })?;
                let mut_writebacks = writeback_id
                    .and_then(|id| caller_chunk.writebacks.get(id as usize).cloned())
                    .unwrap_or_default();
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.pop(pc, span)?);
                }
                args.reverse();

                if name == "__in" && args.len() == 2 {
                    let (_, line) = call_method(
                        &args[1],
                        "contains",
                        &[args[0].clone()],
                        &mut self.heap,
                        &mut self.cache,
                    )?;
                    cache_line = line;
                    let (res, _) = call_method(
                        &args[1],
                        "contains",
                        &[args[0].clone()],
                        &mut self.heap,
                        &mut self.cache,
                    )?;
                    self.stack.push(res);
                } else if name == "__slice" && args.len() == 3 {
                    let s = args[1].as_usize().unwrap_or(0);
                    let e = args[2].as_usize().unwrap_or(0);
                    let res = self.eval_slice(&args[0], s, e, pc, span)?;
                    self.stack.push(res);
                } else if name == "__range" && args.len() == 2 {
                    let a = args[0].as_i64().unwrap_or(0);
                    let b = args[1].as_i64().unwrap_or(0);
                    let h = self.heap.alloc(HeapObject::List(vec![
                        RuntimeValue::Str("__ueas_range__".to_string()),
                        RuntimeValue::Int(BigInt::from(a)),
                        RuntimeValue::Int(BigInt::from(b)),
                    ]))?;
                    self.stack.push(RuntimeValue::Handle(h));
                } else if let Some(val) =
                    call_builtin(&name, &args, &mut self.heap, &mut self.prng)?
                {
                    self.stack.push(val);
                } else if let Some(&target_idx) = self.program.chunk_index.get(&name) {
                    let target_chunk = &self.program.chunks[target_idx];
                    if args.len() != target_chunk.params.len() {
                        return Err(self.trap(
                            TrapCode::ArityMismatch,
                            format!(
                                "'{name}' expects {} args, got {}",
                                target_chunk.params.len(),
                                args.len()
                            ),
                            pc,
                            span,
                        ));
                    }
                    let mut locals = vec![RuntimeValue::Unit; target_chunk.local_names.len()];
                    let mut old_locals = if target_chunk.uses_old {
                        vec![RuntimeValue::Unit; target_chunk.local_names.len()]
                    } else {
                        Vec::new()
                    };
                    for (i, arg) in args.into_iter().enumerate() {
                        if target_chunk.uses_old {
                            self.heap.retain_value(&arg);
                            old_locals[i] = arg.clone();
                        }
                        locals[i] = arg;
                    }
                    self.frames.push(CallFrame {
                        chunk_idx: target_idx,
                        pc: if self.config.enforce_contracts {
                            0
                        } else {
                            target_chunk.body_start_pc
                        },
                        locals,
                        old_locals,
                        stack_base: self.stack.len(),
                        mut_writebacks,
                    });
                } else {
                    return Err(self.trap(
                        TrapCode::UndefinedSymbol,
                        format!("undefined algorithm or builtin '{name}'"),
                        pc,
                        span,
                    ));
                }
            }
            Op::CallMethod {
                method_idx,
                arg_count,
            } => {
                let method = self.program.chunks[chunk_idx]
                    .strings
                    .get(method_idx as usize)
                    .cloned()
                    .ok_or_else(|| {
                        self.trap(TrapCode::InvalidBytecode, "bad method idx", pc, span)
                    })?;
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.pop(pc, span)?);
                }
                args.reverse();
                let receiver = self.pop(pc, span)?;
                let (res, line) =
                    call_method(&receiver, &method, &args, &mut self.heap, &mut self.cache)?;
                cache_line = line;
                self.stack.push(res);
            }
            Op::Return => {
                let ret_val = if let Some(frame) = self.frames.last() {
                    if self.stack.len() > frame.stack_base {
                        self.pop(pc, span)?
                    } else {
                        RuntimeValue::Unit
                    }
                } else {
                    RuntimeValue::Unit
                };

                if let Some(popped) = self.frames.pop() {
                    if let Some(caller) = self.frames.last_mut() {
                        for (callee_slot, caller_slot) in popped.mut_writebacks {
                            if let Some(updated) = popped.locals.get(callee_slot as usize).cloned()
                            {
                                self.heap.retain_value(&updated);
                                if let Some(cell) = caller.locals.get_mut(caller_slot as usize) {
                                    let prev = std::mem::replace(cell, updated);
                                    self.heap.release_value(&prev);
                                }
                            }
                        }
                    }
                    for local in &popped.locals {
                        self.heap.release_value(local);
                    }
                    for old_local in &popped.old_locals {
                        self.heap.release_value(old_local);
                    }
                }

                if self.frames.is_empty() {
                    self.halted_value = Some(ret_val.clone());
                    return Ok(StepOutcome::Halted(ret_val));
                } else {
                    self.stack.push(ret_val);
                }
            }
            Op::MakeList(count) => {
                let mut items = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    items.push(self.pop(pc, span)?);
                }
                items.reverse();
                let h = self.heap.alloc(HeapObject::List(items))?;
                self.stack.push(RuntimeValue::Handle(h));
            }
            Op::MakeSet(count) => {
                let mut items = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    items.push(self.pop(pc, span)?);
                }
                items.reverse();
                let mut set = IndexSet::new();
                for it in items {
                    set.insert(it.to_key()?);
                }
                let h = self.heap.alloc(HeapObject::Set(set))?;
                self.stack.push(RuntimeValue::Handle(h));
            }
            Op::MakeMap(count) => {
                let mut pairs = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let v = self.pop(pc, span)?;
                    let k = self.pop(pc, span)?;
                    pairs.push((k, v));
                }
                pairs.reverse();
                let mut map = IndexMap::new();
                for (k, v) in pairs {
                    map.insert(k.to_key()?, v);
                }
                let h = self.heap.alloc(HeapObject::Map(map))?;
                self.stack.push(RuntimeValue::Handle(h));
            }
            Op::MakeTuple(count) => {
                let mut items = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    items.push(self.pop(pc, span)?);
                }
                items.reverse();
                self.stack.push(RuntimeValue::Tuple(items));
            }
            Op::MakeRecord {
                type_idx,
                field_count,
            } => {
                let type_name = self.program.chunks[chunk_idx]
                    .strings
                    .get(type_idx as usize)
                    .cloned()
                    .unwrap_or_else(|| "Record".to_string());
                let mut pairs = Vec::with_capacity(field_count as usize);
                for _ in 0..field_count {
                    let v = self.pop(pc, span)?;
                    let k = self.pop(pc, span)?;
                    pairs.push((k.to_string(), v));
                }
                pairs.reverse();
                let mut fields = IndexMap::new();
                for (k, v) in pairs {
                    fields.insert(k, v);
                }
                let h = self.heap.alloc(HeapObject::Record { type_name, fields })?;
                self.stack.push(RuntimeValue::Handle(h));
            }
            Op::IndexGet => {
                let idx_val = self.pop(pc, span)?;
                let target = self.pop(pc, span)?;
                let (val, line) = self.eval_index_get(&target, &idx_val, pc, span)?;
                self.heap.retain_value(&val);
                self.heap.release_value(&target);
                self.heap.release_value(&idx_val);
                cache_line = line;
                self.stack.push(val);
            }
            Op::IndexSet => {
                let val = self.pop(pc, span)?;
                let idx_val = self.pop(pc, span)?;
                let target = self.pop(pc, span)?;
                cache_line = self.eval_index_set(&target, &idx_val, val, pc, span)?;
            }
            Op::FieldGet(f_idx) => {
                let field = self.program.chunks[chunk_idx]
                    .strings
                    .get(f_idx as usize)
                    .cloned()
                    .ok_or_else(|| {
                        self.trap(TrapCode::InvalidBytecode, "bad field idx", pc, span)
                    })?;
                let target = self.pop(pc, span)?;
                let res_val = match target {
                    RuntimeValue::Handle(h) => {
                        let (obj, base) = self.heap.get(h)?;
                        cache_line = Some(self.cache.access(base, 8));
                        match obj {
                            HeapObject::Record { fields, .. } => {
                                let v = fields.get(&field).cloned().ok_or_else(|| {
                                    self.trap(
                                        TrapCode::UndefinedSymbol,
                                        format!("no field '{field}' on record"),
                                        pc,
                                        span,
                                    )
                                })?;
                                v
                            }
                            _ => {
                                return Err(self.trap(
                                    TrapCode::TypeMismatch,
                                    "field access requires Record",
                                    pc,
                                    span,
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(self.trap(
                            TrapCode::TypeMismatch,
                            "field access requires Record handle",
                            pc,
                            span,
                        ))
                    }
                };
                self.heap.retain_value(&res_val);
                self.heap.release_value(&target);
                self.stack.push(res_val);
            }
            Op::FieldSet(f_idx) => {
                let field = self.program.chunks[chunk_idx]
                    .strings
                    .get(f_idx as usize)
                    .cloned()
                    .ok_or_else(|| {
                        self.trap(TrapCode::InvalidBytecode, "bad field idx", pc, span)
                    })?;
                let val = self.pop(pc, span)?;
                let target = self.pop(pc, span)?;
                let cloned = self.heap.deep_clone_value(&val)?;
                match target {
                    RuntimeValue::Handle(h) => {
                        let line = self.heap.with_mut(h, |obj, base| match obj {
                            HeapObject::Record { fields, .. } => {
                                fields.insert(field, cloned);
                                Ok(self.cache.access(base, 8))
                            }
                            _ => Err(TrapError::new(
                                TrapCode::TypeMismatch,
                                "field assignment requires Record",
                                pc,
                                self.steps,
                                span,
                            )),
                        })?;
                        cache_line = Some(line);
                    }
                    _ => {
                        return Err(self.trap(
                            TrapCode::TypeMismatch,
                            "field assignment requires Record handle",
                            pc,
                            span,
                        ))
                    }
                }
            }
            Op::MakeSome => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::OptionVal(Some(Box::new(v))));
            }
            Op::MakeNone => {
                self.stack.push(RuntimeValue::OptionVal(None));
            }
            Op::MakeOk => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::ResultVal(Ok(Box::new(v))));
            }
            Op::MakeErr => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::ResultVal(Err(Box::new(v))));
            }
            Op::IsSome => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::Bool(matches!(
                    v,
                    RuntimeValue::OptionVal(Some(_))
                )));
            }
            Op::IsNone => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::Bool(matches!(
                    v,
                    RuntimeValue::OptionVal(None)
                )));
            }
            Op::IsOk => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::Bool(matches!(
                    v,
                    RuntimeValue::ResultVal(Ok(_))
                )));
            }
            Op::IsErr => {
                let v = self.pop(pc, span)?;
                self.stack.push(RuntimeValue::Bool(matches!(
                    v,
                    RuntimeValue::ResultVal(Err(_))
                )));
            }
            Op::Unwrap => {
                let v = self.pop(pc, span)?;
                match v {
                    RuntimeValue::OptionVal(Some(inner))
                    | RuntimeValue::ResultVal(Ok(inner))
                    | RuntimeValue::ResultVal(Err(inner)) => self.stack.push(*inner),
                    _ => {
                        return Err(self.trap(
                            TrapCode::AssertionFailed,
                            "unwrap failed on None/non-algebraic value",
                            pc,
                            span,
                        ))
                    }
                }
            }
            Op::TupleGet(idx) => {
                let v = self.pop(pc, span)?;
                match v {
                    RuntimeValue::Tuple(items) => {
                        let item = items.get(idx as usize).cloned().ok_or_else(|| {
                            self.trap(TrapCode::IndexOutOfBounds, "tuple index OOB", pc, span)
                        })?;
                        self.stack.push(item);
                    }
                    _ => return Err(self.trap(TrapCode::TypeMismatch, "expected Tuple", pc, span)),
                }
            }
            Op::CheckRequire(msg_idx)
            | Op::CheckEnsure(msg_idx)
            | Op::CheckInvariant(msg_idx)
            | Op::Assert(msg_idx) => {
                let cond = self.pop_bool(pc, span)?;
                if self.config.enforce_contracts && !cond {
                    let msg = self.program.chunks[chunk_idx]
                        .strings
                        .get(msg_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| "contract violated".to_string());
                    let trap = match op {
                        Op::CheckRequire(_) => TrapCode::PreconditionViolated,
                        Op::CheckEnsure(_) => TrapCode::PostconditionViolated,
                        Op::CheckInvariant(_) => TrapCode::InvariantViolated,
                        _ => TrapCode::AssertionFailed,
                    };
                    return Err(self.trap(trap, msg, pc, span));
                }
            }
            Op::CheckDecreases(msg_idx) => {
                let prev = self.pop(pc, span)?;
                let curr = self.pop(pc, span)?;
                if self.config.enforce_contracts {
                    let c = curr.as_bigint().unwrap_or_default();
                    let p = prev.as_bigint().unwrap_or_else(|| BigInt::from(-1));
                    if c < BigInt::from(0) {
                        return Err(self.trap(
                            TrapCode::TerminationMeasureViolated,
                            "decreases rank became negative",
                            pc,
                            span,
                        ));
                    }
                    if p >= BigInt::from(0) && c >= p {
                        let msg = self.program.chunks[chunk_idx]
                            .strings
                            .get(msg_idx as usize)
                            .cloned()
                            .unwrap_or_else(|| "termination rank did not decrease".to_string());
                        return Err(self.trap(TrapCode::TerminationMeasureViolated, msg, pc, span));
                    }
                }
            }
            Op::SelectBranchless => {
                let b = self.pop(pc, span)?;
                let a = self.pop(pc, span)?;
                let cond = self.pop_bool(pc, span)?;
                // Branchless arithmetic selection (no jump/branch event emitted)
                self.stack.push(if cond { a } else { b });
            }
            Op::ForkScope => self.profiler.fork_scope(),
            Op::BeginBranch => self.profiler.begin_branch(),
            Op::EndBranch => self.profiler.end_branch(),
            Op::SyncScope => self.profiler.sync_scope(),
            Op::Nop => {}
        }

        if self.config.record_trace {
            self.trace.push(TraceEvent {
                pc,
                cache_line,
                branch_taken,
            });
        }

        if let Some(sp) = span {
            let chunk_name = &self.program.chunks[chunk_idx].name;
            let line = sp.start_line as u32;
            if self.breakpoints.contains(&(chunk_name.clone(), line)) {
                return Ok(StepOutcome::BreakpointHit { line, pc });
            }
        }

        Ok(StepOutcome::Continue)
    }

    fn trap(
        &self,
        code: TrapCode,
        msg: impl Into<String>,
        pc: usize,
        span: Option<SourceSpan>,
    ) -> TrapError {
        TrapError::new(code, msg, pc, self.steps, span)
    }

    fn pop(&mut self, pc: usize, span: Option<SourceSpan>) -> Result<RuntimeValue, TrapError> {
        self.stack.pop().ok_or_else(|| {
            self.trap(
                TrapCode::StackUnderflow,
                "operand stack underflow",
                pc,
                span,
            )
        })
    }

    fn pop_bool(&mut self, pc: usize, span: Option<SourceSpan>) -> Result<bool, TrapError> {
        match self.pop(pc, span)? {
            RuntimeValue::Bool(b) => Ok(b),
            other => Err(self.trap(
                TrapCode::TypeMismatch,
                format!("expected Bool, got {other}"),
                pc,
                span,
            )),
        }
    }

    fn values_equal(&self, a: &RuntimeValue, b: &RuntimeValue) -> Result<bool, TrapError> {
        match (a, b) {
            (RuntimeValue::OptionVal(Some(x)), RuntimeValue::OptionVal(Some(y))) => {
                self.values_equal(x, y)
            }
            (RuntimeValue::ResultVal(Ok(x)), RuntimeValue::ResultVal(Ok(y)))
            | (RuntimeValue::ResultVal(Err(x)), RuntimeValue::ResultVal(Err(y))) => {
                self.values_equal(x, y)
            }
            (RuntimeValue::Tuple(xs), RuntimeValue::Tuple(ys)) => {
                if xs.len() != ys.len() {
                    return Ok(false);
                }
                for (x, y) in xs.iter().zip(ys.iter()) {
                    if !self.values_equal(x, y)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (RuntimeValue::Handle(h1), RuntimeValue::Handle(h2)) => {
                if h1 == h2 {
                    return Ok(true);
                }
                let (o1, _) = self.heap.get(*h1)?;
                let (o2, _) = self.heap.get(*h2)?;
                match (o1, o2) {
                    (HeapObject::List(xs), HeapObject::List(ys)) => {
                        if xs.len() != ys.len() {
                            return Ok(false);
                        }
                        for (x, y) in xs.iter().zip(ys.iter()) {
                            if !self.values_equal(x, y)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    (
                        HeapObject::Record {
                            type_name: t1,
                            fields: f1,
                        },
                        HeapObject::Record {
                            type_name: t2,
                            fields: f2,
                        },
                    ) => {
                        if t1 != t2 || f1.len() != f2.len() {
                            return Ok(false);
                        }
                        for (k, v1) in f1 {
                            let Some(v2) = f2.get(k) else {
                                return Ok(false);
                            };
                            if !self.values_equal(v1, v2)? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(o1 == o2),
                }
            }
            _ => {
                if let (Some(x), Some(y)) = (a.as_bigint(), b.as_bigint()) {
                    return Ok(x == y);
                }
                Ok(a == b)
            }
        }
    }

    fn eval_arith(
        &self,
        op: Op,
        a: &RuntimeValue,
        b: &RuntimeValue,
        pc: usize,
        span: Option<SourceSpan>,
    ) -> Result<RuntimeValue, TrapError> {
        match (a, b) {
            (RuntimeValue::Str(x), RuntimeValue::Str(y)) if op == Op::Add => {
                Ok(RuntimeValue::Str(format!("{x}{y}")))
            }
            (RuntimeValue::Real(_), _) | (_, RuntimeValue::Real(_)) => {
                let x = a.as_f64().ok_or_else(|| {
                    self.trap(TrapCode::TypeMismatch, "expected numeric", pc, span)
                })?;
                let y = b.as_f64().ok_or_else(|| {
                    self.trap(TrapCode::TypeMismatch, "expected numeric", pc, span)
                })?;
                let out = match op {
                    Op::Add => x + y,
                    Op::Sub => x - y,
                    Op::Mul => x * y,
                    Op::Div => {
                        if y == 0.0 {
                            return Err(self.trap(
                                TrapCode::DivisionByZero,
                                "float division by zero",
                                pc,
                                span,
                            ));
                        }
                        x / y
                    }
                    Op::Mod => {
                        if y == 0.0 {
                            return Err(self.trap(
                                TrapCode::DivisionByZero,
                                "float mod by zero",
                                pc,
                                span,
                            ));
                        }
                        x % y
                    }
                    Op::Pow => x.powf(y),
                    _ => unreachable!(),
                };
                Ok(RuntimeValue::Real(CanonicalF64::new(out)?))
            }
            (RuntimeValue::Int64(x), RuntimeValue::Int64(y)) => {
                let out = match op {
                    Op::Add => x.checked_add(*y),
                    Op::Sub => x.checked_sub(*y),
                    Op::Mul => x.checked_mul(*y),
                    Op::Div => {
                        if *y == 0 {
                            return Err(self.trap(
                                TrapCode::DivisionByZero,
                                "i64 division by zero",
                                pc,
                                span,
                            ));
                        }
                        x.checked_div(*y)
                    }
                    Op::Mod => {
                        if *y == 0 {
                            return Err(self.trap(
                                TrapCode::DivisionByZero,
                                "i64 mod by zero",
                                pc,
                                span,
                            ));
                        }
                        x.checked_rem(*y)
                    }
                    Op::Pow => x.checked_pow((*y).max(0) as u32),
                    _ => unreachable!(),
                }
                .ok_or_else(|| self.trap(TrapCode::IntegerOverflow, "Int64 overflow", pc, span))?;
                Ok(RuntimeValue::Int64(out))
            }
            _ => {
                let x = a.as_bigint().ok_or_else(|| {
                    self.trap(
                        TrapCode::TypeMismatch,
                        format!("expected Int, got {a}"),
                        pc,
                        span,
                    )
                })?;
                let y = b.as_bigint().ok_or_else(|| {
                    self.trap(
                        TrapCode::TypeMismatch,
                        format!("expected Int, got {b}"),
                        pc,
                        span,
                    )
                })?;
                let out = match op {
                    Op::Add => x + y,
                    Op::Sub => x - y,
                    Op::Mul => x * y,
                    Op::Div => {
                        if y == BigInt::from(0) {
                            return Err(self.trap(
                                TrapCode::DivisionByZero,
                                "integer division by zero",
                                pc,
                                span,
                            ));
                        }
                        x / y
                    }
                    Op::Mod => {
                        if y == BigInt::from(0) {
                            return Err(self.trap(
                                TrapCode::DivisionByZero,
                                "integer modulo by zero",
                                pc,
                                span,
                            ));
                        }
                        x % y
                    }
                    Op::Pow => {
                        let exp: u32 = y.try_into().map_err(|_| {
                            self.trap(TrapCode::IntegerOverflow, "invalid exponent", pc, span)
                        })?;
                        x.pow(exp)
                    }
                    _ => unreachable!(),
                };
                Ok(RuntimeValue::Int(out))
            }
        }
    }

    fn eval_index_get(
        &mut self,
        target: &RuntimeValue,
        idx_val: &RuntimeValue,
        pc: usize,
        span: Option<SourceSpan>,
    ) -> Result<(RuntimeValue, Option<u64>), TrapError> {
        match target {
            RuntimeValue::Str(s) => {
                if let RuntimeValue::Handle(rh) = idx_val {
                    if let Ok((HeapObject::List(ritems), _)) = self.heap.get(*rh) {
                        if matches!(ritems.first(), Some(RuntimeValue::Str(tag)) if tag == "__ueas_range__")
                        {
                            let start = ritems.get(1).and_then(RuntimeValue::as_usize).unwrap_or(0);
                            let end = ritems.get(2).and_then(RuntimeValue::as_usize).unwrap_or(0);
                            let sliced: String = s
                                .chars()
                                .skip(start)
                                .take(end.saturating_sub(start))
                                .collect();
                            return Ok((RuntimeValue::Str(sliced), None));
                        }
                    }
                }
                let idx = idx_val.as_usize().ok_or_else(|| {
                    self.trap(TrapCode::TypeMismatch, "string index must be Int", pc, span)
                })?;
                let ch = s.chars().nth(idx).ok_or_else(|| {
                    self.trap(TrapCode::IndexOutOfBounds, "string index OOB", pc, span)
                })?;
                Ok((RuntimeValue::Str(ch.to_string()), None))
            }
            RuntimeValue::Tuple(items) => {
                let idx = idx_val.as_usize().ok_or_else(|| {
                    self.trap(TrapCode::TypeMismatch, "tuple index must be Int", pc, span)
                })?;
                let item = items.get(idx).cloned().ok_or_else(|| {
                    self.trap(TrapCode::IndexOutOfBounds, "tuple index OOB", pc, span)
                })?;
                Ok((item, None))
            }
            RuntimeValue::Handle(h) => {
                let (obj, base) = self.heap.get(*h)?;
                match obj {
                    HeapObject::List(v) => {
                        if let RuntimeValue::Handle(rh) = idx_val {
                            if let Ok((HeapObject::List(ritems), _)) = self.heap.get(*rh) {
                                if matches!(ritems.first(), Some(RuntimeValue::Str(tag)) if tag == "__ueas_range__")
                                {
                                    let start = ritems
                                        .get(1)
                                        .and_then(RuntimeValue::as_usize)
                                        .unwrap_or(0)
                                        .min(v.len());
                                    let end = ritems
                                        .get(2)
                                        .and_then(RuntimeValue::as_usize)
                                        .unwrap_or(0)
                                        .min(v.len())
                                        .max(start);
                                    let sliced = v[start..end].to_vec();
                                    let new_h = self.heap.alloc(HeapObject::List(sliced))?;
                                    return Ok((RuntimeValue::Handle(new_h), None));
                                }
                            }
                        }
                        let idx = idx_val.as_usize().ok_or_else(|| {
                            self.trap(
                                TrapCode::IndexOutOfBounds,
                                "list index must be non-negative Int",
                                pc,
                                span,
                            )
                        })?;
                        let line = self.cache.access(base + (idx as u64) * 8, 8);
                        let item = v.get(idx).cloned().ok_or_else(|| {
                            self.trap(
                                TrapCode::IndexOutOfBounds,
                                format!("index {idx} out of bounds for len {}", v.len()),
                                pc,
                                span,
                            )
                        })?;
                        Ok((item, Some(line)))
                    }
                    HeapObject::Map(m) => {
                        let key = idx_val.to_key()?;
                        let line = self.cache.access(base, 8);
                        let item = m.get(&key).cloned().ok_or_else(|| {
                            self.trap(TrapCode::IndexOutOfBounds, "map key not found", pc, span)
                        })?;
                        Ok((item, Some(line)))
                    }
                    _ => Err(self.trap(
                        TrapCode::TypeMismatch,
                        "index unsupported on object",
                        pc,
                        span,
                    )),
                }
            }
            _ => Err(self.trap(
                TrapCode::TypeMismatch,
                "index target must be List/Map/Str",
                pc,
                span,
            )),
        }
    }

    fn eval_index_set(
        &mut self,
        target: &RuntimeValue,
        idx_val: &RuntimeValue,
        val: RuntimeValue,
        pc: usize,
        span: Option<SourceSpan>,
    ) -> Result<Option<u64>, TrapError> {
        let RuntimeValue::Handle(h) = target else {
            return Err(self.trap(
                TrapCode::TypeMismatch,
                "index assignment requires List/Map",
                pc,
                span,
            ));
        };
        let cloned = self.heap.deep_clone_value(&val)?;
        let handle = *h;
        let line = self.heap.with_mut(handle, |obj, base| match obj {
            HeapObject::List(v) => {
                let idx = idx_val.as_usize().ok_or_else(|| {
                    TrapError::new(
                        TrapCode::IndexOutOfBounds,
                        "list index must be non-negative",
                        pc,
                        self.steps,
                        span,
                    )
                })?;
                if idx >= v.len() {
                    return Err(TrapError::new(
                        TrapCode::IndexOutOfBounds,
                        format!("index {idx} >= len {}", v.len()),
                        pc,
                        self.steps,
                        span,
                    ));
                }
                v[idx] = cloned;
                Ok(self.cache.access(base + (idx as u64) * 8, 8))
            }
            HeapObject::Map(m) => {
                let key = idx_val.to_key()?;
                m.insert(key, cloned);
                Ok(self.cache.access(base, 8))
            }
            _ => Err(TrapError::new(
                TrapCode::TypeMismatch,
                "index assignment unsupported",
                pc,
                self.steps,
                span,
            )),
        })?;
        Ok(Some(line))
    }

    fn eval_slice(
        &mut self,
        target: &RuntimeValue,
        start: usize,
        end: usize,
        pc: usize,
        span: Option<SourceSpan>,
    ) -> Result<RuntimeValue, TrapError> {
        match target {
            RuntimeValue::Str(s) => {
                let slice: String = s
                    .chars()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .collect();
                Ok(RuntimeValue::Str(slice))
            }
            RuntimeValue::Handle(h) => {
                let slice_vec = {
                    let (obj, _) = self.heap.get(*h)?;
                    match obj {
                        HeapObject::List(v) => {
                            let s = start.min(v.len());
                            let e = end.min(v.len()).max(s);
                            v[s..e].to_vec()
                        }
                        _ => {
                            return Err(self.trap(
                                TrapCode::TypeMismatch,
                                "slice only supported on List/Str",
                                pc,
                                span,
                            ))
                        }
                    }
                };
                let new_h = self.heap.alloc(HeapObject::List(slice_vec))?;
                Ok(RuntimeValue::Handle(new_h))
            }
            _ => Err(self.trap(TrapCode::TypeMismatch, "slice unsupported", pc, span)),
        }
    }
}

/// Convenience helper: compile HIR and run `entry` with `args`.
pub fn run_hir(
    program: &ueas_hir::hir::Program,
    entry: &str,
    args: Vec<RuntimeValue>,
    config: VmConfig,
) -> Result<ExecutionResult, String> {
    let compiled = crate::compile::compile_program(program)?;
    let mut session = VmSession::new(compiled, entry, args, config).map_err(|e| e.to_string())?;
    session.run().map_err(|e| e.to_string())
}
