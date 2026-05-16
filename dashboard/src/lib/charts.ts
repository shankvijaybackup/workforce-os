import { clickhouse } from './clickhouse';

export interface WorkBlock {
  category: 'DEEP_WORK' | 'COLLABORATION' | 'ADMINISTRATIVE';
  widthPercentage: number;
}

export async function getDailyDistribution(tenantId: string, ouId: string): Promise<WorkBlock[]> {
  // Mock data injection for UI verification since actual ClickHouse is not running
  if (process.env.NODE_ENV === 'development' && !process.env.CLICKHOUSE_HOST) {
    return [
      { category: 'DEEP_WORK', widthPercentage: 45 },
      { category: 'COLLABORATION', widthPercentage: 30 },
      { category: 'ADMINISTRATIVE', widthPercentage: 25 },
    ];
  }

  // Aggregate today's metrics across the specific OU
  const query = `
    SELECT 
      sum(total_deep_work_sec) AS deep_work,
      sum(total_collab_sec) AS collab,
      (28800 * uniqExact(user_id) - sum(total_deep_work_sec) - sum(total_collab_sec)) AS administrative
    FROM telemetry_aggregated_5m
    WHERE tenant_id = {tenant_id: UUID}
      AND user_id IN (SELECT user_id FROM users WHERE ou_id = {ou_id: UUID})
      AND toDate(window_start) = today()
  `;

  try {
    const resultSet = await clickhouse.query({
      query,
      query_params: { tenant_id: tenantId, ou_id: ouId },
      format: 'JSONEachRow',
    });

    const data = await resultSet.json<{ deep_work: number, collab: number, administrative: number }>();
    if (data.length === 0) return [];

    const row = data[0];
    const totalSeconds = row.deep_work + row.collab + row.administrative;

    if (totalSeconds === 0) return [];

    // Calculate CSS width percentages strictly bounded to 100%
    return [
      { category: 'DEEP_WORK', widthPercentage: (row.deep_work / totalSeconds) * 100 },
      { category: 'COLLABORATION', widthPercentage: (row.collab / totalSeconds) * 100 },
      { category: 'ADMINISTRATIVE', widthPercentage: (row.administrative / totalSeconds) * 100 },
    ];
  } catch (err) {
    console.error("ClickHouse fetch failed:", err);
    return [];
  }
}

export interface ScatterPoint {
  xTimePercentage: number; // 0% = 00:00, 100% = 23:59
  yVolumePercentage: number; // Relative to the max switches observed
}

export async function getScatterPlotData(tenantId: string, ouId: string): Promise<ScatterPoint[]> {
  // Mock data injection for UI verification since actual ClickHouse is not running
  if (process.env.NODE_ENV === 'development' && !process.env.CLICKHOUSE_HOST) {
    return [
      { xTimePercentage: 20, yVolumePercentage: 80 },
      { xTimePercentage: 40, yVolumePercentage: 70 },
      { xTimePercentage: 60, yVolumePercentage: 90 },
      { xTimePercentage: 80, yVolumePercentage: 30 },
    ];
  }

  const query = `
    SELECT 
      toHour(window_start) + (toMinute(window_start) / 60) AS hour_decimal,
      sum(context_switches) AS volume
    FROM telemetry_aggregated_5m
    WHERE tenant_id = {tenant_id: UUID}
      AND user_id IN (SELECT user_id FROM users WHERE ou_id = {ou_id: UUID})
      AND toDate(window_start) = today()
    GROUP BY window_start
    ORDER BY window_start
  `;

  try {
    const resultSet = await clickhouse.query({
      query,
      query_params: { tenant_id: tenantId, ou_id: ouId },
      format: 'JSONEachRow',
    });

    const data = await resultSet.json<{ hour_decimal: number, volume: number }>();
    if (data.length === 0) return [];

    // Determine the highest volume ceiling for Y-axis normalization
    const maxVolume = Math.max(...data.map(d => d.volume), 1);

    return data.map(point => ({
      // X-Axis: 0 to 24 hours converted to 0-100%
      xTimePercentage: (point.hour_decimal / 24) * 100,
      // Y-Axis: Normalized against the peak volume (0-100%)
      yVolumePercentage: (point.volume / maxVolume) * 100,
    }));
  } catch (err) {
    console.error("ClickHouse fetch failed:", err);
    return [];
  }
}
