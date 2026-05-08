-- Add migration script here
CREATE TABLE stock_prices(
    id INTEGER PRIMARY KEY,
    ticker TEXT NOT NULL UNIQUE,
    price_cents INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'AUD',
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
)