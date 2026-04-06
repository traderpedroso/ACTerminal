use crate::infrastructure::datafeed::parsing::{dom as parse_dom, quote as parse_quote};
use crate::infrastructure::datafeed::types::TickData;
use crate::infrastructure::datafeed::types::TickPacket;
use crate::infrastructure::datafeed::{DataFeedProvider, DataType};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicI64, AtomicI8, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use zenoh::config::Config;

#[allow(clippy::let_underscore_future)]
static LAST_PRICE_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_BID_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_ASK_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_DIRECTION: AtomicI8 = AtomicI8::new(0);

pub struct ZenohProvider {
    subscriptions: Arc<Mutex<HashMap<String, DataType>>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
    cancel_txs: Arc<Mutex<HashMap<String, mpsc::Sender<()>>>>,
}

impl ZenohProvider {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(None)),
            cancel_txs: Arc::new(Mutex::new(HashMap::new())),
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
        
        if let Some(cancel_tx) = self.cancel_txs.lock().ok().and_then(|mut g| g.remove(&key)) {
            let _ = cancel_tx.send(());
            drop(cancel_tx);
        }
        
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
        let (tx, rx) = mpsc::channel(100);
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        if let Ok(mut guard) = self.shutdown_tx.lock() {
            *guard = Some(shutdown_tx);
        }

        let subscriptions = self.subscriptions.clone();
        let cancel_txs = self.cancel_txs.clone();

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime");

            rt.block_on(async {
                let endpoint = std::env::var("ZENOH_ENDPOINT").unwrap_or_else(|_| "tcp/127.0.0.1:7447".to_string());
                
                let config = Config::from_json5(&format!(r#"{{
                    "mode": "client",
                    "connect": {{
                        "endpoints": ["{}"]
                    }}
                }}"#, endpoint)).expect("Invalid Zenoh config");

                let session = match zenoh::open(config).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[ZENOH] Failed to connect: {}", e);
                        return;
                    }
                };

                let tx = tx.clone();
                let subscriptions = subscriptions.clone();
                let cancel_txs = cancel_txs.clone();
                let mut shutdown = shutdown_rx;
                
                let active_subscribers: Arc<Mutex<HashMap<String, Arc<()>>>> = 
                    Arc::new(Mutex::new(HashMap::new()));

                loop {
                    let subs = {
                        let guard = subscriptions.lock().unwrap();
                        guard.clone()
                    };

                    let active_keys: Vec<String> = {
                        let active = active_subscribers.lock().unwrap();
                        active.keys().cloned().collect()
                    };

                    for key in &active_keys {
                        if !subs.contains_key(key) {
                            if let Some(cancel_tx) = cancel_txs.lock().ok().and_then(|mut g| g.remove(key)) {
                                let _ = cancel_tx.send(());
                                drop(cancel_tx);
                            }
                            if let Ok(mut active) = active_subscribers.lock() {
                                active.remove(key);
                            }
                        }
                    }

                    for (key, data_type) in &subs {
                        let is_active = active_subscribers
                            .lock()
                            .map(|active| active.contains_key(key))
                            .unwrap_or(false);
                        
                        if !is_active {
                            let keyexpr = key.clone();
                            let tx = tx.clone();
                            let symbol = key.strip_prefix(data_type.prefix()).unwrap_or(key).to_string();
                            let symbol_clone = symbol.clone();
                            let data_type_clone = *data_type;
                            let keyexpr_clone = keyexpr.clone();

                            let session_clone = session.clone();
                            let active_clone = active_subscribers.clone();
                            let cancel_txs_clone = cancel_txs.clone();
                            
                            let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
                            
                            if let Ok(mut cts) = cancel_txs.lock() {
                                cts.insert(key.clone(), cancel_tx);
                            }
                            
                            let handle = if let Ok(mut active) = active_subscribers.lock() {
                                let h = Arc::new(());
                                active.insert(key.clone(), h.clone());
                                h
                            } else {
                                continue;
                            };
                            let _handle = handle;

                            tokio::spawn(async move {
                                let subscriber = match session_clone.declare_subscriber(&keyexpr_clone).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        eprintln!("[ZENOH] Falha subscriber: {}", e);
                                        if let Ok(mut active) = active_clone.lock() {
                                            active.remove(&keyexpr_clone);
                                        }
                                        return;
                                    }
                                };
                                
                                loop {
                                    tokio::select! {
                                        result = subscriber.recv_async() => {
                                            match result {
                                                Ok(sample) => {
                                                    let payload = sample.payload().to_bytes().to_vec();
                                                    let payload_str = String::from_utf8_lossy(&payload).to_string();
                                                    
                                                    let result = match data_type_clone {
                                                        DataType::Tick => process_tick_message(&payload_str),
                                                        DataType::Dom => parse_dom::to_json_string(&payload_str),
                                                        DataType::Quote => parse_quote::to_json_string(&payload_str),
                                                    };

                                                    if let Some(formatted) = result {
                                                        let _ = tx.send((symbol_clone.clone(), data_type_clone, formatted)).await;
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("[ZENOH] Erro: {}", e);
                                                    break;
                                                }
                                            }
                                        }
                                        _ = cancel_rx.recv() => {
                                            let _ = subscriber.undeclare().await;
                                            break;
                                        }
                                    }
                                }
                                
                                if let Ok(mut active) = active_clone.lock() {
                                    active.remove(&keyexpr_clone);
                                }
                                if let Ok(mut cts) = cancel_txs_clone.lock() {
                                    cts.remove(&keyexpr_clone);
                                }
                            });
                        }
                    }

                    tokio::select! {
                        _ = shutdown.recv() => {
                            let keys: Vec<String> = active_subscribers
                                .lock()
                                .map(|m| m.keys().cloned().collect())
                                .unwrap_or_default();
                            
                            for key in &keys {
                                if let Some(cancel_tx) = cancel_txs.lock().ok().and_then(|mut g| g.remove(key)) {
                                    let _ = cancel_tx.send(());
                                    drop(cancel_tx);
                                }
                            }
                            break;
                        }
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                    }
                }
            });
        });

        rx
    }

    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.shutdown_tx.lock()
            && let Some(tx) = guard.take()
        {
            drop(tx);
        }
    }

    #[allow(dead_code)]
    pub fn reset_state() {
        LAST_PRICE_TICKS.store(0, Ordering::SeqCst);
        LAST_BID_TICKS.store(0, Ordering::SeqCst);
        LAST_ASK_TICKS.store(0, Ordering::SeqCst);
        LAST_DIRECTION.store(0, Ordering::SeqCst);
    }
}

fn process_tick_message(json: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct TickMessage {
        #[serde(rename = "ticks")]
        ticks: Vec<TickPacket>,
    }

    let msg: TickMessage = serde_json::from_str(json).ok()?;
    if msg.ticks.is_empty() {
        return None;
    }

    let mut results = Vec::new();

    for packet in msg.ticks {
        if packet.eoh {
            continue;
        }

        let base_price = packet.base_price;
        let base_time = packet.base_timestamp;
        let tick_size = packet.tick_size;

        let mut current_ask_ticks = LAST_ASK_TICKS.load(Ordering::SeqCst);
        let mut current_bid_ticks = LAST_BID_TICKS.load(Ordering::SeqCst);

        for tk in packet.ticks {
            let trade_ticks = base_price + tk.price;
            let trade_time = base_time + tk.time;

            if let Some(a_val) = tk.ask {
                current_ask_ticks = base_price + a_val;
                LAST_ASK_TICKS.store(current_ask_ticks, Ordering::SeqCst);
            }
            if let Some(b_val) = tk.bid {
                current_bid_ticks = base_price + b_val;
                LAST_BID_TICKS.store(current_bid_ticks, Ordering::SeqCst);
            }

            let bid_size = tk.bid_size.unwrap_or(0.0);
            let ask_size = tk.ask_size.unwrap_or(0.0);

            let last_price_ticks = LAST_PRICE_TICKS.load(Ordering::SeqCst);
            let last_dir_code = LAST_DIRECTION.load(Ordering::SeqCst);

            let (side_str, dir_code) = if last_price_ticks > 0 {
                if trade_ticks > last_price_ticks {
                    ("BUY_MARKET", 1)
                } else if trade_ticks < last_price_ticks {
                    ("SELL_MARKET", -1)
                } else {
                    match last_dir_code {
                        1 | 2 => ("BUY_PENDING", 2),
                        -1 | -2 => ("SELL_PENDING", -2),
                        _ => ("MID_PRICE_TRADE", 0),
                    }
                }
            } else if current_ask_ticks > 0 && trade_ticks >= current_ask_ticks {
                ("BUY_MARKET", 1)
            } else if current_bid_ticks > 0 && trade_ticks <= current_bid_ticks {
                ("SELL_MARKET", -1)
            } else {
                ("MID_PRICE_TRADE", 0)
            };

            LAST_PRICE_TICKS.store(trade_ticks, Ordering::SeqCst);
            LAST_DIRECTION.store(dir_code, Ordering::SeqCst);

            let trade_price = ((trade_ticks as f64) * tick_size * 100000.0).round() / 100000.0;
            let bid_price = ((current_bid_ticks as f64) * tick_size * 100000.0).round() / 100000.0;
            let ask_price = ((current_ask_ticks as f64) * tick_size * 100000.0).round() / 100000.0;

            results.push(TickData {
                id: tk.tick_id,
                time: trade_time,
                price_ticks: trade_ticks,
                price: trade_price,
                size: tk.size,
                side: side_str.to_string(),
                bid_price,
                bid_size,
                ask_price,
                ask_size,
            });
        }
    }

    if results.is_empty() {
        None
    } else {
        serde_json::to_string_pretty(&results).ok()
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
