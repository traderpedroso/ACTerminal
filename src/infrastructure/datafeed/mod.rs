pub mod parsing;
pub mod provider;
pub mod symbols;
pub mod transformer;
pub mod trade_side;
pub mod types;

pub use symbols::*;
pub use trade_side::TradeSide;
pub use types::*;

use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataFeedBackend {
    #[default]
    Zenoh,
}

#[allow(dead_code)]
pub trait DataFeedProvider: Send + Sync {
    fn subscribe(&self, symbol: &str, data_type: DataType);
    fn unsubscribe(&self, symbol: &str, data_type: DataType);
    fn is_subscribed(&self, symbol: &str, data_type: DataType) -> bool;
    fn subscriptions_arc(&self) -> Arc<std::sync::Mutex<std::collections::HashMap<String, DataType>>>;
    fn start(&self) -> mpsc::Receiver<(String, DataType, String)>;
    fn shutdown(&self);
}

pub fn create_provider(backend: DataFeedBackend) -> Arc<dyn DataFeedProvider> {
    match backend {
        DataFeedBackend::Zenoh => Arc::new(provider::ZenohProvider::new()),
    }
}

pub use transformer::{transform_dom, transform_quote, transform_tick};
pub use symbols::{get_display_name, get_symbol_from_display};
