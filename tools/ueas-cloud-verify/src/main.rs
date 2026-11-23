//! Hardened UEAS Cloud Run formal proof microservice (Dafny/Z3 & Lean 4).
#![allow(missing_docs)]

use axum::{
    extract::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tower_http::limit::RequestBodyLimitLayer;
use ueas_codegen::{generate_target, TargetKind};
use ueas_hir::check::linker::NullModuleResolver;
use ueas_hir::compile_to_hir;

#[derive(Deserialize)]
struct VerifyRequest {
    source: String,
    prover: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    ok: bool,
    prover: String,
    generated_spec: Option<String>,
    stdout: String,
    stderr: String,
}

async fn healthz() -> &'static str {
    "ok"
}

async fn verify_handler(Json(req): Json<VerifyRequest>) -> (StatusCode, Json<VerifyResponse>) {
    let target = match req.prover.to_ascii_lowercase().as_str() {
        "dafny" => TargetKind::Dafny,
        "lean" | "lean4" => TargetKind::Lean4,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(VerifyResponse {
                    ok: false,
                    prover: other.to_string(),
                    generated_spec: None,
                    stdout: String::new(),
                    stderr: "prover must be 'dafny' or 'lean4'".to_string(),
                }),
            );
        }
    };

    let prog = match compile_to_hir(&req.source, &NullModuleResolver) {
        Ok(p) => p,
        Err(errs) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(VerifyResponse {
                    ok: false,
                    prover: req.prover,
                    generated_spec: None,
                    stdout: String::new(),
                    stderr: errs.join("\n"),
                }),
            );
        }
    };

    let artifact = match generate_target(&prog, target) {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(VerifyResponse {
                    ok: false,
                    prover: req.prover,
                    generated_spec: None,
                    stdout: String::new(),
                    stderr: e,
                }),
            );
        }
    };

    let Ok(tmp) = TempDir::new() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(VerifyResponse {
                ok: false,
                prover: req.prover,
                generated_spec: Some(artifact.code),
                stdout: String::new(),
                stderr: "failed to create isolated temp directory".to_string(),
            }),
        );
    };

    let file_path = tmp.path().join(format!("verify.{}", artifact.extension));
    if tokio::fs::write(&file_path, &artifact.code).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(VerifyResponse {
                ok: false,
                prover: req.prover,
                generated_spec: Some(artifact.code),
                stdout: String::new(),
                stderr: "failed to write temp spec file".to_string(),
            }),
        );
    }

    let mut cmd = match target {
        TargetKind::Dafny => {
            let mut c = Command::new("dafny");
            c.arg("verify").arg(&file_path);
            c
        }
        _ => {
            let mut c = Command::new("lean");
            c.arg(&file_path);
            c
        }
    };

    match tokio::time::timeout(Duration::from_secs(15), cmd.output()).await {
        Ok(Ok(out)) => (
            StatusCode::OK,
            Json(VerifyResponse {
                ok: out.status.success(),
                prover: req.prover,
                generated_spec: Some(artifact.code),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            }),
        ),
        Ok(Err(e)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(VerifyResponse {
                ok: false,
                prover: req.prover,
                generated_spec: Some(artifact.code),
                stdout: String::new(),
                stderr: format!("prover binary unavailable: {e}"),
            }),
        ),
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(VerifyResponse {
                ok: false,
                prover: req.prover,
                generated_spec: Some(artifact.code),
                stdout: String::new(),
                stderr: "prover execution exceeded 15s timeout".to_string(),
            }),
        ),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/verify", post(verify_handler))
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
