//! Zero-I/O abstract module linker resolving `import` declarations via `ModuleResolver`.

use crate::hir::Program;
use crate::lower::lower_parse;
use std::collections::HashSet;
use ueas_syntax::{parse, SourceSpan};

/// Abstract module source resolver (keeps `ueas-hir` 100% free of host filesystem I/O).
pub trait ModuleResolver {
    /// Load the `.ueas` source text for a dotted module path like `"std.math.gcd"`.
    fn resolve(&self, dotted_path: &str) -> Result<String, String>;
}

/// No-op resolver for single-file / WASM / browser execution without external modules.
pub struct NullModuleResolver;

impl ModuleResolver for NullModuleResolver {
    fn resolve(&self, dotted_path: &str) -> Result<String, String> {
        Err(format!(
            "module '{dotted_path}' not found (no module resolver configured)"
        ))
    }
}

/// Composes a primary resolver with a fallback resolver (e.g., workspace FS -> `EmbeddedStdlibResolver`).
pub struct ChainedModuleResolver<A, B> {
    pub primary: A,
    pub fallback: B,
}

impl<A: ModuleResolver, B: ModuleResolver> ModuleResolver for ChainedModuleResolver<A, B> {
    fn resolve(&self, dotted_path: &str) -> Result<String, String> {
        self.primary
            .resolve(dotted_path)
            .or_else(|_| self.fallback.resolve(dotted_path))
    }
}

/// Link all `import` declarations recursively into a single merged `Program`,
/// detecting circular imports and preserving exact parsed ASTs.
pub fn link_program(
    entry: Program,
    resolver: &dyn ModuleResolver,
) -> Result<Program, Vec<(SourceSpan, String)>> {
    let mut visited = HashSet::new();
    let mut active_stack = HashSet::new();
    let mut merged = entry.clone();
    let mut errors = Vec::new();

    if let Some(m) = &entry.module {
        visited.insert(m.clone());
        active_stack.insert(m.clone());
    }

    for imp in &entry.imports {
        link_recursive(
            &imp.path,
            imp.span,
            resolver,
            &mut visited,
            &mut active_stack,
            &mut merged,
            &mut errors,
        );
    }

    if errors.is_empty() {
        Ok(merged)
    } else {
        Err(errors)
    }
}

fn link_recursive(
    path: &str,
    span: SourceSpan,
    resolver: &dyn ModuleResolver,
    visited: &mut HashSet<String>,
    active_stack: &mut HashSet<String>,
    merged: &mut Program,
    errors: &mut Vec<(SourceSpan, String)>,
) {
    if active_stack.contains(path) {
        errors.push((span, format!("circular import detected for '{path}'")));
        return;
    }
    if visited.contains(path) {
        return;
    }

    let src = match resolver.resolve(path) {
        Ok(s) => s,
        Err(e) => {
            errors.push((span, e));
            return;
        }
    };

    let parsed = parse(&src);
    if !parsed.is_ok() {
        for d in parsed.diagnostics {
            errors.push((d.span, format!("in imported '{path}': {}", d.message)));
        }
        return;
    }

    let sub = match lower_parse(&parsed) {
        Ok(p) => p,
        Err(errs) => {
            for e in errs {
                errors.push((span, format!("in imported '{path}': {e}")));
            }
            return;
        }
    };

    active_stack.insert(path.to_string());
    for sub_imp in &sub.imports {
        link_recursive(
            &sub_imp.path,
            sub_imp.span,
            resolver,
            visited,
            active_stack,
            merged,
            errors,
        );
    }
    active_stack.remove(path);
    visited.insert(path.to_string());

    for rec in sub.records {
        if !merged.records.iter().any(|r| r.name == rec.name) {
            merged.records.push(rec);
        }
    }
    for algo in sub.algorithms {
        if !merged.algorithms.iter().any(|a| a.name == algo.name) {
            merged.algorithms.push(algo);
        }
    }
}
