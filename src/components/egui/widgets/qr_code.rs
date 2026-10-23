use super::*;
component!(
    &[
        ("errorLevel", Enum(&["L", "M", "Q", "H"])),
        ("bordered", Bool),
        ("status", Enum(&["active", "expired", "loading", "scanned"]))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        rect,
        state,
        value,
        ..
    } = c;
    let level = match contract::string(n, state, "errorLevel", "M") {
        "L" => qrcode::EcLevel::L,
        "Q" => qrcode::EcLevel::Q,
        "H" => qrcode::EcLevel::H,
        _ => qrcode::EcLevel::M,
    };
    match qrcode::QrCode::with_error_correction_level(
        value.as_str().unwrap_or("").as_bytes(),
        level,
    ) {
        Ok(code) => {
            let size = code.width();
            let cell = rect.width().min(rect.height()) / (size + 8) as f32;
            ui.painter().rect_filled(rect, 0., Color32::WHITE);
            for y in 0..size {
                for x in 0..size {
                    if code[(x, y)] == qrcode::Color::Dark {
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                rect.min + Vec2::new((x + 4) as f32 * cell, (y + 4) as f32 * cell),
                                Vec2::splat(cell + 0.1),
                            ),
                            0.,
                            Color32::BLACK,
                        );
                    }
                }
            }
            let status = contract::string(n, state, "status", "active");
            if status != "active" {
                ui.painter()
                    .rect_filled(rect, 0., Color32::from_white_alpha(220));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    match status {
                        "loading" => "生成中…",
                        "expired" => "二维码已过期",
                        _ => "已扫描",
                    },
                    FontId::proportional(16.),
                    Color32::BLACK,
                );
            }
        }
        Err(error) => {
            ui.colored_label(token(ui, Token::Danger), format!("二维码：{error}"));
        }
    }
    if contract::flag(n, state, "bordered", true) {
        ui.painter().rect_stroke(
            rect,
            4.,
            Stroke::new(1_f32, token(ui, Token::Border)),
            StrokeKind::Inside,
        );
    }
}
