use crate::service::portfolio_service::StockTradeRepository;
use crate::service::price_update_service::{PriceUpdateService, StockPriceRepository, TickerPriceProvider};

pub struct PriceUpdateTask<T: TickerPriceProvider, S: StockPriceRepository, P: StockTradeRepository> {
    price_update_service: PriceUpdateService<T, S>,
    stocks_repo: P
}


impl<T: TickerPriceProvider, S: StockPriceRepository, P: StockTradeRepository> PriceUpdateTask<T, S, P> {
    pub fn new(price_update_service: PriceUpdateService<T, S>, stocks_repo: P) -> Self {
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
