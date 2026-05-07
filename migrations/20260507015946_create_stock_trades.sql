-- Add migration script here
CREATE TABLE stock_trades(
    id INTEGER PRIMARY KEY,
    ticker TEXT NOT NULL,
    trade_type TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    units REAL NOT NULL,
    market_price_cents INTEGER NOT NULL,
    fees_cents INTEGER NOT NULL,
    amount_cents INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'AUD',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
)