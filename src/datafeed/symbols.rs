use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolDataType {
    Tick,
    Dom,
    Quote,
}

impl SymbolDataType {
    pub fn prefix(&self) -> &'static str {
        match self {
            SymbolDataType::Tick => "tick.",
            SymbolDataType::Dom => "dom.",
            SymbolDataType::Quote => "quote.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolConfig {
    pub name: String,
    pub enabled_types: Vec<SymbolDataType>,
}

impl SymbolConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled_types: vec![
                SymbolDataType::Tick,
                SymbolDataType::Dom,
                SymbolDataType::Quote,
            ],
        }
    }

    pub fn with_types(mut self, types: Vec<SymbolDataType>) -> Self {
        self.enabled_types = types;
        self
    }
}

pub struct SymbolManager {
    current_symbol: Arc<RwLock<Option<String>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<SymbolDataType>>>>,
    change_tx: mpsc::Sender<SymbolChangeEvent>,
}

#[derive(Debug, Clone)]
pub struct SymbolChangeEvent {
    pub old_symbol: Option<String>,
    pub new_symbol: Option<String>,
    pub data_types: Vec<SymbolDataType>,
}

impl SymbolManager {
    pub fn new(change_tx: mpsc::Sender<SymbolChangeEvent>) -> Self {
        Self {
            current_symbol: Arc::new(RwLock::new(None)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            change_tx,
        }
    }

    pub async fn set_symbol(&self, symbol: &str, data_types: Vec<SymbolDataType>) {
        let old_symbol = self.current_symbol.read().await.clone();
        let new_symbol = Some(symbol.to_string());

        *self.current_symbol.write().await = new_symbol.clone();
        self.subscriptions
            .write()
            .await
            .insert(symbol.to_string(), data_types.clone());

        let _ = self
            .change_tx
            .send(SymbolChangeEvent {
                old_symbol,
                new_symbol,
                data_types,
            })
            .await;
    }

    pub async fn get_current_symbol(&self) -> Option<String> {
        self.current_symbol.read().await.clone()
    }

    pub async fn get_subscriptions(&self) -> HashMap<String, Vec<SymbolDataType>> {
        self.subscriptions.read().await.clone()
    }

    pub fn subscriptions_arc(&self) -> Arc<RwLock<HashMap<String, Vec<SymbolDataType>>>> {
        self.subscriptions.clone()
    }

    pub fn current_symbol_arc(&self) -> Arc<RwLock<Option<String>>> {
        self.current_symbol.clone()
    }
}

pub fn get_available_symbols() -> Vec<SymbolConfig> {
    vec![
        SymbolConfig::new("6L").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6E").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6A").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6B").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6C").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6J").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6S").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
        SymbolConfig::new("6N").with_types(vec![
            SymbolDataType::Tick,
            SymbolDataType::Dom,
            SymbolDataType::Quote,
        ]),
    ]
}
