use crate::datafeed::{DataFeedProvider, DataType, dom_to_json, quote_to_json, types::TickData};
use crate::datafeed::types::{TickPacket, Tick};
use log;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicI64, AtomicI8, Ordering};
use tokio::sync::mpsc;
use zmq::Context;

static LAST_PRICE_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_BID_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_ASK_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_DIRECTION: AtomicI8 = AtomicI8::new(0);

pub struct ZmqProvider {
    context: Arc<Context>,
    subscriptions: Arc<Mutex<HashMap<String, DataType>>>,
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

impl ZmqProvider {
    pub fn new() -> Self {
        Self {
            context: Arc::new(Context::new()),
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
        let (tx, rx) = mpsc::channel(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let subscriptions = self.subscriptions.clone();
        let context = self.context.clone();

        if let Ok(mut guard) = self.shutdown_tx.lock() {
            *guard = Some(shutdown_tx);
        }

        thread::spawn(move || {
            let socket = match context.socket(zmq::SUB) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[SUB] Failed to create socket: {}", e);
                    return;
                }
            };

            if let Err(e) = socket.connect("tcp://127.0.0.1:5555") {
                log::error!("[SUB] Failed to connect: {}", e);
                return;
            }

            let socket = Arc::new(Mutex::new(socket));
            let socket_clone = socket.clone();
            let subscriptions_clone = subscriptions.clone();

            thread::spawn(move || {
                let mut current_subscriptions: HashMap<String, DataType> = HashMap::new();

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(50));

                    let subs = match subscriptions_clone.lock() {
                        Ok(guard) => guard.clone(),
                        Err(_) => continue,
                    };

                    log::debug!(
                        "[SUB-MGR] Current ZeroMQ subs: {:?}",
                        current_subscriptions.keys().collect::<Vec<_>>()
                    );
                    log::debug!(
                        "[SUB-MGR] Target subs: {:?}",
                        subs.keys().collect::<Vec<_>>()
                    );

                    let mut to_remove: Vec<String> = Vec::new();
                    for key in current_subscriptions.keys() {
                        if !subs.contains_key(key) {
                            to_remove.push(key.clone());
                        }
                    }

                    for key in to_remove {
                        if current_subscriptions.remove(&key).is_some()
                            && let Ok(socket) = socket_clone.lock()
                        {
                            log::debug!("[SUB-MGR] Unsubscribing: {}", key);
                            let _ = socket.set_unsubscribe(key.as_bytes());
                        }
                    }

                    for (key, data_type) in &subs {
                        if !current_subscriptions.contains_key(key)
                            && let Ok(socket) = socket_clone.lock()
                        {
                            log::debug!("[SUB-MGR] Subscribing: {}", key);
                            let filter = if key.is_empty() { b"" } else { key.as_bytes() };
                            if socket.set_subscribe(filter).is_ok() {
                                current_subscriptions.insert(key.clone(), *data_type);
                            }
                        }
                    }
                }
            });

            loop {
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }

                let topic_result = {
                    let socket = match socket.lock() {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let _ = socket.set_rcvtimeo(100);
                    socket.recv_bytes(0)
                };

                match topic_result {
                    Ok(topic_bytes) if !topic_bytes.is_empty() => {
                        let msg_bytes = {
                            match socket.lock() {
                                Ok(s) => s.recv_bytes(0).unwrap_or_default(),
                                Err(_) => continue,
                            }
                        };

                        let topic_str = String::from_utf8_lossy(&topic_bytes).to_string();
                        let msg = String::from_utf8_lossy(&msg_bytes).to_string();

                        let data_type = if topic_str.starts_with("ticks.") {
                            DataType::Tick
                        } else if topic_str.starts_with("doms.") {
                            DataType::Dom
                        } else if topic_str.starts_with("quotes.") {
                            DataType::Quote
                        } else {
                            log::warn!("[SUB] Unknown topic: {}", topic_str);
                            continue;
                        };

                        let result = match data_type {
                            DataType::Tick => process_tick_message(&msg),
                            DataType::Dom => dom_to_json(&msg),
                            DataType::Quote => quote_to_json(&msg),
                        };

                        if let Some(formatted) = result {
                            let symbol = topic_str
                                .trim_start_matches("ticks.")
                                .trim_start_matches("doms.")
                                .trim_start_matches("quotes.")
                                .to_string();

                            log::trace!("[SUB] Sending to UI: {} - {}", symbol, data_type.prefix());
                            let _ = tx.blocking_send((symbol, data_type, formatted));
                        } else {
                            log::warn!("[SUB] Processing failed for topic: {}", topic_str);
                        }
                    }
                    _ => {}
                }
            }
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

impl Default for ZmqProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataFeedProvider for ZmqProvider {
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
