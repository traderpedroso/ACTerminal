use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolDataType {
    Tick,
    Dom,
    Quote,
}

impl SymbolDataType {
    #[allow(dead_code)]
    pub fn prefix(&self) -> &'static str {
        match self {
            SymbolDataType::Tick => "ticks.",
            SymbolDataType::Dom => "doms.",
            SymbolDataType::Quote => "quotes.",
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

    pub async fn set_symbol(
        &self,
        symbol: &str,
        data_types: Vec<SymbolDataType>,
    ) -> Result<(), mpsc::error::SendError<SymbolChangeEvent>> {
        let old_symbol = self.current_symbol.read().await.clone();
        let new_symbol = Some(symbol.to_string());

        *self.current_symbol.write().await = new_symbol.clone();
        self.subscriptions
            .write()
            .await
            .insert(symbol.to_string(), data_types.clone());

        self.change_tx
            .send(SymbolChangeEvent {
                old_symbol,
                new_symbol,
                data_types,
            })
            .await
    }
}

/// Convert a CME futures code to its display name (Forex pair format).
/// Used in the symbol selector UI.
pub fn get_display_name(symbol: &str) -> &'static str {
    match symbol {
        "6E" => "EURUSD",
        "6B" => "GBPUSD",
        "6A" => "AUDUSD",
        "6N" => "NZDUSD",
        "6L" => "USDBRL",
        "6J" => "USDJPY",
        "6C" => "USDCAD",
        "6S" => "USDCHF",
        "6M" => "USDMXN",
        _ => "UNKNOWN",
    }
}

/// Convert a display name back to the internal CME futures code.
pub fn get_symbol_from_display(display: &str) -> &'static str {
    match display {
        "EURUSD" => "6E",
        "GBPUSD" => "6B",
        "AUDUSD" => "6A",
        "NZDUSD" => "6N",
        "USDBRL" => "6L",
        "USDJPY" => "6J",
        "USDCAD" => "6C",
        "USDCHF" => "6S",
        "USDMXN" => "6M",
        _ => "6E", // safe fallback
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
        SymbolConfig::new("6M").with_types(vec![
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
