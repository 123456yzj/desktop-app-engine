//! Native chart painting and pointer interaction over the pure chart algorithms.
use super::*;

pub(super) struct SeriesContext<'a> {
    pub painter: &'a egui::Painter,
    pub points: &'a [Option<Point>],
    pub sampled: &'a [usize],
    pub transform: Transform,
    pub color: Color32,
    pub pointer: Option<Pos2>,
    pub enabled: bool,
    pub monotonic: bool,
    pub visible_series: usize,
    pub group: usize,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_chart(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    n: &protocol::Node,
    rect: egui::Rect,
    enabled: bool,
    state: &crate::binding::State,
    version: u64,
    c: &mut ChartState,
    include_zero: bool,
    paint_series: fn(SeriesContext<'_>) -> Option<usize>,
) {
    let series = n.series.as_ref().unwrap();
    if c.version != version {
        c.points = series
            .iter()
            .map(|s| chart::series_points(s, state).unwrap())
            .collect();
        c.version = version;
        c.sample_width = 0;
    }
    let legend = n.legend.unwrap_or(true);
    let plot = plot_rect(rect, legend);
    if plot.width() <= 0.0 || plot.height() <= 0.0 {
        return;
    }
    let mut x = chart::axis_range(
        c.points
            .iter()
            .enumerate()
            .filter(|(i, _)| !c.hidden.contains(i))
            .flat_map(|(_, s)| s.iter().flatten().map(|p| p.x)),
        n.x_axis.as_ref(),
        false,
    );
    if n.kind == Kind::BarChart {
        let spacing = c
            .points
            .iter()
            .enumerate()
            .filter(|(i, _)| !c.hidden.contains(i))
            .map(|(_, points)| chart::bar_spacing(points))
            .fold(f64::INFINITY, f64::min);
        let padding = if spacing.is_finite() {
            spacing * 0.5
        } else {
            0.5
        };
        if n.x_axis.as_ref().and_then(|a| a.min).is_none() {
            x.min -= padding;
        }
        if n.x_axis.as_ref().and_then(|a| a.max).is_none() {
            x.max += padding;
        }
    }
    let y = chart::axis_range(
        c.points
            .iter()
            .enumerate()
            .filter(|(i, _)| !c.hidden.contains(i))
            .flat_map(|(_, s)| s.iter().flatten().map(|p| p.y)),
        n.y_axis.as_ref(),
        include_zero,
    );
    if !c.manual {
        c.viewport = Some((x, y));
    }
    let response = ui.interact(
        plot.intersect(ui.clip_rect()),
        egui::Id::new((&n.id, "chart-input")),
        if enabled {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        },
    );
    if response.double_clicked()
        || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Home))
    {
        c.manual = false;
        c.viewport = Some((x, y));
    }
    if n.pan == Some(true)
        && response.dragged()
        && let Some((x, y)) = c.viewport
    {
        let delta = ui.input(|i| i.pointer.delta());
        c.viewport = Some((
            chart::pan_range(x, -delta.x as f64 / plot.width() as f64 * (x.max - x.min)),
            chart::pan_range(y, delta.y as f64 / plot.height() as f64 * (y.max - y.min)),
        ));
        c.manual = true;
        ui.ctx().request_repaint();
    }
    let (x, y) = c.viewport.unwrap_or((x, y));
    let transform = Transform {
        x,
        y,
        left: plot.left() as f64,
        top: plot.top() as f64,
        width: plot.width() as f64,
        height: plot.height() as f64,
    };
    let range_key = (
        x.min.to_bits(),
        x.max.to_bits(),
        y.min.to_bits(),
        y.max.to_bits(),
    );
    if c.sample_width != plot.width() as usize || c.sample_range != Some(range_key) {
        c.sampled = c
            .points
            .iter()
            .zip(series)
            .map(|(p, s)| {
                if s.monotonic_x {
                    chart::downsample(p, x, plot.width() as usize)
                } else {
                    (0..p.len()).collect()
                }
            })
            .collect();
        c.sample_width = plot.width() as usize;
        c.sample_range = Some(range_key);
    }
    let font = n.style.font_size.unwrap_or(14.0);
    let color = token(ui, n.style.color.unwrap_or(Token::TextMuted));
    let border = Stroke::new(1.0_f32, token(ui, Token::Border));
    for tick in chart::ticks(x, plot.width() as f64) {
        let pos = transform.screen(Point { x: tick, y: y.min });
        p.line_segment(
            [
                Pos2::new(pos.x as f32, plot.top()),
                Pos2::new(pos.x as f32, plot.bottom()),
            ],
            border,
        );
        label(
            ui,
            p,
            egui::Rect::from_min_size(
                Pos2::new(pos.x as f32 - 22.0, plot.bottom() + 5.0),
                Vec2::new(70.0, 20.0),
            ),
            &chart::tick_text(tick),
            font * 0.857,
            color,
        );
    }
    for tick in chart::ticks(y, plot.height() as f64) {
        let pos = transform.screen(Point { x: x.min, y: tick });
        p.line_segment(
            [
                Pos2::new(plot.left(), pos.y as f32),
                Pos2::new(plot.right(), pos.y as f32),
            ],
            border,
        );
        label(
            ui,
            p,
            egui::Rect::from_min_size(
                Pos2::new(rect.left(), pos.y as f32 - 7.0),
                Vec2::new(60.0, 20.0),
            ),
            &chart::tick_text(tick),
            font * 0.857,
            color,
        );
    }
    let pp = p.with_clip_rect(plot.intersect(p.clip_rect()));
    let mut hover = None;
    let pointer = ui
        .input(|i| i.pointer.hover_pos())
        .filter(|p| plot.contains(*p) && ui.clip_rect().contains(*p));
    let visible_series = series.len().saturating_sub(c.hidden.len()).max(1);
    let mut group = 0;
    for (i, s) in series.iter().enumerate() {
        if c.hidden.contains(&i) {
            continue;
        }
        if let Some(index) = paint_series(SeriesContext {
            painter: &pp,
            points: &c.points[i],
            sampled: &c.sampled[i],
            transform,
            color: series_color(ui, i),
            pointer,
            enabled,
            monotonic: s.monotonic_x,
            visible_series,
            group,
        }) {
            hover = Some((i, index));
        }
        group += 1;
    }
    let chart_response = response;
    chart_response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Label,
            enabled,
            format!(
                "图表 {}，系列 {}，{} 个数据点",
                n.id,
                series
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("、"),
                c.points.iter().map(Vec::len).sum::<usize>()
            ),
        )
    });
    if let Some((i, j)) = hover {
        let point = c.points[i][j].unwrap();
        pp.circle_stroke(
            point_pos(transform.screen(point)),
            5.0,
            Stroke::new(2.0_f32, series_color(ui, i)),
        );
        let text = format!("{}\nX: {}\nY: {}", series[i].name, point.x, point.y);
        let tooltip = egui::Rect::from_min_size(
            plot.right_top() + Vec2::new(-150.0, 8.0),
            Vec2::new(142.0, 68.0),
        );
        p.rect_filled(tooltip, 4.0, token(ui, Token::SurfaceRaised));
        label(
            ui,
            p,
            tooltip.shrink(6.0),
            &text,
            font,
            token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
        );
        chart_response.on_hover_text(text);
    }
    if c.points.iter().all(|p| p.iter().all(Option::is_none)) {
        label(ui, p, plot.shrink(12.0), "暂无数据", 16.0, color);
    }
    if legend {
        let mut x = plot.left();
        for (i, s) in series.iter().enumerate() {
            let width = p
                .layout_no_wrap(s.name.clone(), FontId::proportional(font), color)
                .size()
                .x
                + 34.0;
            let r = egui::Rect::from_min_size(
                Pos2::new(x, rect.bottom() - 22.0),
                Vec2::new(width, 22.0),
            );
            p.circle_filled(
                r.left_center() + Vec2::new(6.0, 0.0),
                4.0,
                if c.hidden.contains(&i) {
                    color
                } else {
                    series_color(ui, i)
                },
            );
            label(
                ui,
                p,
                egui::Rect::from_min_max(r.min + Vec2::new(16.0, 0.0), r.max),
                &s.name,
                font,
                color,
            );
            let response = ui.interact(
                r,
                egui::Id::new((&n.id, &s.id, "legend")),
                if enabled {
                    egui::Sense::click()
                } else {
                    egui::Sense::hover()
                },
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    enabled,
                    !c.hidden.contains(&i),
                    &s.name,
                )
            });
            if response.clicked() {
                if !c.hidden.remove(&i) {
                    c.hidden.insert(i);
                }
                c.sample_width = 0;
                ui.ctx().request_repaint();
            }
            x += width;
        }
    }
}
