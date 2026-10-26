use super::*;
component!(
    &[
        ("items", Items),
        ("column", Integer(1, 24)),
        ("bordered", Bool),
        ("colon", Bool),
        ("direction", Enum(&["vertical", "horizontal"]))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui, n, rect, state, ..
    } = c;
    let columns = contract::number(n, state, "column", 3.) as usize;
    let bordered = contract::flag(n, state, "bordered", false);
    let vertical = contract::string(n, state, "direction", "horizontal") == "vertical";
    let items = contract::items(n, state);
    let width = rect.width() / columns as f32;
    let font = style_font(&n.style, ui.ctx());
    let stroke = Stroke::new(1.0_f32, token(ui, Token::Border));
    let mut top = rect.top();
    for row in items.chunks(columns) {
        let cells: Vec<_> = row
            .iter()
            .map(|item| {
                let label_width = if vertical { width } else { width * 0.34 };
                let content_width = if vertical { width } else { width - label_width };
                let label = format!(
                    "{}{}",
                    item_label(item),
                    if !vertical && contract::flag(n, state, "colon", true) {
                        ":"
                    } else {
                        ""
                    }
                );
                let content = item["content"]
                    .as_str()
                    .or_else(|| item["text"].as_str())
                    .unwrap_or("");
                let label = ui.painter().layout(
                    label,
                    font.clone(),
                    token(ui, Token::TextMuted),
                    (label_width - 20.).max(1.),
                );
                let content = ui.painter().layout(
                    content.to_owned(),
                    font.clone(),
                    token(ui, Token::TextPrimary),
                    (content_width - 20.).max(1.),
                );
                (label, content, label_width)
            })
            .collect();
        let height = cells
            .iter()
            .map(|(l, c, _)| {
                if vertical {
                    l.size().y + c.size().y + 8.
                } else {
                    l.size().y.max(c.size().y)
                }
            })
            .fold(0., f32::max)
            + 20.;
        for (col, (label, content, label_width)) in cells.into_iter().enumerate() {
            let cell = egui::Rect::from_min_size(
                Pos2::new(rect.left() + col as f32 * width, top),
                Vec2::new(width, height),
            );
            let split = if vertical {
                cell.top() + label.size().y + 16.
            } else {
                cell.left() + label_width
            };
            if bordered {
                let label_rect = if vertical {
                    egui::Rect::from_min_max(cell.min, Pos2::new(cell.right(), split))
                } else {
                    egui::Rect::from_min_max(cell.min, Pos2::new(split, cell.bottom()))
                };
                ui.painter()
                    .rect_filled(label_rect, 0., token(ui, Token::SurfaceRaised));
                ui.painter()
                    .rect_stroke(cell, 0., stroke, egui::StrokeKind::Inside);
                let line = if vertical {
                    [
                        Pos2::new(cell.left(), split),
                        Pos2::new(cell.right(), split),
                    ]
                } else {
                    [
                        Pos2::new(split, cell.top()),
                        Pos2::new(split, cell.bottom()),
                    ]
                };
                ui.painter().line_segment(line, stroke);
            }
            ui.painter().galley(
                cell.min + Vec2::splat(10.),
                label,
                token(ui, Token::TextMuted),
            );
            let pos = if vertical {
                Pos2::new(cell.left() + 10., split + 4.)
            } else {
                Pos2::new(split + 10., cell.top() + 10.)
            };
            ui.painter()
                .galley(pos, content, token(ui, Token::TextPrimary));
        }
        top += height;
    }
}
