use super::*;
component!(&[], &[]);
#[allow(clippy::too_many_arguments)]
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    n: &protocol::Node,
    rect: egui::Rect,
    enabled: bool,
    state: &crate::binding::State,
    version: u64,
    c: &mut ChartState,
) {
    charts::paint_chart(
        ui,
        p,
        n,
        rect,
        enabled,
        state,
        version,
        c,
        true,
        paint_series,
    );
}
fn paint_series(c: charts::SeriesContext<'_>) -> Option<usize> {
    let x = c.transform.x;
    let spacing = chart::bar_spacing(c.points).min(x.max - x.min);
    let width =
        (spacing / (x.max - x.min) * c.transform.width * 0.75).min(100.) / c.visible_series as f64;
    let mut hover = None;
    for (index, point) in c.points.iter().enumerate() {
        if let Some(point) = point {
            let screen = c.transform.screen(*point);
            let zero = c.transform.screen(Point { x: point.x, y: 0. });
            let left = screen.x - width * c.visible_series as f64 / 2. + width * c.group as f64;
            let bar = egui::Rect::from_min_max(
                point_pos(Point {
                    x: left,
                    y: screen.y.min(zero.y),
                }),
                point_pos(Point {
                    x: left + width - 1.,
                    y: screen.y.max(zero.y),
                }),
            );
            c.painter.rect_filled(bar, 1., c.color);
            if c.enabled && c.pointer.is_some_and(|p| bar.contains(p)) {
                hover = Some(index);
            }
        }
    }
    hover
}
