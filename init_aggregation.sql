-- Create the destination table for the aggregated 5-minute blocks
CREATE TABLE telemetry_aggregated_5m (
    tenant_id UUID,
    user_id UUID,
    window_start DateTime,
    total_deep_work_sec UInt32,
    total_collab_sec UInt32,
    context_switches UInt16
) ENGINE = SummingMergeTree()
PARTITION BY (toYYYYMM(window_start), tenant_id)
ORDER BY (tenant_id, user_id, window_start)
TTL window_start + INTERVAL 365 DAY;

-- Materialized View to calculate temporal fragmentation on insert
CREATE MATERIALIZED VIEW mv_telemetry_aggregation
TO telemetry_aggregated_5m AS
SELECT
    tenant_id,
    user_id,
    toStartOfFiveMinute(timestamp) AS window_start,
    sum(if(category = 'DEEP_WORK', duration_seconds, 0)) AS total_deep_work_sec,
    sum(if(category = 'COLLABORATION', duration_seconds, 0)) AS total_collab_sec,
    -- Context Switch Calculation: Count instances where the category changes chronologically
    countIf(category != neighbor(category, -1, category)) AS context_switches
FROM telemetry_events
GROUP BY tenant_id, user_id, window_start;
