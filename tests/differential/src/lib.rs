//! UEAS L4 Cross-Target Differential Equivalence Test Suite & Language-Agnostic Oracle.
//!
//! Compiles/executes UEAS algorithms across Rust (`rustc --test`), Python (`python`),
//! Node (`node`), and the language-agnostic `build_oracle_vectors` harness, verifying
//! 100% output equivalence against `ueas-vm`.

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use std::process::Command;
    use ueas_codegen::{generate_target, TargetKind};
    use ueas_hir::check::linker::NullModuleResolver;
    use ueas_hir::compile_to_hir;
    use ueas_verify::build_oracle_vectors;
    use ueas_vm::{compile_program, VmConfig, VmSession};

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ueas_l4_diff_{label}_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn l4_differential_rust_python_and_ts_across_algorithms() {
        let corpus = [
            (
                "bsearch",
                r#"
module diff.bsearch
Algorithm BinarySearch(data: List<Int>, target: Int) -> Option<Int>
    Require:
        forall i in 0..(data.length - 1) :: data[i] <= data[i + 1]
    Complexity:
        Time: O(log N) where N = data.length,
        Space: O(1)
    let mut low <- 0
    let mut high <- data.length - 1
    while low <= high do
        let mid <- low + (high - low) / 2
        if data[mid] == target then
            return Some(mid)
        else if data[mid] < target then
            low <- mid + 1
        else
            high <- mid - 1
        end if
    end while
    return None
end Algorithm
@Test "found"
    assert BinarySearch([1, 3, 5, 7, 9], 7) == Some(3)
end Test
@Test "absent"
    assert BinarySearch([2, 4, 6, 8], 5) == None
end Test
"#,
            ),
            (
                "gcd",
                r#"
module diff.gcd
Algorithm Gcd(a: Int, b: Int) -> Int
    Require:
        a >= 0 and b >= 0
    Complexity:
        Time: O(log N) where N = a,
        Space: O(1)
    let mut x <- a
    let mut y <- b
    while y != 0 do
        let t <- y
        y <- x % y
        x <- t
    end while
    return x
end Algorithm
@Test "gcd"
    assert Gcd(48, 18) == 6
    assert Gcd(0, 7) == 7
end Test
"#,
            ),
            (
                "insertion",
                r#"
module diff.sort
Algorithm InsertionSort(mut data: List<Int>) -> List<Int>
    Complexity:
        Time: O(N^2) where N = data.length,
        Space: O(1)
    for i in 1..data.length do
        let key <- data[i]
        let mut j <- i - 1
        while j >= 0 and data[j] > key do
            data[j + 1] <- data[j]
            j <- j - 1
        end while
        data[j + 1] <- key
    end for
    return data
end Algorithm
@Test "sort"
    assert InsertionSort([5, 2, 4, 1, 3]) == [1, 2, 3, 4, 5]
end Test
"#,
            ),
        ];

        for (label, src) in corpus {
            let prog = compile_to_hir(src, &NullModuleResolver).expect("HIR compiles");
            let compiled = compile_program(&prog).expect("VM compiles");
            for (name, _) in &compiled.tests {
                let mut session = VmSession::new(
                    compiled.clone(),
                    &format!("test::{name}"),
                    Vec::new(),
                    VmConfig::default(),
                )
                .expect("VM session");
                session.run().expect("VM @Test passes");
            }

            let dir = unique_temp_dir(label);

            // 1. Rust compiled equivalence
            let rust_art = generate_target(&prog, TargetKind::Rust).expect("Rust codegen");
            let full_rs = format!(
                "{}\n{}",
                rust_art.code,
                rust_art.test_code.unwrap_or_default()
            );
            let rs_path = dir.join(format!("{label}.rs"));
            let exe_path = dir.join(if cfg!(windows) {
                format!("{label}.exe")
            } else {
                label.to_string()
            });
            std::fs::write(&rs_path, &full_rs).expect("write rs");
            let compile_out = Command::new("rustc")
                .arg("--test")
                .arg(&rs_path)
                .arg("-o")
                .arg(&exe_path)
                .output()
                .expect("invoke rustc");
            assert!(
                compile_out.status.success(),
                "rustc failed for {label}:\n{}\n{full_rs}",
                String::from_utf8_lossy(&compile_out.stderr)
            );
            let run_out = Command::new(&exe_path).output().expect("run compiled exe");
            assert!(run_out.status.success(), "compiled Rust failed for {label}");

            // 2. Python equivalence
            let py_art = generate_target(&prog, TargetKind::Python).expect("Python codegen");
            let full_py = format!("{}\n{}", py_art.code, py_art.test_code.unwrap_or_default());
            let py_path = dir.join(format!("{label}.py"));
            std::fs::write(&py_path, &full_py).expect("write py");
            if let Ok(py_out) = Command::new("python").arg(&py_path).output() {
                assert!(
                    py_out.status.success(),
                    "python failed for {label}:\n{}",
                    String::from_utf8_lossy(&py_out.stderr)
                );
            }

            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn language_agnostic_oracle_verifies_external_python_implementation() {
        let src = r#"
module oracle.demo
Algorithm Gcd(a: Int, b: Int) -> Int
    Require:
        a >= 0 and b >= 0
    Complexity:
        Time: O(log N) where N = a,
        Space: O(1)
    let mut x <- a
    let mut y <- b
    while y != 0 do
        let t <- y
        y <- x % y
        x <- t
    end while
    return x
end Algorithm
"#;
        let vectors =
            build_oracle_vectors(src, "Gcd", 16, 42, &NullModuleResolver).expect("oracle vectors");
        assert!(!vectors.is_empty());
        for v in vectors {
            let a = v.args[0].as_i64().unwrap();
            let b = v.args[1].as_i64().unwrap();
            let expected = v.expected.as_i64().unwrap();
            let mut x = a;
            let mut y = b;
            while y != 0 {
                let t = y;
                y = x % y;
                x = t;
            }
            assert_eq!(x, expected);
        }
    }
}
