-- create_stock_prices.sql
CREATE TABLE stock_prices(
    id BIGSERIAL PRIMARY KEY,
    ticker TEXT NOT NULL UNIQUE,
    price_cents BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'AUD',
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)