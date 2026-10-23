use super::*;
component!(
    &[
        ("loading", Bool),
        ("icon", Text),
        ("iconPlacement", Enum(&["start", "end"])),
        ("shape", Enum(&["default", "circle", "round"])),
        (
            "variant",
            Enum(&["outlined", "solid", "dashed", "filled", "text", "link"])
        ),
        ("danger", Bool),
        ("block", Bool),
        ("href", Text)
    ],
    &["click"]
);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    n: &v02::Node,
    r: &Resolved,
    rect: egui::Rect,
    content: egui::Rect,
    state: &crate::binding::State,
    collector: &mut Collector<'_>,
) -> bool {
    let painter = ui.painter().clone();
    let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    let response = ui.interact(
        content,
        egui::Id::new((&n.id, "button")),
        if r.enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, r.enabled, &r.text));
    let variant = contract::string(n, state, "variant", "outlined");
    let danger = contract::flag(n, state, "danger", false);
    let tone = if danger { Token::Danger } else { Token::Accent };
    let fill = if !r.enabled {
        ui.visuals().widgets.noninteractive.bg_fill
    } else if variant == "solid" {
        token(ui, tone)
    } else if variant == "filled" {
        token(ui, tone).gamma_multiply(0.15)
    } else if matches!(variant, "text" | "link") && !response.hovered() {
        Color32::TRANSPARENT
    } else if let Some(background) = n.style.background {
        let base = token(ui, background);
        if response.is_pointer_button_down_on() {
            base.gamma_multiply(0.8)
        } else if response.hovered() {
            base.gamma_multiply(0.9)
        } else {
            base
        }
    } else if response.is_pointer_button_down_on() {
        ui.visuals().widgets.active.bg_fill
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    let visual = if !r.enabled {
        n.style.disabled.as_ref()
    } else if response.is_pointer_button_down_on() {
        n.style.pressed.as_ref()
    } else if response.hovered() {
        n.style.hover.as_ref()
    } else {
        None
    };
    let fill = visual
        .and_then(|s| s.background)
        .map(|c| token(ui, c))
        .unwrap_or(fill);
    let border = visual
        .and_then(|s| s.border_color)
        .or(n.style.border_color)
        .unwrap_or(Token::Border);
    let radius = if contract::string(n, state, "shape", "default") == "default" {
        n.style.radius.max(4.)
    } else {
        rect.height() / 2.
    };
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(
            if matches!(variant, "text" | "link" | "filled" | "solid") {
                0.
            } else if n.style.border_color.is_some() {
                n.style.border_width
            } else {
                1.
            },
            if response.has_focus() {
                token(ui, Token::Focus)
            } else {
                token(ui, border)
            },
        ),
        StrokeKind::Inside,
    );
    let button_text =
        if let Some(icon) = contract::property(n, state, "icon").and_then(Value::as_str) {
            if contract::string(n, state, "iconPlacement", "start") == "end" {
                format!("{} {icon}", r.text)
            } else {
                format!("{icon} {}", r.text)
            }
        } else {
            r.text.clone()
        };
    styled_label(
        ui,
        &painter,
        rect.shrink2(Vec2::new(
            n.style.padding_x.unwrap_or(10.),
            n.style.padding_y.unwrap_or(6.),
        )),
        &button_text,
        &n.style,
        if let Some(c) = visual.and_then(|s| s.color) {
            token(ui, c)
        } else if r.enabled {
            if variant == "solid" {
                token(ui, Token::OnPrimary)
            } else if danger || variant == "link" {
                token(ui, tone)
            } else {
                color
            }
        } else {
            token(ui, Token::TextMuted)
        },
    );
    if response.clicked() {
        let href = contract::string(n, state, "href", "");
        if !href.is_empty() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(href));
        }
        collector.push(ui, "click", None, Value::Null);
    }
    response.clicked()
}
