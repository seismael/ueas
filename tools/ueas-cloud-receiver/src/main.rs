//! UEAS Cloud Run Receiver — Dafny/Z3 verification backend.
//!
//! Accepts POST /verify with UEAS JSON AST, writes .dfy file,
//! runs `dafny verify` for Z3 proof, optionally `dafny build`
//! for code generation (C++, Python, Java, Go, C#, JS).

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use serde_json::json;
use std::net::SocketAddr;
use std::process::Command;
use ueas_backends::{DafnyTarget, TargetGenerator};

async fn handle(req: Request<Incoming>) -> Result<Response<String>, hyper::Error> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Response::new("OK".into()),
        (&Method::POST, "/verify") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&body_bytes);
            match verify_request(&body_str) {
                Ok(result) => Response::new(result),
                Err(e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(e)
                    .unwrap(),
            }
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".into())
            .unwrap(),
    };
    Ok(response)
}

fn verify_request(body: &str) -> Result<String, String> {
    let req: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {}", e))?;
    let ast = req["ast"].to_string();
    let target = req["target"].as_str().unwrap_or("cpp");

    // Generate Dafny source from UEAS AST
    let gen = DafnyTarget;
    let dafny_source = gen
        .generate(&ast)
        .map_err(|e| format!("Transpile error: {}", e.message))?;

    // Write .dfy file and run Dafny
    std::fs::write("/tmp/ueas.dfy", &dafny_source)
        .map_err(|e| format!("Write error: {}", e))?;

    // Step 1: Dafny verify (Z3 proof)
    let verify = Command::new("dafny")
        .args(["verify", "/tmp/ueas.dfy"])
        .output()
        .map_err(|e| format!("Dafny error: {}", e))?;

    let z3_output = String::from_utf8_lossy(&verify.stdout).to_string();
    let verified = verify.status.success();

    // Step 2: Dafny build (code generation)
    let build = Command::new("dafny")
        .args(["build", "--target", target, "/tmp/ueas.dfy"])
        .output()
        .map_err(|e| format!("Build error: {}", e))?;
    let generated = String::from_utf8_lossy(&build.stdout).to_string();

    Ok(
        json!({
            "status": if verified { "verified" } else { "failed" },
            "z3_output": z3_output,
            "source": generated,
            "dafny_source": dafny_source,
        })
        .to_string(),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    eprintln!("UEAS Cloud Receiver listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(stream, service_fn(handle))
                .await
            {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}
