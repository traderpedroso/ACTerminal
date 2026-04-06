use crate::infrastructure::datafeed::types::DomData;

#[derive(Debug, Clone, Default)]
pub struct UiDomData {
    pub bids: Vec<UiDomEntry>,
    pub offers: Vec<UiDomEntry>,
    pub decimals: usize,
}

#[derive(Debug, Clone, Default)]
pub struct UiDomEntry {
    pub price: f64,
    pub size: f64,
}

impl From<DomData> for UiDomData {
    fn from(data: DomData) -> Self {
        Self {
            bids: data.bids.into_iter().map(From::from).collect(),
            offers: data.offers.into_iter().map(From::from).collect(),
            decimals: 5,
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
