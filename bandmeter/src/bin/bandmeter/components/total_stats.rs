use crate::util::format_bytes;
use gpui::{Hsla, IntoElement, ParentElement, RenderOnce, Styled, TextAlign, div, px, rems};
use gpui_component::{
    ActiveTheme, Icon, IconName, h_flex,
    plot::{
        IntoPlot, Plot,
        shape::{Arc, Pie},
    },
    v_flex,
};
use std::f32::consts::FRAC_PI_2;

#[derive(IntoPlot)]
pub struct TotalArc {
    download: u64,
    upload: u64,
    radius: f32,
    thickness: f32,
}

impl TotalArc {
    fn is_empty(&self) -> bool {
        self.download == 0 && self.upload == 0
    }
}

fn color_scale(i: usize, cx: &mut gpui::App) -> Hsla {
    if i == 0 {
        cx.theme().chart_1
    } else {
        cx.theme().chart_2
    }
}

impl Plot for TotalArc {
    fn paint(
        &mut self,
        bounds: gpui::Bounds<gpui::Pixels>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        let data = if self.is_empty() {
            vec![1.]
        } else {
            vec![self.download as f32, self.upload as f32]
        };

        let pie = Pie::new()
            .start_angle(-FRAC_PI_2)
            .end_angle(FRAC_PI_2)
            .value(|d| Some(*d));
        let arcs = pie.arcs(&data);
        let arc_shape = Arc::new()
            .inner_radius(self.radius - self.thickness)
            .outer_radius(self.radius);

        for (i, arc_data) in arcs.iter().enumerate() {
            arc_shape.paint(
                &arc_data,
                if self.is_empty() {
                    cx.theme().muted_foreground.alpha(0.2)
                } else {
                    color_scale(i, cx)
                },
                None, // Override inner radius
                None, // Override outer radius
                &bounds,
                window,
            );
        }
    }
}

#[derive(IntoElement)]
pub struct TotalStats {
    pub download: u64,
    pub upload: u64,
}

impl TotalStats {}

const CELL_WIDTH: f32 = 120.;

impl RenderOnce for TotalStats {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        div().pt_8().pb_2().child(
            h_flex()
                .w_full()
                .items_end()
                .gap_x_3()
                .justify_center()
                .child(
                    h_flex()
                        .gap_x_2()
                        .mb(px(16.))
                        .child(text_w_title(
                            "Download".into(),
                            format_bytes(self.download),
                            TextAlign::Right,
                            cx,
                        ))
                        .child(Icon::new(IconName::ArrowDown).text_color(cx.theme().chart_1)),
                )
                .child(
                    div()
                        .relative()
                        .child(
                            div()
                                .absolute()
                                .left(px(CELL_WIDTH / 2.))
                                .bottom(rems(0.45))
                                .child(TotalArc {
                                    download: self.download,
                                    upload: self.upload,
                                    thickness: 8.,
                                    radius: 60.,
                                }),
                        )
                        .child(text_w_title(
                            "Total usage".into(),
                            format_bytes(self.download + self.upload),
                            TextAlign::Center,
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .gap_x_2()
                        .mb(px(16.))
                        .child(Icon::new(IconName::ArrowUp).text_color(cx.theme().chart_2))
                        .child(text_w_title(
                            "Upload".into(),
                            format_bytes(self.upload),
                            TextAlign::Left,
                            cx,
                        )),
                ),
        )
    }
}

fn text_w_title(
    title: String,
    text: String,
    align: TextAlign,
    cx: &mut gpui::App,
) -> impl IntoElement {
    let text_size = if let TextAlign::Center = align {
        rems(1.)
    } else {
        rems(0.8)
    };

    v_flex()
        .w(px(CELL_WIDTH))
        .text_align(align)
        .child(
            div()
                .text_size(rems(0.7))
                .text_color(cx.theme().muted_foreground)
                .line_height(rems(0.7))
                .child(title),
        )
        .child(div().text_size(text_size).child(text))
}
