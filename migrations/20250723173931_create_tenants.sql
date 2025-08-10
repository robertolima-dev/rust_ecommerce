-- Migration: create_tenants
-- Created at: Qua 23 Jul 2025 17:39:31 -03

CREATE TABLE tenants (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_type TEXT NOT NULL,
    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices para melhor performance
CREATE INDEX idx_tenants_user_id ON tenants(user_id);
CREATE INDEX idx_tenants_dt_created ON tenants(dt_created);
CREATE INDEX idx_tenants_dt_deleted ON tenants(dt_deleted);
