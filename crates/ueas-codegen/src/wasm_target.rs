//! Universal WebAssembly Binary (`.wasm`) & WIT Interface (`.wit`) Code Generator.
//!
//! Powered by pure-Rust `wasm-encoder` (`#![forbid(unsafe_code)]`, `wasm32`-compatible).

use crate::common::{CodeGenerator, GeneratedArtifact};
use ueas_hir::hir::{BinaryOp, Expr, Program, Stmt};
use ueas_hir::types::{PrimitiveType, Type};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

pub struct WasmTarget;
pub struct WitTarget;

impl CodeGenerator for WasmTarget {
    fn target_id(&self) -> &'static str {
        "wasm"
    }
    fn file_extension(&self) -> &'static str {
        "wasm.hex"
    }
    fn generate(&self, program: &Program) -> Result<GeneratedArtifact, String> {
        let bytes = encode_wasm_module(program)?;
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        Ok(GeneratedArtifact {
            target: "wasm".to_string(),
            extension: "wasm.hex".to_string(),
            code: hex,
            test_code: Some(emit_wit_interface(program)),
        })
    }
}

impl CodeGenerator for WitTarget {
    fn target_id(&self) -> &'static str {
        "wit"
    }
    fn file_extension(&self) -> &'static str {
        "wit"
    }
    fn generate(&self, program: &Program) -> Result<GeneratedArtifact, String> {
        Ok(GeneratedArtifact {
            target: "wit".to_string(),
            extension: "wit".to_string(),
            code: emit_wit_interface(program),
            test_code: None,
        })
    }
}

/// Encode pure integer/boolean UEAS algorithms directly to a valid WebAssembly binary module.
pub fn encode_wasm_module(program: &Program) -> Result<Vec<u8>, String> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    let mut funcs = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut codes = CodeSection::new();

    for (idx, algo) in program.algorithms.iter().enumerate() {
        let param_tys: Vec<ValType> = algo
            .parameters
            .iter()
            .map(|p| wasm_val_type(&p.ty))
            .collect();
        let ret_tys: Vec<ValType> = if matches!(
            algo.return_type,
            Type::Primitive {
                prim: PrimitiveType::Void
            }
        ) {
            Vec::new()
        } else {
            vec![wasm_val_type(&algo.return_type)]
        };
        types.ty().function(param_tys, ret_tys);
        funcs.function(idx as u32);
        exports.export(&algo.name, ExportKind::Func, idx as u32);

        let mut locals_map: Vec<String> = algo.parameters.iter().map(|p| p.name.clone()).collect();
        collect_locals(&algo.body, &mut locals_map);
        let extra_locals = locals_map.len().saturating_sub(algo.parameters.len());
        let mut func = Function::new([(extra_locals as u32, ValType::I64)]);
        for stmt in &algo.body {
            emit_wasm_stmt(stmt, &locals_map, &mut func)?;
        }
        if !matches!(
            algo.return_type,
            Type::Primitive {
                prim: PrimitiveType::Void
            }
        ) {
            func.instruction(&Instruction::I64Const(0));
        }
        func.instruction(&Instruction::End);
        codes.function(&func);
    }

    module.section(&types);
    module.section(&funcs);
    module.section(&exports);
    module.section(&codes);
    Ok(module.finish())
}

fn wasm_val_type(ty: &Type) -> ValType {
    match ty {
        Type::Primitive {
            prim: PrimitiveType::Real,
        } => ValType::F64,
        _ => ValType::I64,
    }
}

fn collect_locals(stmts: &[Stmt], out: &mut Vec<String>) {
    for s in stmts {
        match s {
            Stmt::Let { name, .. } | Stmt::Const { name, .. } => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            Stmt::If {
                then_body,
                else_if_branches,
                else_body,
                ..
            } => {
                collect_locals(then_body, out);
                for (_, eb) in else_if_branches {
                    collect_locals(eb, out);
                }
                if let Some(eb) = else_body {
                    collect_locals(eb, out);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => collect_locals(body, out),
            _ => {}
        }
    }
}

fn emit_wasm_stmt(stmt: &Stmt, locals: &[String], func: &mut Function) -> Result<(), String> {
    match stmt {
        Stmt::Let { name, init, .. } | Stmt::Const { name, init, .. } => {
            emit_wasm_expr(init, locals, func)?;
            let idx = locals
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| format!("WasmTarget: unknown local '{name}'"))?;
            func.instruction(&Instruction::LocalSet(idx as u32));
        }
        Stmt::Assign {
            target: ueas_hir::hir::LValue::Var { name, .. },
            value,
            ..
        } => {
            emit_wasm_expr(value, locals, func)?;
            let idx = locals
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| format!("WasmTarget: unknown local '{name}'"))?;
            func.instruction(&Instruction::LocalSet(idx as u32));
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value {
                emit_wasm_expr(v, locals, func)?;
            }
            func.instruction(&Instruction::Return);
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            emit_wasm_expr(condition, locals, func)?;
            func.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
            for s in then_body {
                emit_wasm_stmt(s, locals, func)?;
            }
            if let Some(eb) = else_body {
                func.instruction(&Instruction::Else);
                for s in eb {
                    emit_wasm_stmt(s, locals, func)?;
                }
            }
            func.instruction(&Instruction::End);
        }
        Stmt::While {
            condition, body, ..
        } => {
            func.instruction(&Instruction::Block(wasm_encoder::BlockType::Empty));
            func.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
            emit_wasm_expr(condition, locals, func)?;
            func.instruction(&Instruction::I32Eqz);
            func.instruction(&Instruction::BrIf(1));
            for s in body {
                emit_wasm_stmt(s, locals, func)?;
            }
            func.instruction(&Instruction::Br(0));
            func.instruction(&Instruction::End);
            func.instruction(&Instruction::End);
        }
        _ => {}
    }
    Ok(())
}

fn emit_wasm_expr(expr: &Expr, locals: &[String], func: &mut Function) -> Result<(), String> {
    match expr {
        Expr::IntLit { value, .. } => {
            let v: i64 = value.parse().unwrap_or(0);
            func.instruction(&Instruction::I64Const(v));
        }
        Expr::BoolLit { value, .. } => {
            func.instruction(&Instruction::I32Const(i32::from(*value)));
        }
        Expr::Ident { name, .. } => {
            let idx = locals
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| format!("WasmTarget: unknown ident '{name}'"))?;
            func.instruction(&Instruction::LocalGet(idx as u32));
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            emit_wasm_expr(left, locals, func)?;
            emit_wasm_expr(right, locals, func)?;
            match op {
                BinaryOp::Add => func.instruction(&Instruction::I64Add),
                BinaryOp::Sub => func.instruction(&Instruction::I64Sub),
                BinaryOp::Mul => func.instruction(&Instruction::I64Mul),
                BinaryOp::Div => func.instruction(&Instruction::I64DivS),
                BinaryOp::Mod => func.instruction(&Instruction::I64RemS),
                BinaryOp::Eq => func.instruction(&Instruction::I64Eq),
                BinaryOp::Neq => func.instruction(&Instruction::I64Ne),
                BinaryOp::Lt => func.instruction(&Instruction::I64LtS),
                BinaryOp::Le => func.instruction(&Instruction::I64LeS),
                BinaryOp::Gt => func.instruction(&Instruction::I64GtS),
                BinaryOp::Ge => func.instruction(&Instruction::I64GeS),
                _ => {
                    return Err(format!("WasmTarget: unsupported binary op {op:?}"));
                }
            };
        }
        _ => {
            func.instruction(&Instruction::I64Const(0));
        }
    }
    Ok(())
}

/// Generate a WebAssembly Interface Types (`.wit`) contract from a UEAS `Program`.
pub fn emit_wit_interface(program: &Program) -> String {
    let pkg = program
        .module
        .as_deref()
        .unwrap_or("ueas.module")
        .replace('.', "-");
    let mut out = format!("package ueas:{pkg}@2.0.0;\n\ninterface exports {{\n");
    for algo in &program.algorithms {
        let params: Vec<String> = algo
            .parameters
            .iter()
            .map(|p| format!("{}: {}", p.name.replace('_', "-"), wit_type(&p.ty)))
            .collect();
        let ret = if matches!(
            algo.return_type,
            Type::Primitive {
                prim: PrimitiveType::Void
            }
        ) {
            String::new()
        } else {
            format!(" -> {}", wit_type(&algo.return_type))
        };
        out.push_str(&format!(
            "    {}: func({}){ret};\n",
            algo.name.to_ascii_lowercase().replace('_', "-"),
            params.join(", ")
        ));
    }
    out.push_str("}\n");
    out
}

fn wit_type(ty: &Type) -> String {
    match ty {
        Type::Primitive {
            prim: PrimitiveType::Int | PrimitiveType::Int64,
        } => "s64".to_string(),
        Type::Primitive {
            prim: PrimitiveType::UInt64,
        } => "u64".to_string(),
        Type::Primitive {
            prim: PrimitiveType::Bool,
        } => "bool".to_string(),
        Type::Primitive {
            prim: PrimitiveType::Real,
        } => "f64".to_string(),
        Type::Primitive {
            prim: PrimitiveType::String,
        } => "string".to_string(),
        Type::List { elem } => format!("list<{}>", wit_type(elem)),
        Type::Option { inner } => format!("option<{}>", wit_type(inner)),
        _ => "s64".to_string(),
    }
}
