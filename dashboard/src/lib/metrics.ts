import { clickhouse } from './clickhouse';

export interface DashboardMetrics {
  totalDeepWorkHours: number;
  totalContextSwitches: number;
  activeEndpoints: number;
}

export async function getManagerialMetrics(tenantId: string, ouId: string): Promise<DashboardMetrics> {
  // Mock data injection for UI verification since actual ClickHouse is not running
  if (process.env.NODE_ENV === 'development' && !process.env.CLICKHOUSE_HOST) {
    return {
      totalDeepWorkHours: 142,
      totalContextSwitches: 4281,
      activeEndpoints: 24,
    };
  }

  // Query 1: Aggregate standard metrics for the past 7 days, strictly bounded by OU and Tenant
  const query = `
    SELECT 
      sum(total_deep_work_sec) / 3600 AS total_deep_work_hours,
      sum(context_switches) AS total_context_switches,
      uniqExact(user_id) AS active_endpoints
    FROM telemetry_aggregated_5m
    WHERE tenant_id = {tenant_id: UUID}
      AND user_id IN (SELECT user_id FROM users WHERE ou_id = {ou_id: UUID})
      AND window_start >= now() - INTERVAL 7 DAY
  `;

  try {
    const resultSet = await clickhouse.query({
      query,
      query_params: {
        tenant_id: tenantId,
        ou_id: ouId,
      },
      format: 'JSONEachRow',
    });

    const data = await resultSet.json<{ 
      total_deep_work_hours: number, 
      total_context_switches: number, 
      active_endpoints: number 
    }>();

    if (data.length === 0) {
      return { totalDeepWorkHours: 0, totalContextSwitches: 0, activeEndpoints: 0 };
    }

    return {
      totalDeepWorkHours: Math.round(data[0].total_deep_work_hours),
      totalContextSwitches: data[0].total_context_switches,
      activeEndpoints: data[0].active_endpoints,
    };
  } catch (error) {
    console.error("ClickHouse fetch failed, returning empty dataset:", error);
    return { totalDeepWorkHours: 0, totalContextSwitches: 0, activeEndpoints: 0 };
  }
}

export interface Endpoint {
  id: string;
  user: string;
  os: string;
  version: string;
  status: string;
  lastSync: string;
}

export async function getEndpoints(tenantId: string, ouId: string): Promise<Endpoint[]> {
  // Query ClickHouse to pull unique endpoints over the past 24 hours
  const query = `
    SELECT 
      endpoint_id,
      argMax(os_type, timestamp) AS os_type,
      max(timestamp) AS last_seen,
      any(user_id) AS user_id
    FROM telemetry_events
    WHERE tenant_id = {tenant_id: UUID}
      AND user_id IN (SELECT user_id FROM users WHERE ou_id = {ou_id: UUID})
      AND timestamp >= now() - INTERVAL 1 DAY
    GROUP BY endpoint_id
    ORDER BY last_seen DESC
  `;

  try {
    const resultSet = await clickhouse.query({
      query,
      query_params: { tenant_id: tenantId, ou_id: ouId },
      format: 'JSONEachRow',
    });

    const data = await resultSet.json<{ endpoint_id: string, os_type: string, last_seen: string, user_id: string }>();

    return data.map((row) => {
      // Basic last sync logic
      const lastSeenDate = new Date(row.last_seen);
      const isOnline = (Date.now() - lastSeenDate.getTime()) < 15 * 60 * 1000; // 15 mins
      
      return {
        id: row.endpoint_id.split('-')[0], // Short ID for UI
        user: row.user_id.split('-')[0],
        os: row.os_type === 'darwin' ? 'macOS' : row.os_type === 'win32' ? 'Windows' : 'Linux',
        version: '1.0.0', // Agent version to be tracked in schema later
        status: isOnline ? 'ONLINE' : 'OFFLINE',
        lastSync: lastSeenDate.toLocaleTimeString(),
      };
    });
  } catch (error) {
    console.error("Failed to fetch endpoints from ClickHouse:", error);
    return [];
  }
}
