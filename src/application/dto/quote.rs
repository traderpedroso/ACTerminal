use crate::infrastructure::datafeed::types::QuoteData;

#[derive(Debug, Clone, Default)]
pub struct UiQuoteData {
    pub timestamp: String,
    pub contract_id: i64,
    pub bid_price: Option<f64>,
    pub bid_size: Option<f64>,
    pub ask_price: Option<f64>,
    pub ask_size: Option<f64>,
    pub last_price: Option<f64>,
    pub last_size: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub open_price: Option<f64>,
    pub volume: Option<f64>,
    pub symbol: String,
    pub decimals: usize,
}

impl From<QuoteData> for UiQuoteData {
    fn from(data: QuoteData) -> Self {
        let entries = data.entries;
        let bid = entries.bid.clone();
        let offer = entries.offer.clone();
        let trade = entries.trade.clone();
        let high = entries.high_price.clone();
        let low = entries.low_price.clone();
        let open = entries.opening_price.clone();
        let vol = entries.total_trade_volume.clone();
        Self {
            timestamp: data.timestamp,
            contract_id: data.contract_id,
            bid_price: bid.as_ref().map(|e| e.price),
            bid_size: bid.map(|e| e.size),
            ask_price: offer.as_ref().map(|e| e.price),
            ask_size: offer.map(|e| e.size),
            last_price: trade.as_ref().map(|e| e.price),
            last_size: trade.map(|e| e.size),
            high_price: high.as_ref().map(|e| e.price),
            low_price: low.as_ref().map(|e| e.price),
            open_price: open.as_ref().map(|e| e.price),
            volume: vol.map(|e| e.size),
            symbol: String::new(),
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
