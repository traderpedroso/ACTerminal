use crate::infrastructure::datafeed::types::DomData;

#[derive(Debug, Clone, Default)]
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

impl From<crate::infrastructure::datafeed::types::DomEntry> for UiDomEntry {
    fn from(entry: crate::infrastructure::datafeed::types::DomEntry) -> Self {
        Self {
            price: entry.price,
            size: entry.size,
        }
    }
}
