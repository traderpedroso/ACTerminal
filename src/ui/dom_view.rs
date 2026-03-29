use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::datafeed::DomData;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// Fixed tick size: currency futures are spaced 0.00005 apart.
const TICK_SIZE: f64 = 0.00005;

/// Extra simulated levels to pad beyond the real 10 levels on each side.
/// These are clipped by the overflow_hidden container, giving a responsive zoom.
const SIMULATED_EXTRA: usize = 12;

/// Fixed row height. The body container is justify_center + overflow_hidden,
/// so the spread lives in the middle while outer rows are clipped responsively.
const ROW_H: f32 = 22.0;

// ──────────────────────────────────────────────────────────────────────────────
// DOM Ladder View
// ──────────────────────────────────────────────────────────────────────────────

pub struct DomView {
    pub data: Option<DomData>,
    pub current_price: Option<f64>,
    pub tick_bid_price: Option<f64>,
    pub tick_ask_price: Option<f64>,
}

impl DomView {
    pub fn new() -> Self {
        Self {
            data: None,
            current_price: None,
            tick_bid_price: None,
            tick_ask_price: None,
        }
    }

    pub fn clear(&mut self) {
        self.data = None;
        self.current_price = None;
        self.tick_bid_price = None;
        self.tick_ask_price = None;
    }

    pub fn update_dom(&mut self, data: DomData) {
        self.data = Some(data);
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = Some(price);
    }

    pub fn update_tick_prices(&mut self, bid_price: f64, ask_price: f64) {
        self.tick_bid_price = Some(bid_price);
        self.tick_ask_price = Some(ask_price);
    }

    // ── Colour helpers ────────────────────────────────────────────────────────

    /// Blue gradient for bids — darker at low volume, vivid at high volume.
    fn bid_bar_color(volume_ratio: f64) -> Hsla {
        Hsla {
            h: 215.0 / 360.0, // blue
            s: 0.85,
            l: 0.50,
            a: (0.20 + volume_ratio * 0.80) as f32,
        }
    }

    /// Red gradient for asks — darker at low volume, vivid at high volume.
    fn ask_bar_color(volume_ratio: f64) -> Hsla {
        Hsla {
            h: 4.0 / 360.0, // red
            s: 0.80,
            l: 0.45,
            a: (0.20 + volume_ratio * 0.80) as f32,
        }
    }

    /// Format price to a fixed-decimal string so trailing zeros are preserved.
    fn fmt_price(tick: i64) -> String {
        let price = tick as f64 * TICK_SIZE;
        format!("{:.5}", price)
    }

    // ── Ladder builder ────────────────────────────────────────────────────────

    fn build_ladder(dom: &DomData) -> Vec<(String, f64, f64, i64)> {
        use std::collections::BTreeMap;

        let price_to_ticks = |p: f64| -> i64 { (p / TICK_SIZE).round() as i64 };

        let mut bid_map: BTreeMap<i64, f64> = BTreeMap::new();
        let mut ask_map: BTreeMap<i64, f64> = BTreeMap::new();

        for entry in &dom.bids {
            if entry.size > 0.0 {
                bid_map.insert(price_to_ticks(entry.price), entry.size);
            }
        }
        for entry in &dom.offers {
            if entry.size > 0.0 {
                ask_map.insert(price_to_ticks(entry.price), entry.size);
            }
        }

        if bid_map.is_empty() && ask_map.is_empty() {
            return Vec::new();
        }

        let best_bid_ticks = bid_map.keys().copied().max().unwrap_or(0);
        let best_ask_ticks = ask_map.keys().copied().min().unwrap_or(best_bid_ticks + 1);

        // Extend display range beyond real data
        let lowest_bid = bid_map.keys().copied().min().unwrap_or(best_bid_ticks);
        let highest_ask = ask_map.keys().copied().max().unwrap_or(best_ask_ticks);

        let range_bottom = lowest_bid - SIMULATED_EXTRA as i64;
        let range_top = highest_ask + SIMULATED_EXTRA as i64;

        // Emit highest price first (asks at top, bids at bottom)
        let mut rows = Vec::new();
        let mut tick = range_top;
        while tick >= range_bottom {
            let price_str = Self::fmt_price(tick);
            let bid_size = bid_map.get(&tick).copied().unwrap_or(0.0);
            let ask_size = ask_map.get(&tick).copied().unwrap_or(0.0);
            rows.push((price_str, bid_size, ask_size, tick));
            tick -= 1;
        }
        rows
    }
}

impl Render for DomView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_price = self.current_price;
        let best_bid = self.tick_bid_price;
        let best_ask = self.tick_ask_price;

        let price_to_ticks = |p: f64| -> i64 { (p / TICK_SIZE).round() as i64 };

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

        // ── Volume scale ──────────────────────────────────────────────────────
        let bid_max = ladder
            .iter()
            .map(|(_, b, _, _)| *b)
            .fold(0.0f64, f64::max)
            .max(1.0);
        let ask_max = ladder
            .iter()
            .map(|(_, _, a, _)| *a)
            .fold(0.0f64, f64::max)
            .max(1.0);
        let max_volume = bid_max.max(ask_max);

        // Column widths
        let bar_col_w: f32 = 80.0;
        let price_col_w: f32 = 76.0;

        // Totals
        let total_bids: u64 = ladder.iter().map(|(_, b, _, _)| *b as u64).sum();
        let total_asks: u64 = ladder.iter().map(|(_, _, a, _)| *a as u64).sum();

        // Tick indices for current price and best bid/ask
        let current_tick = current_price.map(price_to_ticks);
        let best_bid_tick = best_bid.map(price_to_ticks);
        let best_ask_tick = best_ask.map(price_to_ticks);

        // Colour constants for header/footer
        let bid_label_color = Hsla {
            h: 215.0 / 360.0,
            s: 0.70,
            l: 0.72,
            a: 1.0,
        };
        let ask_label_color = Hsla {
            h: 4.0 / 360.0,
            s: 0.75,
            l: 0.65,
            a: 1.0,
        };

        // Subtle grid line colour
        let grid_color = Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 0.04,
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
                            .pr(px(4.))
                            .text_color(bid_label_color)
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
                            .pl(px(4.))
                            .text_color(ask_label_color)
                            .child("Asks"),
                    ),
            )
            // ── Ladder body ───────────────────────────────────────────────────
            .child(
                div()
                    .id("dom-body")
                    .w_full()
                    .flex_1() // takes all remaining height
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .h_full()
                            .justify_center() // keeps spread vertically centred; outer rows clip
                            .children(ladder.iter().map(
                                |(price_str, bid_size, ask_size, row_tick)| {
                                    let row_tick = *row_tick;

                                    let is_best_bid =
                                        best_bid_tick.map(|t| t == row_tick).unwrap_or(false);
                                    let is_best_ask =
                                        best_ask_tick.map(|t| t == row_tick).unwrap_or(false);
                                    let is_at_price =
                                        current_tick.map(|t| t == row_tick).unwrap_or(false);

                                    // Price text colour
                                    let price_color: Hsla = if is_best_bid {
                                        // cyan for best bid
                                        Hsla {
                                            h: 185.0 / 360.0,
                                            s: 0.85,
                                            l: 0.65,
                                            a: 1.0,
                                        }
                                    } else if is_best_ask {
                                        // orange for best ask
                                        Hsla {
                                            h: 28.0 / 360.0,
                                            s: 0.90,
                                            l: 0.62,
                                            a: 1.0,
                                        }
                                    } else {
                                        // dim white for empty levels, full white when there's volume
                                        let has_data = *bid_size > 0.0 || *ask_size > 0.0;
                                        Hsla {
                                            h: 0.0,
                                            s: 0.0,
                                            l: 1.0,
                                            a: if has_data { 0.90 } else { 0.35 },
                                        }
                                    };

                                    let bid_ratio = *bid_size / max_volume;
                                    let bid_bar_w = (bid_ratio * bar_col_w as f64) as f32;
                                    let bid_bar_color = Self::bid_bar_color(bid_ratio);

                                    let ask_ratio = *ask_size / max_volume;
                                    let ask_bar_w = (ask_ratio * bar_col_w as f64) as f32;
                                    let ask_bar_color = Self::ask_bar_color(ask_ratio);

                                    // Bracket chars for current price row
                                    let left_bracket = if is_at_price { "[" } else { " " };
                                    let right_bracket = if is_at_price { "]" } else { " " };
                                    let bracket_color = Hsla {
                                        h: 50.0 / 360.0,
                                        s: 0.95,
                                        l: 0.68,
                                        a: 1.0,
                                    };

                                    // Row background for best bid/ask
                                    let row_bg: Option<Hsla> = if is_best_bid {
                                        Some(Hsla {
                                            h: 215.0 / 360.0,
                                            s: 0.60,
                                            l: 0.20,
                                            a: 0.40,
                                        })
                                    } else if is_best_ask {
                                        Some(Hsla {
                                            h: 4.0 / 360.0,
                                            s: 0.60,
                                            l: 0.18,
                                            a: 0.40,
                                        })
                                    } else {
                                        None
                                    };

                                    h_flex()
                                        .w_full()
                                        .h(px(ROW_H))  // fixed height — no flex_1
                                        .justify_center()
                                        .items_center()
                                        .border_b_1()
                                        .border_color(grid_color)
                                        .when_some(row_bg, |el, bg| el.bg(bg))
                                        // ── Bid cell ─────────────────────────────────────────
                                        .child(
                                            div()
                                                .w(px(bar_col_w))
                                                .h_full()
                                                .relative()
                                                .flex()
                                                .items_center()
                                                .justify_end()
                                                // Bar: anchored right, grows leftward
                                                .when(bid_bar_w > 0.0, |el| {
                                                    el.child(
                                                        div()
                                                            .absolute()
                                                            .right_0()
                                                            .top_0()
                                                            .bottom_0()
                                                            .w(px(bid_bar_w))
                                                            .bg(bid_bar_color),
                                                    )
                                                })
                                                // Qty label
                                                .child(
                                                    div()
                                                        .relative()
                                                        .pr(px(3.))
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(if *bid_size > 0.0 {
                                                            gpui::white()
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
                                        // ── Price cell ────────────────────────────────────────
                                        .child(
                                            div()
                                                .w(px(price_col_w))
                                                .h_full()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_sm()
                                                .font_weight(
                                                    if is_best_bid || is_best_ask || is_at_price {
                                                        FontWeight::BOLD
                                                    } else {
                                                        FontWeight::NORMAL
                                                    },
                                                )
                                                // Left bracket
                                                .child(
                                                    div()
                                                        .w(px(9.))
                                                        .flex()
                                                        .justify_center()
                                                        .text_color(bracket_color)
                                                        .child(left_bracket),
                                                )
                                                // Price string
                                                .child(
                                                    div()
                                                        .text_color(price_color)
                                                        .child(price_str.clone()),
                                                )
                                                // Right bracket
                                                .child(
                                                    div()
                                                        .w(px(9.))
                                                        .flex()
                                                        .justify_center()
                                                        .text_color(bracket_color)
                                                        .child(right_bracket),
                                                ),
                                        )
                                        // ── Ask cell ──────────────────────────────────────────
                                        .child(
                                            div()
                                                .w(px(bar_col_w))
                                                .h_full()
                                                .relative()
                                                .flex()
                                                .items_center()
                                                .justify_start()
                                                // Bar: anchored left, grows rightward
                                                .when(ask_bar_w > 0.0, |el| {
                                                    el.child(
                                                        div()
                                                            .absolute()
                                                            .left_0()
                                                            .top_0()
                                                            .bottom_0()
                                                            .w(px(ask_bar_w))
                                                            .bg(ask_bar_color),
                                                    )
                                                })
                                                // Qty label
                                                .child(
                                                    div()
                                                        .relative()
                                                        .pl(px(3.))
                                                        .text_sm()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(if *ask_size > 0.0 {
                                                            gpui::white()
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
                            )),
                    ),
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
                            .pr(px(4.))
                            .text_color(bid_label_color)
                            .child(format!("{}", total_bids)),
                    )
                    .child(div().w(px(price_col_w)))
                    .child(
                        div()
                            .w(px(bar_col_w))
                            .flex()
                            .justify_start()
                            .pl(px(4.))
                            .text_color(ask_label_color)
                            .child(format!("{}", total_asks)),
                    ),
            )
            .into_any_element()
    }
}
