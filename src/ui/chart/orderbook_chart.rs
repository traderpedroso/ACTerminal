use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::h_flex;
use gpui_component::v_flex;

use crate::datafeed::{QuoteData, TickData};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    Line,
    Heatmap,
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChartType::Line => write!(f, "Line"),
            ChartType::Heatmap => write!(f, "Heatmap"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Clone)]
pub struct ChartDataPoint {
    pub time: String,
    pub timestamp: i64,
    pub price: f64,
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub volume: Option<f64>,
    pub side: Option<TradeSide>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone)]
pub struct BookmapLevel {
    pub price: f64,
    pub size: f64,
    pub side: OrderSide,
}

#[derive(Clone)]
pub struct BookmapSnapshot {
    pub timestamp: i64,
    pub time_label: String,
    pub levels: Vec<BookmapLevel>,
}

pub struct OrderbookChart {
    pub dom_prices: Vec<ChartDataPoint>,
    pub bookmap_snapshots: Vec<BookmapSnapshot>,
    pub chart_type: ChartType,
    pub symbol: String,
}

impl OrderbookChart {
    pub fn new() -> Self {
        Self {
            dom_prices: Vec::new(),
            bookmap_snapshots: Vec::new(),
            chart_type: ChartType::Line,
            symbol: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.dom_prices.clear();
        self.bookmap_snapshots.clear();
    }

    fn get_current_price(&self) -> Option<f64> {
        self.dom_prices.last().map(|d| d.price)
    }

    fn get_price_bounds(&self) -> Option<(f64, f64)> {
        const TICK_SIZE: f64 = 0.00005;
        const TICKS_ABOVE_BELOW: i64 = 10;

        let current_price = self.get_current_price()?;

        if current_price <= 0.0 {
            return None;
        }

        let to_tick = |p: f64| -> i64 { (p / TICK_SIZE).round() as i64 };
        let current_tick = to_tick(current_price);

        let min_tick = current_tick - TICKS_ABOVE_BELOW;
        let max_tick = current_tick + TICKS_ABOVE_BELOW;

        let min_price = min_tick as f64 * TICK_SIZE;
        let max_price = max_tick as f64 * TICK_SIZE;

        Some((min_price, max_price))
    }

    pub fn update_from_tick(&mut self, tick: &TickData, symbol: &str) {
        self.symbol = symbol.to_string();
    }

    pub fn update_from_quote(&mut self, quote: &QuoteData, symbol: &str) {
        self.symbol = symbol.to_string();
    }

    pub fn update_from_dom(&mut self, dom: &crate::datafeed::DomData) {
        let timestamp = parse_timestamp(&dom.timestamp);
        let time_label = format_time(timestamp);

        let best_bid = dom.bids.first().map(|b| b.price);
        let best_ask = dom.offers.first().map(|o| o.price);

        if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            let mid_price = (bid + ask) / 2.0;

            let point = ChartDataPoint {
                time: time_label.clone(),
                timestamp,
                price: mid_price,
                bid_price: Some(bid),
                ask_price: Some(ask),
                open: None,
                high: None,
                low: None,
                close: None,
                volume: None,
                side: None,
            };

            self.dom_prices.push(point);
        }

        let mut levels: Vec<BookmapLevel> = Vec::new();

        for bid in &dom.bids {
            if bid.size > 0.0 {
                levels.push(BookmapLevel {
                    price: bid.price,
                    size: bid.size,
                    side: OrderSide::Bid,
                });
            }
        }

        for offer in &dom.offers {
            if offer.size > 0.0 {
                levels.push(BookmapLevel {
                    price: offer.price,
                    size: offer.size,
                    side: OrderSide::Ask,
                });
            }
        }

        let snapshot = BookmapSnapshot {
            timestamp,
            time_label,
            levels,
        };

        self.bookmap_snapshots.push(snapshot);
    }

    pub fn set_chart_type(&mut self, chart_type: ChartType) {
        self.chart_type = chart_type;
    }

    fn build_price_scale(&self, theme: &gpui::Hsla) -> impl IntoElement {
        const TICK_SIZE: f64 = 0.00005;
        const TICKS_ABOVE_BELOW: i64 = 10;

        let current_price = match self.get_current_price() {
            Some(p) => p,
            None => return div().w(px(60.)).child("No price"),
        };

        if current_price <= 0.0 {
            return div().w(px(60.)).child("No price");
        }

        let to_tick = |p: f64| -> i64 { (p / TICK_SIZE).round() as i64 };
        let current_tick = to_tick(current_price);

        let min_tick = current_tick - TICKS_ABOVE_BELOW;
        let max_tick = current_tick + TICKS_ABOVE_BELOW;

        let labels: Vec<String> = (min_tick..=max_tick)
            .rev()
            .map(|tick| {
                let price = tick as f64 * TICK_SIZE;
                format!("{:.5}", price)
            })
            .collect();

        let bg_color = *theme;
        let border_color = gpui::rgb(0x333333);
        let text_color = gpui::rgb(0x888888);

        v_flex()
            .w(px(60.))
            .h_full()
            .bg(bg_color)
            .border_l_1()
            .border_color(border_color)
            .justify_between()
            .p_1()
            .children(
                labels
                    .into_iter()
                    .map(|label| div().text_xs().text_color(text_color).child(label)),
            )
    }

    fn render_line_chart(&self, cx: &mut Context<Self>) -> impl IntoElement + 'static {
        let theme = cx.theme();
        let theme_bg = theme.background;

        if self.dom_prices.is_empty() {
            return h_flex().flex_1().items_center().justify_center().child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Waiting for DOM data..."),
            );
        }

        let (min_price, max_price) = match self.get_price_bounds() {
            Some(bounds) => bounds,
            None => {
                return h_flex().flex_1().items_center().justify_center().child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("No price data..."),
                );
            }
        };

        let price_range = max_price - min_price;
        let data = self.dom_prices.clone();
        let data_count = data.len();
        let theme_stroke = theme.info;

        let price_scale = self.build_price_scale(&theme_bg);

        h_flex()
            .flex_1()
            .h_full()
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .relative()
                    .children(data.iter().enumerate().map(|(i, point)| {
                        let x_frac = if data_count > 1 {
                            i as f64 / (data_count - 1) as f64
                        } else {
                            0.5
                        };

                        let y_frac = if price_range > 0.0 {
                            (point.price - min_price) / price_range
                        } else {
                            0.5
                        };

                        let x = x_frac * 100.0;
                        let y = (1.0 - y_frac) * 100.0;

                        div()
                            .absolute()
                            .left(px(x as f32 * 3.0))
                            .top(px(y as f32 * 3.0))
                            .w_px()
                            .h_px()
                            .bg(theme_stroke)
                    })),
            )
            .child(price_scale)
    }

    fn render_heatmap(&self, cx: &mut Context<Self>) -> impl IntoElement + 'static {
        let theme = cx.theme();
        let theme_bg = theme.background;

        if self.bookmap_snapshots.is_empty() {
            return h_flex().flex_1().items_center().justify_center().child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Waiting for DOM data..."),
            );
        }

        let (min_price, max_price) = match self.get_price_bounds() {
            Some(bounds) => bounds,
            None => {
                return h_flex().flex_1().items_center().justify_center().child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("No price data..."),
                );
            }
        };

        let snapshots = self.bookmap_snapshots.clone();
        let snapshot_count = snapshots.len();
        let row_count = 20;

        let price_scale = self.build_price_scale(&theme_bg);

        h_flex()
            .flex_1()
            .h_full()
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .children(snapshots.iter().enumerate().map(|(i, snapshot)| {
                        let x_fraction = if snapshot_count > 1 {
                            i as f64 / (snapshot_count - 1).max(1) as f64
                        } else {
                            0.5
                        };

                        div()
                            .flex_1()
                            .h_full()
                            .border_r_1()
                            .border_color(theme.border)
                            .children((0..row_count).map(move |row| {
                                let row_price = min_price
                                    + (max_price - min_price)
                                        * (1.0 - row as f64 / row_count as f64);

                                let mut best_match: Option<&BookmapLevel> = None;
                                let mut best_diff = f64::MAX;

                                for level in &snapshot.levels {
                                    let diff = (level.price - row_price).abs();
                                    if diff < best_diff {
                                        best_diff = diff;
                                        best_match = Some(level);
                                    }
                                }

                                let threshold = (max_price - min_price) / row_count as f64;
                                let is_valid = best_diff < threshold;

                                let color = if is_valid {
                                    if let Some(level) = best_match {
                                        let max_size = snapshot
                                            .levels
                                            .iter()
                                            .map(|l| l.size)
                                            .fold(1.0f64, f64::max);
                                        let intensity = (level.size / max_size).min(1.0) as f32;
                                        let alpha = (0.3 + intensity as f64 * 0.7) as f32;

                                        match level.side {
                                            OrderSide::Bid => {
                                                let r = (46.0
                                                    + (209.0 - 46.0) * (1.0 - intensity as f64))
                                                    as f32
                                                    / 255.0;
                                                let g = (134.0
                                                    + (233.0 - 134.0) * (1.0 - intensity as f64))
                                                    as f32
                                                    / 255.0;
                                                let b = (193.0
                                                    + (255.0 - 193.0) * (1.0 - intensity as f64))
                                                    as f32
                                                    / 255.0;
                                                hsla(r, g, b, alpha)
                                            }
                                            OrderSide::Ask => {
                                                let r = (243.0
                                                    + (255.0 - 243.0) * (1.0 - intensity as f64))
                                                    as f32
                                                    / 255.0;
                                                let g = (156.0
                                                    + (140. - 156.0) * (1.0 - intensity as f64))
                                                    as f32
                                                    / 255.0;
                                                let b = (18.0
                                                    + (60.0 - 18.0) * (1.0 - intensity as f64))
                                                    as f32
                                                    / 255.0;
                                                hsla(r, g, b, alpha)
                                            }
                                        }
                                    } else {
                                        theme_bg.opacity(0.1)
                                    }
                                } else {
                                    theme_bg.opacity(0.1)
                                };

                                div()
                                    .h_px()
                                    .w_full()
                                    .bg(color)
                                    .border_b_1()
                                    .border_color(theme.border.opacity(0.3))
                            }))
                    })),
            )
            .child(price_scale)
    }
}

impl Default for OrderbookChart {
    fn default() -> Self {
        Self::new()
    }
}

fn format_time(timestamp: i64) -> String {
    let secs = timestamp / 1000;
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn parse_timestamp(s: &str) -> i64 {
    let cleaned = s.trim().trim_end_matches('Z');
    cleaned.parse::<i64>().unwrap_or(0)
}

impl Render for OrderbookChart {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.chart_type == ChartType::Heatmap {
            return self.render_heatmap(cx).into_any_element();
        }

        self.render_line_chart(cx).into_any_element()
    }
}
