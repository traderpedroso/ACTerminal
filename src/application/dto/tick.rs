use crate::infrastructure::datafeed::TradeSide;

#[derive(Debug, Clone, Default)]
pub struct UiTickData {
    pub time: String,
    pub price: f64,
    pub size: u64,
    pub side: TradeSide,
    pub decimals: usize,
}
