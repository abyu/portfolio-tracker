use csv::Reader;

use crate::models::stock_trade::NewStockTrade;

#[derive(Debug, serde::Deserialize)]
pub struct CSVRecord {
    #[serde(rename = "Ticker")]
    pub ticker: String,
    #[serde(rename = "Trade Type")]
    pub trade_type: String,
    #[serde(rename = "Trade Date")]
    pub trade_date: String,
    #[serde(rename = "Units")]
    pub units: f64,
    #[serde(rename = "Unit Price")]
    pub market_price_cents: i64,
    #[serde(rename = "Fees")]
    pub fees_cents: i64,
    #[serde(rename = "Total amount")]
    pub amount_cents: i64,
    #[serde(rename = "Currency")]
    pub currency: String,
}

impl CSVRecord {
    pub fn into_trade(self) -> Result<NewStockTrade, ParseError> {
        let date = chrono::NaiveDate::parse_from_str(&self.trade_date, "%d %b %Y")
            .map_err(|_| ParseError::InvalidDate(self.trade_date))?;

        Ok(NewStockTrade {
            ticker: self.ticker,
            trade_type: self.trade_type,
            trade_date: date.to_string(),
            units: self.units,
            market_price_cents: self.market_price_cents,
            fees_cents: self.fees_cents,
            amount_cents: self.amount_cents,
            currency: self.currency,
        })
    }
}

pub fn parse_csv_to_trades(csv_content: &str) -> Result<Vec<NewStockTrade>, ParseError> {
    let mut reader = Reader::from_reader(csv_content.as_bytes());

    reader
        .deserialize::<CSVRecord>()
        .map(|r| {
            r.map_err(|e| ParseError::InvalidContent(e.to_string()))
                .and_then(|record| record.into_trade())
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid csv content {0}")]
    InvalidContent(String),
    #[error("Invalid date {0}")]
    InvalidDate(String),
}
