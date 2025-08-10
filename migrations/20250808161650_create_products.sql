-- Migration: create_products
-- Created at: Sex  8 Ago 2025 16:16:50 -03

CREATE TABLE products (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    short_description TEXT,
    description TEXT,
    price NUMERIC(12,2) NOT NULL CHECK (price >= 0),
    stock_quantity INTEGER NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
    attributes JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices para melhor performance
CREATE INDEX idx_products_tenant_id  ON products(tenant_id);
CREATE INDEX idx_products_is_active  ON products(is_active);
CREATE INDEX idx_products_dt_created ON products(dt_created);
CREATE INDEX idx_products_dt_updated ON products(dt_updated);
CREATE INDEX idx_products_dt_deleted ON products(dt_deleted);
