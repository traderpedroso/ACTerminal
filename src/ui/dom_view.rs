use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::datafeed::DomData;

// ──────────────────────────────────────────────────────────────────────────────
// DOM Ladder View
// ──────────────────────────────────────────────────────────────────────────────

/// Renders a compact Depth-of-Market ladder with volume bars.
/// Layout: [Bid Volume] [Price] [Ask Volume]
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

    fn volume_bar_color(side: &str, volume_ratio: f64) -> Hsla {
        let base_saturation: f32 = 0.85;
        let base_lightness: f32 = if side == "bid" { 0.60 } else { 0.55 };
        let base_hue: f32 = if side == "bid" { 215.0 / 360.0 } else { 0.0 };

        let intensity = (0.3 + (volume_ratio * 0.7)) as f32;

        Hsla {
            h: base_hue,
            s: base_saturation,
            l: base_lightness,
            a: intensity,
        }
    }

    fn price_text_color() -> Hsla {
        gpui::white()
    }
}

impl Render for DomView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_price = self.current_price;

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

        let mut asks: Vec<_> = dom.offers.iter().collect();
        asks.sort_by(|a, b| {
            b.price
                .partial_cmp(&a.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut bids: Vec<_> = dom.bids.iter().collect();
        bids.sort_by(|a, b| {
            b.price
                .partial_cmp(&a.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut all_prices: Vec<f64> = asks.iter().chain(bids.iter()).map(|e| e.price).collect();
        all_prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut min_diff = f64::MAX;
        for w in all_prices.windows(2) {
            let diff = (w[1] - w[0]).abs();
            if diff > 1e-8 && diff < min_diff {
                min_diff = diff;
            }
        }
        let tick_size = if min_diff < f64::MAX {
            (min_diff * 1_000_000.0).round() / 1_000_000.0
        } else {
            0.00005
        };

        let best_ask_price = self.tick_ask_price;
        let best_bid_price = self.tick_bid_price;

        // Get all unique prices that have actual bid or ask size
        let mut all_prices_with_size: Vec<f64> = Vec::new();
        for e in &asks {
            if e.size > 0.0 {
                all_prices_with_size.push(e.price);
            }
        }
        for e in &bids {
            if e.size > 0.0 {
                all_prices_with_size.push(e.price);
            }
        }
        all_prices_with_size.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        all_prices_with_size.dedup_by(|a, b| (*a - *b).abs() < tick_size * 0.1);

        // Only show levels with actual size
        let unified_dom: Vec<(f64, f64, f64)> = all_prices_with_size
            .iter()
            .map(|&p| {
                let ask_size = asks
                    .iter()
                    .find(|e| (e.price - p).abs() < tick_size * 0.1)
                    .map(|e| e.size)
                    .unwrap_or(0.0);

                let bid_size = bids
                    .iter()
                    .find(|e| (e.price - p).abs() < tick_size * 0.1)
                    .map(|e| e.size)
                    .unwrap_or(0.0);

                (p, ask_size, bid_size)
            })
            .collect();

        // If no data, show message
        if unified_dom.is_empty() {
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
        }

        let ask_max = asks.iter().map(|e| e.size as f64).fold(0.0f64, f64::max);
        let bid_max = bids.iter().map(|e| e.size as f64).fold(0.0f64, f64::max);
        let max_volume = ask_max.max(bid_max).max(1.0);

        let max_bar_width = 70.0;
        let price_col_width = 80.0;

        let header_color = Self::price_text_color();

        v_flex()
            .size_full()
            .min_w(px(240.))
            // Header: [Bid] [Price] [Ask]
            .child(
                h_flex()
                    .w_full()
                    .justify_center()
                    .px_2()
                    .py_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().tab_bar)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        div()
                            .w(px(max_bar_width as f32))
                            .text_color(header_color)
                            .child("Bid"),
                    )
                    .child(
                        div()
                            .w(px(price_col_width as f32))
                            .justify_center()
                            .child("Price"),
                    )
                    .child(
                        div()
                            .w(px(max_bar_width as f32))
                            .justify_end()
                            .text_color(header_color)
                            .child("Ask"),
                    ),
            )
            // Static DOM body (Overflow Hidden + Centered)
            .child(
                div()
                    .id("dom-container")
                    .w_full()
                    .h_full()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .h_full()
                            .justify_center()
                            // Unified Ladder – displayed top-down, highest price first
                            // Right-aligned (justify_end) so ask side is near right edge
                            .children(unified_dom.iter().map(|&(price, ask_size, bid_size)| {
                                let is_best_bid = best_bid_price
                                    .map(|bp| (bp - price).abs() < tick_size * 0.1)
                                    .unwrap_or(false);

                                let is_best_ask = best_ask_price
                                    .map(|ap| (ap - price).abs() < tick_size * 0.1)
                                    .unwrap_or(false);

                                let price_color = if is_best_bid {
                                    Hsla {
                                        h: 0.0,
                                        s: 0.9,
                                        l: 0.5,
                                        a: 1.0,
                                    }
                                } else if is_best_ask {
                                    Hsla {
                                        h: 210.0 / 360.0,
                                        s: 0.9,
                                        l: 0.5,
                                        a: 1.0,
                                    }
                                } else {
                                    Self::price_text_color()
                                };

                                let ask_ratio = ask_size / max_volume;
                                let ask_bar_width = (ask_ratio * max_bar_width as f64) as f32;
                                let ask_bar_color = Self::volume_bar_color("ask", ask_ratio);

                                let bid_ratio = bid_size / max_volume;
                                let bid_bar_width = (bid_ratio * max_bar_width as f64) as f32;
                                let bid_bar_color = Self::volume_bar_color("bid", bid_ratio);

                                let is_at_price = current_price
                                    .map(|cp| (cp - price).abs() < tick_size * 0.1)
                                    .unwrap_or(false);

                                h_flex()
                                    .w_full()
                                    .justify_end()
                                    .px_2()
                                    .py(px(1.))
                                    .gap_0()
                                    .text_xs()
                                    // Bid side (left of price) - 2px gap from price
                                    .child(
                                        div()
                                            .w(px(max_bar_width as f32))
                                            .relative()
                                            .flex()
                                            .justify_end()
                                            .mr_1()
                                            .child(
                                                div()
                                                    .absolute()
                                                    .right_0()
                                                    .top_0()
                                                    .bottom_0()
                                                    .w(px(bid_bar_width as f32))
                                                    .bg(bid_bar_color),
                                            )
                                            .child(
                                                div()
                                                    .relative()
                                                    .text_color(if bid_size > 0.0 {
                                                        gpui::white()
                                                    } else {
                                                        Hsla::default()
                                                    })
                                                    .child(if bid_size > 0.0 {
                                                        format!("{}", bid_size as u64)
                                                    } else {
                                                        String::new()
                                                    }),
                                            ),
                                    )
                                    // Price (center) - clean, no bars
                                    .child(
                                        div()
                                            .w(px(price_col_width as f32))
                                            .justify_center()
                                            .px_1()
                                            .rounded_sm()
                                            .when(is_at_price, |e| {
                                                e.bg(Hsla {
                                                    h: 50.0 / 360.0,
                                                    s: 0.9,
                                                    l: 0.45,
                                                    a: 0.3,
                                                })
                                            })
                                            .font_weight(
                                                if is_best_bid || is_best_ask || is_at_price {
                                                    FontWeight::BOLD
                                                } else {
                                                    FontWeight::NORMAL
                                                },
                                            )
                                            .text_color(price_color)
                                            .child(format!("{:.5}", price)),
                                    )
                                    // Ask side (right of price) - bars grow from price (left) toward right
                                    .child(
                                        div()
                                            .w(px(max_bar_width as f32))
                                            .relative()
                                            .flex()
                                            .justify_start()
                                            .child(
                                                div()
                                                    .absolute()
                                                    .left(px(-24.))
                                                    .top_0()
                                                    .bottom_0()
                                                    .w(px(ask_bar_width as f32))
                                                    .bg(ask_bar_color),
                                            )
                                            .child(
                                                div()
                                                    .relative()
                                                    .text_color(if ask_size > 0.0 {
                                                        gpui::white()
                                                    } else {
                                                        Hsla::default()
                                                    })
                                                    .child(if ask_size > 0.0 {
                                                        format!("{}", ask_size as u64)
                                                    } else {
                                                        String::new()
                                                    }),
                                            ),
                                    )
                            })),
                    ),
            )
            .into_any_element()
    }
}
