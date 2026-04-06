use std::sync::Arc;
use tokio::sync::RwLock;

use crate::application::dto::{DomData, QuoteData, TickData};

#[derive(Clone)]
pub struct MarketDataState {
    last_tick: Arc<RwLock<Option<TickData>>>,
    last_dom: Arc<RwLock<Option<DomData>>>,
    last_quote: Arc<RwLock<Option<QuoteData>>>,
}

impl MarketDataState {
    pub fn new(_initial_symbol: &str) -> Self {
        Self {
            last_tick: Arc::new(RwLock::new(None)),
            last_dom: Arc::new(RwLock::new(None)),
            last_quote: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn update_tick(&self, tick: TickData) {
        let mut last = self.last_tick.write().await;
        *last = Some(tick);
    }

    pub async fn update_dom(&self, dom: DomData) {
        let mut last = self.last_dom.write().await;
        *last = Some(dom);
    }

    pub async fn update_quote(&self, quote: QuoteData) {
        let mut last = self.last_quote.write().await;
        *last = Some(quote);
    }
}
