-- create_trade_confirmation_import_jobs.sql
CREATE TABLE trade_confirmation_import_jobs(
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    file_name TEXT,
    file_content BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    imported_rows INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
