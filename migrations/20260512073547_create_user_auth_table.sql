-- Add migration script here
CREATE TABLE user_auth (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    provider TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    credential TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_id)
);