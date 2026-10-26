use super::*;
component!(
    &[
        ("showInfo", Bool),
        ("percent", Number(0., 100.)),
        (
            "status",
            Enum(&["normal", "active", "success", "exception"])
        ),
        ("progressType", Enum(&["line", "circle", "dashboard"])),
        ("strokeWidth", Number(1., 64.))
    ],
    &[]
);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    n: &protocol::Node,
    r: &Resolved,
    content: egui::Rect,
    state: &crate::binding::State,
) {
    let painter = ui.painter().clone();
    let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    let value = contract::property(n, state, "percent")
        .and_then(Value::as_f64)
        .map(|v| v / 100.)
        .or_else(|| r.value.as_ref().and_then(Value::as_f64))
        .unwrap_or(0.) as f32;
    let tone = match contract::string(n, state, "status", "normal") {
        "success" => Token::Success,
        "exception" => Token::Danger,
        _ => Token::Accent,
    };
    let thickness = contract::number(n, state, "strokeWidth", 8.) as f32;
    let progress_type = contract::string(n, state, "progressType", "line");
    if progress_type != "line" {
        let center = content.center();
        let radius = (content.width().min(content.height()) / 2. - thickness).max(0.);
        let sweep = if progress_type == "dashboard" {
            std::f32::consts::TAU * 0.75
        } else {
            std::f32::consts::TAU
        };
        let start = if progress_type == "dashboard" {
            std::f32::consts::PI * 0.75
        } else {
            -std::f32::consts::FRAC_PI_2
        };
        for (portion, color) in [(1., token(ui, Token::Border)), (value, token(ui, tone))] {
            let points = (0..=100)
                .map(|i| center + Vec2::angled(start + sweep * portion * i as f32 / 100.) * radius)
                .collect();
            painter.add(egui::Shape::line(points, Stroke::new(thickness, color)));
        }
        if contract::flag(n, state, "showInfo", true) {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{:.0}%", value * 100.),
                style_font(&n.style, ui.ctx()),
                color,
            );
        }
        return;
    }
    let bar = egui::Rect::from_min_size(
        content.min + Vec2::new(0., content.height() - thickness),
        Vec2::new(content.width(), thickness),
    );
    painter.rect_filled(bar, 4., token(ui, Token::Border));
    painter.rect_filled(
        egui::Rect::from_min_size(bar.min, Vec2::new(bar.width() * value, bar.height())),
        4.,
        token(ui, tone),
    );
    if contract::flag(n, state, "showInfo", true) {
        let percentage = format!("{:.0}%", value * 100.);
        label(
            ui,
            &painter,
            content,
            if r.text.is_empty() {
                &percentage
            } else {
                &r.text
            },
            n.style.font_size.unwrap_or(16.),
            color,
        );
    }
}
