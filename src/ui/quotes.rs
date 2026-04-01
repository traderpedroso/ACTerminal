use gpui::*;
use gpui_component::{h_flex, ActiveTheme};

use crate::datafeed::dto::UiQuoteData;

pub struct QuotesView {
    pub data: Option<UiQuoteData>,
}

impl QuotesView {
    pub fn new() -> Self {
        Self { data: None }
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
                .bg(cx.theme().tab_bar)
                .border_b_1()
                .border_color(cx.theme().border)
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

        let high_color = Hsla {
            h: 0.0,
            s: 0.8,
            l: 0.45,
            a: 1.0,
        };
        let low_color = Hsla {
            h: 0.55,
            s: 0.8,
            l: 0.45,
            a: 1.0,
        };

        h_flex()
            .w_full()
            .h(px(28.))
            .px_2()
            .gap_3()
            .items_center()
            .justify_between()
            .bg(cx.theme().tab_bar)
            .border_b_1()
            .border_color(cx.theme().border)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Open:"),
                    )
                    .child(div().text_color(cx.theme().foreground).child(open_str)),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(
                        div()
                            .text_color(high_color)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("High:"),
                    )
                    .child(div().text_color(high_color).child(high_str)),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(
                        div()
                            .text_color(low_color)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Low:"),
                    )
                    .child(div().text_color(low_color).child(low_str)),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Range:"),
                    )
                    .child(div().text_color(cx.theme().foreground).child(range_str)),
            )
            .into_any_element()
    }
}
