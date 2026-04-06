use std::collections::VecDeque;

use gpui::{linear_color_stop, linear_gradient, *};
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::infrastructure::datafeed::TradeSide;

const MAX_ROWS: usize = 500;

#[derive(Clone, Debug)]
pub struct TicksEntry {
    pub time: String,
    pub price: f64,
    pub size: u64,
    pub side: TradeSide,
    pub decimals: usize,
}

impl TicksEntry {
    pub fn from_ui_tick_data(td: &crate::application::dto::UiTickData) -> Self {
        Self {
            time: td.time.clone(),
            price: td.price,
            size: td.size,
            side: td.side.clone(),
            decimals: td.decimals,
        }
    }
}

pub struct TicksView {
    rows: VecDeque<TicksEntry>,
}

impl TicksView {
    pub fn new() -> Self {
        Self {
            rows: VecDeque::with_capacity(MAX_ROWS),
        }
    }

    pub fn clear(&mut self) {
        self.rows.clear();
    }

    pub fn push(&mut self, entry: TicksEntry) {
        if self.rows.len() >= MAX_ROWS {
            self.rows.pop_back();
        }
        self.rows.push_front(entry);
    }

    pub fn render_rows(&self, cx: &Context<Self>) -> impl IntoElement {
        let rows: Vec<TicksEntry> = self.rows.iter().cloned().collect();

        v_flex().size_full().child(self.render_header(cx)).child(
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
            .px_1()
            .py_1()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().tab_bar)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(div().w(px(60.)).child("Time"))
            .child(div().w(px(70.)).child("Price"))
            .child(div().w(px(40.)).child("Size"))
            .child(div().flex_1())
    }

    fn render_row(_i: usize, entry: TicksEntry, cx: &Context<Self>) -> impl IntoElement {
        let side = entry.side;
        let text = side.text_color();
        let is_market = matches!(side, TradeSide::BuyMarket | TradeSide::SellMarket);
        let is_pending = matches!(side, TradeSide::BuyPending | TradeSide::SellPending);

        let row = match side {
            TradeSide::BuyMarket => {
                let grad = linear_gradient(
                    90.0,
                    linear_color_stop(
                        crate::presentation::components::theme::buy_market_gradient_top(),
                        0.0,
                    ),
                    linear_color_stop(
                        crate::presentation::components::theme::buy_market_gradient_bottom(),
                        1.0,
                    ),
                );
                h_flex().px_1().py(px(1.)).gap_1().bg(grad).text_xs()
            }
            TradeSide::SellMarket => {
                let grad = linear_gradient(
                    90.0,
                    linear_color_stop(
                        crate::presentation::components::theme::sell_market_gradient_top(),
                        0.0,
                    ),
                    linear_color_stop(
                        crate::presentation::components::theme::sell_market_gradient_bottom(),
                        1.0,
                    ),
                );
                h_flex().px_1().py(px(1.)).gap_1().bg(grad).text_xs()
            }
            TradeSide::BuyPending | TradeSide::SellPending => h_flex()
                .px_1()
                .py(px(1.))
                .gap_1()
                .bg(crate::presentation::components::theme::buy_pending_bg())
                .text_xs(),
            TradeSide::Mid => h_flex()
                .px_1()
                .py(px(1.))
                .gap_1()
                .bg(cx.theme().background)
                .text_xs(),
        };

        let price_str = match entry.decimals {
            2 => format!("{:.2}", entry.price),
            3 => format!("{:.3}", entry.price),
            _ => format!("{:.5}", entry.price),
        };

        let time_color = if is_market || is_pending {
            text
        } else {
            cx.theme().muted_foreground
        };

        row.text_color(text)
            .child(
                div()
                    .w(px(60.))
                    .text_color(time_color)
                    .child(entry.time.clone()),
            )
            .child(
                div()
                    .w(px(70.))
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
                    .child(price_str),
            )
            .child(
                div()
                    .w(px(40.))
                    .text_color(if is_pending {
                        text
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(format!("{}", entry.size)),
            )
            .child(div().flex_1())
    }
}

impl Render for TicksView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_rows(cx)
    }
}
