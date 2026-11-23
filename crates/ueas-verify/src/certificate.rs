//! Cryptographic SHA-256 verification certificate generator (`spec/schemas/ueas-cert.v2.schema.json`).

use crate::complexity_fit::ComplexityFitReport;
use crate::constant_time::ConstantTimeReport;
use crate::diagnostic::{RepairDiagnostic, VerificationLevel};
use crate::fuzzer::FuzzReport;
use crate::runner::TestOutcome;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Signed/hashed verification certificate for a UEAS module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationCertificate {
    pub schema_version: String,
    pub engine_version: String,
    pub module: Option<String>,
    pub source_sha256: String,
    pub hir_sha256: String,
    pub achieved_level: VerificationLevel,
    pub passed: bool,
    pub tests: Vec<TestOutcome>,
    pub fuzz_reports: Vec<FuzzReport>,
    pub complexity_reports: Vec<ComplexityFitReport>,
    pub constant_time_reports: Vec<ConstantTimeReport>,
    pub diagnostics: Vec<RepairDiagnostic>,
    pub certificate_sha256: String,
}

impl VerificationCertificate {
    /// Compute SHA-256 hex digest of UTF-8 bytes.
    pub fn sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Finalize and seal `certificate_sha256` over all deterministic certificate fields.
    pub fn seal(&mut self) {
        self.certificate_sha256.clear();
        let json = serde_json::to_vec(self).unwrap_or_default();
        self.certificate_sha256 = Self::sha256_hex(&json);
    }

    /// Cryptographically verify this certificate against a UEAS source string and module resolver.
    pub fn verify_against(
        &self,
        source: &str,
        resolver: &dyn ueas_hir::check::linker::ModuleResolver,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let actual_src_hash = Self::sha256_hex(source.as_bytes());
        if self.source_sha256 != actual_src_hash {
            errors.push(format!(
                "source_sha256 mismatch: expected {}, got {actual_src_hash}",
                self.source_sha256
            ));
        }

        if let Ok(prog) = ueas_hir::compile_to_hir(source, resolver) {
            let hir_bytes = serde_json::to_vec(&prog).unwrap_or_default();
            let actual_hir_hash = Self::sha256_hex(&hir_bytes);
            if !self.hir_sha256.is_empty() && self.hir_sha256 != actual_hir_hash {
                errors.push(format!(
                    "hir_sha256 mismatch: expected {}, got {actual_hir_hash}",
                    self.hir_sha256
                ));
            }
        } else if !self.hir_sha256.is_empty() {
            errors.push(
                "source failed L0 HIR compilation while certificate carries hir_sha256".to_string(),
            );
        }

        let mut copy = self.clone();
        copy.seal();
        if copy.certificate_sha256 != self.certificate_sha256 {
            errors.push(format!(
                "certificate_sha256 tampered or invalid: expected {}, got {}",
                self.certificate_sha256, copy.certificate_sha256
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
