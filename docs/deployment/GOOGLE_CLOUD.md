# Google Cloud Run Deployment

Dafny/Z3 verification backend — scale-to-zero, $0/month.

## Prerequisites

1. **Google Cloud account** (free tier, credit card for verification only)
   - Sign up at https://console.cloud.google.com

2. **Install Google Cloud CLI:**
   ```powershell
   # Windows (PowerShell as Admin)
   (New-Object Net.WebClient).DownloadFile("https://dl.google.com/dl/cloudsdk/channels/rapid/GoogleCloudSDKInstaller.exe", "$env:TEMP\gc-sdk.exe")
   & "$env:TEMP\gc-sdk.exe"
   ```

3. **Authenticate:**
   ```bash
   gcloud auth login
   gcloud config set project YOUR_PROJECT_ID
   ```

## Step 1: Create Project
```bash
gcloud projects create ueas-verify --name="UEAS Verification"
gcloud config set project ueas-verify
gcloud services enable run.googleapis.com artifactregistry.googleapis.com
```

## Step 2: Build & Push Docker Image
```bash
cd tools/ueas-cloud-receiver

# Build
docker build -t gcr.io/YOUR_PROJECT_ID/ueas-verify .

# Push
gcloud auth configure-docker
docker push gcr.io/YOUR_PROJECT_ID/ueas-verify
```

## Step 3: Deploy to Cloud Run
```bash
gcloud run deploy ueas-verify \
  --image gcr.io/YOUR_PROJECT_ID/ueas-verify \
  --region us-central1 \
  --memory 1Gi \
  --cpu 1 \
  --min-instances 0 \
  --max-instances 5 \
  --concurrency 80 \
  --allow-unauthenticated \
  --port 8080
```

After deployment, note the **Service URL** (e.g., `https://ueas-verify-xyz-uc.a.run.app`).

## Step 4: Verify
```bash
# Health check
curl https://ueas-verify-xyz-uc.a.run.app/health
# → "OK"

# Verify an algorithm
curl -X POST https://ueas-verify-xyz-uc.a.run.app/verify \
  -H "Content-Type: application/json" \
  -d '{"ast": "...", "target": "cpp"}'
```

## Free Tier Limits

| Resource | Monthly Free | Enough For |
|----------|-------------|------------|
| Requests | 2,000,000 | ~1 verify every 2 seconds |
| vCPU | 180,000 sec | 50 hours of active processing |
| Memory | 360,000 GiB-sec | 100 hours of 1GiB RAM |

The service scales to **0** when idle — $0 cost when not in use.
