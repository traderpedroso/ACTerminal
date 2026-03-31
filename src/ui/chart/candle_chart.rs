use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{canvas, div, px, rgb, Bounds, MouseButton, Pixels, ScrollDelta};
use gpui_component::v_flex;

use gpui_component::ActiveTheme;

use super::orderbook_chart::{ChartDataPoint, TradeSide};
use crate::datafeed::types::QuoteData;
use crate::datafeed::TickData;

const CHART_HEIGHT: f32 = 400.0;
const PRICE_AXIS_WIDTH: f32 = 70.0;
const CHART_PX_PADDING: f32 = 8.0;
const CANDLE_GAP: f32 = 2.0;

pub struct CandleChartComponent {
    pub data: Vec<ChartDataPoint>,
    pub visible_count: usize,
    pub scroll_offset: usize,
    pub is_dragging: bool,
    pub drag_start_x: f32,
    pub drag_start_offset: usize,
    pub chart_area_origin: Rc<Cell<(f32, f32)>>,
    pub chart_area_width: Rc<Cell<f32>>,
}

impl CandleChartComponent {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            visible_count: 60,
            scroll_offset: 0,
            is_dragging: false,
            drag_start_x: 0.0,
            drag_start_offset: 0,
            chart_area_origin: Rc::new(Cell::new((0.0, 0.0))),
            chart_area_width: Rc::new(Cell::new(0.0)),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.scroll_offset = 0;
    }

    pub fn add_tick(&mut self, tick: &TickData) {
        let side = match tick.side.as_str() {
            "BUY_MARKET" | "BUY_PENDING" => Some(TradeSide::Buy),
            "SELL_MARKET" | "SELL_PENDING" => Some(TradeSide::Sell),
            _ => None,
        };

        let mid_price = if tick.bid_price > 0.0 && tick.ask_price > 0.0 {
            (tick.bid_price + tick.ask_price) / 2.0
        } else {
            tick.price
        };

        let time = format_time(tick.time);

        let point = ChartDataPoint {
            time,
            timestamp: tick.time,
            price: mid_price,
            bid_price: Some(tick.bid_price),
            ask_price: Some(tick.ask_price),
            open: Some(tick.price),
            high: Some(tick.price),
            low: Some(tick.price),
            close: Some(tick.price),
            volume: Some(tick.size as f64),
            side,
        };

        self.data.push(point);
    }

    pub fn add_quote(&mut self, quote: &QuoteData) {
        let time = quote.timestamp.clone();
        let timestamp = parse_timestamp(&time);

        let open = quote.entries.trade.as_ref().map(|t| t.price);
        let high = quote.entries.high_price.as_ref().map(|t| t.price);
        let low = quote.entries.low_price.as_ref().map(|t| t.price);
        let close = quote.entries.trade.as_ref().map(|t| t.price);
        let volume = quote.entries.trade.as_ref().map(|t| t.size);

        let bid = quote.entries.bid.as_ref().map(|b| b.price).unwrap_or(0.0);
        let ask = quote.entries.offer.as_ref().map(|o| o.price).unwrap_or(0.0);

        let mid_price = if bid > 0.0 && ask > 0.0 {
            (bid + ask) / 2.0
        } else {
            close.unwrap_or(0.0)
        };

        let point = ChartDataPoint {
            time,
            timestamp,
            price: mid_price,
            bid_price: Some(bid),
            ask_price: Some(ask),
            open,
            high,
            low,
            close,
            volume: volume.map(|v| v as f64),
            side: None,
        };

        self.data.push(point);
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset = self.data.len().saturating_sub(self.visible_count);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    fn visible_range(&self) -> (usize, usize) {
        if self.data.is_empty() {
            return (0, 0);
        }
        let end = self.data.len().saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(self.visible_count);
        (start, end)
    }

    fn visible_data(&self) -> Vec<ChartDataPoint> {
        let (start, end) = self.visible_range();
        if start == end {
            return Vec::new();
        }
        self.data[start..end].to_vec()
    }

    fn candle_width(&self) -> f32 {
        let container_w = self.chart_area_width.get();
        let n = self.visible_count as f32;
        if container_w > CHART_PX_PADDING * 2.0 && n > 0.0 {
            ((container_w - CHART_PX_PADDING * 2.0 - CANDLE_GAP * (n - 1.0)) / n).clamp(2.0, 20.0)
        } else {
            (8.0f32 * 60.0 / self.visible_count as f32).clamp(2.0, 20.0)
        }
    }

    fn price_range(&self) -> (f64, f64) {
        let visible = self.visible_data();
        if visible.is_empty() {
            return (0.0, 100.0);
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for c in &visible {
            if let Some(high) = c.high {
                max = max.max(high);
            }
            if let Some(low) = c.low {
                min = min.min(low);
            }
            max = max.max(c.price);
            min = min.min(c.price);
        }

        if min == f64::MAX || max == f64::MIN {
            return (0.0, 100.0);
        }

        let padding = (max - min) * 0.05;
        (min - padding, max + padding)
    }
}

impl Default for CandleChartComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for CandleChartComponent {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        if self.data.is_empty() {
            return div()
                .flex_1()
                .size_full()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Waiting for data..."),
                );
        }

        let (price_min, price_max) = self.price_range();
        let price_range = (price_max - price_min).max(0.01);
        let visible = self.visible_data();
        let candle_width = self.candle_width();
        let theme = cx.theme();
        let bg_color = theme.background;
        let muted_fg = theme.muted_foreground;

        let origin_cell = self.chart_area_origin.clone();
        let width_cell = self.chart_area_width.clone();

        v_flex()
            .flex_1()
            .bg(bg_color)
            .on_scroll_wheel(
                cx.listener(|this, event: &gpui::ScrollWheelEvent, _window, cx| {
                    let delta_y = match event.delta {
                        ScrollDelta::Pixels(pt) => f32::from(pt.y),
                        ScrollDelta::Lines(pt) => pt.y * 20.0,
                    };
                    let old_count = this.visible_count;
                    let zoom_step = (old_count as f32 * 0.1).max(2.0) as i32;
                    let new_count = if delta_y > 0.0 {
                        (old_count as i32 + zoom_step).min(200).max(20) as usize
                    } else {
                        (old_count as i32 - zoom_step).min(200).max(20) as usize
                    };
                    if old_count > 0 && new_count != old_count {
                        let right_edge = this.data.len().saturating_sub(this.scroll_offset);
                        let center_candle = right_edge.saturating_sub(old_count / 2);
                        let new_right_edge = center_candle + new_count / 2;
                        this.scroll_offset = this.data.len().saturating_sub(new_right_edge);
                    }
                    this.visible_count = new_count;
                    this.clamp_scroll_offset();
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                    this.is_dragging = true;
                    this.drag_start_x = f32::from(event.position.x);
                    this.drag_start_offset = this.scroll_offset;
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |this, _event: &gpui::MouseUpEvent, _window, cx| {
                    this.is_dragging = false;
                    cx.notify();
                }),
            )
            .on_mouse_move(
                cx.listener(move |this, event: &gpui::MouseMoveEvent, _window, cx| {
                    if this.is_dragging {
                        let delta_x = f32::from(event.position.x) - this.drag_start_x;
                        let cw = this.candle_width();
                        let candle_step = cw + CANDLE_GAP;
                        if candle_step > 0.0 {
                            let candle_delta = (delta_x / candle_step).round() as isize;
                            let new_offset =
                                (this.drag_start_offset as isize - candle_delta).max(0) as usize;
                            this.scroll_offset = new_offset;
                            this.clamp_scroll_offset();
                        }
                    }
                    cx.notify();
                }),
            )
            .children([div()
                .flex()
                .w_full()
                .h(px(CHART_HEIGHT))
                .child(
                    div()
                        .h(px(CHART_HEIGHT))
                        .flex_grow()
                        .min_w_0()
                        .relative()
                        .overflow_hidden()
                        .child(
                            canvas(
                                move |bounds: Bounds<Pixels>, _window, _cx| {
                                    origin_cell.set((
                                        f32::from(bounds.origin.x),
                                        f32::from(bounds.origin.y),
                                    ));
                                    width_cell.set(f32::from(bounds.size.width));
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_full(),
                        )
                        .child(
                            div()
                                .h(px(CHART_HEIGHT))
                                .w_full()
                                .flex()
                                .items_end()
                                .px(px(CHART_PX_PADDING))
                                .gap(px(CANDLE_GAP))
                                .children(visible.iter().enumerate().map(|(i, candle)| {
                                    render_candle(
                                        i,
                                        candle,
                                        CHART_HEIGHT,
                                        price_min,
                                        price_range,
                                        candle_width,
                                    )
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_between()
                        .h(px(CHART_HEIGHT))
                        .pl(px(4.))
                        .pr(px(4.))
                        .w(px(PRICE_AXIS_WIDTH))
                        .flex_shrink_0()
                        .children((0..5).rev().map(|i| {
                            let price = price_min + (i as f64 / 4.0) * (price_max - price_min);
                            div()
                                .text_size(px(10.))
                                .text_color(muted_fg)
                                .child(format!("{:.5}", price))
                        })),
                )])
    }
}

fn render_candle(
    _index: usize,
    candle: &ChartDataPoint,
    chart_height: f32,
    price_min: f64,
    price_range: f64,
    candle_width: f32,
) -> impl IntoElement {
    let open = candle.open.unwrap_or(candle.price);
    let close = candle.close.unwrap_or(candle.price);
    let high = candle.high.unwrap_or(candle.price.max(open).max(close));
    let low = candle.low.unwrap_or(candle.price.min(open).min(close));

    let is_bullish = close >= open;
    let color = if is_bullish {
        rgb(0x22c55e)
    } else {
        rgb(0xef4444)
    };

    let body_top = if is_bullish { close } else { open };
    let body_bottom = if is_bullish { open } else { close };

    let high_px = ((high - price_min) / price_range * chart_height as f64) as f32;
    let low_px = ((low - price_min) / price_range * chart_height as f64) as f32;
    let body_top_px = ((body_top - price_min) / price_range * chart_height as f64) as f32;
    let body_bottom_px = ((body_bottom - price_min) / price_range * chart_height as f64) as f32;

    let top_spacer = (chart_height - high_px).max(0.0);
    let upper_wick = (high_px - body_top_px).max(0.0);
    let body_h = (body_top_px - body_bottom_px).max(1.0);
    let lower_wick = (body_bottom_px - low_px).max(0.0);
    let bottom_spacer = low_px.max(0.0);

    let wick_width: f32 = 1.0;
    div()
        .w(px(candle_width))
        .h(px(chart_height))
        .flex()
        .flex_col()
        .items_center()
        .child(div().w(px(candle_width)).h(px(top_spacer)))
        .child(div().w(px(wick_width)).h(px(upper_wick)).bg(color))
        .child(div().w(px(candle_width)).h(px(body_h)).bg(color))
        .child(div().w(px(wick_width)).h(px(lower_wick)).bg(color))
        .child(div().w(px(candle_width)).h(px(bottom_spacer)))
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
