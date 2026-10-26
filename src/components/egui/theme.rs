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
        Token::Accent => v.selection.stroke.color,
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

/// Consistent controls inside a component scope, including native egui widgets.
pub(super) fn component_visuals(ui: &mut egui::Ui) {
    let dark = ui.visuals().dark_mode;
    let (surface, subtle, hover, pressed, border, text) = if dark {
        (
            Color32::from_rgb(24, 29, 39),
            Color32::from_rgb(32, 39, 51),
            Color32::from_rgb(38, 48, 65),
            Color32::from_rgb(42, 58, 82),
            Color32::from_rgb(65, 77, 96),
            Color32::from_rgb(226, 232, 240),
        )
    } else {
        (
            Color32::WHITE,
            Color32::from_rgb(248, 250, 252),
            Color32::from_rgb(239, 246, 255),
            Color32::from_rgb(219, 234, 254),
            Color32::from_rgb(203, 213, 225),
            Color32::from_rgb(30, 41, 59),
        )
    };
    let accent = Color32::from_rgb(59, 130, 246);
    let s = ui.style_mut();
    s.spacing.button_padding = Vec2::new(10., 5.);
    s.spacing.interact_size.y = 30.;
    s.spacing.item_spacing = Vec2::new(8., 6.);
    let v = &mut s.visuals;
    v.panel_fill = surface;
    v.window_fill = surface;
    v.extreme_bg_color = surface;
    v.faint_bg_color = subtle;
    v.selection.bg_fill = if dark {
        pressed
    } else {
        Color32::from_rgb(219, 234, 254)
    };
    v.selection.stroke = Stroke::new(1_f32, accent);
    for (widget, fill, stroke, fg) in [
        (&mut v.widgets.noninteractive, subtle, border, text),
        (&mut v.widgets.inactive, surface, border, text),
        (&mut v.widgets.hovered, hover, accent, accent),
        (&mut v.widgets.active, pressed, accent, accent),
        (&mut v.widgets.open, hover, accent, text),
    ] {
        widget.bg_fill = fill;
        widget.weak_bg_fill = fill;
        widget.bg_stroke = Stroke::new(1_f32, stroke);
        widget.fg_stroke = Stroke::new(1_f32, fg);
        widget.corner_radius = egui::CornerRadius::same(6);
        widget.expansion = 0.;
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
