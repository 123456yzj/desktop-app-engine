//! Host text metrics and clipped accessible labels share the same font selection.
use super::{Color32, FontId, Pos2, Size, Vec2, egui, v02};
use crate::layout::TextMeasure;

pub(super) struct EguiMeasure(pub(super) egui::Painter);
impl EguiMeasure {
    fn size(&self, galley: &egui::Galley) -> Size {
        // Arrangement snaps edges to physical pixels. Round natural text sizes
        // outward so snapping cannot push the last glyph onto a clipped new line.
        let scale = self.0.ctx().pixels_per_point();
        Size {
            w: (galley.size().x * scale).ceil() / scale,
            h: (galley.size().y * scale).ceil() / scale,
        }
    }
}
impl TextMeasure for EguiMeasure {
    fn styled_text(&self, text: &str, width: f32, style: &v02::Style) -> Size {
        let g = self.0.layout(
            text.into(),
            style_font(style, self.0.ctx()),
            Color32::WHITE,
            if style.text_wrap == Some(false) {
                100_000.0
            } else {
                width.max(1.0)
            },
        );
        self.size(&g)
    }
    fn text(&self, text: &str, width: f32, font_size: f32) -> Size {
        let g = self.0.layout(
            text.to_owned(),
            FontId::proportional(font_size),
            Color32::WHITE,
            width.max(1.0),
        );
        self.size(&g)
    }
}
pub(super) fn style_font(style: &v02::Style, ctx: &egui::Context) -> FontId {
    let bold = egui::FontFamily::Name("bold".into());
    let family = if matches!(style.font_family, Some(v02::FontFamily::Monospace)) {
        egui::FontFamily::Monospace
    } else if matches!(style.font_weight, Some(v02::FontWeight::Bold))
        && ctx.fonts(|f| f.families().contains(&bold))
    {
        bold
    } else {
        egui::FontFamily::Proportional
    };
    FontId::new(style.font_size.unwrap_or(16.0), family)
}
/// Lay out decorated text exactly like [`styled_label`] so typography flags keep
/// `textAlign`/`textWrap` (egui's `Label` only centers within its own galley).
pub(super) fn decorated_label(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    r: egui::Rect,
    text: egui::WidgetText,
    style: &v02::Style,
    color: Color32,
    wrap_mode: egui::TextWrapMode,
) {
    if !r.is_positive() {
        return;
    }
    let label = text.text().to_owned();
    let galley = text.into_galley(
        ui,
        Some(wrap_mode),
        r.width().max(1.),
        egui::TextStyle::Body,
    );
    let size = galley.size();
    let pos = style_position(r, size, style);
    painter
        .with_clip_rect(r.intersect(painter.clip_rect()))
        .galley(pos, galley, color);
    let response = ui.interact(
        r.intersect(ui.clip_rect()),
        ui.id().with((&label, r.min.x.to_bits(), r.min.y.to_bits())),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &label));
    if style.text_overflow_tooltip != Some(false)
        && (size.x > r.width() + 0.5 || size.y > r.height() + 0.5)
    {
        response.on_hover_text(label);
    }
}
fn style_position(r: egui::Rect, size: Vec2, style: &v02::Style) -> Pos2 {
    match style.text_align {
        Some(v02::TextAlign::Center) => {
            r.min
                + Vec2::new(
                    (r.width() - size.x).max(0.0) / 2.0,
                    (r.height() - size.y).max(0.0) / 2.0,
                )
        }
        Some(v02::TextAlign::End) => r.min + Vec2::new((r.width() - size.x).max(0.0), 0.0),
        _ => r.min,
    }
}
pub(super) fn styled_label(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    r: egui::Rect,
    text: &str,
    style: &v02::Style,
    color: Color32,
) {
    if text.is_empty() || !r.is_positive() {
        return;
    }
    let g = painter.layout(
        text.into(),
        style_font(style, painter.ctx()),
        color,
        if style.text_wrap == Some(false) {
            100_000.0
        } else {
            r.width()
        },
    );
    let size = g.size();
    let pos = style_position(r, size, style);
    painter
        .with_clip_rect(r.intersect(painter.clip_rect()))
        .galley(pos, g, color);
    let response = ui.interact(
        r.intersect(ui.clip_rect()),
        ui.id().with((text, r.min.x.to_bits(), r.min.y.to_bits())),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    if style.text_overflow_tooltip != Some(false) && (size.x > r.width() || size.y > r.height()) {
        response.on_hover_text(text);
    }
}
pub(super) fn label(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    r: egui::Rect,
    text: &str,
    font: f32,
    color: Color32,
) {
    text_label(ui, painter, r, text, font, color, true);
}
pub(super) fn label_line(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    r: egui::Rect,
    text: &str,
    font: f32,
    color: Color32,
) {
    text_label(ui, painter, r, text, font, color, false);
}
#[allow(clippy::too_many_arguments)]
fn text_label(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    r: egui::Rect,
    text: &str,
    font: f32,
    color: Color32,
    wrap: bool,
) {
    if text.is_empty() || r.width() <= 0.0 || r.height() <= 0.0 {
        return;
    }
    let g = if wrap {
        painter.layout(
            text.to_owned(),
            FontId::proportional(font),
            color,
            r.width(),
        )
    } else {
        painter.layout_no_wrap(text.to_owned(), FontId::proportional(font), color)
    };
    painter
        .with_clip_rect(r.intersect(painter.clip_rect()))
        .galley(r.min, g, color);
    let clipped = r.intersect(painter.clip_rect()).intersect(ui.clip_rect());
    if !clipped.is_positive() {
        return;
    }
    let response = ui.interact(
        clipped,
        ui.id().with((text, r.min.x.to_bits(), r.min.y.to_bits())),
        egui::Sense::hover(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
}
