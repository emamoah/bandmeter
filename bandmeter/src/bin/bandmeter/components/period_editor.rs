use chrono::{DateTime, Datelike, Local, NaiveDate};
use gpui::{
    App, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, RenderOnce, Styled,
    Window, div, px,
};
use gpui_component::{
    Disableable, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
};

use crate::period::*;

pub struct PeriodChangeEvent(pub Period);

pub struct PeriodEditorState {
    period: Period,
}

fn display_date(date: NaiveDate) -> String {
    let now = Local::now();

    let year_fmt = if date.year() == now.year() {
        date.format("")
    } else {
        date.format(" %Y")
    };

    format!("{}{}", date.format("%d %b"), year_fmt)
}

fn display_date_if_not_today(dt: &DateTime<Local>) -> Option<String> {
    let now = Local::now();
    let dt_date = dt.date_naive();
    let today = now.date_naive();

    if dt_date == today {
        return None;
    }

    Some(display_date(dt_date))
}

impl PeriodEditorState {
    pub fn new(period: Period, _: &mut Context<Self>) -> Self {
        Self { period }
    }

    fn format_period(&self) -> String {
        match self.period {
            Period::Hour(_) => {
                let (start, end) = self.period.bounds();

                format!(
                    "{}{} - {}{}",
                    display_date_if_not_today(&start)
                        .map(|d| d + ", ")
                        .unwrap_or_default(),
                    start.format("%R"),
                    display_date_if_not_today(&end)
                        .map(|d| d + ", ")
                        .unwrap_or_default(),
                    end.format("%R"),
                )
            }
            Period::Day(d) => display_date(d),
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
