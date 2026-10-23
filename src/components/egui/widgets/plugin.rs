use super::*;
component!(&[], &[]);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    n: &v02::Node,
    content: egui::Rect,
    enabled: bool,
    texture: Option<&egui::TextureHandle>,
    host: &crate::plugin::Host,
) -> Result<(), String> {
    if let Some(texture) = texture {
        painter.image(
            texture.id(),
            content,
            egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
            Color32::WHITE,
        );
    }
    let response = ui.interact(
        content.intersect(ui.clip_rect()),
        egui::Id::new((&n.id, "plugin")),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Image,
            enabled,
            n.component.as_deref().unwrap_or("plugin"),
        )
    });
    if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        host.send(&n.id, json!({"op":"input","event":{"kind":"click","x":p.x-content.left(),"y":p.y-content.top()}}), None, None)?;
    }
    Ok(())
}
