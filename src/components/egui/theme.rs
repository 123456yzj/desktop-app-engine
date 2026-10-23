//! Translate protocol color and control styles into egui visuals.
use super::{Color32, Stroke, Token, Vec2, egui, text::style_font, v02};

pub(super) fn token(ui: &egui::Ui, t: Token) -> Color32 {
    let v = ui.visuals();
    match t {
        Token::Surface => v.panel_fill,
        Token::SurfaceRaised => v.faint_bg_color,
        Token::TextPrimary => v.text_color(),
        Token::TextMuted => v.weak_text_color(),
        Token::Border => v.widgets.noninteractive.bg_stroke.color,
        Token::Accent => v.selection.bg_fill,
        Token::Success => {
            if v.dark_mode {
                Color32::from_rgb(90, 210, 150)
            } else {
                Color32::from_rgb(20, 130, 80)
            }
        }
        Token::Warning => {
            if v.dark_mode {
                Color32::from_rgb(240, 195, 80)
            } else {
                Color32::from_rgb(150, 100, 0)
            }
        }
        Token::Danger => v.error_fg_color,
        Token::Focus => v.selection.stroke.color,
        Token::Primary => Color32::from_rgb(58, 94, 220),
        Token::OnPrimary => Color32::WHITE,
        Token::Hex([r, g, b, a]) => Color32::from_rgba_unmultiplied(r, g, b, a),
    }
}
pub(super) fn control_theme(ui: &mut egui::Ui, style: &v02::Style, height: f32) {
    let bg = style.background.map(|c| token(ui, c));
    let fg = style.color.map(|c| token(ui, c));
    let border = style.border_color.map(|c| token(ui, c));
    let states = [
        style.disabled.as_ref(),
        None,
        style.hover.as_ref(),
        style.pressed.as_ref(),
        style.hover.as_ref(),
    ];
    let colors: Vec<_> = states
        .iter()
        .map(|s| {
            s.map(|s| {
                (
                    s.background.map(|c| token(ui, c)),
                    s.color.map(|c| token(ui, c)),
                    s.border_color.map(|c| token(ui, c)),
                )
            })
        })
        .collect();
    let custom = bg.is_some()
        || fg.is_some()
        || border.is_some()
        || style.hover.is_some()
        || style.pressed.is_some()
        || style.disabled.is_some()
        || style.font_size.is_some()
        || style.font_family.is_some()
        || style.font_weight.is_some()
        || style.padding_x.is_some()
        || style.padding_y.is_some()
        || style.radius > 0.0;
    if !custom {
        return;
    }
    let font = style_font(style, ui.ctx());
    let s = ui.style_mut();
    s.override_font_id = Some(font);
    s.spacing.interact_size.y = height;
    s.spacing.button_padding = Vec2::new(
        style.padding_x.unwrap_or(8.0),
        style.padding_y.unwrap_or(5.0),
    );
    if let Some(bg) = bg {
        s.visuals.extreme_bg_color = bg;
    }
    if let Some(fg) = fg {
        s.visuals.override_text_color = Some(fg);
    }
    let w = &mut s.visuals.widgets;
    for (widget, state) in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ]
    .into_iter()
    .zip(colors)
    {
        let (over_bg, over_fg, over_border) = state.unwrap_or((None, None, None));
        if let Some(c) = over_bg.or(bg) {
            widget.bg_fill = c;
            widget.weak_bg_fill = c;
        }
        if let Some(c) = over_fg.or(fg) {
            widget.fg_stroke.color = c;
        }
        if let Some(c) = over_border.or(border) {
            widget.bg_stroke = Stroke::new(style.border_width, c);
        }
        widget.corner_radius = egui::CornerRadius::same(style.radius.clamp(0.0, 255.0) as u8);
    }
}
