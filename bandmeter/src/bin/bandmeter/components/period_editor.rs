use crate::period::*;
use chrono::{DateTime, Datelike, Local, TimeZone};
use gpui::{App, ClickEvent, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};
use gpui_component::{
    Disableable, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
};

#[derive(IntoElement)]
pub struct PeriodEditor {
    period: Period,
    on_prev: Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
    on_next: Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
}

impl PeriodEditor {
    pub fn new(
        period: Period,
        on_prev: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
        on_next: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            period,
            on_prev: Box::new(on_prev),
            on_next: Box::new(on_next),
        }
    }

    fn format_period(&self) -> String {
        match self.period {
            Period::Hour(_) => {
                let (start, end) = self.period.bounds();
                let local_start = Local.timestamp_opt(start, 0).unwrap();
                let local_end = Local.timestamp_opt(end, 0).unwrap();

                format!(
                    "{}{} - {}{}",
                    display_date_if_not_today(&local_start)
                        .map(|d| d + ", ")
                        .unwrap_or_default(),
                    local_start.format("%R"),
                    display_date_if_not_today(&local_end)
                        .map(|d| d + ", ")
                        .unwrap_or_default(),
                    local_end.format("%R"),
                )
            }
            Period::Day(d) => {
                let local = Local.timestamp_opt(d, 0).unwrap();
                display_date(&local)
            }
        }
    }
}

fn display_date(dt: &DateTime<Local>) -> String {
    let now = Local::now();

    let year_fmt = if dt.year() == now.year() {
        dt.format("")
    } else {
        dt.format(" %Y")
    };

    format!("{}{}", dt.format("%d %b"), year_fmt)
}

fn display_date_if_not_today(dt: &DateTime<Local>) -> Option<String> {
    let now = Local::now();
    let now_time = now.timestamp();
    let dt_time = dt.timestamp();

    if now_time - now_time % SECS_DAY == dt_time - dt_time % SECS_DAY {
        return None;
    }

    Some(display_date(dt))
}

impl RenderOnce for PeriodEditor {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let period_string = self.format_period();

        h_flex()
            .gap_x_1()
            .child(
                Button::new("period-prev")
                    .small()
                    .ghost()
                    .on_click(self.on_prev)
                    .icon(IconName::ChevronLeft),
            )
            .child(
                Button::new("period")
                    .min_w(px(116.))
                    .small()
                    .text_xs()
                    .ghost()
                    .child(div().text_xs().child(period_string)),
            )
            .child(
                Button::new("period-next")
                    .disabled(self.period.is_current())
                    .small()
                    .ghost()
                    .on_click(self.on_next)
                    .icon(IconName::ChevronRight),
            )
    }
}
