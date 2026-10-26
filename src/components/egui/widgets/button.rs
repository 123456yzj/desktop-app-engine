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
    n: &protocol::Node,
    r: &Resolved,
    rect: egui::Rect,
    content: egui::Rect,
    state: &crate::binding::State,
    collector: &mut Collector<'_>,
) -> bool {
    let painter = ui.painter().clone();
    let loading = contract::flag(n, state, "loading", false);
    let enabled = r.enabled && !loading;
    let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    let response = ui.interact(
        content,
        egui::Id::new((&n.id, "button")),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &r.text));
    let variant = contract::string(n, state, "variant", "outlined");
    let danger = contract::flag(n, state, "danger", false);
    let tone = if danger { Token::Danger } else { Token::Accent };
    let fill = if matches!(variant, "link") {
        Color32::TRANSPARENT
    } else if !enabled {
        ui.visuals().widgets.noninteractive.bg_fill
    } else if let Some(background) = n.style.background {
        let base = token(ui, background);
        if response.is_pointer_button_down_on() {
            base.gamma_multiply(0.8)
        } else if response.hovered() {
            base.gamma_multiply(0.9)
        } else {
            base
        }
    } else if variant == "solid" {
        token(ui, tone)
    } else if variant == "filled" {
        token(ui, tone).gamma_multiply(0.15)
    } else if matches!(variant, "text" | "link") && !response.hovered() {
        Color32::TRANSPARENT
    } else if response.is_pointer_button_down_on() {
        ui.visuals().widgets.active.bg_fill
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };
    let visual = if !enabled {
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
        n.style.radius
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
            } else {
                n.style.border_width
            },
            if response.has_focus() {
                token(ui, Token::Focus)
            } else {
                token(ui, border)
            },
        ),
        StrokeKind::Inside,
    );
    if enabled && response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.),
            radius + 2.,
            Stroke::new(1_f32, token(ui, Token::Focus)),
            StrokeKind::Inside,
        );
    }
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
    let mut label_content = content;
    if loading {
        let spinner = egui::Rect::from_center_size(
            Pos2::new(content.left() + 18., content.center().y),
            Vec2::splat(14.),
        );
        ui.put(
            spinner,
            egui::Spinner::new()
                .size(14.)
                .color(token(ui, Token::TextMuted)),
        );
        label_content.min.x += 22.;
    }
    styled_label(
        ui,
        &painter,
        label_content.shrink2(Vec2::new(
            n.style.padding_x.unwrap_or(10.),
            n.style.padding_y.unwrap_or(6.),
        )),
        &button_text,
        &n.style,
        if let Some(c) = visual.and_then(|s| s.color) {
            token(ui, c)
        } else if enabled {
            if let Some(color) = n.style.color {
                token(ui, color)
            } else if variant == "solid" {
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
    if enabled && response.clicked() {
        response.request_focus();
        let href = contract::string(n, state, "href", "");
        if !href.is_empty() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(href));
        }
        collector.push(ui, "click", None, Value::Null);
    }
    enabled && response.clicked()
}
