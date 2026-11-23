//! HIR -> Bytecode compiler for UEAS v2.0.

use crate::chunk::{Chunk, CompiledProgram, Constant};
use crate::op::Op;
use std::collections::HashMap;
use ueas_hir::complexity::ComplexitySpec;
use ueas_hir::hir::{
    AlgorithmDecl, BinaryOp, Expr, LValue, MatchArm, Pattern, Program, Stmt, UnaryOp,
};
use ueas_syntax::SourceSpan;

struct LoopCtx {
    continue_jumps: Vec<usize>,
    break_jumps: Vec<usize>,
}

struct Compiler {
    chunk: Chunk,
    scopes: Vec<HashMap<String, u16>>,
    loop_stack: Vec<LoopCtx>,
    counter: usize,
    postconditions: Vec<Expr>,
    postcondition_binding: Option<String>,
}

impl Compiler {
    fn new(
        name: String,
        params: Vec<String>,
        mut_param_slots: Vec<u16>,
        is_constant_time: bool,
        complexity: ComplexitySpec,
    ) -> Self {
        let mut root_scope = HashMap::new();
        for (i, p) in params.iter().enumerate() {
            root_scope.insert(p.clone(), i as u16);
        }
        let mut chunk = Chunk::new(name, params, is_constant_time, complexity);
        chunk.mut_param_slots = mut_param_slots;
        Self {
            chunk,
            scopes: vec![root_scope],
            loop_stack: Vec::new(),
            counter: 0,
            postconditions: Vec::new(),
            postcondition_binding: None,
        }
    }

    fn fresh_name(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("__ueas_{prefix}_{}", self.counter)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self, _span: Option<SourceSpan>) {
        let _ = self.scopes.pop();
    }

    fn alloc_local(&mut self, name: &str) -> u16 {
        if let Some(slot) = self.chunk.local_names.iter().position(|n| n == name) {
            if let Some(top) = self.scopes.last_mut() {
                top.insert(name.to_string(), slot as u16);
            }
            return slot as u16;
        }
        let slot = self.chunk.local_names.len() as u16;
        self.chunk.local_names.push(name.to_string());
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name.to_string(), slot);
        }
        slot
    }

    fn resolve_local(&self, name: &str) -> Option<u16> {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    fn compile_algorithm(&mut self, algo: &AlgorithmDecl) -> Result<(), String> {
        self.postconditions = algo.postconditions.clone();
        self.postcondition_binding = algo.postcondition_binding.clone();

        // Check Require preconditions at entry
        for (idx, req) in algo.preconditions.iter().enumerate() {
            self.compile_expr(req)?;
            let msg_idx = self
                .chunk
                .add_string(&format!("Require clause #{idx} failed"));
            self.chunk.emit(Op::CheckRequire(msg_idx), Some(req.span()));
        }
        self.chunk.body_start_pc = self.chunk.ops.len();

        self.compile_stmts(&algo.body)?;

        // Implicit unit return if control reaches end without Return
        if !algo.postconditions.is_empty() {
            let unit_idx = self.chunk.add_const(Constant::Unit);
            self.chunk.emit(Op::LoadConst(unit_idx), Some(algo.span));
            self.emit_postconditions(algo)?;
        } else {
            let unit_idx = self.chunk.add_const(Constant::Unit);
            self.chunk.emit(Op::LoadConst(unit_idx), Some(algo.span));
        }
        self.chunk.emit(Op::Return, Some(algo.span));
        Ok(())
    }

    fn emit_postconditions(&mut self, algo: &AlgorithmDecl) -> Result<(), String> {
        if algo.postconditions.is_empty() {
            return Ok(());
        }
        let res_name = algo.postcondition_binding.as_deref().unwrap_or("result");
        let res_slot = self.alloc_local(res_name);
        self.chunk.emit(Op::InitLocal(res_slot), Some(algo.span));
        for (idx, post) in algo.postconditions.iter().enumerate() {
            self.compile_expr(post)?;
            let msg_idx = self
                .chunk
                .add_string(&format!("Ensure clause #{idx} failed"));
            self.chunk.emit(Op::CheckEnsure(msg_idx), Some(post.span()));
        }
        self.chunk.emit(Op::LoadLocal(res_slot), Some(algo.span));
        Ok(())
    }

    fn compile_stmts(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            self.compile_stmt(stmt, &[])?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt, active_postconditions: &[Expr]) -> Result<(), String> {
        match stmt {
            Stmt::Let {
                name, init, span, ..
            }
            | Stmt::Const {
                name, init, span, ..
            } => {
                self.compile_expr(init)?;
                let slot = self.alloc_local(name);
                self.chunk.emit(Op::InitLocal(slot), Some(*span));
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                self.compile_assign(target, value, *span)?;
            }
            Stmt::Expr { expr, span } => {
                self.compile_expr(expr)?;
                self.chunk.emit(Op::Pop, Some(*span));
            }
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    let c = self.chunk.add_const(Constant::Unit);
                    self.chunk.emit(Op::LoadConst(c), Some(*span));
                }
                let _ = active_postconditions;
                if !self.postconditions.is_empty() {
                    let res_name = self
                        .postcondition_binding
                        .as_deref()
                        .unwrap_or("result")
                        .to_string();
                    let res_slot = self.alloc_local(&res_name);
                    self.chunk.emit(Op::InitLocal(res_slot), Some(*span));
                    let posts = self.postconditions.clone();
                    for (idx, post) in posts.iter().enumerate() {
                        self.compile_expr(post)?;
                        let msg_idx = self
                            .chunk
                            .add_string(&format!("Ensure clause #{idx} failed"));
                        self.chunk.emit(Op::CheckEnsure(msg_idx), Some(post.span()));
                    }
                    self.chunk.emit(Op::LoadLocal(res_slot), Some(*span));
                }
                self.chunk.emit(Op::Return, Some(*span));
            }
            Stmt::If {
                condition,
                then_body,
                else_if_branches,
                else_body,
                span,
            } => {
                let mut end_jumps = Vec::new();
                self.compile_expr(condition)?;
                let mut next_branch_jump = self.chunk.emit(Op::JumpIfFalse(0), Some(*span));
                self.push_scope();
                self.compile_stmts(then_body)?;
                self.pop_scope(Some(*span));
                end_jumps.push(self.chunk.emit(Op::Jump(0), Some(*span)));

                for (cond, body) in else_if_branches {
                    let target = self.chunk.ops.len();
                    self.chunk.patch_jump(next_branch_jump, target);
                    self.compile_expr(cond)?;
                    next_branch_jump = self.chunk.emit(Op::JumpIfFalse(0), Some(cond.span()));
                    self.push_scope();
                    self.compile_stmts(body)?;
                    self.pop_scope(Some(*span));
                    end_jumps.push(self.chunk.emit(Op::Jump(0), Some(*span)));
                }

                let else_target = self.chunk.ops.len();
                self.chunk.patch_jump(next_branch_jump, else_target);
                if let Some(eb) = else_body {
                    self.push_scope();
                    self.compile_stmts(eb)?;
                    self.pop_scope(Some(*span));
                }

                let end_target = self.chunk.ops.len();
                for j in end_jumps {
                    self.chunk.patch_jump(j, end_target);
                }
            }
            Stmt::While {
                condition,
                invariants,
                decreases,
                body,
                span,
            } => {
                let prev_rank_slot = if decreases.is_some() {
                    let n = self.fresh_name("rank");
                    let slot = self.alloc_local(&n);
                    let none_c = self.chunk.add_const(Constant::Int("-1".to_string()));
                    self.chunk.emit(Op::LoadConst(none_c), Some(*span));
                    self.chunk.emit(Op::InitLocal(slot), Some(*span));
                    Some(slot)
                } else {
                    None
                };

                let loop_start = self.chunk.ops.len();
                for (i, inv) in invariants.iter().enumerate() {
                    self.compile_expr(inv)?;
                    let msg = self
                        .chunk
                        .add_string(&format!("while invariant #{i} failed"));
                    self.chunk.emit(Op::CheckInvariant(msg), Some(inv.span()));
                }

                self.compile_expr(condition)?;
                let exit_jump = self.chunk.emit(Op::JumpIfFalse(0), Some(*span));

                if let (Some(dec_expr), Some(rank_slot)) = (decreases, prev_rank_slot) {
                    self.compile_expr(dec_expr)?;
                    self.chunk
                        .emit(Op::LoadLocal(rank_slot), Some(dec_expr.span()));
                    let msg = self
                        .chunk
                        .add_string("decreases rank did not strictly decrease");
                    self.chunk
                        .emit(Op::CheckDecreases(msg), Some(dec_expr.span()));
                    self.compile_expr(dec_expr)?;
                    self.chunk
                        .emit(Op::InitLocal(rank_slot), Some(dec_expr.span()));
                }

                self.loop_stack.push(LoopCtx {
                    continue_jumps: Vec::new(),
                    break_jumps: Vec::new(),
                });
                self.push_scope();
                self.compile_stmts(body)?;
                self.pop_scope(Some(*span));

                let ctx = self
                    .loop_stack
                    .pop()
                    .ok_or_else(|| "internal compiler loop stack underflow".to_string())?;
                for cj in ctx.continue_jumps {
                    self.chunk.patch_jump(cj, loop_start);
                }
                self.chunk.emit(Op::Jump(loop_start), Some(*span));

                let exit_target = self.chunk.ops.len();
                self.chunk.patch_jump(exit_jump, exit_target);
                for bj in ctx.break_jumps {
                    self.chunk.patch_jump(bj, exit_target);
                }
            }
            Stmt::For {
                var,
                iterable,
                is_parallel,
                invariants,
                body,
                span,
            } => {
                if *is_parallel {
                    self.chunk.emit(Op::ForkScope, Some(*span));
                }
                match iterable {
                    Expr::Range {
                        start,
                        end,
                        inclusive,
                        ..
                    } => {
                        let idx_slot = self.alloc_local(var);
                        let end_name = self.fresh_name("end");
                        let end_slot = self.alloc_local(&end_name);
                        self.compile_expr(start)?;
                        self.chunk.emit(Op::InitLocal(idx_slot), Some(*span));
                        self.compile_expr(end)?;
                        self.chunk.emit(Op::InitLocal(end_slot), Some(*span));

                        let loop_start = self.chunk.ops.len();
                        for (i, inv) in invariants.iter().enumerate() {
                            self.compile_expr(inv)?;
                            let msg = self.chunk.add_string(&format!("for invariant #{i} failed"));
                            self.chunk.emit(Op::CheckInvariant(msg), Some(inv.span()));
                        }

                        self.chunk.emit(Op::LoadLocal(idx_slot), Some(*span));
                        self.chunk.emit(Op::LoadLocal(end_slot), Some(*span));
                        if *inclusive {
                            self.chunk.emit(Op::Le, Some(*span));
                        } else {
                            self.chunk.emit(Op::Lt, Some(*span));
                        }
                        let exit_jump = self.chunk.emit(Op::JumpIfFalse(0), Some(*span));

                        if *is_parallel {
                            self.chunk.emit(Op::BeginBranch, Some(*span));
                        }

                        self.loop_stack.push(LoopCtx {
                            continue_jumps: Vec::new(),
                            break_jumps: Vec::new(),
                        });
                        self.push_scope();
                        self.compile_stmts(body)?;
                        self.pop_scope(Some(*span));

                        if *is_parallel {
                            self.chunk.emit(Op::EndBranch, Some(*span));
                        }

                        let inc_target = self.chunk.ops.len();
                        let ctx = self
                            .loop_stack
                            .pop()
                            .ok_or_else(|| "internal compiler loop stack underflow".to_string())?;
                        for cj in ctx.continue_jumps {
                            self.chunk.patch_jump(cj, inc_target);
                        }

                        self.chunk.emit(Op::LoadLocal(idx_slot), Some(*span));
                        let one = self.chunk.add_const(Constant::Int("1".to_string()));
                        self.chunk.emit(Op::LoadConst(one), Some(*span));
                        self.chunk.emit(Op::Add, Some(*span));
                        self.chunk.emit(Op::InitLocal(idx_slot), Some(*span));
                        self.chunk.emit(Op::Jump(loop_start), Some(*span));

                        let exit_target = self.chunk.ops.len();
                        self.chunk.patch_jump(exit_jump, exit_target);
                        for bj in ctx.break_jumps {
                            self.chunk.patch_jump(bj, exit_target);
                        }
                    }
                    other => {
                        let col_name = self.fresh_name("col");
                        let col_slot = self.alloc_local(&col_name);
                        let i_name = self.fresh_name("i");
                        let i_slot = self.alloc_local(&i_name);
                        let var_slot = self.alloc_local(var);

                        self.compile_expr(other)?;
                        self.chunk.emit(Op::InitLocal(col_slot), Some(*span));
                        let zero = self.chunk.add_const(Constant::Int("0".to_string()));
                        self.chunk.emit(Op::LoadConst(zero), Some(*span));
                        self.chunk.emit(Op::InitLocal(i_slot), Some(*span));

                        let loop_start = self.chunk.ops.len();
                        self.chunk.emit(Op::LoadLocal(i_slot), Some(*span));
                        self.chunk.emit(Op::LoadLocal(col_slot), Some(*span));
                        let len_idx = self.chunk.add_string("length");
                        self.chunk.emit(
                            Op::CallMethod {
                                method_idx: len_idx,
                                arg_count: 0,
                            },
                            Some(*span),
                        );
                        self.chunk.emit(Op::Lt, Some(*span));
                        let exit_jump = self.chunk.emit(Op::JumpIfFalse(0), Some(*span));

                        self.chunk.emit(Op::LoadLocal(col_slot), Some(*span));
                        self.chunk.emit(Op::LoadLocal(i_slot), Some(*span));
                        self.chunk.emit(Op::IndexGet, Some(*span));
                        self.chunk.emit(Op::InitLocal(var_slot), Some(*span));

                        if *is_parallel {
                            self.chunk.emit(Op::BeginBranch, Some(*span));
                        }
                        self.loop_stack.push(LoopCtx {
                            continue_jumps: Vec::new(),
                            break_jumps: Vec::new(),
                        });
                        self.push_scope();
                        self.compile_stmts(body)?;
                        self.pop_scope(Some(*span));
                        if *is_parallel {
                            self.chunk.emit(Op::EndBranch, Some(*span));
                        }

                        let inc_target = self.chunk.ops.len();
                        let ctx = self
                            .loop_stack
                            .pop()
                            .ok_or_else(|| "internal compiler loop stack underflow".to_string())?;
                        for cj in ctx.continue_jumps {
                            self.chunk.patch_jump(cj, inc_target);
                        }

                        self.chunk.emit(Op::LoadLocal(i_slot), Some(*span));
                        let one = self.chunk.add_const(Constant::Int("1".to_string()));
                        self.chunk.emit(Op::LoadConst(one), Some(*span));
                        self.chunk.emit(Op::Add, Some(*span));
                        self.chunk.emit(Op::InitLocal(i_slot), Some(*span));
                        self.chunk.emit(Op::Jump(loop_start), Some(*span));

                        let exit_target = self.chunk.ops.len();
                        self.chunk.patch_jump(exit_jump, exit_target);
                        for bj in ctx.break_jumps {
                            self.chunk.patch_jump(bj, exit_target);
                        }
                    }
                }
                if *is_parallel {
                    self.chunk.emit(Op::SyncScope, Some(*span));
                }
            }
            Stmt::Match {
                scrutinee,
                arms,
                span,
            } => {
                self.compile_match(scrutinee, arms, *span)?;
            }
            Stmt::Assert {
                condition,
                message,
                span,
            } => {
                self.compile_expr(condition)?;
                let msg = self
                    .chunk
                    .add_string(message.as_deref().unwrap_or("assertion failed"));
                self.chunk.emit(Op::Assert(msg), Some(*span));
            }
            Stmt::Invariant {
                condition,
                message,
                span,
            } => {
                self.compile_expr(condition)?;
                let msg = self
                    .chunk
                    .add_string(message.as_deref().unwrap_or("invariant failed"));
                self.chunk.emit(Op::CheckInvariant(msg), Some(*span));
            }
            Stmt::Decreases { rank, span } => {
                self.compile_expr(rank)?;
                self.chunk.emit(Op::Pop, Some(*span));
            }
            Stmt::Break { span } => {
                let j = self.chunk.emit(Op::Jump(0), Some(*span));
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.break_jumps.push(j);
                }
            }
            Stmt::Continue { span } => {
                let j = self.chunk.emit(Op::Jump(0), Some(*span));
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.continue_jumps.push(j);
                }
            }
            Stmt::Sync { span } => {
                self.chunk.emit(Op::SyncScope, Some(*span));
            }
        }
        Ok(())
    }

    fn compile_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        span: SourceSpan,
    ) -> Result<(), String> {
        let scr_name = self.fresh_name("match");
        let scr_slot = self.alloc_local(&scr_name);
        self.compile_expr(scrutinee)?;
        self.chunk.emit(Op::InitLocal(scr_slot), Some(span));

        let mut end_jumps = Vec::new();
        for arm in arms {
            let mut fail_jump: Option<usize> = None;
            self.push_scope();
            match &arm.pattern {
                Pattern::Wildcard => {}
                Pattern::Ident { name } => {
                    let s = self.alloc_local(name);
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    self.chunk.emit(Op::InitLocal(s), Some(arm.span));
                }
                Pattern::None => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    self.chunk.emit(Op::IsNone, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                }
                Pattern::Some { inner } => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    self.chunk.emit(Op::IsSome, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                    if let Pattern::Ident { name } = inner.as_ref() {
                        let s = self.alloc_local(name);
                        self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                        self.chunk.emit(Op::Unwrap, Some(arm.span));
                        self.chunk.emit(Op::InitLocal(s), Some(arm.span));
                    }
                }
                Pattern::Ok { inner } => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    self.chunk.emit(Op::IsOk, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                    if let Pattern::Ident { name } = inner.as_ref() {
                        let s = self.alloc_local(name);
                        self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                        self.chunk.emit(Op::Unwrap, Some(arm.span));
                        self.chunk.emit(Op::InitLocal(s), Some(arm.span));
                    }
                }
                Pattern::Err { inner } => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    self.chunk.emit(Op::IsErr, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                    if let Pattern::Ident { name } = inner.as_ref() {
                        let s = self.alloc_local(name);
                        self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                        self.chunk.emit(Op::Unwrap, Some(arm.span));
                        self.chunk.emit(Op::InitLocal(s), Some(arm.span));
                    }
                }
                Pattern::IntLit { value } => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    let c = self.chunk.add_const(Constant::Int(value.clone()));
                    self.chunk.emit(Op::LoadConst(c), Some(arm.span));
                    self.chunk.emit(Op::Eq, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                }
                Pattern::BoolLit { value } => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    let c = self.chunk.add_const(Constant::Bool(*value));
                    self.chunk.emit(Op::LoadConst(c), Some(arm.span));
                    self.chunk.emit(Op::Eq, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                }
                Pattern::StringLit { value } => {
                    self.chunk.emit(Op::LoadLocal(scr_slot), Some(arm.span));
                    let c = self.chunk.add_const(Constant::Str(value.clone()));
                    self.chunk.emit(Op::LoadConst(c), Some(arm.span));
                    self.chunk.emit(Op::Eq, Some(arm.span));
                    fail_jump = Some(self.chunk.emit(Op::JumpIfFalse(0), Some(arm.span)));
                }
            }

            self.compile_stmts(&arm.body)?;
            self.pop_scope(Some(arm.span));
            end_jumps.push(self.chunk.emit(Op::Jump(0), Some(arm.span)));
            if let Some(fj) = fail_jump {
                let next = self.chunk.ops.len();
                self.chunk.patch_jump(fj, next);
            }
        }
        let end = self.chunk.ops.len();
        for j in end_jumps {
            self.chunk.patch_jump(j, end);
        }
        Ok(())
    }

    fn compile_assign(
        &mut self,
        target: &LValue,
        value: &Expr,
        span: SourceSpan,
    ) -> Result<(), String> {
        match target {
            LValue::Var { name, .. } => {
                self.compile_expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.emit(Op::StoreLocal(slot), Some(span));
                } else {
                    let slot = self.alloc_local(name);
                    self.chunk.emit(Op::InitLocal(slot), Some(span));
                }
            }
            LValue::Index { target, index, .. } => {
                self.compile_lvalue_read(target)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.chunk.emit(Op::IndexSet, Some(span));
            }
            LValue::Field { target, field, .. } => {
                self.compile_lvalue_read(target)?;
                self.compile_expr(value)?;
                let f_idx = self.chunk.add_string(field);
                self.chunk.emit(Op::FieldSet(f_idx), Some(span));
            }
        }
        Ok(())
    }

    fn compile_lvalue_read(&mut self, target: &LValue) -> Result<(), String> {
        match target {
            LValue::Var { name, span } => {
                let slot = self
                    .resolve_local(name)
                    .ok_or_else(|| format!("undefined variable '{name}'"))?;
                self.chunk.emit(Op::LoadLocal(slot), Some(*span));
            }
            LValue::Index {
                target,
                index,
                span,
            } => {
                self.compile_lvalue_read(target)?;
                self.compile_expr(index)?;
                self.chunk.emit(Op::IndexGet, Some(*span));
            }
            LValue::Field {
                target,
                field,
                span,
            } => {
                self.compile_lvalue_read(target)?;
                let f_idx = self.chunk.add_string(field);
                self.chunk.emit(Op::FieldGet(f_idx), Some(*span));
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit { value, span } => {
                let c = self.chunk.add_const(Constant::Int(value.clone()));
                self.chunk.emit(Op::LoadConst(c), Some(*span));
            }
            Expr::RealLit { value, span } => {
                let c = self.chunk.add_const(Constant::Real(*value));
                self.chunk.emit(Op::LoadConst(c), Some(*span));
            }
            Expr::BoolLit { value, span } => {
                let c = self.chunk.add_const(Constant::Bool(*value));
                self.chunk.emit(Op::LoadConst(c), Some(*span));
            }
            Expr::StringLit { value, span } => {
                let c = self.chunk.add_const(Constant::Str(value.clone()));
                self.chunk.emit(Op::LoadConst(c), Some(*span));
            }
            Expr::NoneLit { span } => {
                self.chunk.emit(Op::MakeNone, Some(*span));
            }
            Expr::InfinityLit { span } => {
                let c = self.chunk.add_const(Constant::Real(f64::INFINITY));
                self.chunk.emit(Op::LoadConst(c), Some(*span));
            }
            Expr::NanLit { span } => {
                let c = self.chunk.add_const(Constant::Real(f64::NAN));
                self.chunk.emit(Op::LoadConst(c), Some(*span));
            }
            Expr::Ident { name, span } => {
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.emit(Op::LoadLocal(slot), Some(*span));
                } else {
                    return Err(format!("undefined identifier '{name}' in VM compiler"));
                }
            }
            Expr::ListLit { elements, span } => {
                for el in elements {
                    self.compile_expr(el)?;
                }
                self.chunk
                    .emit(Op::MakeList(elements.len() as u16), Some(*span));
            }
            Expr::SetLit { elements, span } => {
                for el in elements {
                    self.compile_expr(el)?;
                }
                self.chunk
                    .emit(Op::MakeSet(elements.len() as u16), Some(*span));
            }
            Expr::MapLit { entries, span } => {
                for (k, v) in entries {
                    self.compile_expr(k)?;
                    self.compile_expr(v)?;
                }
                self.chunk
                    .emit(Op::MakeMap(entries.len() as u16), Some(*span));
            }
            Expr::TupleLit { elements, span } => {
                for el in elements {
                    self.compile_expr(el)?;
                }
                self.chunk
                    .emit(Op::MakeTuple(elements.len() as u16), Some(*span));
            }
            Expr::RecordLit { name, fields, span } => {
                for (fname, fval) in fields {
                    let f_c = self.chunk.add_const(Constant::Str(fname.clone()));
                    self.chunk.emit(Op::LoadConst(f_c), Some(*span));
                    self.compile_expr(fval)?;
                }
                let type_idx = self.chunk.add_string(name);
                self.chunk.emit(
                    Op::MakeRecord {
                        type_idx,
                        field_count: fields.len() as u16,
                    },
                    Some(*span),
                );
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                match op {
                    BinaryOp::And => {
                        self.compile_expr(left)?;
                        let skip = self.chunk.emit(Op::JumpIfFalse(0), Some(*span));
                        self.compile_expr(right)?;
                        let end_j = self.chunk.emit(Op::Jump(0), Some(*span));
                        let false_t = self.chunk.ops.len();
                        self.chunk.patch_jump(skip, false_t);
                        let f = self.chunk.add_const(Constant::Bool(false));
                        self.chunk.emit(Op::LoadConst(f), Some(*span));
                        let end_t = self.chunk.ops.len();
                        self.chunk.patch_jump(end_j, end_t);
                        return Ok(());
                    }
                    BinaryOp::Or => {
                        self.compile_expr(left)?;
                        let skip = self.chunk.emit(Op::JumpIfTrue(0), Some(*span));
                        self.compile_expr(right)?;
                        let end_j = self.chunk.emit(Op::Jump(0), Some(*span));
                        let true_t = self.chunk.ops.len();
                        self.chunk.patch_jump(skip, true_t);
                        let t = self.chunk.add_const(Constant::Bool(true));
                        self.chunk.emit(Op::LoadConst(t), Some(*span));
                        let end_t = self.chunk.ops.len();
                        self.chunk.patch_jump(end_j, end_t);
                        return Ok(());
                    }
                    BinaryOp::Implies => {
                        self.compile_expr(left)?;
                        self.chunk.emit(Op::Not, Some(*span));
                        let skip = self.chunk.emit(Op::JumpIfTrue(0), Some(*span));
                        self.compile_expr(right)?;
                        let end_j = self.chunk.emit(Op::Jump(0), Some(*span));
                        let true_t = self.chunk.ops.len();
                        self.chunk.patch_jump(skip, true_t);
                        let t = self.chunk.add_const(Constant::Bool(true));
                        self.chunk.emit(Op::LoadConst(t), Some(*span));
                        let end_t = self.chunk.ops.len();
                        self.chunk.patch_jump(end_j, end_t);
                        return Ok(());
                    }
                    _ => {}
                }

                self.compile_expr(left)?;
                self.compile_expr(right)?;
                let instr = match op {
                    BinaryOp::Add => Op::Add,
                    BinaryOp::Sub => Op::Sub,
                    BinaryOp::Mul => Op::Mul,
                    BinaryOp::Div => Op::Div,
                    BinaryOp::Mod => Op::Mod,
                    BinaryOp::Eq => Op::Eq,
                    BinaryOp::Neq => Op::Ne,
                    BinaryOp::Lt => Op::Lt,
                    BinaryOp::Le => Op::Le,
                    BinaryOp::Gt => Op::Gt,
                    BinaryOp::Ge => Op::Ge,
                    BinaryOp::Iff => Op::Eq,
                    BinaryOp::In => {
                        let m = self.chunk.add_string("contains");
                        // Stack has [left, right] -> swap not needed if handled via CallMethod? Wait: receiver is right!
                        // For `x in col`, compile_expr(right) then compile_expr(left) would put receiver first.
                        // Emit CallNamed "__in" to avoid stack order issue.
                        let _ = m;
                        let n = self.chunk.add_string("__in");
                        self.chunk.emit(
                            Op::CallNamed {
                                name_idx: n,
                                arg_count: 2,
                                writeback_id: None,
                            },
                            Some(*span),
                        );
                        return Ok(());
                    }
                    BinaryOp::NotIn => {
                        let n = self.chunk.add_string("__in");
                        self.chunk.emit(
                            Op::CallNamed {
                                name_idx: n,
                                arg_count: 2,
                                writeback_id: None,
                            },
                            Some(*span),
                        );
                        self.chunk.emit(Op::Not, Some(*span));
                        return Ok(());
                    }
                    BinaryOp::BitAnd => Op::BitAnd,
                    BinaryOp::BitOr => Op::BitOr,
                    BinaryOp::BitXor => Op::BitXor,
                    BinaryOp::Shl => Op::Shl,
                    BinaryOp::Shr => Op::Shr,
                    BinaryOp::And | BinaryOp::Or | BinaryOp::Implies => unreachable!(),
                };
                self.chunk.emit(instr, Some(*span));
            }
            Expr::Unary { op, operand, span } => {
                self.compile_expr(operand)?;
                let instr = match op {
                    UnaryOp::Neg => Op::Neg,
                    UnaryOp::Not => Op::Not,
                    UnaryOp::BitNot => Op::Not,
                };
                self.chunk.emit(instr, Some(*span));
            }
            Expr::Cast { expr, .. } => {
                self.compile_expr(expr)?;
            }
            Expr::Call { callee, args, span } => {
                if callee == "select" && args.len() == 3 {
                    for a in args {
                        self.compile_expr(&a.value)?;
                    }
                    self.chunk.emit(Op::SelectBranchless, Some(*span));
                    return Ok(());
                }
                if callee == "old" && args.len() == 1 {
                    if let Expr::Ident { name, .. } = &args[0].value {
                        if let Some(slot) = self.resolve_local(name) {
                            self.chunk.uses_old = true;
                            self.chunk.emit(Op::LoadOldLocal(slot), Some(*span));
                            return Ok(());
                        }
                    }
                }
                let mut wb_pairs = Vec::new();
                for (param_idx, a) in args.iter().enumerate() {
                    if a.is_mut {
                        if let Expr::Ident { name, .. } = &a.value {
                            if let Some(caller_slot) = self.resolve_local(name) {
                                wb_pairs.push((param_idx as u16, caller_slot));
                            }
                        }
                    }
                    self.compile_expr(&a.value)?;
                }
                let writeback_id = if wb_pairs.is_empty() {
                    None
                } else {
                    let id = self.chunk.writebacks.len() as u16;
                    self.chunk.writebacks.push(wb_pairs);
                    Some(id)
                };
                let name_idx = self.chunk.add_string(callee);
                self.chunk.emit(
                    Op::CallNamed {
                        name_idx,
                        arg_count: args.len() as u8,
                        writeback_id,
                    },
                    Some(*span),
                );
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
                span,
            } => {
                self.compile_expr(receiver)?;
                for a in args {
                    self.compile_expr(&a.value)?;
                }
                let method_idx = self.chunk.add_string(method);
                self.chunk.emit(
                    Op::CallMethod {
                        method_idx,
                        arg_count: args.len() as u8,
                    },
                    Some(*span),
                );
            }
            Expr::Index {
                target,
                index,
                span,
            } => {
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.chunk.emit(Op::IndexGet, Some(*span));
            }
            Expr::Slice {
                target,
                start,
                end,
                span,
            } => {
                self.compile_expr(target)?;
                if let Some(s) = start {
                    self.compile_expr(s)?;
                } else {
                    let z = self.chunk.add_const(Constant::Int("0".to_string()));
                    self.chunk.emit(Op::LoadConst(z), Some(*span));
                }
                if let Some(e) = end {
                    self.compile_expr(e)?;
                } else {
                    self.compile_expr(target)?;
                    let m = self.chunk.add_string("length");
                    self.chunk.emit(
                        Op::CallMethod {
                            method_idx: m,
                            arg_count: 0,
                        },
                        Some(*span),
                    );
                }
                let slice_name = self.chunk.add_string("__slice");
                self.chunk.emit(
                    Op::CallNamed {
                        name_idx: slice_name,
                        arg_count: 3,
                        writeback_id: None,
                    },
                    Some(*span),
                );
            }
            Expr::Field {
                target,
                field,
                span,
            } => {
                self.compile_expr(target)?;
                if field == "length" || field == "len" {
                    let m = self.chunk.add_string("length");
                    self.chunk.emit(
                        Op::CallMethod {
                            method_idx: m,
                            arg_count: 0,
                        },
                        Some(*span),
                    );
                } else {
                    let f_idx = self.chunk.add_string(field);
                    self.chunk.emit(Op::FieldGet(f_idx), Some(*span));
                }
            }
            Expr::Range {
                start, end, span, ..
            } => {
                self.compile_expr(start)?;
                self.compile_expr(end)?;
                let r_name = self.chunk.add_string("__range");
                self.chunk.emit(
                    Op::CallNamed {
                        name_idx: r_name,
                        arg_count: 2,
                        writeback_id: None,
                    },
                    Some(*span),
                );
            }
            Expr::Quantifier {
                is_forall,
                var,
                iterable,
                body,
                span,
            } => {
                self.compile_quantifier(*is_forall, var, iterable, body, *span)?;
            }
            Expr::Spawn { expr, span } => {
                self.chunk.emit(Op::ForkScope, Some(*span));
                self.chunk.emit(Op::BeginBranch, Some(*span));
                self.compile_expr(expr)?;
                self.chunk.emit(Op::EndBranch, Some(*span));
                self.chunk.emit(Op::SyncScope, Some(*span));
            }
        }
        Ok(())
    }

    fn compile_quantifier(
        &mut self,
        is_forall: bool,
        var: &str,
        iterable: &Expr,
        body: &Expr,
        span: SourceSpan,
    ) -> Result<(), String> {
        let res_name = self.fresh_name("quant_res");
        let res_slot = self.alloc_local(&res_name);
        let init_c = self.chunk.add_const(Constant::Bool(is_forall));
        self.chunk.emit(Op::LoadConst(init_c), Some(span));
        self.chunk.emit(Op::InitLocal(res_slot), Some(span));

        let idx_slot = self.alloc_local(var);
        match iterable {
            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                let end_name = self.fresh_name("qend");
                let end_slot = self.alloc_local(&end_name);
                self.compile_expr(start)?;
                self.chunk.emit(Op::InitLocal(idx_slot), Some(span));
                self.compile_expr(end)?;
                self.chunk.emit(Op::InitLocal(end_slot), Some(span));

                let loop_start = self.chunk.ops.len();
                self.chunk.emit(Op::LoadLocal(idx_slot), Some(span));
                self.chunk.emit(Op::LoadLocal(end_slot), Some(span));
                self.chunk
                    .emit(if *inclusive { Op::Le } else { Op::Lt }, Some(span));
                let exit_j = self.chunk.emit(Op::JumpIfFalse(0), Some(span));

                self.compile_expr(body)?;
                let check_j = if is_forall {
                    self.chunk.emit(Op::JumpIfTrue(0), Some(span))
                } else {
                    self.chunk.emit(Op::JumpIfFalse(0), Some(span))
                };
                let fail_c = self.chunk.add_const(Constant::Bool(!is_forall));
                self.chunk.emit(Op::LoadConst(fail_c), Some(span));
                self.chunk.emit(Op::InitLocal(res_slot), Some(span));
                let break_j = self.chunk.emit(Op::Jump(0), Some(span));

                let next_iter = self.chunk.ops.len();
                self.chunk.patch_jump(check_j, next_iter);
                self.chunk.emit(Op::LoadLocal(idx_slot), Some(span));
                let one = self.chunk.add_const(Constant::Int("1".to_string()));
                self.chunk.emit(Op::LoadConst(one), Some(span));
                self.chunk.emit(Op::Add, Some(span));
                self.chunk.emit(Op::InitLocal(idx_slot), Some(span));
                self.chunk.emit(Op::Jump(loop_start), Some(span));

                let exit_t = self.chunk.ops.len();
                self.chunk.patch_jump(exit_j, exit_t);
                self.chunk.patch_jump(break_j, exit_t);
            }
            other => {
                let col_name = self.fresh_name("qcol");
                let col_slot = self.alloc_local(&col_name);
                let i_name = self.fresh_name("qi");
                let i_slot = self.alloc_local(&i_name);
                self.compile_expr(other)?;
                self.chunk.emit(Op::InitLocal(col_slot), Some(span));
                let zero = self.chunk.add_const(Constant::Int("0".to_string()));
                self.chunk.emit(Op::LoadConst(zero), Some(span));
                self.chunk.emit(Op::InitLocal(i_slot), Some(span));

                let loop_start = self.chunk.ops.len();
                self.chunk.emit(Op::LoadLocal(i_slot), Some(span));
                self.chunk.emit(Op::LoadLocal(col_slot), Some(span));
                let len_m = self.chunk.add_string("length");
                self.chunk.emit(
                    Op::CallMethod {
                        method_idx: len_m,
                        arg_count: 0,
                    },
                    Some(span),
                );
                self.chunk.emit(Op::Lt, Some(span));
                let exit_j = self.chunk.emit(Op::JumpIfFalse(0), Some(span));

                self.chunk.emit(Op::LoadLocal(col_slot), Some(span));
                self.chunk.emit(Op::LoadLocal(i_slot), Some(span));
                self.chunk.emit(Op::IndexGet, Some(span));
                self.chunk.emit(Op::InitLocal(idx_slot), Some(span));

                self.compile_expr(body)?;
                let check_j = if is_forall {
                    self.chunk.emit(Op::JumpIfTrue(0), Some(span))
                } else {
                    self.chunk.emit(Op::JumpIfFalse(0), Some(span))
                };
                let fail_c = self.chunk.add_const(Constant::Bool(!is_forall));
                self.chunk.emit(Op::LoadConst(fail_c), Some(span));
                self.chunk.emit(Op::InitLocal(res_slot), Some(span));
                let break_j = self.chunk.emit(Op::Jump(0), Some(span));

                let next_iter = self.chunk.ops.len();
                self.chunk.patch_jump(check_j, next_iter);
                self.chunk.emit(Op::LoadLocal(i_slot), Some(span));
                let one = self.chunk.add_const(Constant::Int("1".to_string()));
                self.chunk.emit(Op::LoadConst(one), Some(span));
                self.chunk.emit(Op::Add, Some(span));
                self.chunk.emit(Op::InitLocal(i_slot), Some(span));
                self.chunk.emit(Op::Jump(loop_start), Some(span));

                let exit_t = self.chunk.ops.len();
                self.chunk.patch_jump(exit_j, exit_t);
                self.chunk.patch_jump(break_j, exit_t);
            }
        }

        self.chunk.emit(Op::LoadLocal(res_slot), Some(span));
        Ok(())
    }
}

/// Compile a typed UEAS `Program` into a `CompiledProgram` of bytecode chunks.
pub fn compile_program(program: &Program) -> Result<CompiledProgram, String> {
    let mut chunks = Vec::new();
    let mut chunk_index = HashMap::new();
    let mut tests = Vec::new();

    for algo in &program.algorithms {
        let params: Vec<String> = algo.parameters.iter().map(|p| p.name.clone()).collect();
        let mut_slots: Vec<u16> = algo
            .parameters
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_mut)
            .map(|(i, _)| i as u16)
            .collect();
        let mut comp = Compiler::new(
            algo.name.clone(),
            params,
            mut_slots,
            algo.constant_time,
            algo.complexity.clone(),
        );
        comp.compile_algorithm(algo)?;
        let idx = chunks.len();
        chunk_index.insert(algo.name.clone(), idx);
        chunks.push(comp.chunk);
    }

    for test in &program.tests {
        let mut comp = Compiler::new(
            format!("test::{}", test.name),
            Vec::new(),
            Vec::new(),
            false,
            ComplexitySpec::default(),
        );
        comp.compile_stmts(&test.body)?;
        let unit = comp.chunk.add_const(Constant::Unit);
        comp.chunk.emit(Op::LoadConst(unit), Some(test.span));
        comp.chunk.emit(Op::Return, Some(test.span));
        let idx = chunks.len();
        chunk_index.insert(comp.chunk.name.clone(), idx);
        tests.push((test.name.clone(), idx));
        chunks.push(comp.chunk);
    }

    Ok(CompiledProgram {
        chunks,
        chunk_index,
        tests,
        records: program.records.clone(),
    })
}
