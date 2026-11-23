//! UEAS workspace automation (`cargo xtask`).

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask", about = "UEAS workspace build & verification tasks")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Verify zero leftovers, single Cargo.lock, uniform versions, no ignored equivalence tests.
    VerifyClean,
    /// Build ueas-wasm for Browser Web Worker (--target web) and Cloudflare Edge (--target bundler).
    BuildWasm,
    /// Run all workspace quality checks (fmt, clippy, test, verify-clean).
    CheckAll,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = workspace_root()?;
    match cli.command {
        Cmd::VerifyClean => cmd_verify_clean(&root),
        Cmd::BuildWasm => cmd_build_wasm(&root),
        Cmd::CheckAll => {
            run(&root, "cargo", &["fmt", "--all", "--", "--check"])?;
            run(
                &root,
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            run(&root, "cargo", &["test", "--workspace"])?;
            cmd_verify_clean(&root)?;
            println!("xtask check-all: ALL PASSED");
            Ok(())
        }
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("xtask must live inside workspace root")
}

fn run(cwd: &Path, bin: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(bin)
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("failed to spawn {bin}"))?;
    if !status.success() {
        bail!("command `{bin} {}` failed with {status}", args.join(" "));
    }
    Ok(())
}

fn cmd_build_wasm(root: &Path) -> Result<()> {
    let wasm_crate = root.join("crates/ueas-wasm");
    let playground_pkg = root.join("tools/ueas-playground/src/wasm");
    let mcp_pkg = root.join("tools/ueas-mcp-edge/src/wasm");
    std::fs::create_dir_all(&playground_pkg)?;
    std::fs::create_dir_all(&mcp_pkg)?;

    run(
        &wasm_crate,
        "wasm-pack",
        &[
            "build",
            "--target",
            "web",
            "--out-dir",
            playground_pkg
                .to_str()
                .context("non-utf8 playground path")?,
        ],
    )?;
    run(
        &wasm_crate,
        "wasm-pack",
        &[
            "build",
            "--target",
            "bundler",
            "--out-dir",
            mcp_pkg.to_str().context("non-utf8 mcp path")?,
        ],
    )?;
    println!("WASM built cleanly for playground (web) and mcp-edge (bundler).");
    Ok(())
}

fn cmd_verify_clean(root: &Path) -> Result<()> {
    let mut lockfiles = Vec::new();
    collect_files(root, &mut |p| {
        if p.file_name().and_then(|n| n.to_str()) == Some("Cargo.lock") {
            lockfiles.push(p.to_path_buf());
        }
    })?;
    if lockfiles.len() != 1 || lockfiles[0] != root.join("Cargo.lock") {
        bail!("expected exactly 1 root Cargo.lock, found: {lockfiles:?}");
    }

    let forbidden_legacy = [
        "CLA.md",
        "VERSION",
        "implementation",
        "specification",
        "tools/ueas-cloud-receiver",
        "tools/ueas-mcp",
    ];
    for rel in forbidden_legacy {
        if root.join(rel).exists() {
            bail!("legacy path still exists: {rel}");
        }
    }

    let spec_ver = std::fs::read_to_string(root.join("spec/VERSION"))?
        .trim()
        .to_string();
    if spec_ver != env!("CARGO_PKG_VERSION") {
        bail!(
            "version mismatch: spec/VERSION={spec_ver} vs Cargo.toml={}",
            env!("CARGO_PKG_VERSION")
        );
    }

    for dir in ["crates", "tools", "tests"] {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        collect_files(&base, &mut |p| {
            if p.extension().and_then(|e| e.to_str()) != Some("rs") {
                return;
            }
            if let Ok(content) = std::fs::read_to_string(p) {
                for pat in ["todo!(", "unimplemented!(", "Box::leak"] {
                    if content.contains(pat) {
                        eprintln!("ERROR: `{pat}` found in {}", p.display());
                    }
                }
            }
        })?;
    }

    // Hard-fail on any forbidden patterns in crates/ and tools/
    let mut errors = Vec::new();
    for dir in ["crates", "tools", "tests"] {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        collect_files(&base, &mut |p| {
            if p.extension().and_then(|e| e.to_str()) != Some("rs") {
                return;
            }
            if let Ok(content) = std::fs::read_to_string(p) {
                for pat in ["todo!(", "unimplemented!(", "Box::leak"] {
                    if content.contains(pat) {
                        errors.push(format!("`{pat}` in {}", p.display()));
                    }
                }
                if (dir == "tests") && content.contains("#[ignore]") {
                    errors.push(format!("`#[ignore]` in {}", p.display()));
                }
            }
        })?;
    }
    if !errors.is_empty() {
        bail!("verify-clean failed:\n  {}", errors.join("\n  "));
    }

    println!("verify-clean: OK (1 Cargo.lock, 0 legacy paths, 0 todo!/Box::leak/#[ignore])");
    Ok(())
}

fn collect_files(dir: &Path, cb: &mut dyn FnMut(&Path)) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(name, "target" | ".git" | "node_modules" | "dist" | ".venv") {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, cb)?;
        } else {
            cb(&path);
        }
    }
    Ok(())
}
