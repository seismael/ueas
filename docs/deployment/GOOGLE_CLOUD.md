# Google Cloud Run Deployment (`ueas-cloud-verify`)

Hardened Axum formal verification backend for Dafny 4 and Lean 4 — scale-to-zero, isolated `TempDir` per request, 1 MB body limit, and 15-second subprocess timeout.

## Build & Push Docker Image

From the repository root (so workspace crates are in Docker build context):

```bash
docker build -f tools/ueas-cloud-verify/Dockerfile -t gcr.io/YOUR_PROJECT_ID/ueas-cloud-verify .
docker push gcr.io/YOUR_PROJECT_ID/ueas-cloud-verify
```

## Deploy to Cloud Run

```bash
gcloud run deploy ueas-cloud-verify \
  --image gcr.io/YOUR_PROJECT_ID/ueas-cloud-verify \
  --region us-central1 \
  --memory 1Gi \
  --cpu 1 \
  --min-instances 0 \
  --max-instances 5 \
  --allow-unauthenticated \
  --port 8080
```

## Verify

```bash
curl https://<service-url>/healthz
# -> ok

curl -X POST https://<service-url>/verify \
  -H "Content-Type: application/json" \
  -d '{"source": "module demo\nAlgorithm Id(x: Int) -> Int\n    return x\nend Algorithm", "prover": "dafny"}'
```
