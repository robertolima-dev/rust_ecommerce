-- Migration: create_orchestrators
-- Created at: Ter  1 Jul 2025 09:55:50 -03

CREATE TABLE orchestrators (
    id UUID PRIMARY KEY,
    app_name TEXT NOT NULL,
    app_url TEXT NOT NULL,
    app_token UUID NOT NULL UNIQUE,
    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices para melhor performance
CREATE INDEX idx_orchestrators_app_name ON orchestrators(app_name);
CREATE INDEX idx_orchestrators_app_token ON orchestrators(app_token);
CREATE INDEX idx_orchestrators_dt_deleted ON orchestrators(dt_deleted);
