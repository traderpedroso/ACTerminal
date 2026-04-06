use crate::application::dto::{UiDomData, UiDomEntry, UiQuoteData, UiTickData};
use crate::infrastructure::datafeed::trade_side::TradeSide;
use crate::infrastructure::datafeed::types::{DomData, QuoteData, TickData};

pub fn needs_inversion(symbol: &str) -> bool {
    matches!(symbol, "6J" | "6L" | "6M" | "6C" | "6S")
}

pub fn get_decimals_for_symbol(symbol: &str) -> usize {
    match symbol {
        "6L" => 2,
        "6J" => 2,
        "6M" => 4,
        "6C" => 4,
        "6S" => 5,
        _ => 5,
    }
}

fn invert_price(price: f64, symbol: &str) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }

    let inv = 1.0 / price;

    match symbol {
        "6L" => {
            let wdo = inv * 1000.0;
            (wdo / 0.5).round() * 0.5
        }
        "6J" => (inv * 100.0).round() / 100.0,
        "6S" => (inv * 100_000.0).round() / 100_000.0,
        _ => (inv * 10_000.0).round() / 10_000.0,
    }
}

fn invert_side(side: TradeSide) -> TradeSide {
    match side {
        TradeSide::BuyMarket => TradeSide::SellMarket,
        TradeSide::SellMarket => TradeSide::BuyMarket,
        TradeSide::BuyPending => TradeSide::SellPending,
        TradeSide::SellPending => TradeSide::BuyPending,
        TradeSide::Mid => TradeSide::Mid,
    }
}

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
        let price = invert_price(tick.price, symbol);
        let side = invert_side(raw_side);

        UiTickData {
            time: format!("{:02}:{:02}:{:02}", h, m, s),
            price,
            size: tick.size,
            side,
            decimals,
        }
    } else {
        UiTickData {
            time: format!("{:02}:{:02}:{:02}", h, m, s),
            price: tick.price,
            size: tick.size,
            side: raw_side,
            decimals,
        }
    }
}

pub fn transform_dom(dom: &DomData, symbol: &str) -> UiDomData {
    let invert = needs_inversion(symbol);
    let decimals = get_decimals_for_symbol(symbol);

    let mut result = UiDomData {
        bids: Vec::new(),
        offers: Vec::new(),
        decimals,
    };

    if invert {
        let mut bids: Vec<UiDomEntry> = dom
            .offers
            .iter()
            .filter(|e| e.size > 0.0 && e.price > 0.0)
            .map(|e| UiDomEntry {
                price: invert_price(e.price, symbol),
                size: e.size,
            })
            .collect();

        let mut offers: Vec<UiDomEntry> = dom
            .bids
            .iter()
            .filter(|e| e.size > 0.0 && e.price > 0.0)
            .map(|e| UiDomEntry {
                price: invert_price(e.price, symbol),
                size: e.size,
            })
            .collect();

        bids.sort_by(|a, b| {
            b.price
                .partial_cmp(&a.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        offers.sort_by(|a, b| {
            a.price
                .partial_cmp(&b.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result.bids = bids;
        result.offers = offers;
    } else {
        result.bids = dom
            .bids
            .iter()
            .filter(|e| e.size > 0.0)
            .map(|e| UiDomEntry {
                price: e.price,
                size: e.size,
            })
            .collect();

        result.offers = dom
            .offers
            .iter()
            .filter(|e| e.size > 0.0)
            .map(|e| UiDomEntry {
                price: e.price,
                size: e.size,
            })
            .collect();
    }

    result
}

pub fn transform_quote(quote: &QuoteData, symbol: &str) -> UiQuoteData {
    let decimals = get_decimals_for_symbol(symbol);
    let invert = needs_inversion(symbol);

    let inv = |p: f64| invert_price(p, symbol);

    if invert {
        UiQuoteData {
            bid_price: quote.entries.offer.as_ref().map(|e| inv(e.price)),
            ask_price: quote.entries.bid.as_ref().map(|e| inv(e.price)),
            last_price: quote.entries.trade.as_ref().map(|e| inv(e.price)),
            high_price: quote.entries.low_price.as_ref().map(|e| inv(e.price)),
            low_price: quote.entries.high_price.as_ref().map(|e| inv(e.price)),
            open_price: quote.entries.opening_price.as_ref().map(|e| inv(e.price)),
            decimals,
        }
    } else {
        UiQuoteData {
            bid_price: quote.entries.bid.as_ref().map(|e| e.price),
            ask_price: quote.entries.offer.as_ref().map(|e| e.price),
            last_price: quote.entries.trade.as_ref().map(|e| e.price),
            high_price: quote.entries.high_price.as_ref().map(|e| e.price),
            low_price: quote.entries.low_price.as_ref().map(|e| e.price),
            open_price: quote.entries.opening_price.as_ref().map(|e| e.price),
            decimals,
        }
    }
}
