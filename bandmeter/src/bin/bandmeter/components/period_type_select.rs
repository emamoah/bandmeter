use crate::period::*;
use gpui::{Entity, IntoElement, ParentElement, RenderOnce, Styled};
use gpui_component::{
    ActiveTheme, Sizable, h_flex,
    select::{Select, SelectState},
};

type PeriodTypeSelectState = Entity<SelectState<Vec<PeriodType>>>;

#[derive(IntoElement)]
pub struct PeriodTypeSelect {
    state: PeriodTypeSelectState,
}

impl PeriodTypeSelect {
    pub fn new(state: &PeriodTypeSelectState) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for PeriodTypeSelect {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        h_flex().child(
            Select::new(&self.state)
                .small()
                .text_xs()
                .bg(cx.theme().transparent)
                .shadow_none()
                .border_color(cx.theme().transparent),
        )
    }
}
