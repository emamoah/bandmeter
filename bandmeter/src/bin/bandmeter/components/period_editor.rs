use chrono::{DateTime, Datelike, Local, NaiveDate};
use gpui::{
    App, AppContext, Bounds, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, RenderOnce, Styled,
    Subscription, Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, Sizable,
    button::{Button, ButtonVariants},
    calendar::{self, Calendar, CalendarEvent, CalendarState, Date},
    h_flex,
    popover::Popover,
};

use crate::period::*;

pub struct PeriodChangeEvent(pub Period);

pub struct PeriodEditorState {
    period: Period,
    calendar: Entity<CalendarState>,
    open: bool,
    trigger_bounds: Entity<Bounds<Pixels>>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
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
    pub fn new(period: Period, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let calendar = cx.new(|cx| {
            let date = period.bounds().0.date_naive();
            let tomorrow = Local::now().date_naive().next();
            let mut cal = CalendarState::new(window, cx)
                .disabled_matcher(calendar::Matcher::range(Some(tomorrow), None));
            cal.set_date(date, window, cx);
            cal
        });

        let _subscriptions = vec![cx.subscribe_in(&calendar, window, Self::on_day_change)];

        Self {
            period,
            calendar,
            open: false,
            trigger_bounds: cx.new(|_| Bounds::default()),
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }

    fn format_period(&self) -> String {
        match self.period {
            Period::Hour(_) => {
                let TimeBounds(start, end) = self.period.bounds();

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
            Period::Week(w) => {
                let (start, end) = (w, w.add_days(6));

                format!("{} - {}", display_date(start), display_date(end))
            }
            Period::Month(m) => {
                let this_year = Local::now().year();

                let fmt = if m.year() == this_year {
                    m.format("%B")
                } else {
                    m.format("%B %Y")
                };

                format!("{fmt}")
            }
        }
    }

    fn update_period(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        update: impl Fn(&mut Period) -> bool,
    ) {
        if update(&mut self.period) {
            if let Period::Day(date) = self.period {
                self.calendar.update(cx, |it, cx| {
                    it.set_date(date, window, cx);
                })
            }

            cx.emit(PeriodChangeEvent(self.period));
            cx.notify();
        }
    }

    fn on_day_change(
        &mut self,
        _: &Entity<CalendarState>,
        ev: &CalendarEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match ev {
            CalendarEvent::Selected(Date::Single(Some(date))) => {
                self.update_period(window, cx, |p| {
                    *p = Period::Day(*date);
                    true
                });

                self.open = false;
                self.focus_handle.focus(window);
            }
            _ => {}
        }
    }

    pub fn switch_period_type(
        &mut self,
        period_type: &PeriodType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_period(window, cx, |p| {
            p.switch(period_type);
            true
        });

        self.open = false;
    }

    pub fn prev(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_period(window, cx, |p| p.prev());
    }

    pub fn next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_period(window, cx, |p| p.next());
    }

    fn toggle_open(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        self.focus_handle.focus(window);
        cx.notify();
    }
}

impl Focusable for PeriodEditorState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
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
                div()
                    .relative()
                    .child(
                        // Mad hack to get a centered popover. Better API in next version, I hope.
                        canvas(
                            {
                                let trigger_bounds = self.trigger_bounds.clone();
                                move |bounds, _, cx| trigger_bounds.update(cx, |it, _| *it = bounds)
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        Popover::new("period-popover")
                            .open(self.open)
                            .w(self.trigger_bounds.read(cx).size.width)
                            .appearance(false)
                            .mt_px()
                            .trigger(
                                Button::new("period")
                                    .min_w(px(116.))
                                    .flex()
                                    .small()
                                    .text_xs()
                                    .ghost()
                                    .on_click(cx.listener(Self::toggle_open))
                                    .child(div().text_xs().child(period_string)),
                            )
                            .child(div().flex().justify_center().when(
                                matches!(self.period, Period::Day(_)),
                                |this| {
                                    this.child(
                                        Calendar::new(&self.calendar)
                                            .small()
                                            .bg(cx.theme().popover),
                                    )
                                },
                            )),
                    ),
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
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .track_focus(&self.state.focus_handle(cx))
            .child(self.state)
    }
}
