use gpui::*;
use gpui_component::{h_flex, ActiveTheme};

use crate::datafeed::dto::UiQuoteData;
use crate::domains::trading_display::views::quotes::QuotesViewState;

pub struct QuotesView {
    pub data: Option<UiQuoteData>,
    #[allow(dead_code)]
    state: Option<QuotesViewState>,
}

impl QuotesView {
    pub fn new(state: Option<QuotesViewState>) -> Self {
        Self { data: None, state }
    }

    pub fn clear(&mut self) {
        self.data = None;
    }

    pub fn update_quote(&mut self, data: UiQuoteData) {
        self.data = Some(data);
    }
}

impl Render for QuotesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(ref quote) = self.data else {
            return h_flex()
                .w_full()
                .h(px(28.))
                .px_2()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Waiting for quote data…"),
                )
                .into_any_element();
        };

        let decimals = quote.decimals;
        let fmt = |p: f64| match decimals {
            2 => format!("{:.2}", p),
            3 => format!("{:.3}", p),
            _ => format!("{:.5}", p),
        };

        let high_str = fmt(quote.high());
        let low_str = fmt(quote.low());
        let open_str = fmt(quote.open());
        let range = quote.high() - quote.low();
        let range_str = fmt(range);

        h_flex()
            .w_full()
            .h(px(28.))
            .pl(px(8.))
            .gap_4()
            .items_center()
            .justify_start()
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_baseline()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .font_weight(FontWeight::BOLD)
                            .child("OPEN:"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(open_str),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_baseline()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .font_weight(FontWeight::BOLD)
                            .child("HIGH:"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(Hsla {
                                h: 120.0 / 360.0,
                                s: 0.7,
                                l: 0.5,
                                a: 1.0,
                            })
                            .child(high_str),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_baseline()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .font_weight(FontWeight::BOLD)
                            .child("LOW:"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(Hsla {
                                h: 0.0 / 360.0,
                                s: 0.8,
                                l: 0.55,
                                a: 1.0,
                            })
                            .child(low_str),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_baseline()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .font_weight(FontWeight::BOLD)
                            .child("RANGE:"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(range_str),
                    ),
            )
            .into_any_element()
    }
}
