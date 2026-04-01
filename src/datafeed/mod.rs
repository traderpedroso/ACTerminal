pub mod dom;
pub mod dto;
pub mod provider;
pub mod quote;
pub mod symbols;
pub mod trade_side;
pub mod transformer;
pub mod types;

pub use dom::{process as process_dom, to_json_string as dom_to_json};
pub use quote::{process as process_quote, to_json_string as quote_to_json};
pub use symbols::*;
pub use trade_side::TradeSide;
pub use types::*;
pub use dto::{UiDomData, UiDomEntry, UiQuoteData, UiTickData};
pub use transformer::{transform_dom, transform_quote, transform_tick, get_decimals_for_symbol, get_tick_size_for_symbol, needs_inversion};

use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataFeedBackend {
    #[default]
    Zmq,
    #[allow(dead_code)]
    Zenoh,
}

#[allow(dead_code)]
pub trait DataFeedProvider: Send + Sync {
    fn subscribe(&self, symbol: &str, data_type: DataType);
    fn unsubscribe(&self, symbol: &str, data_type: DataType);
    fn is_subscribed(&self, symbol: &str, data_type: DataType) -> bool;
    #[allow(dead_code)]
    fn subscriptions_arc(&self) -> Arc<std::sync::Mutex<std::collections::HashMap<String, DataType>>>;
    fn start(&self) -> mpsc::Receiver<(String, DataType, String)>;
    fn shutdown(&self);
}

pub fn create_provider(backend: DataFeedBackend) -> Arc<dyn DataFeedProvider> {
    match backend {
        DataFeedBackend::Zmq => Arc::new(provider::zmq::ZmqProvider::new()),
        DataFeedBackend::Zenoh => Arc::new(provider::zenoh::ZenohProvider::new()),
    }
}

pub use DataFeedProvider as DataFeedSubscriber;
