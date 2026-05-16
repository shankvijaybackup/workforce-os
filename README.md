# Workforce OS – Enterprise Telemetry & Intelligence

## Overview
This repository implements the **first slice** of the enterprise‑grade Workforce OS platform described in the user's plan.  It focuses on:
- Secure telemetry ingestion (REST endpoint with mTLS/JWT)
- Cross‑platform agents (macOS & Windows) that collect non‑intrusive work context signals
- Infrastructure as Code (Terraform) for AWS components: API Gateway, MSK, EKS, S3, PostgreSQL and ClickHouse
- Privacy‑by‑Design Zero‑Knowledge encryption of telemetry payloads
- Initial data schemas for tenant/configuration metadata and high‑frequency telemetry events

## Repository Layout
```
workforce_os/
│   README.md               # ← You are reading this
│   LICENSE
│
├─ backend/
│   ├─ api/
│   │   └─ openapi.yaml     # OpenAPI contract for telemetry ingress
│   ├─ app/
│   │   └─ main.py          # FastAPI server stub
│   └─ terraform/
│       └─ main.tf          # Minimal IaC skeleton
│
├─ agents/
│   ├─ macos/
│   │   └─ telemetry_agent.py  # Python macOS agent skeleton
│   └─ windows/
│       └─ telemetry_agent.py  # Placeholder for future Windows implementation
│
└─ docs/
    └─ design.md            # High‑level architecture & security design
```

## Getting Started
1. **Set up the workspace** – The repository lives under the default scratch directory.
   ```bash
   cd /Users/vijayshankar/.gemini/antigravity/scratch/workforce_os
   ```
2. **Deploy infrastructure** – See `backend/terraform/main.tf` for a Terraform entry point.
3. **Run the API locally** – `python backend/app/main.py` (requires FastAPI & uvicorn).
4. **Start the macOS agent** – `python agents/macos/telemetry_agent.py` (needs `pyobjc` for AppleScript interaction).

---
*The next steps will flesh out the OpenAPI spec, Terraform resources, and the macOS telemetry agent implementation.*
