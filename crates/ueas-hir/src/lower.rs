//! Lowering from lossless `ueas-syntax` Rowan CST to typed canonical `ueas-hir`.

use crate::complexity::{parse_complexity_bound, ComplexitySpec};
use crate::hir::*;
use crate::types::{PrimitiveType, Type};
use ueas_syntax::{LineIndex, Parse, SourceSpan, SyntaxKind, SyntaxNode, SyntaxToken};

/// Spec version embedded from `spec/VERSION`.
pub const SPEC_VERSION: &str = include_str!("../../../spec/VERSION");
/// Engine version from Cargo workspace.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lower a parsed CST into a `Program` HIR, returning any lowering errors.
pub fn lower_parse(parse: &Parse) -> Result<Program, Vec<String>> {
    let root = parse.syntax();
    let ctx = LowerCtx {
        lines: &parse.line_index,
        errors: Vec::new(),
    };
    ctx.lower_program(&root)
}

struct LowerCtx<'a> {
    lines: &'a LineIndex,
    errors: Vec<String>,
}

impl<'a> LowerCtx<'a> {
    fn span_of(&self, node: &SyntaxNode) -> SourceSpan {
        let range = node.text_range();
        self.lines
            .span(usize::from(range.start()), usize::from(range.end()))
    }

    fn lower_program(mut self, root: &SyntaxNode) -> Result<Program, Vec<String>> {
        let mut module = None;
        let mut imports = Vec::new();
        let mut records = Vec::new();
        let mut algorithms = Vec::new();
        let mut tests = Vec::new();
        let mut benchmarks = Vec::new();

        for child in root.children() {
            match child.kind() {
                SyntaxKind::MODULE_DECL => {
                    if let Some(di) = child
                        .children()
                        .find(|n| n.kind() == SyntaxKind::DOTTED_IDENT)
                    {
                        module = Some(collect_dotted_ident(&di));
                    }
                }
                SyntaxKind::IMPORT_DECL => {
                    if let Some(di) = child
                        .children()
                        .find(|n| n.kind() == SyntaxKind::DOTTED_IDENT)
                    {
                        imports.push(ImportDecl {
                            path: collect_dotted_ident(&di),
                            span: self.span_of(&child),
                        });
                    }
                }
                SyntaxKind::RECORD_DECL => {
                    if let Some(rec) = self.lower_record(&child) {
                        records.push(rec);
                    }
                }
                SyntaxKind::ALGORITHM_DECL => {
                    if let Some(algo) = self.lower_algorithm(&child) {
                        algorithms.push(algo);
                    }
                }
                SyntaxKind::TEST_DECL => {
                    if let Some(t) = self.lower_test(&child) {
                        tests.push(t);
                    }
                }
                SyntaxKind::BENCHMARK_DECL => {
                    if let Some(b) = self.lower_benchmark(&child) {
                        benchmarks.push(b);
                    }
                }
                _ => {}
            }
        }

        if self.errors.is_empty() {
            Ok(Program {
                spec_version: SPEC_VERSION.trim().to_string(),
                engine_version: ENGINE_VERSION.to_string(),
                module,
                imports,
                records,
                algorithms,
                tests,
                benchmarks,
            })
        } else {
            Err(self.errors)
        }
    }

    fn lower_record(&mut self, node: &SyntaxNode) -> Option<RecordDecl> {
        let name = first_ident_text(node)?;
        let type_params = node
            .children()
            .find(|n| n.kind() == SyntaxKind::TYPE_PARAMS)
            .map(|n| self.lower_type_params(&n))
            .unwrap_or_default();
        let mut fields = Vec::new();
        for f in node
            .children()
            .filter(|n| n.kind() == SyntaxKind::FIELD_DECL)
        {
            let fname = first_ident_text(&f).unwrap_or_else(|| "_".to_string());
            let fty = f
                .children()
                .find(|n| matches!(n.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE))
                .map(|n| self.lower_type(&n))
                .unwrap_or(Type::Infer);
            fields.push(FieldDecl {
                name: fname,
                ty: fty,
                span: self.span_of(&f),
            });
        }
        Some(RecordDecl {
            name,
            type_params,
            fields,
            span: self.span_of(node),
        })
    }

    fn lower_type_params(&self, node: &SyntaxNode) -> Vec<TypeParam> {
        let mut out = Vec::new();
        for tp in node
            .children()
            .filter(|n| n.kind() == SyntaxKind::TYPE_PARAM)
        {
            let mut idents = ident_tokens(&tp);
            if let Some(name) = idents.first().map(|t| t.text().to_string()) {
                let mut bounds = Vec::new();
                for b in idents.drain(1..) {
                    match b.text() {
                        "Eq" => bounds.push(TraitBound::Eq),
                        "Ord" => bounds.push(TraitBound::Ord),
                        "Hash" => bounds.push(TraitBound::Hash),
                        "Numeric" => bounds.push(TraitBound::Numeric),
                        _ => {}
                    }
                }
                out.push(TypeParam { name, bounds });
            }
        }
        out
    }

    fn lower_algorithm(&mut self, node: &SyntaxNode) -> Option<AlgorithmDecl> {
        let name = first_ident_text(node)?;
        let type_params = node
            .children()
            .find(|n| n.kind() == SyntaxKind::TYPE_PARAMS)
            .map(|n| self.lower_type_params(&n))
            .unwrap_or_default();

        let mut constant_time = false;
        let mut hardware_profile = None;
        for dec in node
            .children()
            .filter(|n| n.kind() == SyntaxKind::DECORATOR)
        {
            if let Some(dname) = first_ident_text(&dec) {
                if dname.eq_ignore_ascii_case("ConstantTime") {
                    constant_time = true;
                } else if dname.eq_ignore_ascii_case("HardwareProfile") {
                    hardware_profile = Some(self.lower_hw_profile(&dec));
                }
            }
        }

        let mut parameters = Vec::new();
        if let Some(pl) = node.children().find(|n| n.kind() == SyntaxKind::PARAM_LIST) {
            for p in pl.children().filter(|n| n.kind() == SyntaxKind::PARAM) {
                parameters.push(self.lower_param(&p));
            }
        }

        let mut return_type = node
            .children()
            .find(|n| matches!(n.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE))
            .map(|n| self.lower_type(&n))
            .unwrap_or(Type::void());

        let mut preconditions = Vec::new();
        for req in node
            .children()
            .filter(|n| n.kind() == SyntaxKind::REQUIRE_BLOCK)
        {
            for child in req.children() {
                if child.kind() == SyntaxKind::PARAM {
                    // Legacy v3.0 `Require: x: Real, y: Real` parameter type bindings!
                    let rp = self.lower_param(&child);
                    if let Some(existing) = parameters.iter_mut().find(|p| p.name == rp.name) {
                        if existing.ty == Type::Infer {
                            existing.ty = rp.ty;
                            existing.is_mut |= rp.is_mut;
                        }
                    } else {
                        parameters.push(rp);
                    }
                } else if is_expr_node(child.kind()) {
                    preconditions.push(self.lower_expr(&child));
                }
            }
        }

        let mut postcondition_binding = None;
        let mut postconditions = Vec::new();
        for ens in node
            .children()
            .filter(|n| n.kind() == SyntaxKind::ENSURE_BLOCK)
        {
            if let Some(binding_tok) = ens
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| t.kind() == SyntaxKind::IDENT)
            {
                postcondition_binding = Some(binding_tok.text().to_string());
            }
            for child in ens.children() {
                if matches!(
                    child.kind(),
                    SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE
                ) {
                    if return_type == Type::void() {
                        return_type = self.lower_type(&child);
                    }
                } else if is_expr_node(child.kind()) {
                    postconditions.push(self.lower_expr(&child));
                }
            }
        }

        let complexity = self.lower_complexity(node);

        let body = node
            .children()
            .find(|n| n.kind() == SyntaxKind::BLOCK)
            .map(|b| self.lower_block(&b))
            .unwrap_or_default();

        Some(AlgorithmDecl {
            name,
            type_params,
            parameters,
            return_type,
            preconditions,
            postcondition_binding,
            postconditions,
            complexity,
            hardware_profile,
            constant_time,
            declassifications: Vec::new(),
            body,
            span: self.span_of(node),
        })
    }

    fn lower_hw_profile(&self, dec: &SyntaxNode) -> HardwareProfile {
        let mut prof = HardwareProfile::default();
        for hp in dec.children().filter(|n| n.kind() == SyntaxKind::HW_PARAM) {
            let idents = ident_tokens(&hp);
            let key = idents.first().map(|t| t.text()).unwrap_or("");
            let unit = idents.get(1).map(|t| t.text()).unwrap_or("B");
            let num: u64 = hp
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| t.kind() == SyntaxKind::INT_LIT)
                .and_then(|t| t.text().parse().ok())
                .unwrap_or(0);
            let mult = match unit.to_ascii_uppercase().as_str() {
                "MB" => 1024 * 1024,
                "KB" => 1024,
                _ => 1,
            };
            let bytes = num.saturating_mul(mult);
            match key.to_ascii_uppercase().as_str() {
                "L1" => prof.l1_bytes = bytes,
                "L2" => prof.l2_bytes = bytes,
                "L3" => prof.l3_bytes = bytes,
                "CACHELINE" | "LINE" => prof.line_bytes = bytes.max(16),
                _ => {}
            }
        }
        prof
    }

    fn lower_param(&self, node: &SyntaxNode) -> Param {
        let is_mut = node
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .any(|t| t.kind() == SyntaxKind::KW_MUT);
        let name = first_ident_text(node).unwrap_or_else(|| "_".to_string());
        let ty = node
            .children()
            .find(|n| matches!(n.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE))
            .map(|n| self.lower_type(&n))
            .unwrap_or(Type::Infer);
        Param {
            name,
            ty,
            is_mut,
            span: self.span_of(node),
        }
    }

    fn lower_complexity(&mut self, algo: &SyntaxNode) -> ComplexitySpec {
        let mut spec = ComplexitySpec {
            time: None,
            space: None,
            work: None,
            span_bound: None,
            expected: None,
            bindings: Vec::new(),
        };
        for cb in algo
            .children()
            .filter(|n| n.kind() == SyntaxKind::COMPLEXITY_BLOCK)
        {
            let is_memory_block = cb
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .any(|t| t.kind() == SyntaxKind::KW_MEMORY);

            for entry in cb
                .children()
                .filter(|n| n.kind() == SyntaxKind::COMPLEXITY_ENTRY)
            {
                let tokens: Vec<_> = entry
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .filter(|t| !t.kind().is_trivia())
                    .collect();

                let mut key: Option<String> = None;
                let has_key = tokens
                    .get(1)
                    .is_some_and(|t| matches!(t.kind(), SyntaxKind::COLON | SyntaxKind::ASSIGN_EQ));
                if has_key {
                    key = tokens.first().map(|t| t.text().to_string());
                }

                // Check for `where` bindings inside entry
                let where_pos = tokens.iter().position(|t| t.kind() == SyntaxKind::KW_WHERE);
                if let Some(wp) = where_pos {
                    let _ = wp;
                    let mut expr_iter = entry.children().filter(|n| is_expr_node(n.kind()));
                    let binding_idents: Vec<_> = tokens
                        .iter()
                        .skip(wp + 1)
                        .filter(|t| t.kind() == SyntaxKind::IDENT)
                        .collect();
                    // Each `var = expr` produces 1 expr child after the main expr (if any)
                    for (idx, eid) in binding_idents.iter().enumerate() {
                        let _ = idx;
                        if let Some(ex_node) = expr_iter.next() {
                            spec.bindings
                                .push((eid.text().to_string(), self.lower_expr(&ex_node)));
                        }
                    }
                }

                // Extract formula text (from STRING_LIT or O(...) tokens)
                let raw_formula = if let Some(lit) = entry
                    .children()
                    .find(|n| n.kind() == SyntaxKind::LITERAL_EXPR)
                {
                    unquote_str(&lit.text().to_string())
                } else {
                    let start = if has_key { 2 } else { 0 };
                    let end = where_pos.unwrap_or(tokens.len());
                    tokens[start..end]
                        .iter()
                        .map(|t| t.text())
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                if is_memory_block {
                    if let Ok(b) = parse_complexity_bound(&raw_formula) {
                        spec.space = Some(b);
                    }
                    continue;
                }

                match key.as_deref().map(str::to_ascii_lowercase).as_deref() {
                    None | Some("time") => match parse_complexity_bound(&raw_formula) {
                        Ok(b) => spec.time = Some(b),
                        Err(e) => self.errors.push(e),
                    },
                    Some("space") | Some("memory") => match parse_complexity_bound(&raw_formula) {
                        Ok(b) => spec.space = Some(b),
                        Err(e) => self.errors.push(e),
                    },
                    Some("work") => {
                        if let Ok(b) = parse_complexity_bound(&raw_formula) {
                            spec.work = Some(b);
                        }
                    }
                    Some("span") => {
                        if let Ok(b) = parse_complexity_bound(&raw_formula) {
                            spec.span_bound = Some(b);
                        }
                    }
                    Some("expected") => {
                        if let Ok(b) = parse_complexity_bound(&raw_formula) {
                            spec.expected = Some(b);
                        }
                    }
                    Some(other_var) => {
                        if let Some(ex) = entry.children().find(|n| is_expr_node(n.kind())) {
                            spec.bindings
                                .push((other_var.to_string(), self.lower_expr(&ex)));
                        }
                    }
                }
            }
        }
        if spec.time.is_none() {
            spec.time = Some(crate::complexity::ComplexityBound {
                raw: "O(1)".to_string(),
                class: crate::complexity::AsymptoticClass::Constant,
            });
        }
        spec
    }

    fn lower_test(&mut self, node: &SyntaxNode) -> Option<TestDecl> {
        let name = node
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::STRING_LIT)
            .map(|t| unquote_str(t.text()))
            .unwrap_or_else(|| "unnamed".to_string());
        let body = node
            .children()
            .find(|n| n.kind() == SyntaxKind::BLOCK)
            .map(|b| self.lower_block(&b))
            .unwrap_or_default();
        Some(TestDecl {
            name,
            body,
            span: self.span_of(node),
        })
    }

    fn lower_benchmark(&mut self, node: &SyntaxNode) -> Option<BenchmarkDecl> {
        let name = node
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::STRING_LIT)
            .map(|t| unquote_str(t.text()))
            .unwrap_or_else(|| "benchmark".to_string());
        let scales: Vec<u64> = node
            .children()
            .filter(|n| n.kind() == SyntaxKind::LITERAL_EXPR)
            .filter_map(|n| n.text().to_string().trim().parse().ok())
            .collect();
        let generator_param = first_ident_text(node).unwrap_or_else(|| "N".to_string());
        let generator_body = node
            .children()
            .filter(|n| is_expr_node(n.kind()) && n.kind() != SyntaxKind::LITERAL_EXPR)
            .last()
            .map(|n| self.lower_expr(&n));
        Some(BenchmarkDecl {
            name,
            scales,
            generator_param,
            generator_body,
            span: self.span_of(node),
        })
    }

    fn lower_type(&self, node: &SyntaxNode) -> Type {
        let name_node = if node.kind() == SyntaxKind::GENERIC_TYPE {
            node.children().find(|n| n.kind() == SyntaxKind::NAMED_TYPE)
        } else {
            Some(node.clone())
        };
        let raw_name = name_node
            .as_ref()
            .and_then(first_ident_text)
            .unwrap_or_else(|| "Int".to_string());

        let args: Vec<Type> = node
            .children()
            .find(|n| n.kind() == SyntaxKind::TYPE_ARG_LIST)
            .map(|al| {
                al.children()
                    .filter(|c| {
                        matches!(c.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE)
                    })
                    .map(|c| self.lower_type(&c))
                    .collect()
            })
            .unwrap_or_default();

        match raw_name.as_str() {
            "Int" | "Integer" => Type::int(),
            "Int64" => Type::int64(),
            "UInt64" => Type::Primitive {
                prim: PrimitiveType::UInt64,
            },
            "UInt32" => Type::Primitive {
                prim: PrimitiveType::UInt32,
            },
            "Byte" => Type::Primitive {
                prim: PrimitiveType::Byte,
            },
            "Real" => Type::real(),
            "Bool" | "Boolean" => Type::bool(),
            "String" => Type::string(),
            "Void" => Type::void(),
            "List" => Type::List {
                elem: Box::new(args.into_iter().next().unwrap_or(Type::Infer)),
            },
            "Set" => Type::Set {
                elem: Box::new(args.into_iter().next().unwrap_or(Type::Infer)),
            },
            "Map" => {
                let mut it = args.into_iter();
                Type::Map {
                    key: Box::new(it.next().unwrap_or(Type::Infer)),
                    val: Box::new(it.next().unwrap_or(Type::Infer)),
                }
            }
            "Graph" | "DirectedGraph" | "UndirectedGraph" => {
                let mut it = args.into_iter();
                Type::Graph {
                    node: Box::new(it.next().unwrap_or(Type::int())),
                    weight: Box::new(it.next().unwrap_or(Type::real())),
                    directed: raw_name == "DirectedGraph",
                }
            }
            "Matrix" => Type::Matrix {
                elem: Box::new(args.into_iter().last().unwrap_or(Type::real())),
            },
            "Option" => Type::Option {
                inner: Box::new(args.into_iter().next().unwrap_or(Type::Infer)),
            },
            "Result" => {
                let mut it = args.into_iter();
                Type::Result {
                    ok: Box::new(it.next().unwrap_or(Type::Infer)),
                    err: Box::new(it.next().unwrap_or(Type::string())),
                }
            }
            "Tuple" => Type::Tuple { elems: args },
            "Secret" => Type::Secret {
                inner: Box::new(args.into_iter().next().unwrap_or(Type::int64())),
            },
            other => Type::Record {
                name: other.to_string(),
                type_args: args,
            },
        }
    }

    fn lower_block(&mut self, block: &SyntaxNode) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for child in block.children() {
            if let Some(s) = self.lower_stmt(&child) {
                stmts.push(s);
            }
        }
        stmts
    }

    fn lower_stmt(&mut self, node: &SyntaxNode) -> Option<Stmt> {
        let span = self.span_of(node);
        match node.kind() {
            SyntaxKind::LET_STMT => {
                let is_mut = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .any(|t| t.kind() == SyntaxKind::KW_MUT);
                let name = first_ident_text(node)?;
                let ty = node
                    .children()
                    .find(|n| matches!(n.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE))
                    .map(|n| self.lower_type(&n));
                let init = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                Some(Stmt::Let {
                    name,
                    is_mut,
                    ty,
                    init,
                    span,
                })
            }
            SyntaxKind::CONST_STMT => {
                let name = first_ident_text(node)?;
                let ty = node
                    .children()
                    .find(|n| matches!(n.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE))
                    .map(|n| self.lower_type(&n));
                let init = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                Some(Stmt::Const {
                    name,
                    ty,
                    init,
                    span,
                })
            }
            SyntaxKind::ASSIGN_STMT => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let lhs_node = exprs.next()?;
                let rhs_node = exprs.next()?;
                let target = self.lower_lvalue(&lhs_node)?;
                let value = self.lower_expr(&rhs_node);
                Some(Stmt::Assign {
                    target,
                    value,
                    span,
                })
            }
            SyntaxKind::EXPR_STMT => {
                let expr_node = node.children().find(|n| is_expr_node(n.kind()))?;
                Some(Stmt::Expr {
                    expr: self.lower_expr(&expr_node),
                    span,
                })
            }
            SyntaxKind::RETURN_STMT => {
                let value = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n));
                Some(Stmt::Return { value, span })
            }
            SyntaxKind::IF_STMT => {
                let condition = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                let then_body = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::BLOCK)
                    .map(|b| self.lower_block(&b))
                    .unwrap_or_default();
                let mut else_if_branches = Vec::new();
                for eb in node
                    .children()
                    .filter(|n| n.kind() == SyntaxKind::ELSE_IF_BRANCH)
                {
                    let c = eb
                        .children()
                        .find(|n| is_expr_node(n.kind()))
                        .map(|n| self.lower_expr(&n))
                        .unwrap_or(Expr::BoolLit { value: false, span });
                    let b = eb
                        .children()
                        .find(|n| n.kind() == SyntaxKind::BLOCK)
                        .map(|bl| self.lower_block(&bl))
                        .unwrap_or_default();
                    else_if_branches.push((c, b));
                }
                let else_body = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::ELSE_BRANCH)
                    .and_then(|eb| eb.children().find(|n| n.kind() == SyntaxKind::BLOCK))
                    .map(|b| self.lower_block(&b));
                Some(Stmt::If {
                    condition,
                    then_body,
                    else_if_branches,
                    else_body,
                    span,
                })
            }
            SyntaxKind::WHILE_STMT => {
                let condition = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                let raw_body = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::BLOCK)
                    .map(|b| self.lower_block(&b))
                    .unwrap_or_default();
                let mut invariants = Vec::new();
                let mut decreases = None;
                let mut body = Vec::new();
                for s in raw_body {
                    match s {
                        Stmt::Invariant { condition, .. } => invariants.push(condition),
                        Stmt::Decreases { rank, .. } => decreases = Some(rank),
                        other => body.push(other),
                    }
                }
                Some(Stmt::While {
                    condition,
                    invariants,
                    decreases,
                    body,
                    span,
                })
            }
            SyntaxKind::FOR_STMT => {
                let is_parallel = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .any(|t| t.kind() == SyntaxKind::KW_PARALLEL);
                let var = first_ident_text(node)?;
                let iterable = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                let raw_body = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::BLOCK)
                    .map(|b| self.lower_block(&b))
                    .unwrap_or_default();
                let mut invariants = Vec::new();
                let mut body = Vec::new();
                for s in raw_body {
                    match s {
                        Stmt::Invariant { condition, .. } => invariants.push(condition),
                        other => body.push(other),
                    }
                }
                Some(Stmt::For {
                    var,
                    iterable,
                    is_parallel,
                    invariants,
                    body,
                    span,
                })
            }
            SyntaxKind::MATCH_STMT => {
                let scrutinee = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                let mut arms = Vec::new();
                for arm in node
                    .children()
                    .filter(|n| n.kind() == SyntaxKind::MATCH_ARM)
                {
                    let pat_expr = arm
                        .children()
                        .find(|n| is_expr_node(n.kind()))
                        .map(|n| self.lower_expr(&n));
                    let pattern = pat_expr
                        .as_ref()
                        .map(expr_to_pattern)
                        .unwrap_or(Pattern::Wildcard);
                    let body = arm
                        .children()
                        .find(|n| n.kind() == SyntaxKind::BLOCK)
                        .map(|b| self.lower_block(&b))
                        .unwrap_or_default();
                    arms.push(MatchArm {
                        pattern,
                        body,
                        span: self.span_of(&arm),
                    });
                }
                Some(Stmt::Match {
                    scrutinee,
                    arms,
                    span,
                })
            }
            SyntaxKind::ASSERT_STMT => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let condition = exprs.next().map(|n| self.lower_expr(&n))?;
                let message = exprs.next().and_then(|n| match self.lower_expr(&n) {
                    Expr::StringLit { value, .. } => Some(value),
                    _ => None,
                });
                Some(Stmt::Assert {
                    condition,
                    message,
                    span,
                })
            }
            SyntaxKind::INVARIANT_STMT => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let condition = exprs.next().map(|n| self.lower_expr(&n))?;
                let message = exprs.next().and_then(|n| match self.lower_expr(&n) {
                    Expr::StringLit { value, .. } => Some(value),
                    _ => None,
                });
                Some(Stmt::Invariant {
                    condition,
                    message,
                    span,
                })
            }
            SyntaxKind::DECREASES_STMT => {
                let rank = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))?;
                Some(Stmt::Decreases { rank, span })
            }
            SyntaxKind::BREAK_STMT => Some(Stmt::Break { span }),
            SyntaxKind::CONTINUE_STMT => Some(Stmt::Continue { span }),
            SyntaxKind::SYNC_STMT => Some(Stmt::Sync { span }),
            _ => None,
        }
    }

    fn lower_lvalue(&mut self, node: &SyntaxNode) -> Option<LValue> {
        let span = self.span_of(node);
        match node.kind() {
            SyntaxKind::IDENT_EXPR => Some(LValue::Var {
                name: node.text().to_string().trim().to_string(),
                span,
            }),
            SyntaxKind::INDEX_EXPR => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let target_node = exprs.next()?;
                let idx_node = exprs.next()?;
                let target = Box::new(self.lower_lvalue(&target_node)?);
                let index = self.lower_expr(&idx_node);
                Some(LValue::Index {
                    target,
                    index,
                    span,
                })
            }
            SyntaxKind::FIELD_EXPR => {
                let target_node = node.children().find(|n| is_expr_node(n.kind()))?;
                let field = first_ident_text(node)?;
                let target = Box::new(self.lower_lvalue(&target_node)?);
                Some(LValue::Field {
                    target,
                    field,
                    span,
                })
            }
            _ => {
                self.errors.push(format!(
                    "line {}: invalid assignment target",
                    span.start_line
                ));
                None
            }
        }
    }

    fn lower_expr(&self, node: &SyntaxNode) -> Expr {
        let span = self.span_of(node);
        match node.kind() {
            SyntaxKind::LITERAL_EXPR => {
                let tok = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .find(|t| !t.kind().is_trivia());
                match tok.as_ref().map(|t| (t.kind(), t.text())) {
                    Some((SyntaxKind::INT_LIT, s)) => Expr::IntLit {
                        value: s.to_string(),
                        span,
                    },
                    Some((SyntaxKind::REAL_LIT, s)) => Expr::RealLit {
                        value: s.parse::<f64>().unwrap_or(0.0),
                        span,
                    },
                    Some((SyntaxKind::STRING_LIT, s)) => Expr::StringLit {
                        value: unquote_str(s),
                        span,
                    },
                    Some((SyntaxKind::KW_TRUE, _)) => Expr::BoolLit { value: true, span },
                    Some((SyntaxKind::KW_FALSE, _)) => Expr::BoolLit { value: false, span },
                    Some((SyntaxKind::KW_NONE, _)) => Expr::NoneLit { span },
                    Some((SyntaxKind::KW_INFINITY, _)) => Expr::InfinityLit { span },
                    Some((SyntaxKind::KW_NAN, _)) => Expr::NanLit { span },
                    _ => Expr::NoneLit { span },
                }
            }
            SyntaxKind::IDENT_EXPR => Expr::Ident {
                name: node.text().to_string().trim().to_string(),
                span,
            },
            SyntaxKind::PAREN_EXPR => node
                .children()
                .find(|n| is_expr_node(n.kind()))
                .map(|n| self.lower_expr(&n))
                .unwrap_or(Expr::NoneLit { span }),
            SyntaxKind::TUPLE_EXPR => Expr::TupleLit {
                elements: node
                    .children()
                    .filter(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .collect(),
                span,
            },
            SyntaxKind::LIST_EXPR => Expr::ListLit {
                elements: node
                    .children()
                    .filter(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .collect(),
                span,
            },
            SyntaxKind::SET_OR_MAP_EXPR => {
                let has_map_entries = node.children().any(|n| n.kind() == SyntaxKind::MAP_ENTRY);
                if has_map_entries {
                    let mut entries = Vec::new();
                    for ch in node.children() {
                        if ch.kind() == SyntaxKind::MAP_ENTRY {
                            let mut ex = ch.children().filter(|n| is_expr_node(n.kind()));
                            if let (Some(k), Some(v)) = (ex.next(), ex.next()) {
                                entries.push((self.lower_expr(&k), self.lower_expr(&v)));
                            }
                        }
                    }
                    Expr::MapLit { entries, span }
                } else {
                    Expr::SetLit {
                        elements: node
                            .children()
                            .filter(|n| is_expr_node(n.kind()))
                            .map(|n| self.lower_expr(&n))
                            .collect(),
                        span,
                    }
                }
            }
            SyntaxKind::RECORD_LIT_EXPR => {
                let name = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::IDENT_EXPR)
                    .map(|n| n.text().to_string().trim().to_string())
                    .unwrap_or_default();
                let mut fields = Vec::new();
                for fi in node
                    .children()
                    .filter(|n| n.kind() == SyntaxKind::RECORD_FIELD_INIT)
                {
                    let fname = first_ident_text(&fi).unwrap_or_default();
                    let val = fi
                        .children()
                        .find(|n| is_expr_node(n.kind()))
                        .map(|n| self.lower_expr(&n))
                        .unwrap_or(Expr::NoneLit { span });
                    fields.push((fname, val));
                }
                Expr::RecordLit { name, fields, span }
            }
            SyntaxKind::UNARY_EXPR => {
                let op_tok = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .find(|t| !t.kind().is_trivia());
                let op = match op_tok.map(|t| t.kind()) {
                    Some(SyntaxKind::KW_NOT) => UnaryOp::Not,
                    Some(SyntaxKind::TILDE) => UnaryOp::BitNot,
                    _ => UnaryOp::Neg,
                };
                let operand = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                Expr::Unary {
                    op,
                    operand: Box::new(operand),
                    span,
                }
            }
            SyntaxKind::BINARY_EXPR => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let left = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let right = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let op_toks: Vec<_> = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .filter(|t| !t.kind().is_trivia())
                    .collect();
                let op = match op_toks.first().map(|t| t.kind()) {
                    Some(SyntaxKind::PLUS) => BinaryOp::Add,
                    Some(SyntaxKind::MINUS) => BinaryOp::Sub,
                    Some(SyntaxKind::STAR) => BinaryOp::Mul,
                    Some(SyntaxKind::SLASH) => BinaryOp::Div,
                    Some(SyntaxKind::KW_MOD) | Some(SyntaxKind::PERCENT) => BinaryOp::Mod,
                    Some(SyntaxKind::EQ) => BinaryOp::Eq,
                    Some(SyntaxKind::NEQ) => BinaryOp::Neq,
                    Some(SyntaxKind::LT) => BinaryOp::Lt,
                    Some(SyntaxKind::LE) => BinaryOp::Le,
                    Some(SyntaxKind::GT) => BinaryOp::Gt,
                    Some(SyntaxKind::GE) => BinaryOp::Ge,
                    Some(SyntaxKind::KW_AND) => BinaryOp::And,
                    Some(SyntaxKind::KW_OR) => BinaryOp::Or,
                    Some(SyntaxKind::IMPLIES) => BinaryOp::Implies,
                    Some(SyntaxKind::IFF) => BinaryOp::Iff,
                    Some(SyntaxKind::KW_IN) => BinaryOp::In,
                    Some(SyntaxKind::KW_NOT) => BinaryOp::NotIn,
                    Some(SyntaxKind::AMP) => BinaryOp::BitAnd,
                    Some(SyntaxKind::PIPE) => BinaryOp::BitOr,
                    Some(SyntaxKind::CARET) => BinaryOp::BitXor,
                    Some(SyntaxKind::LSHIFT) => BinaryOp::Shl,
                    Some(SyntaxKind::RSHIFT) => BinaryOp::Shr,
                    _ => BinaryOp::Add,
                };
                Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                }
            }
            SyntaxKind::CAST_EXPR => {
                let expr = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let target_type = node
                    .children()
                    .find(|n| matches!(n.kind(), SyntaxKind::NAMED_TYPE | SyntaxKind::GENERIC_TYPE))
                    .map(|n| self.lower_type(&n))
                    .unwrap_or(Type::int());
                Expr::Cast {
                    expr: Box::new(expr),
                    target_type,
                    span,
                }
            }
            SyntaxKind::CALL_EXPR => {
                let callee = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::IDENT_EXPR)
                    .map(|n| n.text().to_string().trim().to_string())
                    .unwrap_or_default();
                let args = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::ARG_LIST)
                    .map(|al| self.lower_arg_list(&al))
                    .unwrap_or_default();
                Expr::Call { callee, args, span }
            }
            SyntaxKind::METHOD_CALL_EXPR => {
                let receiver = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let method = first_ident_text(node).unwrap_or_default();
                let args = node
                    .children()
                    .find(|n| n.kind() == SyntaxKind::ARG_LIST)
                    .map(|al| self.lower_arg_list(&al))
                    .unwrap_or_default();
                Expr::MethodCall {
                    receiver: Box::new(receiver),
                    method,
                    args,
                    span,
                }
            }
            SyntaxKind::INDEX_EXPR => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let target = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let index = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                Expr::Index {
                    target: Box::new(target),
                    index: Box::new(index),
                    span,
                }
            }
            SyntaxKind::SLICE_EXPR => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let target = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let start = exprs.next().map(|n| Box::new(self.lower_expr(&n)));
                let end = exprs.next().map(|n| Box::new(self.lower_expr(&n)));
                Expr::Slice {
                    target: Box::new(target),
                    start,
                    end,
                    span,
                }
            }
            SyntaxKind::FIELD_EXPR => {
                let target = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let field = first_ident_text(node).unwrap_or_default();
                Expr::Field {
                    target: Box::new(target),
                    field,
                    span,
                }
            }
            SyntaxKind::RANGE_EXPR => {
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let start = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let end = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let inclusive = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .any(|t| t.kind() == SyntaxKind::DOT_DOT_EQ);
                Expr::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                    inclusive,
                    span,
                }
            }
            SyntaxKind::QUANTIFIER_EXPR => {
                let is_forall = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .any(|t| t.kind() == SyntaxKind::KW_FORALL);
                let var = first_ident_text(node).unwrap_or_else(|| "_".to_string());
                let mut exprs = node.children().filter(|n| is_expr_node(n.kind()));
                let iterable = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                let body = exprs
                    .next()
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::BoolLit { value: true, span });
                Expr::Quantifier {
                    is_forall,
                    var,
                    iterable: Box::new(iterable),
                    body: Box::new(body),
                    span,
                }
            }
            SyntaxKind::SPAWN_EXPR => {
                let expr = node
                    .children()
                    .find(|n| is_expr_node(n.kind()))
                    .map(|n| self.lower_expr(&n))
                    .unwrap_or(Expr::NoneLit { span });
                Expr::Spawn {
                    expr: Box::new(expr),
                    span,
                }
            }
            _ => Expr::NoneLit { span },
        }
    }

    fn lower_arg_list(&self, al: &SyntaxNode) -> Vec<CallArg> {
        let mut args = Vec::new();
        let mut pending_mut = false;
        for item in al.children_with_tokens() {
            if let Some(t) = item.as_token() {
                if t.kind() == SyntaxKind::KW_MUT {
                    pending_mut = true;
                }
            } else if let Some(n) = item.as_node() {
                if is_expr_node(n.kind()) {
                    args.push(CallArg {
                        is_mut: pending_mut,
                        value: self.lower_expr(n),
                    });
                    pending_mut = false;
                }
            }
        }
        args
    }
}

fn expr_to_pattern(expr: &Expr) -> Pattern {
    match expr {
        Expr::Ident { name, .. } if name == "_" => Pattern::Wildcard,
        Expr::Ident { name, .. } => Pattern::Ident { name: name.clone() },
        Expr::NoneLit { .. } => Pattern::None,
        Expr::IntLit { value, .. } => Pattern::IntLit {
            value: value.clone(),
        },
        Expr::BoolLit { value, .. } => Pattern::BoolLit { value: *value },
        Expr::StringLit { value, .. } => Pattern::StringLit {
            value: value.clone(),
        },
        Expr::Call { callee, args, .. } => {
            let inner = args
                .first()
                .map(|a| expr_to_pattern(&a.value))
                .unwrap_or(Pattern::Wildcard);
            match callee.to_ascii_lowercase().as_str() {
                "some" => Pattern::Some {
                    inner: Box::new(inner),
                },
                "ok" => Pattern::Ok {
                    inner: Box::new(inner),
                },
                "err" => Pattern::Err {
                    inner: Box::new(inner),
                },
                _ => Pattern::Wildcard,
            }
        }
        _ => Pattern::Wildcard,
    }
}

fn collect_dotted_ident(node: &SyntaxNode) -> String {
    ident_tokens(node)
        .into_iter()
        .map(|t| t.text().to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn ident_tokens(node: &SyntaxNode) -> Vec<SyntaxToken> {
    node.children_with_tokens()
        .filter_map(|e| e.into_token())
        .filter(|t| {
            matches!(
                t.kind(),
                SyntaxKind::IDENT | SyntaxKind::KW_SCALES | SyntaxKind::KW_GENERATOR
            )
        })
        .collect()
}

fn first_ident_text(node: &SyntaxNode) -> Option<String> {
    ident_tokens(node).first().map(|t| t.text().to_string())
}

fn unquote_str(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

fn is_expr_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LITERAL_EXPR
            | SyntaxKind::IDENT_EXPR
            | SyntaxKind::PAREN_EXPR
            | SyntaxKind::LIST_EXPR
            | SyntaxKind::SET_OR_MAP_EXPR
            | SyntaxKind::TUPLE_EXPR
            | SyntaxKind::RECORD_LIT_EXPR
            | SyntaxKind::BINARY_EXPR
            | SyntaxKind::UNARY_EXPR
            | SyntaxKind::CAST_EXPR
            | SyntaxKind::CALL_EXPR
            | SyntaxKind::INDEX_EXPR
            | SyntaxKind::SLICE_EXPR
            | SyntaxKind::FIELD_EXPR
            | SyntaxKind::METHOD_CALL_EXPR
            | SyntaxKind::RANGE_EXPR
            | SyntaxKind::QUANTIFIER_EXPR
            | SyntaxKind::SPAWN_EXPR
    )
}
