mod components;
mod db;
mod period;
mod util;

use chrono::{Local, TimeZone};
use gpui::*;
use gpui_component::{
    button::*,
    select::{SelectEvent, SelectState},
    table::{Table, TableState},
    *,
};

use crate::{components::*, db::*, period::*};

actions!([Prev, Next]);
const KEY_CX_PERIOD: &str = "period";

#[derive(Clone, Default)]
pub struct TimeStat {
    pub timestamp: i64,
    pub download: u64,
    pub upload: u64,
}

#[derive(Clone)]
pub struct AppStat {
    pub exe: Option<SharedString>,
    pub download: u64,
    pub upload: u64,
}

#[derive(Debug)]
pub struct Stat {
    pub timestamp: i64,
    pub exe: Option<SharedString>,
    pub raddr: SharedString,
    pub send: u64,
    pub recv: u64,
}

#[derive(Default)]
struct Stats {
    raw: Vec<Stat>,
    by_time: Vec<TimeStat>,
    total_download: u64,
    total_upload: u64,
}

pub struct Bandmeter {
    period_type_select: Entity<SelectState<Vec<PeriodType>>>,
    period_editor: Entity<PeriodEditorState>,
    db_manager: DBManager,
    stats: Stats,
    exe_table: Entity<TableState<ExeTable>>,
    focus_handle: FocusHandle,
}

impl Bandmeter {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let period_type_select = cx.new(|cx| {
            SelectState::new(
                vec![PeriodType::Hour, PeriodType::Day],
                Some(IndexPath::new(1)),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &period_type_select,
            window,
            Self::on_period_type_select_change,
        )
        .detach();

        let period_editor = cx.new(|cx| PeriodEditorState::new(Period::default(), cx));
        cx.subscribe_in(&period_editor, window, Self::on_period_change)
            .detach();

        let exe_table = cx.new(|cx| {
            TableState::new(ExeTable::new(), window, cx)
                .col_selectable(false)
                .row_selectable(false)
        });

        let mut bandmeter = Self {
            period_type_select,
            exe_table,
            period_editor,
            db_manager: DBManager::new(),
            stats: Stats::default(),
            focus_handle: cx.focus_handle(),
        };
        bandmeter.query_stats(&Period::default(), cx);

        bandmeter
    }

    fn query_stats(&mut self, period: &Period, cx: &mut Context<Self>) {
        self.stats.total_download = 0;
        self.stats.total_upload = 0;

        self.stats.raw = self.db_manager.query_raw(period);

        let (period_start, period_end) = period.bounds_timestamp();
        let intvl = period.intvl_secs();

        let mut stats_iter = self.stats.raw.iter().peekable();

        let by_time = (period_start..period_end)
            .step_by(intvl as usize)
            .map(|timestamp| {
                let mut time_stat = TimeStat {
                    timestamp,
                    ..Default::default()
                };

                while let Some(next) = stats_iter.peek()
                    && next.timestamp - timestamp < intvl
                {
                    let next = stats_iter.next().unwrap();
                    time_stat.download += next.recv;
                    time_stat.upload += next.send;
                }

                self.stats.total_download += time_stat.download;
                self.stats.total_upload += time_stat.upload;

                time_stat
            })
            .collect::<Vec<_>>();

        self.stats.by_time = by_time;
        self.exe_table.update(cx, |table, cx| {
            table.delegate_mut().filter_stats(&self.stats.raw);
            cx.notify();
        });
    }

    fn on_period_change(
        &mut self,
        _: &Entity<PeriodEditorState>,
        event: &PeriodChangeEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let period = &event.0;
        self.query_stats(period, cx);
        cx.notify();
    }

    fn on_period_type_select_change(
        &mut self,
        _: &Entity<SelectState<Vec<PeriodType>>>,
        event: &SelectEvent<Vec<PeriodType>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(Some(period_type)) => {
                self.period_editor.update(cx, |it, cx| {
                    it.switch_period_type(period_type, cx);
                });
            }
            _ => {}
        }
    }

    fn tick_fmt(&self) -> impl Fn(&TimeStat) -> String + 'static {
        |stat| {
            format!(
                "{}",
                Local.timestamp_opt(stat.timestamp, 0).unwrap().format("%R")
            )
        }
    }
}

impl Focusable for Bandmeter {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Bandmeter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(KEY_CX_PERIOD)
            .on_action(
                cx.listener(|s, _: &Prev, w, cx| {
                    s.period_editor.update(cx, |it, cx| it.prev(w, cx))
                }),
            )
            .on_action(
                cx.listener(|s, _: &Next, w, cx| {
                    s.period_editor.update(cx, |it, cx| it.next(w, cx))
                }),
            )
            .size_full()
            .child(
                h_flex() // Top bar
                    .p(px(6.))
                    .gap_x_2()
                    .justify_between()
                    .text_xs()
                    .child(
                        div()
                            .w_20()
                            .flex_shrink_0()
                            .child(PeriodTypeSelect::new(&self.period_type_select)),
                    )
                    .child(PeriodEditor::new(&self.period_editor))
                    .child(
                        h_flex().justify_end().w_20().child(
                            Button::new("settings")
                                .small()
                                .ghost()
                                .icon(IconName::Settings2),
                        ),
                    ),
            )
            .child(div().h_1()) // Reserved for new stuff
            .child(
                div() // Chart
                    .relative()
                    .h(px(130.))
                    .flex_shrink_0()
                    .child({
                        BandwidthChart {
                            // Consider a custom Entity to persist data
                            data: self.stats.by_time.clone(),
                            tick_margin: 3,
                            tick_fmt: Box::new(self.tick_fmt()),
                        }
                    }),
            )
            .child(TotalStats {
                download: self.stats.total_download,
                upload: self.stats.total_upload,
            })
            .child(
                Table::new(&self.exe_table)
                    .small()
                    .stripe(true)
                    .bordered(false)
                    .scrollbar_visible(true, false),
            )
            .track_focus(&self.focus_handle(cx))
    }
}

fn prepare_theme(window: &mut Window, cx: &mut App) {
    Theme::change(ThemeMode::Dark, Some(window), cx);

    let theme = Theme::global_mut(cx);
    theme.table_row_border = theme.transparent;
    theme.chart_1 = Hsla::from(rgb(0x3DAEE9));
    theme.chart_2 = Hsla::from(rgb(0x58E93D));
    theme.table_active_border = theme.chart_1;
    theme.table_active = theme.chart_1.alpha(0.1);
    theme.table_hover = theme.table_active;
}

fn main() -> anyhow::Result<()> {
    // For now, just error out when `DB_DIR` isn't set
    db_dir()?;

    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.)), cx);
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);
        cx.bind_keys([
            KeyBinding::new("left", Prev, Some(KEY_CX_PERIOD)),
            KeyBinding::new("right", Next, Some(KEY_CX_PERIOD)),
        ]);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..WindowOptions::default()
                },
                |window, cx| {
                    prepare_theme(window, cx);
                    window.set_window_title("Bandmeter");
                    let view = cx.new(|cx| Bandmeter::new(window, cx));

                    let focus_handle = view.focus_handle(cx);
                    window.defer(cx, move |window, _| {
                        focus_handle.focus(window);
                    });
                    // This first level on the window, should be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            anyhow::Ok(())
        })
        .detach();
    });

    Ok(())
}
