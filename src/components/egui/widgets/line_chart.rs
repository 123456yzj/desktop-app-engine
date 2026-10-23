use super::*;
component!(&[], &[]);
#[allow(clippy::too_many_arguments)]
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    n: &v02::Node,
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
        false,
        paint_series,
    );
}
fn paint_series(c: charts::SeriesContext<'_>) -> Option<usize> {
    let mut previous = None;
    for &index in c.sampled {
        let Some(point) = c.points[index] else {
            previous = None;
            continue;
        };
        let point = point_pos(c.transform.screen(point));
        if let Some(last) = previous {
            c.painter
                .line_segment([last, point], Stroke::new(2_f32, c.color));
        } else {
            c.painter.circle_filled(point, 3., c.color);
        }
        previous = Some(point);
    }
    if c.enabled
        && let Some(pointer) = c.pointer
    {
        chart::hit(
            c.points,
            c.transform,
            Point {
                x: pointer.x as f64,
                y: pointer.y as f64,
            },
            c.monotonic,
        )
    } else {
        None
    }
}
