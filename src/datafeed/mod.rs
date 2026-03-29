pub mod dom;
pub mod quote;
pub mod symbols;
pub mod tick;
pub mod types;
pub mod zmq_provider;
pub mod zenoh_provider;

pub use dom::{process as process_dom, to_json_string as dom_to_json};
pub use quote::{process as process_quote, to_json_string as quote_to_json};
pub use symbols::*;
pub use tick::{process as process_tick, to_json_string as tick_to_json};
pub use types::*;

use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataFeedBackend {
    #[default]
    Zmq,
    #[allow(dead_code)]
    Zenoh,
}

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
        DataFeedBackend::Zmq => Arc::new(zmq_provider::ZmqProvider::new()),
        DataFeedBackend::Zenoh => Arc::new(zenoh_provider::ZenohProvider::new()),
    }
}

pub use DataFeedProvider as DataFeedSubscriber;
