use gpui::*;
use gpui_component::{h_flex, progress::Progress, ActiveTheme, Icon, IconName};

pub struct StatusBarData {
    /// App-process CPU usage in percent (0.0–100.0).
    pub app_cpu: f64,
    /// App-process memory in bytes.
    pub app_memory: u64,
}

/// Render the bottom status bar.  This is intentionally a free function so
/// every tab can call it without duplicating logic.
pub fn render_status_bar<T: 'static>(data: StatusBarData, cx: &Context<T>) -> impl IntoElement {
    h_flex()
        .px_3()
        .gap_4()
        .h_7()
        .text_sm()
        .items_center()
        .justify_end()
        .border_t_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().tab_bar)
        .text_color(cx.theme().muted_foreground)
        .child({
            h_flex()
                .gap_4()
                // ── RAM (app) ──────────────────────────────────────────────
                .child({
                    let mem_mb = data.app_memory as f64 / 1024.0 / 1024.0;
                    h_flex()
                        .gap_2()
                        .w(px(135.))
                        .items_center()
                        .child(Icon::new(IconName::MemoryStick))
                        .child(
                            Progress::new("status-mem")
                                .w_12()
                                .h_2()
                                .value(data.app_cpu as f32),
                        )
                        .child(format!("{:.1} MB", mem_mb))
                })
                // ── CPU (app) ──────────────────────────────────────────────
                .child({
                    let cpu_percent = data.app_cpu;
                    h_flex()
                        .gap_2()
                        .w(px(135.))
                        .items_center()
                        .child(Icon::new(IconName::Cpu))
                        .child(
                            Progress::new("status-cpu")
                                .w_12()
                                .h_2()
                                .value(cpu_percent as f32),
                        )
                        .child(format!("{:.1}%", cpu_percent))
                })
        })
}
