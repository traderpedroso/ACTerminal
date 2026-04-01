use crate::datafeed::{DataFeedProvider, DataType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct ZenohProvider {
    subscriptions: Arc<Mutex<HashMap<String, DataType>>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

impl ZenohProvider {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn subscribe(&self, symbol: &str, data_type: DataType) {
        let key = format!("{}{}", data_type.prefix(), symbol);
        if let Ok(mut guard) = self.subscriptions.lock() {
            guard.insert(key, data_type);
        }
    }

    pub fn unsubscribe(&self, symbol: &str, data_type: DataType) {
        let key = format!("{}{}", data_type.prefix(), symbol);
        if let Ok(mut guard) = self.subscriptions.lock() {
            guard.remove(&key);
        }
    }

    pub fn is_subscribed(&self, symbol: &str, data_type: DataType) -> bool {
        let key = format!("{}{}", data_type.prefix(), symbol);
        self.subscriptions
            .lock()
            .map(|guard| guard.contains_key(&key))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn subscriptions_arc(&self) -> Arc<Mutex<HashMap<String, DataType>>> {
        self.subscriptions.clone()
    }

    pub fn start(&self) -> mpsc::Receiver<(String, DataType, String)> {
        let (_tx, rx) = mpsc::channel(100);

        log::warn!("[ZENOH] Provider not yet implemented - using stub");

        rx
    }

    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.shutdown_tx.lock()
            && let Some(tx) = guard.take()
        {
            drop(tx);
        }
    }
}

impl Default for ZenohProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataFeedProvider for ZenohProvider {
    fn subscribe(&self, symbol: &str, data_type: DataType) {
        Self::subscribe(self, symbol, data_type);
    }

    fn unsubscribe(&self, symbol: &str, data_type: DataType) {
        Self::unsubscribe(self, symbol, data_type);
    }

    fn is_subscribed(&self, symbol: &str, data_type: DataType) -> bool {
        Self::is_subscribed(self, symbol, data_type)
    }

    fn subscriptions_arc(&self) -> Arc<Mutex<HashMap<String, DataType>>> {
        Self::subscriptions_arc(self)
    }

    fn start(&self) -> mpsc::Receiver<(String, DataType, String)> {
        Self::start(self)
    }

    fn shutdown(&self) {
        Self::shutdown(self)
    }
}
