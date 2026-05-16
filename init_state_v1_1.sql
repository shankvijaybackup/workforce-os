BEGIN;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE org_units (
    ou_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    parent_ou_id UUID REFERENCES org_units(ou_id),
    ou_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    ou_id UUID REFERENCES org_units(ou_id),
    email_hash VARCHAR(256) UNIQUE NOT NULL,
    role VARCHAR(50) DEFAULT 'employee' CHECK (role IN ('employee', 'manager', 'admin'))
);

-- Alter existing endpoints table to map directly to the user identity
ALTER TABLE endpoints
ADD COLUMN user_id UUID REFERENCES users(user_id);

COMMIT;
