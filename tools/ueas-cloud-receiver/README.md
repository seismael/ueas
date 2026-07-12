# UEAS Cloud Receiver — Google Cloud Run

Dafny/Z3 verification backend. Scale-to-zero, $0/month on free tier.

## Endpoint

https://ueas-verify-504087134780.us-central1.run.app

## API

### POST /verify

```json
{"ast": "<JSON AST from MCP parse>", "target": "cpp"}
```

Returns:
```json
{"status":"verified","z3_output":"...","source":"...","dafny_source":"..."}
```

### GET /health → "OK"

## Architecture

```
MCP parse (CF Workers) → raw AST → POST /verify (GCP) → Dafny → Z3 proof → dafny build → C++/Python/etc.
```

## Deploy

```bash
gcloud run deploy ueas-verify --source . --region us-central1 \
  --memory 1Gi --cpu 1 --min-instances 0 --max-instances 5 \
  --concurrency 80 --allow-unauthenticated --port 8080
```
