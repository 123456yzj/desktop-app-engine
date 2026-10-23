use super::*;
component!(
    &[
        ("copyable", Bool),
        ("editable", Bool),
        ("ellipsis", Bool),
        ("code", Bool),
        ("italic", Bool),
        ("underline", Bool),
        ("delete", Bool),
        ("strong", Bool)
    ],
    &["change", "copy"]
);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    n: &v02::Node,
    r: &Resolved,
    rect: egui::Rect,
    state: &crate::binding::State,
    collector: &mut Collector<'_>,
) -> Option<String> {
    if n.api.is_empty() {
        styled_label(
            ui,
            &ui.painter().clone(),
            rect,
            &r.text,
            &n.style,
            token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
        );
        return None;
    }
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((&n.id, "typography"))
            .max_rect(rect),
    );
    if !r.enabled {
        child.disable();
    }
    let mut edited = None;
    if contract::flag(n, state, "editable", false) {
        let mut text = r.text.clone();
        if child
            .add(egui::TextEdit::multiline(&mut text).desired_width(rect.width()))
            .changed()
        {
            collector.push(&child, "change", None, json!(text));
            edited = Some(text);
        }
    } else {
        let mut text = egui::RichText::new(&r.text)
            .font(style_font(&n.style, ui.ctx()))
            .color(token(ui, n.style.color.unwrap_or(Token::TextPrimary)));
        if contract::flag(n, state, "strong", false) {
            text = text.strong();
        }
        if contract::flag(n, state, "italic", false) {
            text = text.italics();
        }
        if contract::flag(n, state, "underline", false) {
            text = text.underline();
        }
        if contract::flag(n, state, "delete", false) {
            text = text.strikethrough();
        }
        if contract::flag(n, state, "code", false) {
            text = text.code();
        }
        let wrap = if contract::flag(n, state, "ellipsis", false) {
            egui::TextWrapMode::Truncate
        } else if n.style.text_wrap == Some(false) {
            egui::TextWrapMode::Extend
        } else {
            egui::TextWrapMode::Wrap
        };
        // Measurement reserves a 28 point strip for the copy button.
        let copyable = contract::flag(n, state, "copyable", false);
        let text_rect = if copyable {
            egui::Rect::from_min_max(
                rect.min,
                Pos2::new(rect.right(), (rect.bottom() - 28.).max(rect.top())),
            )
        } else {
            rect
        };
        let painter = child.painter().clone();
        decorated_label(
            &mut child,
            &painter,
            text_rect,
            text.into(),
            &n.style,
            token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
            wrap,
        );
    }
    if contract::flag(n, state, "copyable", false) {
        let button = egui::Rect::from_min_size(
            Pos2::new(rect.left(), (rect.bottom() - 24.).max(rect.top())),
            Vec2::new(48., 24.),
        );
        if child
            .put(button, egui::Button::new("复制").small())
            .clicked()
        {
            child.ctx().copy_text(r.text.clone());
            collector.push(&child, "copy", None, json!(r.text));
        }
    }
    edited
}
