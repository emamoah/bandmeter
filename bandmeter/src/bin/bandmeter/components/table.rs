pub mod exe_table;

pub use exe_table::*;

use gpui::{Div, IntoElement, ParentElement, SharedString, Styled, div, px};
use gpui_component::{PixelsExt, Size, StyledExt, Theme, table::Column};
use std::sync::LazyLock;

const KEY_WIDTH: f32 = 420.;
const TOTAL_WIDTH: f32 = 80.;
const CELL_P_X: LazyLock<f32> = LazyLock::new(|| {
    let cell_pl = Size::Small.table_cell_padding().left.as_f32();
    let cell_pr = Size::Small.table_cell_padding().right.as_f32();
    cell_pl + cell_pr
});

fn new_column(key: impl Into<SharedString>, name: impl Into<SharedString>, width: f32) -> Column {
    Column::new(key, name)
        .width(width)
        .resizable(false)
        .movable(false)
}

fn render_key_header(name: impl IntoElement) -> Div {
    div().size_full().text_xs().child(name)
}

fn render_total_header(name: impl IntoElement) -> Div {
    div().size_full().text_xs().text_right().child(name)
}

fn render_key_with_bar(
    key_width: f32,
    key: Option<String>,
    max_val: u64,
    download: u64,
    upload: u64,
    theme: &Theme,
) -> Div {
    let max_bar_width = key_width - *CELL_P_X;

    let download_bar_width = (download as f64 / max_val as f64) as f32 * max_bar_width;
    let upload_bar_width = (upload as f64 / max_val as f64) as f32 * max_bar_width;

    let (key, key_color) = if let Some(key) = key {
        (key, theme.foreground)
    } else {
        ("unknown".into(), theme.muted_foreground)
    };

    div().text_xs().size_full().flex().items_center().child(
        div() // Key & Bar
            .pb_1()
            .w_full()
            .v_flex()
            .gap_y_0p5()
            .child(div().text_color(key_color).child(key))
            .child(
                div() // Bar
                    .h_0p5()
                    .flex()
                    .child(
                        div() // Download
                            .flex_shrink_0()
                            .h_full()
                            .min_w(if download > 0 { px(2.) } else { px(0.) })
                            .w(px(download_bar_width))
                            .bg(theme.chart_1),
                    )
                    .child(
                        div() // Upload
                            .flex_shrink_0()
                            .h_full()
                            .min_w(if upload > 0 { px(2.) } else { px(0.) })
                            .w(px(upload_bar_width))
                            .bg(theme.chart_2),
                    ),
            ),
    )
}

fn render_total(total: impl IntoElement) -> Div {
    div()
        .grid()
        .size_full()
        .items_center()
        .text_xs()
        .text_right()
        .child(total)
}
