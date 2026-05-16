-- Schema v1.0: Control Plane State

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE tenants (
    tenant_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_name VARCHAR(255) NOT NULL,
    data_residency_region VARCHAR(50) DEFAULT 'ap-southeast-2',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'archived'))
);

CREATE TABLE endpoints (
    endpoint_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    assigned_user_hash VARCHAR(256) NOT NULL,
    os_family VARCHAR(20) NOT NULL CHECK (os_family IN ('darwin', 'win32')),
    agent_version VARCHAR(50) NOT NULL,
    public_key_fingerprint VARCHAR(256),
    last_checkin TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'enrolled' CHECK (status IN ('enrolled', 'active', 'quarantine', 'revoked'))
);

CREATE INDEX idx_endpoints_tenant ON endpoints(tenant_id);
CREATE INDEX idx_endpoints_status ON endpoints(status);
