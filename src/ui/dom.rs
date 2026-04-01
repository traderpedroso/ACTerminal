use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::datafeed::dto::UiDomData;

// ──────────────────────────────────────────────────────────────────────────────
// DOM Ladder View
// ──────────────────────────────────────────────────────────────────────────────

pub struct DomView {
    pub data: Option<UiDomData>,
    pub current_price: Option<f64>,
    pub tick_bid_price: Option<f64>,
    pub tick_ask_price: Option<f64>,
    // Last trade price from Quote - only updates when value changes
    pub show_last_trade: Option<f64>,
    // Previous values for change detection
    pub prev_last_trade: Option<f64>,
    pub prev_best_bid: Option<f64>,
    pub prev_best_ask: Option<f64>,
}

impl DomView {
    pub fn new() -> Self {
        Self {
            data: None,
            current_price: None,
            tick_bid_price: None,
            tick_ask_price: None,
            show_last_trade: None,
            prev_last_trade: None,
            prev_best_bid: None,
            prev_best_ask: None,
        }
    }

    pub fn clear(&mut self) {
        self.data = None;
        self.current_price = None;
        self.tick_bid_price = None;
        self.tick_ask_price = None;
        self.show_last_trade = None;
        self.prev_last_trade = None;
        self.prev_best_bid = None;
        self.prev_best_ask = None;
    }

    pub fn update_dom(&mut self, data: UiDomData) {
        self.data = Some(data);
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = Some(price);
    }

    pub fn update_tick_prices(&mut self, bid_price: f64, ask_price: f64) {
        self.tick_bid_price = Some(bid_price);
        self.tick_ask_price = Some(ask_price);
    }

    pub fn update_show_last_trade(&mut self, price: Option<f64>) {
        // Only update if value actually changed
        if price != self.prev_last_trade {
            self.prev_last_trade = self.show_last_trade;
            self.show_last_trade = price;
        }
    }

    // ── Colour helpers ────────────────────────────────────────────────────────

    fn bid_bar_color(volume_ratio: f64) -> Hsla {
        Hsla {
            h: 215.0 / 360.0, // blue
            s: 0.85,
            l: 0.50,
            a: (0.20 + volume_ratio * 0.80) as f32,
        }
    }

    fn ask_bar_color(volume_ratio: f64) -> Hsla {
        Hsla {
            h: 4.0 / 360.0, // red
            s: 0.80,
            l: 0.45,
            a: (0.20 + volume_ratio * 0.80) as f32,
        }
    }

    // ── Ladder builder ────────────────────────────────────────────────────────
    //
    // Shows ONLY price levels that have actual volume (bid > 0 or ask > 0).
    // Always sorted highest price first (top of DOM), regardless of symbol type,
    // because after the transformer all prices are already in Forex convention.
    //
    // Return type: Vec<(price_str, bid_size, ask_size, is_best_bid, is_best_ask)>

    fn build_ladder(dom: &UiDomData) -> Vec<(String, f64, f64, bool, bool)> {
        let decimals = dom.decimals;
        let fmt = |price: f64| -> String { format!("{:.width$}", price, width = decimals) };

        let bids   = &dom.bids;
        let offers = &dom.offers;

        // Collect all unique prices that carry volume
        let mut all_prices: Vec<f64> = Vec::new();
        for e in bids   { if e.size > 0.0 { all_prices.push(e.price); } }
        for e in offers { if e.size > 0.0 { all_prices.push(e.price); } }

        if all_prices.is_empty() {
            return Vec::new();
        }

        // Highest price at top of DOM for all symbols
        all_prices.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        all_prices.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

        // Best bid = highest bid price; best ask = lowest ask price
        let best_bid_price = bids
            .iter()
            .filter(|e| e.size > 0.0)
            .map(|e| e.price)
            .fold(f64::NEG_INFINITY, f64::max);
        let best_ask_price = offers
            .iter()
            .filter(|e| e.size > 0.0)
            .map(|e| e.price)
            .fold(f64::INFINITY, f64::min);

        let eps = 1e-9;
        let mut rows = Vec::new();
        for price in &all_prices {
            let mut bid = 0.0f64;
            let mut ask = 0.0f64;

            for b in bids {
                if (b.price - price).abs() < eps && b.size > 0.0 {
                    bid = b.size;
                    break;
                }
            }
            for o in offers {
                if (o.price - price).abs() < eps && o.size > 0.0 {
                    ask = o.size;
                    break;
                }
            }

            if bid > 0.0 || ask > 0.0 {
                let is_best_bid = (price - best_bid_price).abs() < eps && bid > 0.0;
                let is_best_ask = (price - best_ask_price).abs() < eps && ask > 0.0;
                rows.push((fmt(*price), bid, ask, is_best_bid, is_best_ask));
            }
        }

        rows
    }
}

impl Render for DomView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport_h = f32::from(window.viewport_size().height);

        // ── Empty state ───────────────────────────────────────────────────────
        let Some(ref dom) = self.data else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Waiting for DOM data…"),
                )
                .into_any_element();
        };

        let ladder = Self::build_ladder(dom);
        if ladder.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("No DOM levels…"),
                )
                .into_any_element();
        }

        // Calculate row height dynamically based on window and number of levels
        // DOM area = viewport - (title bar ~30 + tab bar ~32 + header ~26 + footer ~26 + status ~28)
        let dom_area_h = (viewport_h - 142.0).max(200.0);
        let num_levels = ladder.len() as f32;
        let row_h = (dom_area_h / num_levels).max(16.0).min(32.0); // Clamp between 16-32px

        // ── Column widths ───────────────────────────────────────────────────────
        let bar_col_w: f32 = 80.0;
        let price_col_w: f32 = 80.0;

        // ── Volume scale ──────────────────────────────────────────────────────
        let bid_max = ladder
            .iter()
            .map(|(_, b, _, _, _)| *b)
            .fold(0.0f64, f64::max)
            .max(1.0);
        let ask_max = ladder
            .iter()
            .map(|(_, _, a, _, _)| *a)
            .fold(0.0f64, f64::max)
            .max(1.0);
        let max_vol = bid_max.max(ask_max);

        // ── Totals ────────────────────────────────────────────────────────────
        let total_bids: u64 = ladder.iter().map(|(_, b, _, _, _)| *b as u64).sum();
        let total_asks: u64 = ladder.iter().map(|(_, _, a, _, _)| *a as u64).sum();

        // ── Theme colours ─────────────────────────────────────────────────────
        let bid_label = Hsla {
            h: 210.0 / 360.0, // blue
            s: 0.70,
            l: 0.72,
            a: 1.0,
        };
        let ask_label = Hsla {
            h: 0.0 / 360.0, // red
            s: 0.75,
            l: 0.65,
            a: 1.0,
        };
        let grid = Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 0.05,
        };

        v_flex()
            .size_full()
            // ── Header ───────────────────────────────────────────────────────
            .child(
                h_flex()
                    .w_full()
                    .justify_center()
                    .px_1()
                    .py_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().tab_bar)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(
                        div()
                            .w(px(bar_col_w))
                            .flex()
                            .justify_end()
                            .pr(px(3.))
                            .text_color(bid_label)
                            .child("Bids"),
                    )
                    .child(
                        div()
                            .w(px(price_col_w))
                            .flex()
                            .justify_center()
                            .text_color(cx.theme().muted_foreground)
                            .child("Price"),
                    )
                    .child(
                        div()
                            .w(px(bar_col_w))
                            .flex()
                            .justify_start()
                            .pl(px(3.))
                            .text_color(ask_label)
                            .child("Asks"),
                    ),
            )
            // ── Ladder body ───────────────────────────────────────────────────
            .child(
                div()
                    .id("dom-body")
                    .w_full()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(v_flex().w_full().children(ladder.iter().map(
                        |(price_str, bid_size, ask_size, is_best_bid, is_best_ask)| {
                            // Bid column color: best bid gets blue
                            let bid_color = if *is_best_bid {
                                Hsla {
                                    h: 210.0 / 360.0,
                                    s: 0.85,
                                    l: 0.65,
                                    a: 1.0,
                                }
                            } else {
                                gpui::white()
                            };

                            // Ask column color: best ask gets red
                            let ask_color = if *is_best_ask {
                                Hsla {
                                    h: 0.0 / 360.0,
                                    s: 0.90,
                                    l: 0.60,
                                    a: 1.0,
                                }
                            } else {
                                gpui::white()
                            };

                            let bid_ratio = *bid_size / max_vol;
                            let bid_bar_w = (bid_ratio * bar_col_w as f64) as f32;
                            let ask_ratio = *ask_size / max_vol;
                            let ask_bar_w = (ask_ratio * bar_col_w as f64) as f32;

                            h_flex()
                                .w_full()
                                .h(px(row_h))
                                .justify_center()
                                .items_center()
                                .border_b_1()
                                .border_color(grid)
                                // ── Bid cell ─────────────────────────
                                .child(
                                    div()
                                        .w(px(bar_col_w))
                                        .h_full()
                                        .relative()
                                        .flex()
                                        .items_center()
                                        .justify_end()
                                        .when(bid_bar_w > 0.0, |el| {
                                            el.child(
                                                div()
                                                    .absolute()
                                                    .right_0()
                                                    .top_0()
                                                    .bottom_0()
                                                    .w(px(bid_bar_w))
                                                    .bg(Self::bid_bar_color(bid_ratio)),
                                            )
                                        })
                                        .child(
                                            div()
                                                .relative()
                                                .pr(px(3.))
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if *bid_size > 0.0 {
                                                    bid_color
                                                } else {
                                                    Hsla {
                                                        h: 0.0,
                                                        s: 0.0,
                                                        l: 0.0,
                                                        a: 0.0,
                                                    }
                                                })
                                                .child(if *bid_size > 0.0 {
                                                    format!("{}", *bid_size as u64)
                                                } else {
                                                    String::new()
                                                }),
                                        ),
                                )
                                // ── Price cell ────────────────────────
                                .child(
                                    div()
                                        .w(px(price_col_w))
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_sm()
                                        .font_weight(FontWeight::NORMAL)
                                        .child({
                                            // Parse price for comparison
                                            let price_val: f64 =
                                                price_str.replace('.', "").parse().unwrap_or(0.0);

                                            // Check if this price matches show_last_trade from Quote
                                            let is_last_trade = self
                                                .show_last_trade
                                                .map(|p| {
                                                    let p_str = format!(
                                                        "{:.width$}",
                                                        p,
                                                        width = dom.decimals
                                                    );
                                                    let p_val: f64 = p_str
                                                        .replace('.', "")
                                                        .parse()
                                                        .unwrap_or(0.0);
                                                    (p_val - price_val).abs() < 1.0
                                                })
                                                .unwrap_or(false);

                                            // Determine price color: best bid=blue, best ask=red, else=white
                                            let price_color = if *is_best_bid && !*is_best_ask {
                                                Hsla {
                                                    h: 210.0 / 360.0,
                                                    s: 0.85,
                                                    l: 0.65,
                                                    a: 1.0,
                                                }
                                            } else if *is_best_ask && !*is_best_bid {
                                                Hsla {
                                                    h: 0.0 / 360.0,
                                                    s: 0.90,
                                                    l: 0.60,
                                                    a: 1.0,
                                                }
                                            } else {
                                                gpui::white()
                                            };

                                            // Yellow color for last trade
                                            let yellow = Hsla {
                                                h: 50.0 / 360.0,
                                                s: 0.90,
                                                l: 0.60,
                                                a: 1.0,
                                            };

                                            // If last trade, everything is yellow (brackets + price)
                                            if is_last_trade {
                                                div()
                                                    .text_color(yellow)
                                                    .child(format!("[{}]", price_str.clone()))
                                            } else {
                                                div()
                                                    .text_color(price_color)
                                                    .child(price_str.clone())
                                            }
                                        }),
                                )
                                // ── Ask cell ─────────────────────────
                                .child(
                                    div()
                                        .w(px(bar_col_w))
                                        .h_full()
                                        .relative()
                                        .flex()
                                        .items_center()
                                        .justify_start()
                                        .when(ask_bar_w > 0.0, |el| {
                                            el.child(
                                                div()
                                                    .absolute()
                                                    .left_0()
                                                    .top_0()
                                                    .bottom_0()
                                                    .w(px(ask_bar_w))
                                                    .bg(Self::ask_bar_color(ask_ratio)),
                                            )
                                        })
                                        .child(
                                            div()
                                                .relative()
                                                .pl(px(3.))
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if *ask_size > 0.0 {
                                                    ask_color
                                                } else {
                                                    Hsla {
                                                        h: 0.0,
                                                        s: 0.0,
                                                        l: 0.0,
                                                        a: 0.0,
                                                    }
                                                })
                                                .child(if *ask_size > 0.0 {
                                                    format!("{}", *ask_size as u64)
                                                } else {
                                                    String::new()
                                                }),
                                        ),
                                )
                        },
                    ))),
            )
            // ── Totals footer ─────────────────────────────────────────────────
            .child(
                h_flex()
                    .w_full()
                    .justify_center()
                    .px_1()
                    .py_1()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().tab_bar)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .child(
                        div()
                            .w(px(bar_col_w))
                            .flex()
                            .justify_end()
                            .pr(px(3.))
                            .text_color(bid_label)
                            .child(format!("{}", total_bids)),
                    )
                    .child(
                        div()
                            .w(px(price_col_w))
                            .flex()
                            .justify_center()
                            .child({
                                let total = (total_bids + total_asks) as f64;
                                if total > 0.0 {
                                    let imb = ((total_bids as f64 - total_asks as f64) / total) * 100.0;
                                    let imb_color = if imb > 0.0 { bid_label } else if imb < 0.0 { ask_label } else { cx.theme().muted_foreground };
                                    div()
                                        .text_color(imb_color)
                                        .child(format!("{:.1}%", imb.abs()))
                                } else {
                                    div()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("0%")
                                }
                            })
                    )
                    .child(
                        div()
                            .w(px(bar_col_w))
                            .flex()
                            .justify_start()
                            .pl(px(3.))
                            .text_color(ask_label)
                            .child(format!("{}", total_asks)),
                    ),
            )
            .into_any_element()
    }
}
