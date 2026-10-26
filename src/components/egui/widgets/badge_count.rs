use super::*;
component!(
    &[
        ("count", Integer(0, 1000000000)),
        ("overflowCount", Integer(1, 1000000000)),
        ("showZero", Bool),
        ("dot", Bool),
        (
            "status",
            Enum(&["success", "processing", "default", "error", "warning"])
        )
    ],
    &[]
);
fn indicator(r: &Resolved) -> Option<String> {
    let count = r.api.get("count").and_then(Value::as_f64).unwrap_or(0.) as u64;
    let limit = r
        .api
        .get("overflowCount")
        .and_then(Value::as_f64)
        .unwrap_or(99.) as u64;
    if r.api.get("dot") == Some(&Value::Bool(true)) {
        Some(String::new())
    } else if count > 0 || r.api.get("showZero") == Some(&Value::Bool(true)) {
        Some(if count > limit {
            format!("{limit}+")
        } else {
            count.to_string()
        })
    } else {
        None
    }
}
fn indicator_width(text: &str, measured: f32) -> f32 {
    if text.is_empty() {
        8.
    } else {
        (measured + 12.).max(22.)
    }
}
pub(in crate::components) fn measure(
    r: &Resolved,
    style: &v02::Style,
    inner: f32,
    text: &impl crate::layout::TextMeasure,
) -> Size {
    let marker = indicator(r);
    let width = marker
        .as_ref()
        .map_or(0., |s| indicator_width(s, text.text(s, 100_000., 12.).w));
    let leading = width
        + if marker.is_some() && !r.text.is_empty() {
            8.
        } else {
            0.
        };
    let label = text.styled_text(&r.text, (inner - leading).max(1.), style);
    Size {
        w: leading + label.w,
        h: label.h.max(if marker.is_some() { 22. } else { 0. }),
    }
}
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        ..
    } = c;
    let mut leading = 0.;
    if let Some(text) = indicator(r) {
        let tone = shared::status_color(contract::string(n, state, "status", "error"));
        let galley =
            ui.painter()
                .layout_no_wrap(text.clone(), FontId::proportional(12.), Color32::WHITE);
        let width = indicator_width(&text, galley.size().x);
        leading = width + if r.text.is_empty() { 0. } else { 8. };
        if text.is_empty() {
            ui.painter()
                .circle_filled(rect.left_center() + Vec2::new(4., 0.), 4., token(ui, tone));
        } else {
            let badge = egui::Rect::from_center_size(
                Pos2::new(rect.left() + width / 2., rect.center().y),
                Vec2::new(width, 22.),
            );
            ui.painter().rect_filled(badge, 11., token(ui, tone));
            ui.painter()
                .galley(badge.center() - galley.size() / 2., galley, Color32::WHITE);
        }
    }
    if !r.text.is_empty() {
        let label = egui::Rect::from_min_max(rect.min + Vec2::new(leading, 0.), rect.max);
        let mut style = n.style.clone();
        style.text_align = Some(v02::TextAlign::Start);
        styled_label(
            ui,
            &ui.painter().clone(),
            label,
            &r.text,
            &style,
            token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
        );
    }
}
