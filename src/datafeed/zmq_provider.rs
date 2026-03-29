use crate::datafeed::{dom_to_json, quote_to_json, tick_to_json, DataFeedProvider, DataType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;
use zmq::Context;

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
                    eprintln!("[SUB] Failed to create socket: {}", e);
                    return;
                }
            };

            if let Err(e) = socket.connect("tcp://127.0.0.1:5555") {
                eprintln!("[SUB] Failed to connect: {}", e);
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

                    eprintln!(
                        "[SUB-MGR] Current ZeroMQ subs: {:?}",
                        current_subscriptions.keys().collect::<Vec<_>>()
                    );
                    eprintln!(
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
                            eprintln!("[SUB-MGR] Unsubscribing: {}", key);
                            let _ = socket.set_unsubscribe(key.as_bytes());
                        }
                    }

                    for (key, data_type) in &subs {
                        if !current_subscriptions.contains_key(key)
                            && let Ok(socket) = socket_clone.lock()
                        {
                            eprintln!("[SUB-MGR] Subscribing: {}", key);
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

                        let data_type = if topic_str.starts_with("tick.") {
                            DataType::Tick
                        } else if topic_str.starts_with("dom.") {
                            DataType::Dom
                        } else if topic_str.starts_with("quote.") {
                            DataType::Quote
                        } else {
                            eprintln!("[SUB] Unknown topic: {}", topic_str);
                            continue;
                        };

                        let result = match data_type {
                            DataType::Tick => tick_to_json(&msg),
                            DataType::Dom => dom_to_json(&msg),
                            DataType::Quote => quote_to_json(&msg),
                        };

                        if let Some(formatted) = result {
                            let symbol = topic_str
                                .trim_start_matches("tick.")
                                .trim_start_matches("dom.")
                                .trim_start_matches("quote.")
                                .to_string();

                            eprintln!("[SUB] Sending to UI: {} - {}", symbol, data_type.prefix());
                            let _ = tx.blocking_send((symbol, data_type, formatted));
                        } else {
                            eprintln!("[SUB] Processing failed for topic: {}", topic_str);
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
