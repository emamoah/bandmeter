use super::*;
use crate::{AppStat, Stat, util::*};
use gpui::div;
use gpui_component::{
    ActiveTheme,
    table::{Column, TableDelegate},
};
use std::collections::HashMap;

pub struct ExeTable {
    data: Vec<AppStat>,
    columns: Vec<Column>,
    name_width: f32,
    max_val: u64,
}

impl ExeTable {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            name_width: KEY_WIDTH,
            max_val: 0,
            columns: vec![
                new_column("exe", "Program", KEY_WIDTH),
                new_column("total", "Total", TOTAL_WIDTH),
            ],
        }
    }

    pub fn filter_stats(&mut self, stats: &[Stat]) {
        let mut exe_map = HashMap::<Option<SharedString>, (u64, u64)>::with_capacity(64);

        for stat in stats {
            exe_map
                .entry(stat.exe.clone())
                .and_modify(|tuple| {
                    tuple.0 += stat.recv;
                    tuple.1 += stat.send;
                })
                .or_insert((stat.recv, stat.send));
        }

        let mut data: Vec<AppStat> = exe_map
            .drain()
            .map(|(exe, (download, upload))| AppStat {
                exe,
                download,
                upload,
            })
            .collect();
        data.sort_by(|a, b| (b.download + b.upload).cmp(&(a.download + a.upload)));

        self.data = data;
        self.max_val = self
            .data
            .first()
            .map(|stat| stat.download + stat.upload)
            .unwrap_or(0);
    }
}

impl TableDelegate for ExeTable {
    fn columns_count(&self, _cx: &gpui::App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &gpui::App) -> usize {
        self.data.len()
    }

    fn column(&self, col_ix: usize, _cx: &gpui::App) -> &Column {
        &self.columns[col_ix]
    }

    fn render_empty(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        div()
    }

    fn render_last_empty_col(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        div()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let col = self.column(col_ix, cx);
        match col.key.as_ref() {
            "total" => render_total_header(col.name.clone()),
            "exe" => render_key_header(col.name.clone()),
            _ => div(),
        }
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let row = &self.data[row_ix];
        let col = &self.columns[col_ix];
        let total = row.download + row.upload;

        match col.key.as_ref() {
            "total" => render_total(format_bytes(total)),
            "exe" => render_key_with_bar(
                self.name_width,
                row.exe
                    .as_ref()
                    .and_then(|s| s.split("/").last())
                    .map(String::from),
                self.max_val,
                row.download,
                row.upload,
                cx.theme(),
            ),
            _ => div(),
        }
    }
}
