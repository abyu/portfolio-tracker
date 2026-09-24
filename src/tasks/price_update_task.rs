use std::sync::Arc;

use crate::service::price_service::PriceService;

pub struct PriceUpdateTask<P: StockTradeRepository> {
    price_update_service: Arc<dyn PriceService>,
    stocks_repo: P,
}

#[async_trait::async_trait]
pub trait StockTradeRepository {
    async fn get_tickers(&self) -> Result<Vec<String>, sqlx::Error>;
}

impl<P: StockTradeRepository> PriceUpdateTask<P> {
    pub fn new(price_update_service: Arc<dyn PriceService>, stocks_repo: P) -> Self {
        Self {
            price_update_service,
            stocks_repo,
        }
    }

    pub async fn start(&self) -> Result<(), PriceTaskError> {
        let tickers = self.stocks_repo.get_tickers().await?;
        self.price_update_service.update_prices(tickers).await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PriceTaskError {
    #[error("Update error: {0}")]
    PriceUpdateError(#[from] crate::service::price_service::PriceError),
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error),
}
