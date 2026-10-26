use super::*;
use crate::protocol::{LineVariant, TextAlign};
component!(&[], &[]);
pub(in crate::components::egui) fn paint(
    ui: &egui::Ui,
    painter: &egui::Painter,
    node: &protocol::Node,
    rect: egui::Rect,
    text: &str,
) {
    let stroke = Stroke::new(
        node.style.border_width.max(1.),
        token(ui, node.style.border_color.unwrap_or(Token::Border)),
    );
    let line = |a: Pos2, b: Pos2| line(painter, a, b, stroke, node.variant);
    if node.vertical {
        line(
            Pos2::new(rect.center().x, rect.top()),
            Pos2::new(rect.center().x, rect.bottom()),
        );
    } else if text.is_empty() {
        line(
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        );
    } else {
        let color = token(ui, node.style.color.unwrap_or(Token::TextPrimary));
        let galley =
            painter.layout_no_wrap(text.to_owned(), style_font(&node.style, ui.ctx()), color);
        let width = galley.size().x.min((rect.width() - 32.).max(0.));
        let left = match node.title_placement.unwrap_or(TextAlign::Center) {
            TextAlign::Start => rect.left() + 16.,
            TextAlign::Center => rect.center().x - width / 2.,
            TextAlign::End => rect.right() - 16. - width,
        };
        let title =
            egui::Rect::from_min_size(Pos2::new(left, rect.top()), Vec2::new(width, rect.height()));
        painter
            .with_clip_rect(title.intersect(painter.clip_rect()))
            .galley(
                Pos2::new(left, rect.center().y - galley.size().y / 2.),
                galley,
                color,
            );
        line(
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new((left - 8.).max(rect.left()), rect.center().y),
        );
        line(
            Pos2::new((left + width + 8.).min(rect.right()), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        );
    }
}
fn line(painter: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke, variant: LineVariant) {
    if matches!(variant, LineVariant::Solid) {
        painter.line_segment([a, b], stroke);
        return;
    }
    let horizontal = a.y == b.y;
    let clip = painter.clip_rect();
    let (start, end) = if horizontal {
        (a.x.max(clip.left()), b.x.min(clip.right()))
    } else {
        (a.y.max(clip.top()), b.y.min(clip.bottom()))
    };
    let dash = if matches!(variant, LineVariant::Dotted) {
        stroke.width
    } else {
        6.
    };
    let step = dash + 4.;
    for i in 0..(((end - start).max(0.) / step).ceil() as usize).min(4096) {
        let from = start + i as f32 * step;
        let to = (from + dash).min(end);
        let points = if horizontal {
            [Pos2::new(from, a.y), Pos2::new(to, a.y)]
        } else {
            [Pos2::new(a.x, from), Pos2::new(a.x, to)]
        };
        painter.line_segment(points, stroke);
    }
}
