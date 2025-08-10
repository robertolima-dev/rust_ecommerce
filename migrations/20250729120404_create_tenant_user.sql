-- Migration: create_tenant_user
-- Created at: Qua 23 Jul 2025 17:39:31 -03

CREATE TABLE tenant_users (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices para melhor performance
CREATE INDEX idx_tenant_users_user_id ON tenant_users(user_id);
CREATE INDEX idx_tenant_users_tenant_id ON tenant_users(tenant_id);
CREATE INDEX idx_tenant_users_dt_created ON tenant_users(dt_created);
CREATE INDEX idx_tenant_users_dt_deleted ON tenant_users(dt_deleted);
