use chrono::Datelike;
use gpui::{
    App, Bounds, FontWeight, PathBuilder, Pixels, SharedString, TextAlign, TextRun, Window, point,
    px, rems,
};
use gpui_component::{
    ActiveTheme, PixelsExt,
    plot::{
        AXIS_GAP, AxisText, IntoPlot, Plot, PlotAxis,
        scale::{Scale, ScaleBand, ScaleLinear},
        shape::Bar,
    },
};

use crate::{util::*, *};

#[derive(IntoPlot)]
pub struct BandwidthChart {
    pub data: Vec<TimeStat>,
    pub period_type: PeriodType,
}

impl BandwidthChart {
    fn tick_margin(&self) -> usize {
        match self.period_type {
            PeriodType::Hour | PeriodType::Day => 3,
            PeriodType::Week | PeriodType::Month => 1,
        }
    }

    fn tick_fmt(&self) -> impl Fn(&TimeStat) -> String {
        |stat| {
            let fmt = match self.period_type {
                PeriodType::Hour | PeriodType::Day => {
                    Local.timestamp_opt(stat.timestamp, 0).unwrap().format("%R")
                }
                PeriodType::Week => {
                    let date = Local.timestamp_opt(stat.timestamp, 0).unwrap().date_naive();

                    if date.day() == 1 {
                        date.format("%a %d %b")
                    } else {
                        date.format("%a %d")
                    }
                }
                PeriodType::Month => Local
                    .timestamp_opt(stat.timestamp, 0)
                    .unwrap()
                    .format("%-e"),
            };

            format!("{}", fmt)
        }
    }
}

impl Plot for BandwidthChart {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let tick_margin = self.tick_margin();
        let tick_fmt = self.tick_fmt();

        if self.data.is_empty() {
            return;
        }

        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32() - AXIS_GAP;

        let x_scale = ScaleBand::new(
            self.data.iter().map(|stat| stat.timestamp).collect(),
            vec![0., width],
        )
        .padding_inner(0.35)
        .padding_outer(0.4);

        let band_width = x_scale.band_width(); // No pun intended

        let y_max = self
            .data
            .iter()
            .map(|stat| u64::max(stat.download, stat.upload))
            .max()
            .unwrap_or(0);

        let y_top = paint_max_indicator(format_bytes(y_max).into(), bounds, window, cx);
        let y_bottom = height - 4.;
        let y_min = y_bottom - 1.;

        let y_scale = ScaleLinear::new(vec![y_max as f64, 0.], vec![y_bottom, y_top.as_f32()]);

        let first_tick = self
            .data
            .iter()
            .next()
            .and_then(|d| x_scale.tick(&d.timestamp))
            .unwrap_or(0.);
        let last_tick = self
            .data
            .iter()
            .last()
            .and_then(|d| x_scale.tick(&d.timestamp))
            .unwrap_or(0.);
        // Next two are NaN if data.len() == 1
        let interval = (last_tick - first_tick) / (self.data.len() as f32 - 1.);
        let halfway_between_bands = (interval - band_width) / 2.;

        // Draw X axis
        let x_label = self.data.iter().enumerate().filter_map(|(i, d)| {
            if i > 0 && i % tick_margin == 0 {
                x_scale.tick(&d.timestamp).map(|x_tick| {
                    AxisText::new(
                        tick_fmt(&d),
                        x_tick - halfway_between_bands,
                        cx.theme().muted_foreground,
                    )
                    .align(TextAlign::Center)
                })
            } else {
                None
            }
        });

        PlotAxis::new()
            .x(height)
            .x_label(x_label)
            .paint(&bounds, window, cx);

        let new_band_width = band_width * 0.6;

        // Download bars
        let x_scale1 = x_scale.clone();
        let y_scale1 = y_scale.clone();
        let dl_fill = cx.theme().chart_1;
        let up_fill = cx.theme().chart_2;

        let dlbar = Bar::new()
            .data(&self.data)
            .band_width(new_band_width)
            .x(move |d| x_scale1.tick(&d.timestamp))
            .y0(move |_| y_bottom)
            .y1(move |d| get_y1(&y_scale1, y_min, d.download as f64))
            .fill(move |_| dl_fill);

        // Upload bars
        let upbar = Bar::new()
            .data(&self.data)
            .band_width(new_band_width)
            .x(move |d| {
                x_scale
                    .tick(&d.timestamp)
                    .map(|t| t + (band_width - new_band_width))
            })
            .y0(move |_| y_bottom)
            .y1(move |d| get_y1(&y_scale, y_min, d.upload as f64))
            .fill(move |_| up_fill);

        dlbar.paint(&bounds, window, cx);
        upbar.paint(&bounds, window, cx);

        // Dots

        if self.data.len() < 2 {
            return;
        }

        let dot_width = 2.;
        let half_dot_width = 1.;

        let dots_x_start = first_tick - halfway_between_bands - half_dot_width;
        let dots_x_end = last_tick + band_width + halfway_between_bands + half_dot_width;
        let dots_y = height;

        let mut dots = PathBuilder::stroke(px(dot_width))
            .dash_array(&[px(dot_width), px(interval - dot_width)]);
        dots.move_to(bounds.origin + point(px(dots_x_start), px(dots_y)));
        dots.line_to(bounds.origin + point(px(dots_x_end), px(dots_y)));

        if let Ok(dotted_line) = dots.build() {
            window.paint_path(dotted_line, cx.theme().muted_foreground);
        }

        // Tick dots
        let mut tick_dots = PathBuilder::stroke(px(dot_width * 2.)).dash_array(&[
            px(dot_width),
            px((interval - dot_width) + interval * (tick_margin - 1) as f32),
        ]);
        let dots_x_start = dots_x_start + (interval * tick_margin as f32);
        let dots_x_end = dots_x_end - dot_width;

        tick_dots.move_to(bounds.origin + point(px(dots_x_start), px(dots_y + half_dot_width)));
        tick_dots.line_to(bounds.origin + point(px(dots_x_end), px(dots_y + half_dot_width)));

        if let Ok(dotted_line) = tick_dots.build() {
            window.paint_path(dotted_line, cx.theme().muted_foreground);
        }
    }
}

fn get_y1(y_scale: &ScaleLinear<f64>, y_min: f32, value: f64) -> Option<f32> {
    if value == 0. {
        y_scale.tick(&0.)
    } else {
        y_scale.tick(&value).map(|tick| tick.min(y_min))
    }
}

/// Prints max indicator and returns underline Y location
fn paint_max_indicator(
    text: SharedString,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) -> Pixels {
    let font_weight = FontWeight(400.);
    let text_run = TextRun {
        len: text.len(),
        font: window.text_style().highlight(font_weight).font(),
        color: cx.theme().muted_foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };

    let font_size = rems(0.625).to_pixels(cx.theme().font_size);

    let pad_left = px(6.);
    let pad_top = px(5.);
    let text_line = window
        .text_system()
        .shape_line(text.clone(), font_size, &[text_run], None);
    let line_height = rems(1.2).to_pixels(font_size);
    let start_x = bounds.origin.x + pad_left;
    let start_y = pad_top + bounds.origin.y;
    let origin = point(start_x, start_y);

    let mut builder = PathBuilder::stroke(px(1.));
    let line_y = pad_top + line_height + px(2.);
    builder.move_to(bounds.origin + point(px(0.), line_y));
    builder.line_to(bounds.origin + point(pad_left + text_line.width, line_y));

    if let Ok(line) = builder.build() {
        window.paint_path(line, cx.theme().border);
    }

    let _ = text_line.paint(origin, line_height, window, cx);

    line_y
}
