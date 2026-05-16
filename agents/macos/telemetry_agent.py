import os
import uuid
import json
import hashlib
import time
import platform
import subprocess
import requests
from datetime import datetime, timezone

# Configuration – replace with real values before production
API_ENDPOINT = os.getenv("TELEMETRY_API_URL", "https://api.example.com/v1/telemetry/ingress")
AGENT_VERSION = "1.0.0-rc1"
TENANT_ID = os.getenv("TENANT_ID", "t-placeholder")

# Helper to get the frontmost application bundle identifier (macOS specific)
def get_frontmost_app_bundle():
    try:
        # Use AppleScript to get the bundle identifier of the frontmost app
        script = "tell application \"System Events\" to get bundle identifier of (process 1 where frontmost is true)"
        bundle_id = subprocess.check_output(["osascript", "-e", script], text=True).strip()
        return bundle_id
    except Exception as e:
        return "unknown"

# Helper to get the frontmost window title and hash it
def get_window_title_hash():
    try:
        # Get the name of the frontmost process first
        bundle = get_frontmost_app_bundle()
        if bundle == "unknown":
            return hashlib.sha256(b"unknown").hexdigest()
        # AppleScript to fetch the front window title of the frontmost app
        script = f"tell application \"System Events\" to get value of attribute \"AXTitle\" of UI element 1 of (process 1 where frontmost is true)"
        title = subprocess.check_output(["osascript", "-e", script], text=True).strip()
        # Hash the title – ensures privacy
        return hashlib.sha256(title.encode("utf-8")).hexdigest()
    except Exception:
        # Fallback – hash the bundle identifier only
        return hashlib.sha256(bundle.encode("utf-8")).hexdigest()

# Collect a single telemetry event – this function can be called periodically (e.g., every 2 min)
def collect_event():
    event = {
        "event_id": str(uuid.uuid4()),
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "tenant_id": TENANT_ID,
        "device": {
            "os_type": "darwin",
            "os_version": platform.mac_ver()[0] or "unknown",
            "agent_version": AGENT_VERSION,
            "cpu_overhead_pct": None,  # Placeholder – can be filled via psutil if needed
            "mem_overhead_mb": None,
        },
        "context": {
            "active_app_bundle": get_frontmost_app_bundle(),
            "window_title_hashed": get_window_title_hash(),
            "category_inferred": "unknown",  # Future: infer via known bundle list
            "duration_seconds": 120,  # Fixed example block length
        },
        "metrics": {
            "keystroke_entropy": None,  # Placeholder for future calculation
            "mouse_movement_distance": None,
        }
    }
    return event

# Send payload – in production this would use mTLS client certificates and a short‑lived JWT
def send_payload(event):
    try:
        headers = {"Content-Type": "application/json"}
        # Example: include JWT in Authorization header if available
        jwt = os.getenv("AGENT_JWT")
        if jwt:
            headers["Authorization"] = f"Bearer {jwt}"
        response = requests.post(API_ENDPOINT, json=event, headers=headers, timeout=5)
        response.raise_for_status()
        print(f"[Telemetry] Sent event {event['event_id']} – status {response.status_code}")
    except Exception as e:
        print(f"[Telemetry] Failed to send event: {e}")

if __name__ == "__main__":
    # Simple loop – in a real agent this would run as a background service / launchd daemon
    while True:
        event = collect_event()
        send_payload(event)
        time.sleep(120)  # Wait 2 minutes before next collection
