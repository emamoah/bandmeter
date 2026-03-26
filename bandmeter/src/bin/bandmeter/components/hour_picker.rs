use chrono::{Local, NaiveDate, Timelike};
use gpui::{
    App, AppContext, Context, Div, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, SharedString,
    StyleRefinement, Styled, Subscription, Window, div, prelude::FluentBuilder, px, rems,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, PixelsExt, Selectable, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    calendar::{self, Calendar, CalendarState},
    h_flex, v_flex,
};

use crate::{period::NaiveDateExt, util::*};

const DIAL_CANVAS_SIZE_REMS: f32 = 14.;
const DIAL_RADIUS_1_REMS: f32 = 5.8;
const DIAL_RADIUS_2_REMS: f32 = 3.8;
const DIAL_BTN_SIZE_REMS: f32 = 2.;
const DIAL_BTN_TEXT_REMS: f32 = 0.8;

pub struct HourChangeEvent {
    pub date: NaiveDate,
    pub hour: u32,
}

pub struct HourPickerState {
    date: NaiveDate,
    hour: u32,
    calendar: Entity<CalendarState>,
    show_calendar: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

#[derive(IntoElement)]
pub struct HourPicker {
    state: Entity<HourPickerState>,
    style: StyleRefinement,
}

impl EventEmitter<HourChangeEvent> for HourPickerState {}

impl Focusable for HourPickerState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl HourPickerState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let now = Local::now();
        let calendar = cx.new(|cx| {
            let tomorrow = Local::now().date_naive().next();
            let mut date_picker = CalendarState::new(window, cx)
                .disabled_matcher(calendar::Matcher::range(Some(tomorrow), None));
            date_picker.set_date(now.date_naive(), window, cx);

            date_picker
        });

        let _subscriptions = vec![cx.subscribe_in(&calendar, window, |s, _, _, window, cx| {
            s.show_calendar = false;
            s.focus_handle.focus(window);
            cx.notify();
        })];

        Self {
            date: now.date_naive(),
            hour: now.hour(),
            calendar,
            show_calendar: false,
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }

    pub fn set(&mut self, date: NaiveDate, hour: u32, window: &mut Window, cx: &mut Context<Self>) {
        self.calendar.update(cx, |it, cx| {
            it.set_date(date, window, cx);
        });
        self.date = date;
        self.hour = hour;

        cx.notify();
    }
}

impl HourPicker {
    pub fn new(state: &Entity<HourPickerState>) -> Self {
        Self {
            state: state.clone(),
            style: StyleRefinement::default(),
        }
    }
}

fn get_hour_pos(dial_radius: f32, bearing: f32, cx: &mut App) -> (f32, f32) {
    let r_cos_t = dial_radius * bearing.cos();
    let r_sin_t = dial_radius * bearing.sin();

    let mid_canvas = rems_to_px(DIAL_CANVAS_SIZE_REMS / 2., cx).as_f32();

    let x = r_sin_t + mid_canvas;
    let y = -r_cos_t + mid_canvas;

    (x, y)
}

impl HourPickerState {
    fn render_dial_button(
        &self,
        hour: u32,
        position: (f32, f32),
        cx: &mut Context<Self>,
    ) -> Button {
        let id: SharedString = format!("hour-picker-{hour}").into();

        let half_size = rems_to_px(DIAL_BTN_SIZE_REMS / 2., cx);
        let pos_x = px(position.0) - half_size;
        let pos_y = px(position.1) - half_size;

        let text = if hour == 0 {
            "00".into()
        } else {
            hour.to_string()
        };

        Button::new(id)
            .absolute()
            .top(pos_y)
            .left(pos_x)
            .p_0()
            .size(rems(DIAL_BTN_SIZE_REMS))
            .rounded_full()
            .ghost()
            .child(
                div()
                    .line_height(px(0.))
                    .text_size(rems(DIAL_BTN_TEXT_REMS))
                    .child(text),
            )
    }

    fn render_dial(&self, cx: &mut Context<Self>) -> Div {
        let dial_radius_1: f32 = rems_to_px(DIAL_RADIUS_1_REMS, cx).into();
        let dial_radius_2: f32 = rems_to_px(DIAL_RADIUS_2_REMS, cx).into();
        let bearings = (0u32..360).step_by(30).map(|t| (t as f32).to_radians());

        let date_picker_date = self.calendar.read(cx).date().start();

        div()
            .size(rems(DIAL_CANVAS_SIZE_REMS))
            .rounded_full()
            .bg(cx.theme().secondary.alpha(0.3))
            .relative()
            .children(
                bearings
                    .clone()
                    .chain(bearings)
                    .enumerate()
                    .map(move |(hour, bearing)| {
                        let dial_hour = hour as u32;

                        let now = Local::now();
                        let current_date = now.date_naive();
                        let current_hour = now.hour();

                        let dial_radius = if dial_hour < 12 {
                            dial_radius_1
                        } else {
                            dial_radius_2
                        };

                        self.render_dial_button(
                            dial_hour,
                            get_hour_pos(dial_radius, bearing, cx),
                            cx,
                        )
                        .when(
                            date_picker_date.is_some_and(|pd| pd == self.date)
                                && dial_hour == self.hour,
                            |btn| {
                                btn.selected(true)
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                            },
                        )
                        .disabled(
                            date_picker_date.is_some_and(|date| date == current_date)
                                && dial_hour > current_hour,
                        )
                        .on_click(cx.listener(move |s, _, w, cx| {
                            if let Some(date) = date_picker_date {
                                s.set(date, dial_hour, w, cx);
                                cx.emit(HourChangeEvent {
                                    date,
                                    hour: dial_hour,
                                });
                            }
                        }))
                    }),
            )
    }
}

impl Render for HourPickerState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .track_focus(&self.focus_handle)
            .items_center()
            .justify_center()
            .relative()
            .child(
                v_flex().w_full().justify_start().child(
                    Button::new("hour-picker-cal")
                        .absolute()
                        .small()
                        .ghost()
                        .icon(IconName::Calendar)
                        .when(self.show_calendar, |this| this.invisible())
                        .on_click(cx.listener(|s, _, _, cx| {
                            s.show_calendar = !s.show_calendar;
                            cx.notify();
                        })),
                ),
            )
            .when(self.show_calendar, |this| {
                this.child(
                    Calendar::new(&self.calendar)
                        .absolute()
                        .small()
                        .border_0()
                        .p_0(),
                )
            })
            .child(
                self.render_dial(cx)
                    .when(self.show_calendar, |this| this.invisible()),
            )
    }
}

impl Styled for HourPicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for HourPicker {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .p_2()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_lg)
            .refine_style(&self.style)
            .child(self.state)
    }
}
