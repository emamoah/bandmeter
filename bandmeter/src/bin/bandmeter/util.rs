use gpui::{App, Pixels};
use gpui_component::ActiveTheme;

pub fn rems_to_px(rems: f32, cx: &mut App) -> Pixels {
    gpui::rems(rems).to_pixels(cx.theme().font_size)
}

pub fn format_bytes(num: u64) -> String {
    const KB: u64 = 1000;
    const MB: u64 = KB * 1000;
    const GB: u64 = MB * 1000;
    const TB: u64 = GB * 1000;
    const PB: u64 = TB * 1000;

    if num >= PB {
        format!("{:.2} PB", num as f64 / PB as f64)
    } else if num >= TB {
        format!("{:.2} TB", num as f64 / TB as f64)
    } else if num >= GB {
        format!("{:.2} GB", num as f64 / GB as f64)
    } else if num >= MB {
        format!("{:.2} MB", num as f64 / MB as f64)
    } else if num >= KB {
        format!("{:.2} KB", num as f64 / KB as f64)
    } else {
        format!("{} B", num)
    }
}
