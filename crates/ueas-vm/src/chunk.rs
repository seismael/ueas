//! Compiled bytecode chunks and program container.

use crate::op::Op;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ueas_hir::complexity::ComplexitySpec;
use ueas_hir::hir::RecordDecl;
use ueas_syntax::SourceSpan;

/// Constant pool entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constant {
    Unit,
    Bool(bool),
    Int(String),
    Real(f64),
    Char(char),
    Str(String),
}

/// Compiled bytecode chunk for a single UEAS algorithm or `@Test` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub name: String,
    pub params: Vec<String>,
    pub mut_param_slots: Vec<u16>,
    pub ops: Vec<Op>,
    pub spans: Vec<Option<SourceSpan>>,
    pub constants: Vec<Constant>,
    pub strings: Vec<String>,
    pub local_names: Vec<String>,
    pub is_constant_time: bool,
    pub uses_old: bool,
    pub complexity: ComplexitySpec,
    pub body_start_pc: usize,
    pub writebacks: Vec<Vec<(u16, u16)>>,
}

impl Chunk {
    /// Create a new empty chunk.
    pub fn new(
        name: String,
        params: Vec<String>,
        is_constant_time: bool,
        complexity: ComplexitySpec,
    ) -> Self {
        let local_names = params.clone();
        Self {
            name,
            params,
            mut_param_slots: Vec::new(),
            ops: Vec::new(),
            spans: Vec::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            local_names,
            is_constant_time,
            uses_old: false,
            complexity,
            body_start_pc: 0,
            writebacks: Vec::new(),
        }
    }

    /// Emit an instruction with its source span, returning its PC.
    pub fn emit(&mut self, op: Op, span: Option<SourceSpan>) -> usize {
        let pc = self.ops.len();
        self.ops.push(op);
        self.spans.push(span);
        pc
    }

    /// Patch a jump target at `jump_pc`.
    pub fn patch_jump(&mut self, jump_pc: usize, target: usize) {
        if let Some(Op::Jump(t) | Op::JumpIfFalse(t) | Op::JumpIfTrue(t)) =
            self.ops.get_mut(jump_pc)
        {
            *t = target;
        }
    }

    /// Intern a constant into the chunk's constant pool.
    pub fn add_const(&mut self, c: Constant) -> u16 {
        if let Some(pos) = self.constants.iter().position(|x| x == &c) {
            return pos as u16;
        }
        let idx = self.constants.len();
        self.constants.push(c);
        idx as u16
    }

    /// Intern a string identifier into the chunk's string table.
    pub fn add_string(&mut self, s: &str) -> u16 {
        if let Some(pos) = self.strings.iter().position(|x| x == s) {
            return pos as u16;
        }
        let idx = self.strings.len();
        self.strings.push(s.to_string());
        idx as u16
    }
}

/// Compiled UEAS program ready for execution in `VmSession`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledProgram {
    pub chunks: Vec<Chunk>,
    pub chunk_index: HashMap<String, usize>,
    pub tests: Vec<(String, usize)>,
    pub records: Vec<RecordDecl>,
}

impl CompiledProgram {
    /// Look up a chunk index by algorithm or test name.
    pub fn get_chunk(&self, name: &str) -> Option<&Chunk> {
        self.chunk_index.get(name).and_then(|i| self.chunks.get(*i))
    }
}
