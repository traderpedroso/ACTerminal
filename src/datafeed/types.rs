use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Tick,
    Dom,
    Quote,
}

impl DataType {
    pub fn prefix(&self) -> &'static str {
        match self {
            DataType::Tick => "tick.",
            DataType::Dom => "dom.",
            DataType::Quote => "quote.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickPacket {
    pub id: i64,
    #[serde(rename = "s")]
    pub source: String,
    #[serde(rename = "td")]
    pub trade_date: String,
    #[serde(rename = "bp")]
    pub base_price: i64,
    #[serde(rename = "bt")]
    pub base_timestamp: i64,
    #[serde(rename = "ts")]
    pub tick_size: f64,
    #[serde(rename = "tks")]
    pub ticks: Vec<Tick>,
    #[serde(default)]
    pub eoh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    #[serde(rename = "t")]
    pub time: i64,
    #[serde(rename = "p")]
    pub price: i64,
    #[serde(rename = "s")]
    pub size: u64,
    #[serde(rename = "b", default)]
    pub bid: Option<i64>,
    #[serde(rename = "a", default)]
    pub ask: Option<i64>,
    #[serde(rename = "bs", default)]
    pub bid_size: Option<f64>,
    #[serde(rename = "as", default)]
    pub ask_size: Option<f64>,
    #[serde(rename = "id")]
    pub tick_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickMessage {
    #[serde(rename = "ticks")]
    pub ticks: Vec<TickPacket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickData {
    pub id: i64,
    pub time: i64,
    pub price_ticks: i64,
    pub price: f64,
    pub size: u64,
    pub side: String,
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomEntry {
    #[serde(deserialize_with = "deserialize_price")]
    pub price: f64,
    pub size: f64,
}

fn deserialize_price<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PriceOrString {
        Number(f64),
        String(String),
    }
    let price = PriceOrString::deserialize(deserializer)?;
    match price {
        PriceOrString::Number(n) => Ok(n),
        PriceOrString::String(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomMessage {
    #[serde(rename = "doms")]
    pub doms: Vec<DomData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomData {
    #[serde(rename = "contractId")]
    pub contract_id: i64,
    pub timestamp: String,
    pub bids: Vec<DomEntry>,
    pub offers: Vec<DomEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteEntry {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteEntries {
    #[serde(rename = "Bid", default)]
    pub bid: Option<QuoteEntry>,
    #[serde(rename = "TotalTradeVolume", default)]
    pub total_trade_volume: Option<QuoteEntry>,
    #[serde(rename = "Offer", default)]
    pub offer: Option<QuoteEntry>,
    #[serde(rename = "LowPrice", default)]
    pub low_price: Option<QuoteEntry>,
    #[serde(rename = "Trade", default)]
    pub trade: Option<QuoteEntry>,
    #[serde(rename = "OpenInterest", default)]
    pub open_interest: Option<QuoteEntry>,
    #[serde(rename = "OpeningPrice", default)]
    pub opening_price: Option<QuoteEntry>,
    #[serde(rename = "HighPrice", default)]
    pub high_price: Option<QuoteEntry>,
    #[serde(rename = "SettlementPrice", default)]
    pub settlement_price: Option<QuoteEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    pub timestamp: String,
    #[serde(rename = "contractId")]
    pub contract_id: i64,
    pub entries: QuoteEntries,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteMessage {
    #[serde(rename = "quotes")]
    pub quotes: Vec<QuoteData>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MarketData {
    Tick(TickData),
    Dom(DomData),
    Quote(QuoteData),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Subscription {
    pub symbol: String,
    pub data_type: DataType,
}
