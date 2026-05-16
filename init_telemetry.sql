-- Schema v1.0: Time-Series Telemetry

CREATE TABLE telemetry_events (
    timestamp DateTime64(3, 'UTC'),
    event_id UUID,
    tenant_id UUID,
    endpoint_id UUID,
    os_type Enum8('darwin' = 1, 'win32' = 2),
    app_bundle_hash FixedString(64),
    window_title_hash FixedString(64),
    duration_seconds UInt32,
    keystroke_entropy Float32
) ENGINE = MergeTree()
PARTITION BY (toYYYYMM(timestamp), tenant_id)
ORDER BY (tenant_id, endpoint_id, timestamp)
TTL timestamp + INTERVAL 90 DAY; 
-- Enforces data retention compliance by automatically dropping logs older than 90 days.
