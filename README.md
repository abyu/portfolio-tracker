## DB setup
### Prereqs
- `cargo install sqlx-cli --no-default-features --features sqlite` to instal sqlX

### Creating a new SQLite DB
- `sqlx database create --database-url sqlite:portfolio.db`

### New migration
- `sqlx migrate add create_stock_trades`

### Run migration
- `sqlx migrate run --database-url sqlite:portfolio.db`