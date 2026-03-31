use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
#[allow(unused_imports)]
mod datafeed;
mod ui;

use datafeed::{create_provider, DataFeedBackend, DataFeedProvider, DataType, DomData, QuoteData, SymbolManager, TickData};
use ui::{
    BatteryInfo, DiskInfo, DomView, ProcessTableDelegate, StatusBarData,
    TimesAndSalesEntry, TimesAndSalesView, UiDomData, UiTickData, render_status_bar,
};

use std::time::Duration;

use gpui::{actions, prelude::FluentBuilder as _, *};
use gpui_component::ThemeMode;
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_component::{
    ActiveTheme, IconName, Root, Sizable, Theme, TitleBar,
    chart::AreaChart,
    h_flex,
    tab::{Tab, TabBar},
    table::{DataTable, TableState},
    v_flex,
};
use smol::Timer;
use sysinfo::{Disks, ProcessRefreshKind, RefreshKind, System};
use tokio::sync::mpsc;

// ──────────────────────────────────────────────────────────────────────────────
// Actions
// ──────────────────────────────────────────────────────────────────────────────

actions!(icetrader, [Quit]);

const INTERVAL: Duration = Duration::from_millis(500);
const MAX_DATA_POINTS: usize = 120;

// ──────────────────────────────────────────────────────────────────────────────
// Tab enum
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum MonitorTab {
    #[default]
    OrderFlow = 0,
    System = 1,
    Processes = 2,
}

impl MonitorTab {
    fn from_index(index: usize) -> Self {
        match index {
            0 => MonitorTab::OrderFlow,
            1 => MonitorTab::System,
            2 => MonitorTab::Processes,
            _ => MonitorTab::OrderFlow,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal structs (kept in main.rs — system monitor specific)
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct MetricPoint {
    time: String,
    cpu: f64,
    memory: f64,
}

// ──────────────────────────────────────────────────────────────────────────────
// SystemMonitor — the root GPUI entity
// ──────────────────────────────────────────────────────────────────────────────

pub struct SystemMonitor {
    // ── System info ──────────────────────────────────────────────────────────
    sys: System,
    disks: Disks,
    data: VecDeque<MetricPoint>,
    time_index: usize,
    disk_info: Vec<DiskInfo>,
    battery_info: Vec<BatteryInfo>,
    app_cpu: f64,
    app_memory: u64,

    // ── Tab state ─────────────────────────────────────────────────────────────
    active_tab: MonitorTab,

    // ── Processes tab ─────────────────────────────────────────────────────────
    process_table: Entity<TableState<ProcessTableDelegate>>,

    // ── OrderFlow tab ─────────────────────────────────────────────────────────
    times_and_sales: Entity<TimesAndSalesView>,
    dom_view: Entity<DomView>,
    datafeed_rx: Option<mpsc::Receiver<(String, DataType, String)>>,
    symbol_select: Entity<SelectState<Vec<SharedString>>>,
    current_symbol: Arc<RwLock<String>>,
    subscriber: Arc<dyn DataFeedProvider>,
    #[allow(dead_code)]
    symbol_manager: Arc<SymbolManager>,
}

impl SystemMonitor {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();

        let process_delegate = ProcessTableDelegate::new();
        let process_table = cx.new(|cx| {
            TableState::new(process_delegate, window, cx)
                .col_selectable(false)
                .col_movable(false)
        });

        let times_and_sales = cx.new(|_| TimesAndSalesView::new());
        let dom_view = cx.new(|_| DomView::new());

        // ── Setup symbol dropdown ─────────────────────────────────────────────
        let available_symbols = datafeed::get_available_symbols()
            .into_iter()
            .map(|s| s.name.into())
            .collect::<Vec<SharedString>>();

        let initial_symbol = "6E";

        let symbol_select = cx.new(|cx| SelectState::new(available_symbols, None, window, cx));

        symbol_select.update(cx, |state, cx| {
            state.set_selected_value(&gpui::SharedString::from(initial_symbol), window, cx);
        });

        // ── Setup Symbol Manager ──────────────────────────────────────────────
        let (change_tx, mut change_rx) = tokio::sync::mpsc::channel(100);
        let symbol_manager = Arc::new(SymbolManager::new(change_tx));

        // ── Start datafeed subscriber ─────────────────────────────────────────
        let subscriber = create_provider(DataFeedBackend::Zmq);

        // Subscribe to initial symbol
        subscriber.subscribe("6E", DataType::Tick);
        subscriber.subscribe("6E", DataType::Dom);
        subscriber.subscribe("6E", DataType::Quote);

        let datafeed_rx = subscriber.start();

        // Spawn task to handle SymbolChangeEvents
        cx.background_executor()
            .spawn({
                let subscriber = subscriber.clone();
                async move {
                    while let Some(event) = change_rx.recv().await {
                        // Unsubscribe old symbol
                        if let Some(old) = event.old_symbol {
                            for dt in &event.data_types {
                                let mapped_dt = match dt {
                                    datafeed::symbols::SymbolDataType::Tick => DataType::Tick,
                                    datafeed::symbols::SymbolDataType::Dom => DataType::Dom,
                                    datafeed::symbols::SymbolDataType::Quote => DataType::Quote,
                                };
                                if subscriber.is_subscribed(&old, mapped_dt) {
                                    subscriber.unsubscribe(&old, mapped_dt);
                                }
                            }
                        }
                        // Subscribe new symbol
                        if let Some(new) = event.new_symbol {
                            for dt in &event.data_types {
                                let mapped_dt = match dt {
                                    datafeed::symbols::SymbolDataType::Tick => DataType::Tick,
                                    datafeed::symbols::SymbolDataType::Dom => DataType::Dom,
                                    datafeed::symbols::SymbolDataType::Quote => DataType::Quote,
                                };
                                if !subscriber.is_subscribed(&new, mapped_dt) {
                                    subscriber.subscribe(&new, mapped_dt);
                                }
                            }
                        }
                    }
                }
            })
            .detach();

        // Initialize symbol manager with ESM5
        cx.background_executor()
            .spawn({
                let symbol_manager = symbol_manager.clone();
                let initial = initial_symbol.to_string();
                async move {
                    let _ = symbol_manager
                        .set_symbol(
                            &initial,
                            vec![
                                datafeed::symbols::SymbolDataType::Tick,
                                datafeed::symbols::SymbolDataType::Dom,
                                datafeed::symbols::SymbolDataType::Quote,
                            ],
                        )
                        .await;
                }
            })
            .detach();

        let mut monitor = Self {
            sys,
            disks,
            data: VecDeque::with_capacity(MAX_DATA_POINTS),
            time_index: 0,
            disk_info: Vec::new(),
            battery_info: Vec::new(),
            app_cpu: 0.0,
            app_memory: 0,
            active_tab: MonitorTab::OrderFlow,
            process_table,
            times_and_sales: times_and_sales.clone(),
            dom_view: dom_view.clone(),
            datafeed_rx: Some(datafeed_rx),
            symbol_select: symbol_select.clone(),
            current_symbol: Arc::new(RwLock::new(initial_symbol.to_string())),
            subscriber: subscriber.clone(),
            symbol_manager: symbol_manager.clone(),
        };

        // ── Handle symbol change events ───────────────────────────────────────
        cx.subscribe(&symbol_select, {
            let symbol_manager = symbol_manager.clone();
            let ts_entity = times_and_sales.clone();
            let dom_entity = dom_view.clone();
            move |this: &mut SystemMonitor, _entity, event: &SelectEvent<Vec<SharedString>>, cx| {
                if let SelectEvent::Confirm(Some(new_symbol)) = event {
                    let old_symbol = this.current_symbol.blocking_read().clone();
                    if old_symbol != new_symbol.as_ref() {
                        let new_sym_str = new_symbol.to_string();

                        *this.current_symbol.blocking_write() = new_sym_str.clone();

                        let symbol_manager = symbol_manager.clone();
                        cx.background_executor()
                            .spawn(async move {
                                let _ = symbol_manager
                                    .set_symbol(
                                        &new_sym_str,
                                        vec![
                                            datafeed::symbols::SymbolDataType::Tick,
                                            datafeed::symbols::SymbolDataType::Dom,
                                            datafeed::symbols::SymbolDataType::Quote,
                                        ],
                                    )
                                    .await;
                            })
                            .detach();

                        ts_entity.update(cx, |v, cx| {
                            v.clear();
                            cx.notify();
                        });
                        dom_entity.update(cx, |v, cx| {
                            v.clear();
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();

        monitor.collect_metrics(cx);
        monitor.start_system_loop(cx);
        monitor.start_datafeed_loop(cx);

        monitor
    }

    // ── System metrics loop ───────────────────────────────────────────────────

    fn start_system_loop(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(INTERVAL).await;
                let result = this.update(cx, |this, cx| {
                    this.collect_metrics(cx);
                    cx.notify();
                });
                if result.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    // ── ZeroMQ datafeed loop ──────────────────────────────────────────────────

    fn start_datafeed_loop(&mut self, cx: &mut Context<Self>) {
        let Some(mut rx) = self.datafeed_rx.take() else {
            return;
        };

        let ts_entity = self.times_and_sales.clone();
        let dom_entity = self.dom_view.clone();
        let current_symbol = self.current_symbol.clone();

        cx.spawn(async move |_this, cx| {
            loop {
                // Poll for messages in a non-blocking way
                match rx.try_recv() {
                    Ok((symbol, data_type, json)) => {
                        // Read the current symbol from the shared Arc
                        let current = current_symbol.read().await;

                        // Only process messages for current symbol
                        if symbol != *current {
                            continue;
                        }

                        match data_type {
                            DataType::Tick => {
                                if let Ok(ticks) = serde_json::from_str::<Vec<TickData>>(&json) {
                                    for td in ticks {
                                        let ui_tick = UiTickData::from_tick_data(&td, &symbol);
                                        let entry = TimesAndSalesEntry::from_ui_tick_data(&ui_tick);
                                        let price = td.price;
                                        let bid_price = td.bid_price;
                                        let ask_price = td.ask_price;
                                        ts_entity.update(cx, |view, cx| {
                                            view.push(entry);
                                            cx.notify();
                                        });
                                        dom_entity.update(cx, |view, cx| {
                                            view.update_price(price);
                                            view.update_tick_prices(bid_price, ask_price);
                                            cx.notify();
                                        });
                                    }
                                }
                            }
                            DataType::Dom => {
                                if let Ok(dom) = serde_json::from_str::<DomData>(&json) {
                                    let ui_dom = UiDomData::from(dom);
                                    dom_entity.update(cx, |view, cx| {
                                        view.update_dom(ui_dom);
                                        cx.notify();
                                    });
                                }
                            }
                            DataType::Quote => {
                                if let Ok(_quote) = serde_json::from_str::<QuoteData>(&json) {
                                }
                            }
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // No message yet — yield to avoid busy-spin
                        smol::Timer::after(Duration::from_millis(10)).await;
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        })
        .detach();
    }

    // ── System data collection ────────────────────────────────────────────────

    fn collect_metrics(&mut self, cx: &mut Context<Self>) {
        self.sys.refresh_specifics(
            RefreshKind::everything().with_processes(ProcessRefreshKind::everything()),
        );
        self.disks.refresh(true);

        self.app_cpu = 0.0;
        self.app_memory = 0;
        for process in self.sys.processes().values() {
            let name = process.name().to_string_lossy().to_lowercase();
            if name.contains("icetrader") {
                self.app_cpu = process.cpu_usage() as f64;
                self.app_memory = process.memory();
                break;
            }
        }

        let cpu_usage = self.sys.global_cpu_usage() as f64;
        let total_memory = self.sys.total_memory() as f64;
        let used_memory = self.sys.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 {
            (used_memory / total_memory * 100.0).min(100.0)
        } else {
            0.0
        };

        let point = MetricPoint {
            time: format!("{}s", self.time_index),
            cpu: cpu_usage,
            memory: memory_usage,
        };

        if self.data.len() >= MAX_DATA_POINTS {
            self.data.pop_front();
        }
        self.data.push_back(point);
        self.time_index += 1;

        self.process_table.update(cx, |table, cx| {
            table.delegate_mut().update_processes(&self.sys);
            cx.notify();
        });

        self.disk_info = self
            .disks
            .iter()
            .map(|disk| {
                crate::ui::disk_info_from(
                    disk.total_space(),
                    disk.total_space() - disk.available_space(),
                )
            })
            .collect();

        self.update_battery_info();
    }

    fn update_battery_info(&mut self) {
        self.battery_info.clear();
        if let Ok(manager) = battery::Manager::new()
            && let Ok(batteries) = manager.batteries()
        {
            for battery in batteries.flatten() {
                let icon = match battery.state() {
                    battery::State::Charging => IconName::BatteryCharging,
                    battery::State::Discharging => IconName::BatteryMedium,
                    battery::State::Full => IconName::BatteryFull,
                    battery::State::Empty => IconName::Battery,
                    _ => IconName::Battery,
                };
                self.battery_info.push(BatteryInfo {
                    icon,
                    percentage: battery.state_of_charge().value * 100.0,
                });
            }
        }
    }

    fn set_active_tab(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_tab = MonitorTab::from_index(index);
        cx.notify();
    }

    // ── Status bar (shared across all tabs) ───────────────────────────────────

    fn render_status_bar_view(&self, cx: &Context<Self>) -> impl IntoElement {
        render_status_bar(
            StatusBarData {
                disk_info: &self.disk_info,
                battery_info: &self.battery_info,
                app_cpu: self.app_cpu,
                app_memory: self.app_memory,
            },
            cx,
        )
    }

    // ── System tab ────────────────────────────────────────────────────────────

    fn render_chart(
        &self,
        title: &str,
        data: Vec<MetricPoint>,
        value_fn: impl Fn(&MetricPoint) -> f64 + 'static,
        color: Hsla,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .min_h(px(160.))
            .flex_1()
            .gap_2()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .justify_between()
                    .py_1()
                    .px_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(title.to_string()),
                    )
                    .child({
                        let current_value = data.last().map(&value_fn).unwrap_or(0.0);
                        div()
                            .text_sm()
                            .text_color(color)
                            .child(format!("{:.1}%", current_value))
                    }),
            )
            .child(
                AreaChart::new(data)
                    .x(|d| d.time.clone())
                    .y(value_fn)
                    .stroke(color)
                    .fill(linear_gradient(
                        0.,
                        linear_color_stop(color.opacity(0.4), 1.),
                        linear_color_stop(cx.theme().background.opacity(0.1), 0.),
                    ))
                    .tick_margin(15),
            )
    }

    fn render_system_tab(&self, cx: &Context<Self>) -> impl IntoElement {
        let data: Vec<MetricPoint> = self.data.iter().cloned().collect();
        v_flex()
            .p_3()
            .gap_4()
            .flex_1()
            // CPU in red
            .child(self.render_chart("CPU Usage", data.clone(), |d| d.cpu, cx.theme().red, cx))
            // Memory in blue (was green)
            .child(self.render_chart(
                "Memory Usage",
                data.clone(),
                |d| d.memory,
                cx.theme().blue,
                cx,
            ))
    }

    fn render_processes_tab(&self, _cx: &Context<Self>) -> impl IntoElement {
        v_flex().size_full().child(
            DataTable::new(&self.process_table)
                .bordered(false)
                .stripe(true)
                .small(),
        )
    }

    // ── OrderFlow tab ─────────────────────────────────────────────────────────

    fn render_orderflow_tab(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let select_entity = self.symbol_select.clone();

        // Two-panel layout: DOM on the LEFT, Times & Sales on the RIGHT (fixed widths)
        h_flex()
            .size_full()
            // DOM panel (fixed width on LEFT)
            .child(
                v_flex()
                    .w(px(310.))
                    .h_full()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    // Sub-header
                    .child(
                        h_flex()
                            .px_2()
                            .py_1()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().tab_bar)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
                            .child("DOM"),
                    )
                    .child(div().flex_1().child(self.dom_view.clone())),
            )
            // Times & Sales panel (fills remaining space on RIGHT)
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    // Sub-header with Dropdown
                    .child(
                        h_flex()
                            .px_2()
                            .py(px(4.))
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().tab_bar)
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("Times & Sales"),
                            )
                            .child(Select::new(&select_entity).small().w(px(100.))),
                    )
                    .child(div().flex_1().child(self.times_and_sales.clone())),
            )
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Render
// ──────────────────────────────────────────────────────────────────────────────

impl Render for SystemMonitor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab_index = self.active_tab as usize;

        v_flex()
            .size_full()
            .child(
                TitleBar::new()
                    .child(
                        TabBar::new("monitor-tabs")
                            .mt(px(1.))
                            .segmented()
                            .px_0()
                            .py(px(2.))
                            .bg(cx.theme().title_bar)
                            .selected_index(active_tab_index)
                            .on_click(cx.listener(|this, ix: &usize, window, cx| {
                                this.set_active_tab(*ix, window, cx);
                            }))
                            .child(Tab::new().label("OrderFlow"))
                            .child(Tab::new().label("System"))
                            .child(Tab::new().label("Processes")),
                    )
                    .child(
                        div()
                            .mr_4()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{:.1} GB",
                                self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
                            )),
                    ),
            )
            .bg(cx.theme().background)
            .child(
                div()
                    .id("tab-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .map(|this| match self.active_tab {
                        MonitorTab::OrderFlow => this.child(self.render_orderflow_tab(window, cx)),
                        MonitorTab::System => this.child(self.render_system_tab(cx)),
                        MonitorTab::Processes => this.child(self.render_processes_tab(cx)),
                    }),
            )
            // Status bar present on ALL tabs
            .child(self.render_status_bar_view(cx))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Entry point
// ──────────────────────────────────────────────────────────────────────────────

impl Drop for SystemMonitor {
    fn drop(&mut self) {
        self.subscriber.shutdown();
    }
}

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.bind_keys([
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-q", Quit, None),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("alt-f4", Quit, None),
        ]);

        cx.on_action(|_: &Quit, cx: &mut App| {
            cx.quit();
        });

        let window_options = WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            window_bounds: Some(WindowBounds::centered(size(px(820.), px(780.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                window.activate_window();
                window.set_window_title("OrderFlow");

                Theme::change(ThemeMode::Dark, Some(window), cx);

                let view = cx.new(|cx| SystemMonitor::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
