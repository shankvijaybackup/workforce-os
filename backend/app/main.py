from fastapi import FastAPI, Request, HTTPException, Depends
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field
from typing import Optional
import uvicorn
import json
import logging

app = FastAPI(title="Workforce OS Telemetry Ingress", version="1.0.0")

# Simple bearer token auth – in production replace with JWT verification & mTLS handling
security = HTTPBearer(auto_error=False)

class DeviceInfo(BaseModel):
    os_type: str = Field(..., description="Operating system type: darwin or win32")
    os_version: str = Field(..., description="OS version string")
    agent_version: str = Field(..., description="Agent semantic version")
    cpu_overhead_pct: Optional[float] = None
    mem_overhead_mb: Optional[int] = None

class ContextInfo(BaseModel):
    active_app_bundle: str
    window_title_hashed: str
    category_inferred: str
    duration_seconds: int

class MetricsInfo(BaseModel):
    keystroke_entropy: Optional[float] = None
    mouse_movement_distance: Optional[float] = None

class TelemetryPayload(BaseModel):
    event_id: str
    timestamp: str
    tenant_id: str
    device: DeviceInfo
    context: ContextInfo
    metrics: MetricsInfo

def verify_token(credentials: Optional[HTTPAuthorizationCredentials] = Depends(security)):
    if credentials is None:
        raise HTTPException(status_code=401, detail="Missing Authorization header")
    token = credentials.credentials
    # TODO: validate JWT signature and claims against tenant public key
    if not token.startswith("jwt-"):
        raise HTTPException(status_code=401, detail="Invalid token format")
    return token

@app.post("/v1/telemetry/ingress", status_code=202)
async def ingest(payload: TelemetryPayload, request: Request, token: str = Depends(verify_token)):
    # In a real deployment, the request would be forwarded to Kafka after decryption.
    logging.info("Received telemetry event %s from tenant %s", payload.event_id, payload.tenant_id)
    # Simple echo response for now
    return {"status": "accepted", "event_id": payload.event_id}

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    uvicorn.run(app, host="0.0.0.0", port=8000)
