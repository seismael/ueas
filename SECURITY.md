# Security Policy

## Supported Versions

UEAS `2.0.x` is actively maintained and receives security updates.

## Architectural Security Guarantees

- **Axiom 1 Purity:** Core `crates/*` compile with `#![forbid(unsafe_code)]`, zero host filesystem/network/clock access, and deterministic `StepBudget` / `HeapBudget` enforcement.
- **Constant-Time Enforcement:** `@ConstantTime` algorithms undergo static `Public ⊑ Secret` taint analysis in `ueas-hir` and dynamic `(pc, cache_line, branch_taken)` trace equivalence verification in `ueas-verify`.
- **Cloud Microservice Hardening:** `ueas-cloud-verify` enforces 1 MB request limits, isolated `TempDir` sandboxes, and 15-second subprocess timeouts.

## Reporting a Vulnerability

Please report security issues privately via GitHub Security Advisories or the repository maintainers rather than opening a public issue.
