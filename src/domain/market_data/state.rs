use std::sync::Arc;
use tokio::sync::RwLock;

use crate::application::dto::{DomData, QuoteData, TickData};

#[derive(Clone)]
pub struct MarketDataState {
    current_symbol: Arc<RwLock<String>>,
    last_tick: Arc<RwLock<Option<TickData>>>,
    last_dom: Arc<RwLock<Option<DomData>>>,
    last_quote: Arc<RwLock<Option<QuoteData>>>,
}

impl MarketDataState {
    pub fn new(initial_symbol: &str) -> Self {
        Self {
            current_symbol: Arc::new(RwLock::new(initial_symbol.to_string())),
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

    pub async fn get_last_tick(&self) -> Option<TickData> {
        self.last_tick.read().await.clone()
    }

    pub async fn get_last_dom(&self) -> Option<DomData> {
        self.last_dom.read().await.clone()
    }

    pub async fn get_last_quote(&self) -> Option<QuoteData> {
        self.last_quote.read().await.clone()
    }

    pub async fn get_symbol(&self) -> String {
        self.current_symbol.read().await.clone()
    }

    pub async fn set_symbol(&self, symbol: &str) {
        *self.current_symbol.write().await = symbol.to_string();
    }

    pub async fn clear(&self) {
        *self.last_tick.write().await = None;
        *self.last_dom.write().await = None;
        *self.last_quote.write().await = None;
    }
}
