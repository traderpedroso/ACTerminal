use crate::infrastructure::datafeed::types::TickData;
use crate::infrastructure::datafeed::TradeSide;

#[derive(Debug, Clone, Default)]
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
