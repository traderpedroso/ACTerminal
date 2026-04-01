use crate::datafeed::dto::{UiDomData, UiDomEntry, UiQuoteData, UiTickData};
use crate::datafeed::trade_side::TradeSide;
use crate::datafeed::types::{DomData, QuoteData, TickData};

// ─────────────────────────────────────────────────────────────────────────────
// Symbol metadata
// ─────────────────────────────────────────────────────────────────────────────

/// CME symbols whose price is quoted as USD-per-unit (inverted vs Forex convention).
/// 6E, 6B, 6A, 6N are already "Moeda/USD" — no inversion needed.
/// 6L, 6J, 6M, 6C, 6S come as USD/Moeda — we invert to get the Forex view.
pub fn needs_inversion(symbol: &str) -> bool {
    matches!(symbol, "6J" | "6L" | "6M" | "6C" | "6S")
}

/// Tick size used for DOM ladder spacing (in the *display* currency after inversion).
///
/// * 6L: R$0.50   (WDO standard tick — valid prices end in .00 or .50)
/// * 6J: 0.01     (USD/JPY moves in ~0.01 after inversion)
/// * 6M, 6C: 0.0001
/// * 6S: 0.00001  (needs 5dp to separate every CME level after inversion)
/// * Normal: 0.00005
pub fn get_tick_size_for_symbol(symbol: &str) -> f64 {
    match symbol {
        "6L" => 0.5,     // WDO tick = R$0.50
        "6J" => 0.01,    // USD/JPY ~0.01 per CME level after inversion
        "6M" => 0.0001,
        "6C" => 0.0001,
        "6S" => 0.00001, // CHF gap after inversion ≈ 0.000031 → needs 5dp
        _    => 0.00005,
    }
}

/// Decimal places for price display after inversion.
///
/// * 6L: 2  → "5179.00"  (WDO format: R$ per US$1,000)
/// * 6J: 2  → "150.35"   (USD/JPY)
/// * 6M: 4  → "17.2345"  (USD/MXN)
/// * 6C: 4  → "1.3605"   (USD/CAD)
/// * 6S: 5  → "0.78649"  (USD/CHF — 5dp needed to keep all DOM levels distinct)
pub fn get_decimals_for_symbol(symbol: &str) -> usize {
    match symbol {
        "6L" => 2,
        "6J" => 2,
        "6M" => 4,
        "6C" => 4,
        "6S" => 5,
        _    => 5,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Core price inversion
// ─────────────────────────────────────────────────────────────────────────────

/// Inverts a CME price to its Forex/domestic equivalent and rounds to the
/// smallest visible grid step so the DOM ladder stays clean.
///
/// **6L (Real Brasileiro)**
///   CME quotes BRL/USD (e.g. 0.1931).  
///   WDO (B3 mini-dólar) is expressed as R$ per US$1,000 → multiply by 1000.  
///   1 / 0.1931 × 1000 ≈ 5178.9 → rounded to nearest R$0.50 = 5179.0  
///   Valid prices: …5178.5, 5179.0, 5179.5, 5180.0…
///
/// **6J (Iene)**
///   CME quotes JPY/USD (e.g. 0.006651).  
///   1 / 0.006651 ≈ 150.35 USD/JPY → rounded to 2 dp.
///
/// **6S (Franco Suíço)**
///   CME quotes CHF/USD (e.g. 1.27150).  
///   1 / 1.27150 ≈ 0.78649 USD/CHF → rounded to 5 dp.  
///   (4 dp collapses adjacent CME levels; gap after inversion ≈ 0.000031)
///
/// **6M (Peso), 6C (CAD)**
///   1 / price → rounded to 4 dp.
fn invert_price(price: f64, symbol: &str) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }

    let inv = 1.0 / price;

    match symbol {
        "6L" => {
            // Scale to R$/US$1,000 then round to nearest R$0.50 (WDO tick)
            let wdo = inv * 1000.0;
            (wdo / 0.5).round() * 0.5
        }
        "6J" => {
            // USD/JPY — 2 decimal places (e.g. 150.35)
            (inv * 100.0).round() / 100.0
        }
        "6S" => {
            // USD/CHF — 5 decimal places to keep every CME level distinct
            (inv * 100_000.0).round() / 100_000.0
        }
        _ => {
            // 6M, 6C — 4 decimal places is sufficient
            (inv * 10_000.0).round() / 10_000.0
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Trade side inversion
// ─────────────────────────────────────────────────────────────────────────────

/// For inverted symbols the buy/sell perspective also flips:
/// A "BUY" of JPY/USD (6J goes up) = market bought Yen = USD/JPY fell = SELL in Forex.
fn invert_side(side: TradeSide) -> TradeSide {
    match side {
        TradeSide::BuyMarket   => TradeSide::SellMarket,
        TradeSide::SellMarket  => TradeSide::BuyMarket,
        TradeSide::BuyPending  => TradeSide::SellPending,
        TradeSide::SellPending => TradeSide::BuyPending,
        TradeSide::Mid         => TradeSide::Mid,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tick transform
// ─────────────────────────────────────────────────────────────────────────────

pub fn transform_tick(tick: &TickData, symbol: &str) -> UiTickData {
    let decimals = get_decimals_for_symbol(symbol);

    let time_ms = tick.time;
    let secs = if time_ms > 1_000_000_000 {
        time_ms / 1000
    } else {
        time_ms
    };
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;

    let raw_side = TradeSide::from_str(&tick.side);

    if needs_inversion(symbol) {
        // Invert all prices.
        // bid ↔ ask swap: in CME-inverted space the raw "ask" price is the
        // *lower* CME price which becomes the *higher* Forex bid after inversion.
        let price     = invert_price(tick.price,     symbol);
        let bid_price = invert_price(tick.ask_price,  symbol); // CME ask → Forex bid
        let ask_price = invert_price(tick.bid_price,  symbol); // CME bid → Forex ask
        let side      = invert_side(raw_side);

        UiTickData {
            id: tick.id,
            time: format!("{:02}:{:02}:{:02}", h, m, s),
            price,
            size: tick.size,
            side,
            bid_price,
            ask_price,
            symbol: symbol.to_string(),
            decimals,
        }
    } else {
        UiTickData {
            id: tick.id,
            time: format!("{:02}:{:02}:{:02}", h, m, s),
            price:     tick.price,
            size:      tick.size,
            side:      raw_side,
            bid_price: tick.bid_price,
            ask_price: tick.ask_price,
            symbol:    symbol.to_string(),
            decimals,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DOM transform
// ─────────────────────────────────────────────────────────────────────────────

pub fn transform_dom(dom: &DomData, symbol: &str) -> UiDomData {
    let invert    = needs_inversion(symbol);
    let tick_size = get_tick_size_for_symbol(symbol);
    let decimals  = get_decimals_for_symbol(symbol);

    let mut result = UiDomData {
        contract_id: dom.contract_id,
        timestamp:   dom.timestamp.clone(),
        bids:    Vec::new(),
        offers:  Vec::new(),
        symbol:  symbol.to_string(),
        decimals,
        tick_size,
    };

    if invert {
        // ── Inverted symbols: CME offers → Forex bids, CME bids → Forex offers ──
        //
        // In CME space the "offer" side contains lower raw prices which, after
        // inversion (1/p), become *higher* prices — i.e. the best Forex bid.
        //
        // After building the lists we sort them so the ladder is correct:
        //   bids:   descending (highest first — best bid nearest the spread)
        //   offers: ascending  (lowest first  — best ask nearest the spread)

        let mut bids: Vec<UiDomEntry> = dom
            .offers
            .iter()
            .filter(|e| e.size > 0.0 && e.price > 0.0)
            .map(|e| UiDomEntry {
                price: invert_price(e.price, symbol),
                size:  e.size,
            })
            .collect();

        let mut offers: Vec<UiDomEntry> = dom
            .bids
            .iter()
            .filter(|e| e.size > 0.0 && e.price > 0.0)
            .map(|e| UiDomEntry {
                price: invert_price(e.price, symbol),
                size:  e.size,
            })
            .collect();

        bids.sort_by(|a, b| {
            b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal)
        });
        offers.sort_by(|a, b| {
            a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)
        });

        result.bids   = bids;
        result.offers = offers;
    } else {
        // Normal symbols — pass through as-is (exchange already sorted)
        result.bids = dom
            .bids
            .iter()
            .filter(|e| e.size > 0.0)
            .map(|e| UiDomEntry { price: e.price, size: e.size })
            .collect();

        result.offers = dom
            .offers
            .iter()
            .filter(|e| e.size > 0.0)
            .map(|e| UiDomEntry { price: e.price, size: e.size })
            .collect();
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Quote transform
// ─────────────────────────────────────────────────────────────────────────────

pub fn transform_quote(quote: &QuoteData, symbol: &str) -> UiQuoteData {
    let decimals = get_decimals_for_symbol(symbol);
    let invert   = needs_inversion(symbol);

    let inv = |p: f64| invert_price(p, symbol);

    if invert {
        // For inverted symbols:
        //   • bid ↔ ask swap: CME bid (higher CME price) → Forex ask (lower inverted price)
        //                     CME offer (lower CME price) → Forex bid (higher inverted price)
        //   • high ↔ low swap: CME high (largest raw price = lowest inverted) → Forex low
        //                      CME low  (smallest raw price = highest inverted) → Forex high
        //   • open, last, volume: just invert the price (no swap needed)

        UiQuoteData {
            timestamp:   quote.timestamp.clone(),
            contract_id: quote.contract_id,

            // CME Offer (lower raw price → higher inverted) = Forex Bid
            bid_price: quote.entries.offer.as_ref().map(|e| inv(e.price)),
            bid_size:  quote.entries.offer.as_ref().map(|e| e.size),

            // CME Bid (higher raw price → lower inverted) = Forex Ask
            ask_price: quote.entries.bid.as_ref().map(|e| inv(e.price)),
            ask_size:  quote.entries.bid.as_ref().map(|e| e.size),

            // Last trade — just invert
            last_price: quote.entries.trade.as_ref().map(|e| inv(e.price)),
            last_size:  quote.entries.trade.as_ref().map(|e| e.size),

            // CME High (largest raw) → smallest inverted = Forex Low
            // CME Low  (smallest raw) → largest inverted = Forex High
            high_price: quote.entries.low_price.as_ref().map(|e| inv(e.price)),
            low_price:  quote.entries.high_price.as_ref().map(|e| inv(e.price)),

            // Open: just invert
            open_price: quote.entries.opening_price.as_ref().map(|e| inv(e.price)),

            // Volume: not a price — no inversion
            volume: quote.entries.total_trade_volume.as_ref().map(|e| e.size),

            symbol:   symbol.to_string(),
            decimals,
        }
    } else {
        UiQuoteData {
            timestamp:   quote.timestamp.clone(),
            contract_id: quote.contract_id,
            bid_price:   quote.entries.bid.as_ref().map(|e| e.price),
            bid_size:    quote.entries.bid.as_ref().map(|e| e.size),
            ask_price:   quote.entries.offer.as_ref().map(|e| e.price),
            ask_size:    quote.entries.offer.as_ref().map(|e| e.size),
            last_price:  quote.entries.trade.as_ref().map(|e| e.price),
            last_size:   quote.entries.trade.as_ref().map(|e| e.size),
            high_price:  quote.entries.high_price.as_ref().map(|e| e.price),
            low_price:   quote.entries.low_price.as_ref().map(|e| e.price),
            open_price:  quote.entries.opening_price.as_ref().map(|e| e.price),
            volume:      quote.entries.total_trade_volume.as_ref().map(|e| e.size),
            symbol:      symbol.to_string(),
            decimals,
        }
    }
}
