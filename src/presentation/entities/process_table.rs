use gpui::{div, App, Context, IntoElement, ParentElement, Styled, Window};
use gpui_component::table::{Column, ColumnSort, TableDelegate, TableState};
use gpui_component::ActiveTheme;
use sysinfo::{Pid, System};

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Default)]
pub enum ProcessSortField {
    Pid,
    Name,
    #[default]
    Cpu,
    Memory,
}

pub struct ProcessTableDelegate {
    pub processes: Vec<ProcessInfo>,
    columns: Vec<Column>,
    pub sort_field: ProcessSortField,
    pub sort_order: ColumnSort,
}

impl ProcessTableDelegate {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            columns: vec![
                Column::new("pid", "PID").width(70.).sortable(),
                Column::new("name", "Name").width(380.).sortable(),
                Column::new("cpu", "CPU %")
                    .width(80.)
                    .sortable()
                    .sort(ColumnSort::Descending),
                Column::new("memory", "Memory").width(100.).sortable(),
            ],
            sort_field: ProcessSortField::Cpu,
            sort_order: ColumnSort::Descending,
        }
    }

    pub fn update_processes(&mut self, sys: &System) {
        self.processes = sys
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: *pid,
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            })
            .collect();
        self.sort_processes();
    }

    pub fn sort_processes(&mut self) {
        let is_desc = matches!(self.sort_order, ColumnSort::Descending);
        match self.sort_field {
            ProcessSortField::Pid => self.processes.sort_by(|a, b| {
                let c = a.pid.as_u32().cmp(&b.pid.as_u32());
                if is_desc {
                    c.reverse()
                } else {
                    c
                }
            }),
            ProcessSortField::Name => self.processes.sort_by(|a, b| {
                let c = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                if is_desc {
                    c.reverse()
                } else {
                    c
                }
            }),
            ProcessSortField::Cpu => self.processes.sort_by(|a, b| {
                let c = a
                    .cpu_usage
                    .partial_cmp(&b.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if is_desc {
                    c.reverse()
                } else {
                    c
                }
            }),
            ProcessSortField::Memory => self.processes.sort_by(|a, b| {
                let c = a.memory.cmp(&b.memory);
                if is_desc {
                    c.reverse()
                } else {
                    c
                }
            }),
        }
        self.processes.truncate(200);
    }
}

impl TableDelegate for ProcessTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }
    fn rows_count(&self, _cx: &App) -> usize {
        self.processes.len()
    }
    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(process) = self.processes.get(row_ix) else {
            return div().into_any_element();
        };
        match col_ix {
            0 => div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}", process.pid))
                .into_any_element(),
            1 => div()
                .text_sm()
                .text_color(cx.theme().foreground)
                .truncate()
                .child(process.name.clone())
                .into_any_element(),
            2 => div()
                .text_xs()
                .text_color(if process.cpu_usage > 50.0 {
                    cx.theme().red
                } else if process.cpu_usage > 20.0 {
                    cx.theme().yellow
                } else {
                    cx.theme().blue
                })
                .child(format!("{:.1}%", process.cpu_usage))
                .into_any_element(),
            3 => div()
                .text_xs()
                .text_color(cx.theme().blue)
                .child(format_bytes(process.memory))
                .into_any_element(),
            _ => div().into_any_element(),
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        self.sort_order = sort;
        self.sort_field = match col_ix {
            0 => ProcessSortField::Pid,
            1 => ProcessSortField::Name,
            2 => ProcessSortField::Cpu,
            3 => ProcessSortField::Memory,
            _ => ProcessSortField::Cpu,
        };
        self.sort_processes();
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
