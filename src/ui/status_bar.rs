use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, h_flex, progress::Progress};

/// Disk information snapshot used by the status bar.
#[derive(Clone, Default)]
pub struct DiskInfo {
    pub total: u64,
    pub used: u64,
}

/// Battery information snapshot used by the status bar.
#[derive(Clone)]
pub struct BatteryInfo {
    pub icon: IconName,
    pub percentage: f32,
}

/// Data needed to render the status bar. Callers fill this from their own
/// system-info state and pass it to `render_status_bar`.
pub struct StatusBarData<'a> {
    pub disk_info: &'a [DiskInfo],
    pub battery_info: &'a [BatteryInfo],
    /// App-process CPU usage in percent (0.0–100.0).
    pub app_cpu: f64,
    /// App-process memory in bytes.
    pub app_memory: u64,
}

/// Render the bottom status bar.  This is intentionally a free function so
/// every tab can call it without duplicating logic.
pub fn render_status_bar<T: 'static>(data: StatusBarData<'_>, cx: &Context<T>) -> impl IntoElement {
    let primary_disk = data.disk_info.first();
    let primary_battery = data.battery_info.first();

    h_flex()
        .px_3()
        .gap_4()
        .h_7()
        .text_sm()
        .items_center()
        .justify_between()
        .border_t_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().tab_bar)
        .text_color(cx.theme().muted_foreground)
        .child(
            h_flex()
                .gap_4()
                // ── Disk ──────────────────────────────────────────────────
                .when_some(primary_disk, |this, disk| {
                    let used_percent = if disk.total > 0 {
                        (disk.used as f64 / disk.total as f64 * 100.0) as f32
                    } else {
                        0.0
                    };
                    this.child(
                        h_flex()
                            .gap_2()
                            .w(px(135.))
                            .items_center()
                            .child(Icon::new(IconName::HardDrive))
                            .child(
                                Progress::new("status-disk")
                                    .w_12()
                                    .h_2()
                                    .value(used_percent),
                            )
                            .child(format!("{:.0}%", used_percent)),
                    )
                })
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
                }),
        )
        .child(div().when_some(primary_battery, |this, battery| {
            this.child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(battery.icon.clone()))
                    .child(format!("{:.0}%", battery.percentage)),
            )
        }))
}

/// Helper: convert raw bytes (total/used) from sysinfo disks into `DiskInfo`.
pub fn disk_info_from(total: u64, used: u64) -> DiskInfo {
    DiskInfo { total, used }
}
