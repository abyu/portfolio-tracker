-- create_stock_trades.sql
CREATE TABLE stock_trades(
    id BIGSERIAL PRIMARY KEY,
    ticker TEXT NOT NULL,
    trade_type TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    units DOUBLE PRECISION NOT NULL,
    market_price_cents BIGINT NOT NULL,
    fees_cents BIGINT NOT NULL,
    amount_cents BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'AUD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)