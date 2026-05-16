import { headers } from 'next/headers';
import { getEndpoints } from '@/lib/metrics';

export default async function EndpointsPage() {
  const headersList = await headers();
  const ouId = headersList.get('x-org-unit-id') || 'mock-ou-id';
  const tenantId = headersList.get('x-tenant-id') || 'mock-tenant-id';

  const endpoints = await getEndpoints(tenantId, ouId);

  return (
    <div className="dashboard-container">
      <header className="dashboard-header">
        <h2>Endpoint Management</h2>
        <p>Active telemetry agents across Organizational Unit: <strong>{ouId}</strong></p>
      </header>

      <div className="table-container">
        <table className="monochrome-table">
          <thead>
            <tr>
              <th>Endpoint ID</th>
              <th>Assigned User</th>
              <th>Operating System</th>
              <th>Agent Version</th>
              <th>Status</th>
              <th>Last Sync</th>
            </tr>
          </thead>
          <tbody>
            {endpoints.length === 0 ? (
              <tr>
                <td colSpan={6} style={{ textAlign: 'center', color: '#666' }}>No active endpoints found in ClickHouse for this OU.</td>
              </tr>
            ) : (
              endpoints.map((ep) => (
                <tr key={ep.id}>
                  <td><strong>{ep.id}</strong></td>
                  <td>{ep.user}</td>
                  <td>{ep.os}</td>
                  <td>{ep.version}</td>
                  <td>
                    <span className={`status-badge ${ep.status === 'ONLINE' ? 'solid' : 'hollow'}`}>
                      {ep.status}
                    </span>
                  </td>
                  <td>{ep.lastSync}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
