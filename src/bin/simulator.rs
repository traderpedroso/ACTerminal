use rand::Rng;
use serde::Serialize;
use std::time::Duration;
use zenoh::config::Config;

const SYMBOL: &str = "6E";
const TICK_INTERVAL_MS: u64 = 100;
const DOM_INTERVAL_MS: u64 = 50;
const QUOTE_INTERVAL_MS: u64 = 500;

const BASE_PRICE: f64 = 18405.0;
const TICK_SIZE: f64 = 0.00005;

#[derive(Debug, Clone, Serialize)]
struct QuoteEntry {
    price: f64,
    size: f64,
}

#[derive(Debug, Clone, Serialize)]
struct QuoteEntries {
    #[serde(rename = "Bid")]
    bid: Option<QuoteEntry>,
    #[serde(rename = "TotalTradeVolume")]
    total_trade_volume: Option<QuoteEntry>,
    #[serde(rename = "Offer")]
    offer: Option<QuoteEntry>,
    #[serde(rename = "LowPrice")]
    low_price: Option<QuoteEntry>,
    #[serde(rename = "Trade")]
    trade: Option<QuoteEntry>,
    #[serde(rename = "OpenInterest")]
    open_interest: Option<QuoteEntry>,
    #[serde(rename = "OpeningPrice")]
    opening_price: Option<QuoteEntry>,
    #[serde(rename = "HighPrice")]
    high_price: Option<QuoteEntry>,
    #[serde(rename = "SettlementPrice")]
    settlement_price: Option<QuoteEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct QuoteData {
    timestamp: String,
    #[serde(rename = "contractId")]
    contract_id: i64,
    entries: QuoteEntries,
}

#[derive(Debug, Clone, Serialize)]
struct QuoteMessage {
    quotes: Vec<QuoteData>,
}

#[derive(Debug, Clone, Serialize)]
struct TickInPacket {
    t: i64,
    p: i64,
    s: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "a")]
    ask_rel: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "as")]
    ask_size: Option<f64>,
    id: i64,
}

#[derive(Debug, Clone, Serialize)]
struct TickPacket {
    id: i64,
    s: String,
    td: i64,
    bp: i64,
    bt: i64,
    ts: f64,
    tks: Vec<TickInPacket>,
}

#[derive(Debug, Clone, Serialize)]
struct TickMessage {
    ticks: Vec<TickPacket>,
}

#[derive(Debug, Clone, Serialize)]
struct DomEntry {
    price: f64,
    size: f64,
}

#[derive(Debug, Clone, Serialize)]
struct DomData {
    #[serde(rename = "contractId")]
    contract_id: i64,
    timestamp: String,
    bids: Vec<DomEntry>,
    offers: Vec<DomEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct DomMessage {
    doms: Vec<DomData>,
}

struct SimState {
    base_price: i64,
    current_price: i64,
    bid_price: i64,
    ask_price: i64,
    last_tick_id: i64,
    base_timestamp: i64,
    rng: rand::rngs::ThreadRng,
    total_volume: f64,
    high_price: f64,
    low_price: f64,
    opening_price: f64,
}

impl SimState {
    fn new() -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let base_price = (BASE_PRICE / TICK_SIZE) as i64;
        Self {
            base_price,
            current_price: base_price,
            bid_price: base_price - 20,
            ask_price: base_price + 20,
            last_tick_id: 0,
            base_timestamp: now,
            rng: rand::rng(),
            total_volume: 0.0,
            high_price: BASE_PRICE,
            low_price: BASE_PRICE,
            opening_price: BASE_PRICE,
        }
    }

    fn next_tick(&mut self) -> TickPacket {
        let price_delta = self.rng.random_range(-3..=3);
        self.current_price = (self.base_price + price_delta).max(self.base_price - 100).min(self.base_price + 100);

        let bid_offset = self.rng.random_range(15..=25);
        let ask_offset = self.rng.random_range(15..=25);
        self.bid_price = self.current_price - bid_offset;
        self.ask_price = self.current_price + ask_offset;

        let tick_price = {
            let r: i32 = self.rng.random_range(0..=2);
            match r {
                0 => self.ask_price,
                1 => self.bid_price,
                _ => self.current_price,
            }
        };

        let now = chrono::Utc::now().timestamp_millis();
        let size = self.rng.random_range(1..=50) as u64;

        self.last_tick_id += 1;

        let trade_price = tick_price as f64 * TICK_SIZE;
        if trade_price > self.high_price {
            self.high_price = trade_price;
        }
        if trade_price < self.low_price {
            self.low_price = trade_price;
        }
        self.total_volume += size as f64;

        TickPacket {
            id: 16335,
            s: "db".to_string(),
            td: chrono::Utc::now().format("%Y%m%d").to_string().parse().unwrap(),
            bp: self.base_price,
            bt: self.base_timestamp,
            ts: TICK_SIZE,
            tks: vec![TickInPacket {
                t: now - self.base_timestamp,
                p: tick_price - self.base_price,
                s: size,
                b: Some(self.bid_price - self.base_price),
                ask_rel: Some(self.ask_price - self.base_price),
                bs: Some(self.rng.random_range(100.0..=500.0)),
                ask_size: Some(self.rng.random_range(100.0..=500.0)),
                id: self.last_tick_id,
            }],
        }
    }

    fn next_dom(&mut self) -> DomData {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let mid_price = (self.bid_price + self.ask_price) / 2;

        let bids: Vec<DomEntry> = (0..10)
            .map(|level| {
                let price_offset = (level + 1) as i64;
                let price = ((mid_price - price_offset) as f64) * TICK_SIZE;
                let size = self.rng.random_range(50.0..=500.0);
                DomEntry { price, size }
            })
            .collect();

        let offers: Vec<DomEntry> = (0..10)
            .map(|level| {
                let price_offset = (level + 1) as i64;
                let price = ((mid_price + price_offset) as f64) * TICK_SIZE;
                let size = self.rng.random_range(50.0..=500.0);
                DomEntry { price, size }
            })
            .collect();

        DomData {
            contract_id: 123456,
            timestamp,
            bids,
            offers,
        }
    }

    fn next_quote(&mut self) -> QuoteData {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let bid_price = self.bid_price as f64 * TICK_SIZE;
        let ask_price = self.ask_price as f64 * TICK_SIZE;
        let trade_price = self.current_price as f64 * TICK_SIZE;

        QuoteData {
            timestamp,
            contract_id: 123456,
            entries: QuoteEntries {
                bid: Some(QuoteEntry {
                    price: bid_price,
                    size: self.rng.random_range(100.0..=500.0),
                }),
                offer: Some(QuoteEntry {
                    price: ask_price,
                    size: self.rng.random_range(100.0..=500.0),
                }),
                trade: Some(QuoteEntry {
                    price: trade_price,
                    size: self.rng.random_range(1.0..=10.0),
                }),
                total_trade_volume: Some(QuoteEntry {
                    size: self.total_volume,
                    price: 0.0,
                }),
                open_interest: Some(QuoteEntry {
                    size: self.rng.random_range(40000.0..=50000.0),
                    price: 0.0,
                }),
                opening_price: Some(QuoteEntry {
                    price: self.opening_price,
                    size: 0.0,
                }),
                high_price: Some(QuoteEntry {
                    price: self.high_price,
                    size: 0.0,
                }),
                low_price: Some(QuoteEntry {
                    price: self.low_price,
                    size: 0.0,
                }),
                settlement_price: Some(QuoteEntry {
                    price: self.opening_price,
                    size: 0.0,
                }),
            },
        }
    }
}

#[tokio::main]
async fn main() {
    let endpoint = std::env::var("ZENOH_ENDPOINT").unwrap_or_else(|_| "tcp/127.0.0.1:7447".to_string());

    let config = Config::from_json5(&format!(
        r#"{{
            "mode": "client",
            "connect": {{
                "endpoints": ["{}"]
            }}
        }}"#,
        endpoint
    ))
    .expect("Invalid Zenoh config");

    println!("[SIMULATOR] Connecting to Zenoh at {}", endpoint);
    let session = zenoh::open(config).await.expect("Failed to connect to Zenoh");
    println!("[SIMULATOR] Connected!");

    let tick_key = format!("ticks/{}", SYMBOL);
    let dom_key = format!("doms/{}", SYMBOL);
    let quote_key = format!("quotes/{}", SYMBOL);

    let tick_publisher = session.declare_publisher(&tick_key).await.expect("Failed to declare tick publisher");
    let dom_publisher = session.declare_publisher(&dom_key).await.expect("Failed to declare dom publisher");
    let quote_publisher = session.declare_publisher(&quote_key).await.expect("Failed to declare quote publisher");

    println!("[SIMULATOR] Publishing to:");
    println!("  - {}", tick_key);
    println!("  - {}", dom_key);
    println!("  - {}", quote_key);
    println!("[SIMULATOR] Intervals: Tick={}ms, DOM={}ms, Quote={}ms", TICK_INTERVAL_MS, DOM_INTERVAL_MS, QUOTE_INTERVAL_MS);

    let mut state = SimState::new();

    let mut tick_interval = tokio::time::interval(Duration::from_millis(TICK_INTERVAL_MS));
    let mut dom_interval = tokio::time::interval(Duration::from_millis(DOM_INTERVAL_MS));
    let mut quote_interval = tokio::time::interval(Duration::from_millis(QUOTE_INTERVAL_MS));

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let tick = state.next_tick();
                let msg = TickMessage { ticks: vec![tick] };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = tick_publisher.put(json).await;
                }
            }
            _ = dom_interval.tick() => {
                let dom = state.next_dom();
                let msg = DomMessage { doms: vec![dom] };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = dom_publisher.put(json).await;
                }
            }
            _ = quote_interval.tick() => {
                let quote = state.next_quote();
                let msg = QuoteMessage { quotes: vec![quote] };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = quote_publisher.put(json).await;
                }
            }
        }
    }
}