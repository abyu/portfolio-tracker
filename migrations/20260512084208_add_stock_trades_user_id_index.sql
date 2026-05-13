-- Add migration script here
CREATE INDEX idx_stock_trades_user_id ON stock_trades(user_id);