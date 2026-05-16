import { headers } from 'next/headers';
import { getManagerialMetrics } from '@/lib/metrics';
import { getDailyDistribution, getScatterPlotData } from '@/lib/charts';

export default async function Dashboard() {
  const headersList = await headers();
  // Simulated fallback for architectural review if headers aren't injected by middleware in dev
  const ouId = headersList.get('x-org-unit-id') || 'mock-ou-id';
  const tenantId = headersList.get('x-tenant-id') || 'mock-tenant-id';

  const metrics = await getManagerialMetrics(tenantId, ouId);
  const distribution = await getDailyDistribution(tenantId, ouId);
  const scatterPoints = await getScatterPlotData(tenantId, ouId);

  return (
    <div className="dashboard-container">
      <header className="dashboard-header">
        <h2>Organizational Intelligence Overview</h2>
        <div className="alert-feed">
          <div className="alert-item critical">
            <strong>[CRITICAL ALERT]</strong> Team A is exhibiting a 35% increase in sustained off-hours cognitive load paired with a 20% drop in daytime Deep Work. Burnout risk is CRITICAL.
          </div>
        </div>
      </header>

      <div className="metrics-grid">
        <div className="metric-card">
          <h3>Total Deep Work</h3>
          <p className="metric-value">{metrics.totalDeepWorkHours} Hours</p>
          <span className="trend-down">↓ 12% vs last week</span>
        </div>
        <div className="metric-card">
          <h3>Context Switches</h3>
          <p className="metric-value">{metrics.totalContextSwitches.toLocaleString()}</p>
          <span className="trend-up">↑ 18% vs last week</span>
        </div>
        <div className="metric-card">
          <h3>Active Endpoints</h3>
          <p className="metric-value">{metrics.activeEndpoints}/24</p>
          <span className="trend-neutral">- Stable</span>
        </div>
      </div>

      <div className="charts-container">
        {/* Deep Work vs Fragmentation Chart */}
        <section className="chart-section">
          <h3>Deep Work vs Fragmentation (8-Hour Window)</h3>
          <div className="stacked-bar-container">
            <div className="legend">
              <span className="legend-item"><span className="swatch solid"></span> Deep Work</span>
              <span className="legend-item"><span className="swatch striped"></span> Collaboration</span>
              <span className="legend-item"><span className="swatch hollow"></span> Administrative</span>
            </div>
            
            <div className="bar-wrapper">
              {distribution.map((block) => {
                let styleClass = '';
                if (block.category === 'DEEP_WORK') styleClass = 'solid';
                if (block.category === 'COLLABORATION') styleClass = 'striped';
                if (block.category === 'ADMINISTRATIVE') styleClass = 'hollow';
                
                return (
                  <div 
                    key={block.category} 
                    className={`bar-segment ${styleClass}`} 
                    style={{ width: `${block.widthPercentage}%` }}
                  ></div>
                );
              })}
            </div>
          </div>
        </section>

        {/* Context Switch Scatter Plot */}
        <section className="chart-section">
          <h3>Context Switch Volume by Department</h3>
          <div className="scatter-plot-container">
            <div className="scatter-axis y-axis">Volume</div>
            <div className="scatter-axis x-axis">Time</div>
            <div className="scatter-grid">
              {scatterPoints.map((point, index) => (
                <div 
                  key={index} 
                  className="plot-point" 
                  style={{ bottom: `${point.yVolumePercentage}%`, left: `${point.xTimePercentage}%` }}
                ></div>
              ))}
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
