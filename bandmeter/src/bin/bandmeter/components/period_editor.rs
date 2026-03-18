use crate::period::*;
use chrono::{DateTime, Datelike, Local, TimeZone};
use gpui::{
    App, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, RenderOnce, Styled,
    Window, div, px,
};
use gpui_component::{
    Disableable, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
};

pub struct PeriodChangeEvent(pub Period);

pub struct PeriodEditorState {
    period: Period,
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

impl PeriodEditorState {
    pub fn new(period: Period, _: &mut Context<Self>) -> Self {
        Self { period }
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

    pub fn switch_period_type(&mut self, period_type: &PeriodType, cx: &mut Context<Self>) {
        self.period.switch(period_type);
        cx.emit(PeriodChangeEvent(self.period));
        cx.notify();
    }

    pub fn prev(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.period.prev() {
            cx.emit(PeriodChangeEvent(self.period));
            cx.notify();
        }
    }

    pub fn next(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.period.next() {
            cx.emit(PeriodChangeEvent(self.period));
            cx.notify();
        }
    }
}

impl EventEmitter<PeriodChangeEvent> for PeriodEditorState {}

impl Render for PeriodEditorState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let period_string = self.format_period();

        h_flex()
            .gap_x_1()
            .child(
                Button::new("period-prev")
                    .small()
                    .ghost()
                    .on_click(cx.listener(|s, _, w, cx| s.prev(w, cx)))
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
                    .on_click(cx.listener(|s, _, w, cx| s.next(w, cx)))
                    .icon(IconName::ChevronRight),
            )
    }
}

#[derive(IntoElement)]
pub struct PeriodEditor {
    state: Entity<PeriodEditorState>,
}

impl PeriodEditor {
    pub fn new(state: &Entity<PeriodEditorState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for PeriodEditor {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().child(self.state)
    }
}
