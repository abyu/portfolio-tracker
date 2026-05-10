## DB setup
### Prereqs
- `cargo install sqlx-cli --no-default-features --features postgres` to install sqlx with postgres driver

### Creating a new SQLite DB
- `sqlx database create --database-url postgres://postgres:password@localhost:5432/portfolio_tracker`

### New migration
- `sqlx migrate add create_stock_trades`

### Run migration
- `sqlx migrate run --database-url postgres://postgres:password@localhost:5432/portfolio_tracker`