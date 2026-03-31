use std::collections::VecDeque;

use gpui::{linear_color_stop, linear_gradient, *};
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::datafeed::TradeSide;
use crate::ui::dto::UiTickData;

/// Maximum number of rows kept in memory.
const MAX_ROWS: usize = 500;

// ──────────────────────────────────────────────────────────────────────────────
// Data model
// ──────────────────────────────────────────────────────────────────────────────

/// A single trade entry displayed in the Times & Sales list.
#[derive(Clone, Debug)]
pub struct TimesAndSalesEntry {
    /// Human-readable time string (HH:MM:SS).
    pub time: String,
    /// Trade price.
    pub price: f64,
    /// Trade size.
    pub size: u64,
    /// Trade side as determined by the tick processor.
    pub side: TradeSide,
    /// Contract symbol, e.g. "ESM5".
    pub symbol: String,
}

impl TimesAndSalesEntry {
    /// Build from a parsed `UiTickData` + the subscription symbol.
    pub fn from_ui_tick_data(td: &UiTickData) -> Self {
        Self {
            time: td.time.clone(),
            price: td.price,
            size: td.size,
            side: td.side,
            symbol: td.symbol.clone(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// View entity
// ──────────────────────────────────────────────────────────────────────────────

/// GPUI entity that owns the Times & Sales buffer.
pub struct TimesAndSalesView {
    rows: VecDeque<TimesAndSalesEntry>,
    /// Symbol currently being shown (for the header).
    pub symbol: String,
}

impl TimesAndSalesView {
    pub fn new() -> Self {
        Self {
            rows: VecDeque::with_capacity(MAX_ROWS),
            symbol: String::new(),
        }
    }

    /// Clear the internal buffer (e.g. when subscription changes).
    pub fn clear(&mut self) {
        self.rows.clear();
        self.symbol.clear();
    }

    /// Push a new trade entry, evicting the oldest if needed.
    pub fn push(&mut self, entry: TimesAndSalesEntry) {
        if !entry.symbol.is_empty() {
            self.symbol = entry.symbol.clone();
        }
        if self.rows.len() >= MAX_ROWS {
            self.rows.pop_back();
        }
        self.rows.push_front(entry);
    }

    /// Render just the scrollable rows section (no outer padding needed).
    pub fn render_rows(&self, cx: &Context<Self>) -> impl IntoElement {
        let rows: Vec<TimesAndSalesEntry> = self.rows.iter().cloned().collect();

        v_flex()
            .size_full()
            // Column header
            .child(self.render_header(cx))
            // Scrollable row list
            .child(
                div().id("ts-scroll").flex_1().overflow_y_scroll().children(
                    rows.into_iter()
                        .enumerate()
                        .map(|(i, entry)| Self::render_row(i, entry, cx)),
                ),
            )
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .px_2()
            .py_1()
            .gap_0()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().tab_bar)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(div().w(px(70.)).child("Time"))
            .child(div().w(px(80.)).child("Price"))
            .child(div().w(px(50.)).child("Size"))
            .child(div().flex_1()) // fills remaining space on the right
    }

    fn render_row(_i: usize, entry: TimesAndSalesEntry, cx: &Context<Self>) -> impl IntoElement {
        let side = entry.side;
        let text = side.text_color();
        let is_market = matches!(side, TradeSide::BuyMarket | TradeSide::SellMarket);
        let is_pending = matches!(side, TradeSide::BuyPending | TradeSide::SellPending);

        let row = match side {
            TradeSide::BuyMarket => {
                let grad = linear_gradient(
                    90.0,
                    linear_color_stop(crate::ui::theme::buy_market_gradient_top(), 0.0),
                    linear_color_stop(crate::ui::theme::buy_market_gradient_bottom(), 1.0),
                );
                h_flex().px_2().py(px(1.)).gap_0().bg(grad).text_xs()
            }
            TradeSide::SellMarket => {
                let grad = linear_gradient(
                    90.0,
                    linear_color_stop(crate::ui::theme::sell_market_gradient_top(), 0.0),
                    linear_color_stop(crate::ui::theme::sell_market_gradient_bottom(), 1.0),
                );
                h_flex().px_2().py(px(1.)).gap_0().bg(grad).text_xs()
            }
            TradeSide::BuyPending | TradeSide::SellPending => h_flex()
                .px_2()
                .py(px(1.))
                .gap_0()
                .bg(crate::ui::theme::buy_pending_bg())
                .text_xs(),
            TradeSide::Mid => h_flex()
                .px_2()
                .py(px(1.))
                .gap_0()
                .bg(cx.theme().background)
                .text_xs(),
        };

        // Time gets color for market and pending
        let time_color = if is_market || is_pending {
            text
        } else {
            cx.theme().muted_foreground
        };

        row.text_color(text)
            // Time
            .child(
                div()
                    .w(px(70.))
                    .text_color(time_color)
                    .child(entry.time.clone()),
            )
            // Price
            .child(
                div()
                    .w(px(80.))
                    .font_weight(if is_market || is_pending {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(if is_pending {
                        text
                    } else {
                        cx.theme().foreground
                    })
                    .child(format!("{:.5}", entry.price)),
            )
            // Size
            .child(
                div()
                    .w(px(50.))
                    .text_color(if is_pending {
                        text
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(format!("{}", entry.size)),
            )
            // Spacer fills the rest of the row to the right
            .child(div().flex_1())
    }
}

impl Render for TimesAndSalesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_rows(cx)
    }
}
