import { createClient } from '@clickhouse/client';

export const clickhouse = createClient({
  host: process.env.CLICKHOUSE_HOST ?? 'http://localhost:8123',
  username: process.env.CLICKHOUSE_USER ?? 'default',
  password: process.env.CLICKHOUSE_PASSWORD ?? '',
  database: 'default',
  // Required for enterprise timeouts and connection pooling
  request_timeout: 30000, 
});
