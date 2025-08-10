-- Migration: create_users
-- Created at: Dom 29 Jun 2025 09:56:35 -03

CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    password TEXT NOT NULL,
    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices para melhor performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_dt_created ON users(dt_created);
CREATE INDEX idx_users_dt_deleted ON users(dt_deleted);
