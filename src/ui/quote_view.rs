use std::time::Instant;

use gpui::prelude::*;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme, StyledExt};

use crate::datafeed::QuoteData;

pub struct QuoteView {
    pub data: Option<QuoteData>,
    pub last_update: Option<Instant>,
    current_time: Instant,
}

impl QuoteView {
    pub fn new() -> Self {
        Self {
            data: None,
            last_update: None,
            current_time: Instant::now(),
        }
    }

    pub fn clear(&mut self) {
        self.data = None;
        self.last_update = None;
    }

    pub fn update(&mut self, quote: QuoteData) {
        self.data = Some(quote);
        self.last_update = Some(Instant::now());
    }

    fn get_open(&self) -> f64 {
        self.data
            .as_ref()
            .and_then(|q| q.entries.opening_price.as_ref())
            .map(|e| e.price)
            .unwrap_or(0.0)
    }

    fn get_high(&self) -> f64 {
        self.data
            .as_ref()
            .and_then(|q| q.entries.high_price.as_ref())
            .map(|e| e.price)
            .unwrap_or(0.0)
    }

    fn get_low(&self) -> f64 {
        self.data
            .as_ref()
            .and_then(|q| q.entries.low_price.as_ref())
            .map(|e| e.price)
            .unwrap_or(0.0)
    }

    fn get_last(&self) -> f64 {
        self.data
            .as_ref()
            .and_then(|q| q.entries.trade.as_ref())
            .map(|e| e.price)
            .unwrap_or(0.0)
    }

    fn get_change(&self) -> f64 {
        let open = self.get_open();
        let last = self.get_last();
        if open > 0.0 {
            last - open
        } else {
            0.0
        }
    }

    fn get_change_percent(&self) -> f64 {
        let open = self.get_open();
        let change = self.get_change();
        if open > 0.0 {
            (change / open) * 100.0
        } else {
            0.0
        }
    }

    fn get_amplitude(&self) -> f64 {
        let high = self.get_high();
        let low = self.get_low();
        if low > 0.0 && high > 0.0 {
            high - low
        } else {
            0.0
        }
    }

    fn get_range(&self) -> f64 {
        self.get_amplitude()
    }

    fn format_time(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let hours = (secs / 3600) % 24;
        let mins = (secs / 60) % 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, seconds)
    }

    fn get_seconds_since_update(&self) -> u64 {
        self.last_update
            .map(|last| {
                let elapsed = self.current_time.duration_since(last);
                elapsed.as_secs()
            })
            .unwrap_or(0)
    }
}

impl Render for QuoteView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.current_time = Instant::now();

        let theme = cx.theme();
        let bg = rgb(0x1f2933);
        let card_bg = rgb(0x2b3642);
        let text_primary = rgb(0xffffff);
        let text_secondary = rgb(0x9ca3af);
        let green = rgb(0x22c55e);
        let red = rgb(0xef4444);

        let open = self.get_open();
        let high = self.get_high();
        let low = self.get_low();
        let change = self.get_change();
        let change_pct = self.get_change_percent();
        let amplitude = self.get_amplitude();
        let range = self.get_range();
        let _time = self.format_time();
        let secs_since = self.get_seconds_since_update();

        let is_positive = change >= 0.0;
        let change_color = if is_positive { green } else { red };

        h_flex()
            .w_full()
            .h(px(120.))
            .gap_2()
            .bg(bg)
            .p_2()
            .children([
                v_flex()
                    .flex_1()
                    .h_full()
                    .bg(card_bg)
                    .rounded_md()
                    .p_3()
                    .justify_between()
                    .child(div().text_xs().text_color(text_secondary).child("Abertura"))
                    .child(div().text_2xl().font_bold().text_color(text_primary).child(
                        if open > 0.0 {
                            format!("{:.5}", open)
                        } else {
                            "-".to_string()
                        },
                    ))
                    .child(div().h_px().w_full().bg(theme.border))
                    .child(
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(div().text_sm().text_color(text_secondary).child("Máxima"))
                            .child(div().text_sm().font_semibold().text_color(green).child(
                                if high > 0.0 {
                                    format!("{:.5}", high)
                                } else {
                                    "-".to_string()
                                },
                            )),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(div().text_sm().text_color(text_secondary).child("Mínima"))
                            .child(div().text_sm().font_semibold().text_color(red).child(
                                if low > 0.0 {
                                    format!("{:.5}", low)
                                } else {
                                    "-".to_string()
                                },
                            )),
                    ),
                v_flex()
                    .flex_1()
                    .h_full()
                    .bg(card_bg)
                    .rounded_md()
                    .p_3()
                    .justify_between()
                    .child(div().text_xs().text_color(text_secondary).child("Variação"))
                    .child(div().text_2xl().font_bold().text_color(change_color).child(
                        if open > 0.0 {
                            format!("{:+.2}% ({:+.5})", change_pct, change)
                        } else {
                            "-".to_string()
                        },
                    ))
                    .child(div().h_px().w_full().bg(theme.border))
                    .child(
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(text_secondary)
                                    .child("Amplitude"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(text_primary)
                                    .child(if amplitude > 0.0 {
                                        format!("{:.5}", amplitude)
                                    } else {
                                        "-".to_string()
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(div().text_sm().text_color(text_secondary).child("Range"))
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(text_primary)
                                    .child(if range > 0.0 {
                                        format!("{:.5}", range)
                                    } else {
                                        "-".to_string()
                                    }),
                            ),
                    ),
                v_flex()
                    .flex_1()
                    .h_full()
                    .bg(card_bg)
                    .rounded_md()
                    .p_3()
                    .justify_between()
                    .child(div().text_xs().text_color(text_secondary).child("Última"))
                    .child(div().text_2xl().font_bold().text_color(text_primary).child(
                        if self.data.is_some() {
                            self.format_time()
                        } else {
                            "-".to_string()
                        },
                    ))
                    .child(div().h_px().w_full().bg(theme.border))
                    .child(
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(text_secondary)
                                    .child("Última Atualização"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(text_primary)
                                    .child(if self.last_update.is_some() {
                                        self.format_time()
                                    } else {
                                        "-".to_string()
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(text_secondary)
                                    .child("Tempo Desde"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(text_primary)
                                    .child(if self.last_update.is_some() {
                                        format!("{}s", secs_since)
                                    } else {
                                        "-".to_string()
                                    }),
                            ),
                    ),
            ])
    }
}
