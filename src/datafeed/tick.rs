use crate::datafeed::types::{Tick, TickData, TickMessage};
use std::sync::atomic::{AtomicI64, AtomicI8, Ordering};

static LAST_PRICE_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_BID_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_ASK_TICKS: AtomicI64 = AtomicI64::new(0);
static LAST_DIRECTION: AtomicI8 = AtomicI8::new(0);

pub fn reset_state() {
    LAST_PRICE_TICKS.store(0, Ordering::SeqCst);
    LAST_BID_TICKS.store(0, Ordering::SeqCst);
    LAST_ASK_TICKS.store(0, Ordering::SeqCst);
    LAST_DIRECTION.store(0, Ordering::SeqCst);
}

pub fn parse_message(json: &str) -> Option<TickMessage> {
    serde_json::from_str::<TickMessage>(json).ok()
}

pub fn process(json: &str) -> Option<Vec<TickData>> {
    eprintln!(
        "[TICK-PROCESS] Input JSON: {}",
        &json[..json.len().min(300)]
    );
    let msg = match parse_message(json) {
        Some(m) => m,
        None => {
            eprintln!("[TICK-PROCESS] Failed to parse message!");
            return None;
        }
    };

    if msg.ticks.is_empty() {
        eprintln!("[TICK-PROCESS] No ticks in message!");
        return None;
    }
    eprintln!(
        "[TICK-PROCESS] Parsed successfully, ticks: {}",
        msg.ticks.len()
    );

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
        Some(results)
    }
}

pub fn to_json_string(json: &str) -> Option<String> {
    process(json).and_then(|ticks| serde_json::to_string_pretty(&ticks).ok())
}
