ALTER TABLE stock_trades 
ADD COLUMN user_id BIGINT NOT NULL REFERENCES users(id);

CREATE INDEX idx_stock_trades_user_id_ticker ON stock_trades(user_id, ticker);
