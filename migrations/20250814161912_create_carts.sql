-- Migration: create_carts
-- Created at: Qui 14 Ago 2025 16:19:12 -03

-- 1) Enum de status do carrinho
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'cart_status') THEN
        CREATE TYPE cart_status AS ENUM (
            'ACTIVE',
            'CHECKOUT_IN_PROGRESS',
            'CONVERTED_TO_ORDER',
            'ABANDONED',
            'CANCELLED'
        );
    END IF;
END$$;

-- 2) Tabela de carrinhos
CREATE TABLE IF NOT EXISTS carts (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,

    status cart_status NOT NULL DEFAULT 'ACTIVE',
    currency CHAR(3) NOT NULL DEFAULT 'BRL',

    -- valores em centavos
    subtotal BIGINT NOT NULL DEFAULT 0 CHECK (subtotal >= 0),
    discount_total BIGINT NOT NULL DEFAULT 0 CHECK (discount_total >= 0),
    tax_total BIGINT NOT NULL DEFAULT 0 CHECK (tax_total >= 0),
    shipping_total BIGINT NOT NULL DEFAULT 0 CHECK (shipping_total >= 0),

    -- derivado (sem trigger)
    grand_total BIGINT GENERATED ALWAYS AS (
        subtotal - discount_total + tax_total + shipping_total
    ) STORED,

    version INT NOT NULL DEFAULT 0,
    expires_at TIMESTAMP,

    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices úteis
CREATE INDEX IF NOT EXISTS idx_carts_tenant           ON carts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_carts_user             ON carts(user_id);
CREATE INDEX IF NOT EXISTS idx_carts_status           ON carts(status);
CREATE INDEX IF NOT EXISTS idx_carts_dt_created       ON carts(dt_created);
CREATE INDEX IF NOT EXISTS idx_carts_dt_deleted       ON carts(dt_deleted);

-- Busca rápida por carrinho ativo de um usuário (ignorando soft-delete)
DROP INDEX IF EXISTS idx_carts_tenant_user_status;
CREATE INDEX IF NOT EXISTS idx_carts_tenant_user_status
  ON carts(tenant_id, user_id, status)
  WHERE dt_deleted IS NULL;

-- Garante 1 carrinho ACTIVE por usuário (por tenant)
DROP INDEX IF EXISTS uq_active_cart_per_user;
CREATE UNIQUE INDEX IF NOT EXISTS uq_active_cart_per_user
  ON carts(tenant_id, user_id)
  WHERE status = 'ACTIVE' AND dt_deleted IS NULL;

-- 3) Tabela de itens do carrinho
CREATE TABLE IF NOT EXISTS cart_items (
    id UUID PRIMARY KEY,
    cart_id UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,

    product_id UUID NOT NULL,
    variant_id UUID,

    -- snapshot de preço unitário em centavos
    unit_price BIGINT NOT NULL CHECK (unit_price >= 0),

    quantity INT NOT NULL CHECK (quantity > 0),

    -- totais por linha (centavos)
    line_discount_total BIGINT NOT NULL DEFAULT 0 CHECK (line_discount_total >= 0),
    line_tax_total      BIGINT NOT NULL DEFAULT 0 CHECK (line_tax_total >= 0),

    -- derivado (sem trigger)
    line_total BIGINT GENERATED ALWAYS AS (
        (unit_price * quantity) - line_discount_total + line_tax_total
    ) STORED,

    attributes_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,

    -- hash para deduplicação por atributos
    attributes_hash TEXT GENERATED ALWAYS AS (md5(attributes_snapshot::text)) STORED,

    dt_created TIMESTAMP NOT NULL DEFAULT now(),
    dt_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dt_deleted TIMESTAMP
);

-- Índices úteis
CREATE INDEX IF NOT EXISTS idx_cart_items_cart_id     ON cart_items(cart_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_product     ON cart_items(product_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_dt_deleted  ON cart_items(dt_deleted);

-- Itens ativos por carrinho (consulta frequente)
DROP INDEX IF EXISTS idx_cart_items_cart_active;
CREATE INDEX IF NOT EXISTS idx_cart_items_cart_active
  ON cart_items(cart_id)
  WHERE dt_deleted IS NULL;

-- Evita duplicar "mesmo item" no carrinho (mesmo produto/variante/atributos)
DROP INDEX IF EXISTS uq_cart_item_dedup;
CREATE UNIQUE INDEX IF NOT EXISTS uq_cart_item_dedup
  ON cart_items(cart_id, product_id, variant_id, attributes_hash)
  WHERE dt_deleted IS NULL;

-- (Opcional futuro)
-- ALTER TABLE cart_items
--   ADD CONSTRAINT fk_cart_items_product
--   FOREIGN KEY (product_id) REFERENCES products(id);
-- ALTER TABLE cart_items
--   ADD CONSTRAINT fk_cart_items_variant
--   FOREIGN KEY (variant_id) REFERENCES product_variants(id);
