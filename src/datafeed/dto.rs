use crate::datafeed::{DomData, QuoteData, TickData, TradeSide};

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct UiTickData {
    pub id: i64,
    pub time: String,
    pub price: f64,
    pub size: u64,
    pub side: TradeSide,
    pub bid_price: f64,
    pub ask_price: f64,
    pub symbol: String,
    pub decimals: usize,
}

impl UiTickData {
    #[allow(dead_code)]
    pub fn from_tick_data(td: &TickData, symbol: &str, decimals: usize) -> Self {
        let time_ms = td.time;
        let secs = if time_ms > 1_000_000_000 {
            time_ms / 1000
        } else {
            time_ms
        };

        let h = (secs / 3600) % 24;
        let m = (secs / 60) % 60;
        let s = secs % 60;

        Self {
            id: td.id,
            time: format!("{:02}:{:02}:{:02}", h, m, s),
            price: td.price,
            size: td.size,
            side: TradeSide::from_str(&td.side),
            bid_price: td.bid_price,
            ask_price: td.ask_price,
            symbol: symbol.to_string(),
            decimals,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct UiDomData {
    pub contract_id: i64,
    pub timestamp: String,
    pub bids: Vec<UiDomEntry>,
    pub offers: Vec<UiDomEntry>,
    pub symbol: String,
    pub decimals: usize,
    pub tick_size: f64,
}

#[derive(Debug, Clone, Default)]
pub struct UiDomEntry {
    pub price: f64,
    pub size: f64,
}

impl From<DomData> for UiDomData {
    fn from(data: DomData) -> Self {
        Self {
            contract_id: data.contract_id,
            timestamp: data.timestamp,
            bids: data.bids.into_iter().map(From::from).collect(),
            offers: data.offers.into_iter().map(From::from).collect(),
            symbol: String::new(),
            decimals: 5,
            tick_size: 0.00005,
        }
    }
}

impl From<crate::datafeed::DomEntry> for UiDomEntry {
    fn from(entry: crate::datafeed::DomEntry) -> Self {
        Self {
            price: entry.price,
            size: entry.size,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
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
