# Contributing to UEAS

Thank you for contributing to the **Universal Executable Algorithm Standard (UEAS)**!

## 1. Engineering Protocol

Before opening a Pull Request, read [AGENTS.md](AGENTS.md). It defines our workspace architecture, zero-I/O kernel invariants, idiomatic Rust rules, and mandatory CI quality gates.

## 2. Developer Certificate of Origin (DCO)

UEAS uses the lightweight **Developer Certificate of Origin (DCO)** under Apache License 2.0 (no separate CLA required). By signing off your commits (`git commit -s`), you certify that you have the right to submit your work under the project's license:

```text
Signed-off-by: Your Name <you@example.com>
```

## 3. Local Development & Quality Gates

```bash
# Format, lint, and test the entire Rust workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo xtask verify-clean
```

## 4. Submitting a Pull Request

- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`).
- Include unit/integration tests for every change; specification changes must update `spec/SPEC.md` and `tests/ucts/`.
- All CI checks must pass with zero warnings.
