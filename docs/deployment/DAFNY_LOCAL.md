# Local Dafny CLI Testing

Test the Dafny transpiler locally before deploying to Google Cloud Run.

## Install Dafny

```powershell
# Download Dafny 4.6.0
Invoke-WebRequest -Uri "https://github.com/dafny-lang/dafny/releases/download/v4.6.0/dafny-4.6.0-x64-windows-2022.zip" -OutFile "$env:TEMP\dafny.zip"
Expand-Archive -Path "$env:TEMP\dafny.zip" -DestinationPath "C:\tools\dafny"
$env:Path += ";C:\tools\dafny"
```

Requires .NET 8.0 SDK: https://dotnet.microsoft.com/download/dotnet/8.0

## Verify Installation

```bash
dafny --version
# → Dafny 4.6.0
```

## Test Dafny Transpile

```bash
cd C:\dev\projects\ueas

# Transpile UEAS to Dafny
cargo run --bin ueas -- transpile examples/core/euclidean.ueas --target dafny > test.dfy

# Z3 proof verification
dafny verify test.dfy

# Expected output:
# Dafny program verifier finished with 1 verified, 0 errors

# Generate Python
dafny build --target:py test.dfy
# Creates test-py/test.py
```

## Test Cloud Receiver Locally

```bash
cd tools/ueas-cloud-receiver

# Build
cargo build --release

# Run (listens on port 8080)
$env:PORT="8080"
./target/release/ueas-cloud-receiver

# In another terminal, test:
curl -X POST http://localhost:8080/verify \
  -H "Content-Type: application/json" \
  -d "{\"ast\":\"$(cat ../../examples/core/euclidean.ueas | jq -Rs .)\", \"target\":\"cpp\"}"
```
