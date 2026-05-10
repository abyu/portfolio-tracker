use crate::service::price_update_service::{PriceUpdateService, StockPriceRepository, TickerPriceProvider};
use crate::db::sqlite_stock_trade_repository::SqliteStockTradeRepository;

pub struct PriceUpdateTask<T: TickerPriceProvider, S: StockPriceRepository> {
    price_update_service: PriceUpdateService<T, S>,
    stocks_repo: SqliteStockTradeRepository
}

impl<T: TickerPriceProvider, S: StockPriceRepository> PriceUpdateTask<T, S> {
    pub fn new(price_update_service: PriceUpdateService<T, S>, stocks_repo: SqliteStockTradeRepository) -> Self {
        Self { price_update_service, stocks_repo }
    }

    pub async fn start(&self) -> Result<(), PriceTaskError> {
        let tickers = self.stocks_repo.get_tickers().await?;
        self.price_update_service.update_prices(tickers).await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PriceTaskError{
    #[error("Update error: {0}")]
    PriceUpdateError(#[from] crate::service::price_update_service::PriceUpdateError),
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error)
}
