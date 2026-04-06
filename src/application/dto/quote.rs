use crate::infrastructure::datafeed::types::QuoteData;

#[derive(Debug, Clone, Default)]
pub struct UiQuoteData {
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub last_price: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub open_price: Option<f64>,
    pub decimals: usize,
}

impl From<QuoteData> for UiQuoteData {
    fn from(data: QuoteData) -> Self {
        let entries = data.entries;
        let high = entries.high_price.clone();
        let low = entries.low_price.clone();
        let open = entries.opening_price.clone();
        Self {
            bid_price: entries.bid.as_ref().map(|e| e.price),
            ask_price: entries.offer.as_ref().map(|e| e.price),
            last_price: entries.trade.as_ref().map(|e| e.price),
            high_price: high.as_ref().map(|e| e.price),
            low_price: low.as_ref().map(|e| e.price),
            open_price: open.as_ref().map(|e| e.price),
            decimals: 5,
        }
    }
}

impl UiQuoteData {
    pub fn high(&self) -> f64 {
        self.high_price.unwrap_or(0.0)
    }

    pub fn low(&self) -> f64 {
        self.low_price.unwrap_or(0.0)
    }

    pub fn open(&self) -> f64 {
        self.open_price.unwrap_or(0.0)
    }
}
