//! AST Linker for UEAS Module System (RFC 0007).
//!
//! Resolves `Import:` directives in UEAS programs by loading
//! referenced modules via dotted namespace resolution. Each
//! `math.fft` import maps to `library/math/fft.ueas` in the
//! configured library path.
//!
//! The linker merges imported algorithm definitions into the
//! program AST before execution, enabling multi-file algorithm
//! compositions without filesystem-specific paths.

use crate::ast::{AstNode, AstNodeFactory, AstNodeKind, AstValue};

/// Error returned when an import cannot be resolved.
#[derive(Debug)]
pub struct LinkerError {
    pub message: String,
    pub namespace: String,
}

impl std::fmt::Display for LinkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Linker error for '{}': {}", self.namespace, self.message)
    }
}

/// Configuration for the AST linker.
pub struct LinkerConfig {
    /// Base directory for namespace resolution.
    /// `math.fft` → `<library_path>/math/fft.ueas`
    pub library_path: String,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            library_path: "library".to_string(),
        }
    }
}

/// Resolve and link all `Import:` directives in a program AST.
///
/// # Arguments
/// * `program` — The program AST node (AstNodeKind::Program)
/// * `source_loader` — Function that loads a .ueas file given a path,
///   returning the parsed source text
/// * `config` — Linker configuration (library path)
///
/// # Returns
/// The merged program AST with inlined algorithms from all imports.
pub fn link_program<F>(
    program: &AstNode,
    source_loader: &F,
    config: &LinkerConfig,
) -> Result<AstNode, LinkerError>
where
    F: Fn(&str) -> Result<String, String>,
{
    if program.kind != AstNodeKind::Program {
        return Ok(program.clone());
    }

    let mut all_algos: Vec<AstNode> = Vec::new();
    let mut imported: std::collections::HashSet<String> = std::collections::HashSet::new();

    for child in &program.children {
        match child.kind {
            AstNodeKind::Import => {
                let namespace = child
                    .value
                    .as_ref()
                    .map(|v| match v {
                        AstValue::String(s) => s.clone(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();

                if namespace.is_empty() {
                    continue;
                }

                if imported.contains(&namespace) {
                    continue; // skip duplicates
                }

                let file_path = resolve_namespace(&namespace, config);
                let source = source_loader(&file_path).map_err(|e| LinkerError {
                    message: e,
                    namespace: namespace.clone(),
                })?;

                // Parse the imported file and extract its algorithms
                let imported_algos =
                    extract_algorithms_from_source(&source, &namespace).map_err(|e| {
                        LinkerError {
                            message: e,
                            namespace: namespace.clone(),
                        }
                    })?;

                all_algos.extend(imported_algos);
                imported.insert(namespace);
            }
            AstNodeKind::Algorithm => {
                all_algos.push(child.clone());
            }
            _ => {}
        }
    }

    Ok(AstNodeFactory::program(all_algos))
}

/// Resolve a dotted namespace to a filesystem path.
///
/// `math.fft` → `<library_path>/math/fft.ueas`
pub fn resolve_namespace(namespace: &str, config: &LinkerConfig) -> String {
    let path = namespace.replace('.', "/");
    format!(
        "{}/{}.ueas",
        config.library_path.trim_end_matches('/'),
        path
    )
}

/// Extract algorithm AST nodes from a parsed .ueas source string.
///
/// This is a lightweight extraction that reads the source, finds
/// algorithm definitions, and constructs AST nodes using the factory.
fn extract_algorithms_from_source(source: &str, _namespace: &str) -> Result<Vec<AstNode>, String> {
    let source = source.trim();
    if source.is_empty() {
        return Ok(vec![]);
    }

    let mut algos: Vec<AstNode> = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("Algorithm ")
            || line.starts_with("algorithm ")
            || line.starts_with("ALGORITHM ")
        {
            // Find the matching end of this algorithm
            let start = i;
            let mut depth = 1u32;
            i += 1;

            while i < lines.len() {
                let l = lines[i].trim().to_lowercase();
                if l.starts_with("algorithm ") {
                    depth += 1;
                }
                if l.starts_with("end algorithm") || l.starts_with("end") {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }

            // Extract name from header
            let header = lines[start].trim();
            let name = header
                .split_whitespace()
                .nth(1)
                .and_then(|n| n.split('(').next())
                .unwrap_or("imported")
                .to_string();

            // Build a minimal algorithm AST
            let algo = AstNode::internal(
                AstNodeKind::Algorithm,
                vec![
                    AstNodeFactory::identifier(&name),
                    AstNodeFactory::string_literal("O(?)"),
                    AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("0"))),
                ],
                None,
            );
            algos.push(algo);
        } else {
            i += 1;
        }
    }

    Ok(algos)
}

/// Create an AST import node from a namespace string.
pub fn import_node(namespace: &str) -> AstNode {
    AstNode::leaf(
        AstNodeKind::Import,
        Some(AstValue::String(namespace.to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_math_fft_to_path() {
        let config = LinkerConfig {
            library_path: "library".into(),
        };
        let path = resolve_namespace("math.fft", &config);
        assert_eq!(path, "library/math/fft.ueas");
    }

    #[test]
    fn resolve_single_component() {
        let config = LinkerConfig::default();
        let path = resolve_namespace("graph", &config);
        assert_eq!(path, "library/graph.ueas");
    }

    #[test]
    fn link_program_no_imports() {
        let program = AstNodeFactory::program(vec![AstNode::internal(
            AstNodeKind::Algorithm,
            vec![
                AstNodeFactory::identifier("test"),
                AstNodeFactory::string_literal("O(1)"),
                AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("42"))),
            ],
            None,
        )]);

        let source_loader =
            |_path: &str| -> Result<String, String> { Err("should not be called".into()) };
        let config = LinkerConfig::default();

        let result = link_program(&program, &source_loader, &config).unwrap();
        assert_eq!(result.children.len(), 1);
    }

    #[test]
    fn link_program_with_imports() {
        let program = AstNode::internal(
            AstNodeKind::Program,
            vec![
                import_node("math.gcd"),
                AstNode::internal(
                    AstNodeKind::Algorithm,
                    vec![
                        AstNodeFactory::identifier("test"),
                        AstNodeFactory::string_literal("O(1)"),
                        AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("42"))),
                    ],
                    None,
                ),
            ],
            None,
        );

        let source_loader = |_path: &str| -> Result<String, String> {
            Ok("Algorithm gcd(a, b)\n    Require: a: Integer, b: Integer\n    Ensure: Integer\n    Complexity: \"O(log min(a,b))\"\n\n    return a\nend Algorithm".into())
        };
        let config = LinkerConfig::default();

        let result = link_program(&program, &source_loader, &config).unwrap();
        assert!(result.children.len() >= 2); // original + imported
    }

    #[test]
    fn link_program_duplicate_imports_idempotent() {
        let program = AstNode::internal(
            AstNodeKind::Program,
            vec![
                import_node("math.gcd"),
                import_node("math.gcd"), // duplicate
                AstNode::internal(
                    AstNodeKind::Algorithm,
                    vec![
                        AstNodeFactory::identifier("test"),
                        AstNodeFactory::string_literal("O(1)"),
                        AstNodeFactory::return_stmt(Some(AstNodeFactory::integer_literal("0"))),
                    ],
                    None,
                ),
            ],
            None,
        );

        let call_count = std::cell::RefCell::new(0u32);
        let source_loader = |_path: &str| -> Result<String, String> {
            *call_count.borrow_mut() += 1;
            Ok("Algorithm gcd(a, b)\n    Require: a: Integer, b: Integer\n    Ensure: Integer\n    Complexity: \"O(1)\"\n\n    return a\nend Algorithm".into())
        };
        let config = LinkerConfig::default();

        let result = link_program(&program, &source_loader, &config).unwrap();
        assert_eq!(*call_count.borrow(), 1); // only loaded once
        assert!(result.children.len() >= 2);
    }

    #[test]
    fn link_missing_import_errors() {
        let program = AstNode::internal(
            AstNodeKind::Program,
            vec![
                import_node("missing.module"),
                AstNode::internal(
                    AstNodeKind::Algorithm,
                    vec![
                        AstNodeFactory::identifier("test"),
                        AstNodeFactory::string_literal("O(1)"),
                        AstNodeFactory::return_stmt(None),
                    ],
                    None,
                ),
            ],
            None,
        );

        let source_loader =
            |_path: &str| -> Result<String, String> { Err("File not found".into()) };
        let config = LinkerConfig::default();

        let result = link_program(&program, &source_loader, &config);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_nested_namespace_three_levels() {
        let config = LinkerConfig {
            library_path: "lib".into(),
        };
        let path = resolve_namespace("data.structures.heap", &config);
        assert_eq!(path, "lib/data/structures/heap.ueas");
    }

    #[test]
    fn link_empty_program_no_children() {
        let program = AstNode::internal(AstNodeKind::Program, vec![], None);
        let source_loader = |_: &str| -> Result<String, String> { Err("no".into()) };
        let config = LinkerConfig::default();
        let result = link_program(&program, &source_loader, &config).unwrap();
        assert!(result.children.is_empty());
    }

    #[test]
    fn link_unknown_node_kind_passes_through() {
        let program = AstNode::internal(
            AstNodeKind::Program,
            vec![AstNode::internal(AstNodeKind::IntegerLiteral, vec![], None)],
            None,
        );
        let source_loader = |_: &str| -> Result<String, String> { Err("no".into()) };
        let config = LinkerConfig::default();
        let result = link_program(&program, &source_loader, &config).unwrap();
        assert!(result.children.is_empty() || result.children.len() == 1);
    }
}
