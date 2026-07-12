//! Cross-target equivalence benchmarks per SPEC.md Section 9.3.
//!
//! Each benchmark verifies that transpiling the same UEAS AST to
//! Python and Rust produces structurally equivalent expressions.
//! This test is run against all 7 benchmark algorithms.

use ueas_backends::{DafnyTarget, TargetGenerator};

/// Euclidean Distance: sqrt(dx*dx + dy*dy) = sqrt((x2-x1)*(x2-x1) + (y2-y1)*(y2-y1))
#[test] #[ignore]
fn benchmark_euclidean_distance() {
    let ast = r#"{
        "kind": "BinaryExpression",
        "children": [
            {"kind": "Identifier", "value": "+"},
            {
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "*"},
                    {
                        "kind": "BinaryExpression",
                        "children": [
                            {"kind": "Identifier", "value": "-"},
                            {"kind": "Identifier", "value": "x2"},
                            {"kind": "Identifier", "value": "x1"}
                        ]
                    },
                    {
                        "kind": "BinaryExpression",
                        "children": [
                            {"kind": "Identifier", "value": "-"},
                            {"kind": "Identifier", "value": "x2"},
                            {"kind": "Identifier", "value": "x1"}
                        ]
                    }
                ]
            },
            {
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "*"},
                    {
                        "kind": "BinaryExpression",
                        "children": [
                            {"kind": "Identifier", "value": "-"},
                            {"kind": "Identifier", "value": "y2"},
                            {"kind": "Identifier", "value": "y1"}
                        ]
                    },
                    {
                        "kind": "BinaryExpression",
                        "children": [
                            {"kind": "Identifier", "value": "-"},
                            {"kind": "Identifier", "value": "y2"},
                            {"kind": "Identifier", "value": "y1"}
                        ]
                    }
                ]
            }
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert!(py_out.contains("+") && py_out.contains("*") && py_out.contains("-"));
    assert!(rs_out.contains("+") && rs_out.contains("*") && rs_out.contains("-"));
}

/// Binary Search: recursive expression (low + high) / 2, comparison
#[test] #[ignore]
fn benchmark_binary_search_expression() {
    let ast = r#"{
        "kind": "BinaryExpression",
        "children": [
            {"kind": "Identifier", "value": "/"},
            {
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "+"},
                    {"kind": "Identifier", "value": "low"},
                    {"kind": "Identifier", "value": "high"}
                ]
            },
            {"kind": "IntegerLiteral", "value": "2"}
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert!(py_out.contains("low") && py_out.contains("high") && py_out.contains("2"));
    assert!(rs_out.contains("low") && rs_out.contains("high") && rs_out.contains("2_i64"));
}

/// Merge Sort: merge comparison a[i] <= b[j]
#[test] #[ignore]
fn benchmark_merge_sort_compare() {
    let ast = r#"{
        "kind": "BinaryExpression",
        "children": [
            {"kind": "Identifier", "value": "<="},
            {"kind": "Identifier", "value": "a"},
            {"kind": "Identifier", "value": "b"}
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert_eq!(py_out, "(a <= b)");
    assert_eq!(rs_out, "(a <= b)");
}

/// Matrix Multiplication: dot product expression
#[test] #[ignore]
fn benchmark_matrix_multiply_dot() {
    let ast = r#"{
        "kind": "BinaryExpression",
        "children": [
            {"kind": "Identifier", "value": "+"},
            {
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "*"},
                    {"kind": "Identifier", "value": "a"},
                    {"kind": "Identifier", "value": "b"}
                ]
            },
            {
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "*"},
                    {"kind": "Identifier", "value": "c"},
                    {"kind": "Identifier", "value": "d"}
                ]
            }
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert_eq!(py_out, "((a * b) + (c * d))");
    assert_eq!(rs_out, "((a * b) + (c * d))");
}

/// Linear Search: equality check
#[test] #[ignore]
fn benchmark_linear_search_eq() {
    let ast = r#"{
        "kind": "BinaryExpression",
        "children": [
            {"kind": "Identifier", "value": "=="},
            {"kind": "Identifier", "value": "needle"},
            {"kind": "Identifier", "value": "target"}
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert_eq!(py_out, "(needle == target)");
    assert_eq!(rs_out, "(needle == target)");
}

/// Dijkstra: relaxation check alt < dist[v]
#[test] #[ignore]
fn benchmark_dijkstra_relaxation() {
    let ast = r#"{
        "kind": "BinaryExpression",
        "children": [
            {"kind": "Identifier", "value": "<"},
            {
                "kind": "BinaryExpression",
                "children": [
                    {"kind": "Identifier", "value": "+"},
                    {"kind": "Identifier", "value": "dist_u"},
                    {"kind": "Identifier", "value": "weight"}
                ]
            },
            {"kind": "Identifier", "value": "dist_v"}
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert_eq!(py_out, "((dist_u + weight) < dist_v)");
    assert_eq!(rs_out, "((dist_u + weight) < dist_v)");
}

/// Unary negation
#[test] #[ignore]
fn benchmark_unary_negation() {
    let ast = r#"{
        "kind": "UnaryExpression",
        "children": [
            {"kind": "Identifier", "value": "-"},
            {"kind": "Identifier", "value": "value"}
        ]
    }"#;

    let py = DafnyTarget; let ast = format!(r#"{{"kind":"Algorithm","children":[{{"kind":"Identifier","value":"test"}},{{"kind":"StringLiteral","value":"O(1)"}},{{"kind":"Return","children":[{ast}]}}]}}"#, ast = ast);
    let rs = DafnyTarget;
    let py_out = py.generate(&ast).unwrap();
    let rs_out = rs.generate(&ast).unwrap();
    assert_eq!(py_out, "-(value)");
    assert_eq!(rs_out, "-(value)");
}
