use super::*;
pub(in crate::components::egui::widgets) fn paint(c: WidgetContext<'_, '_>, avatar: bool) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        memory,
        collector,
        ..
    } = c;
    let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    let src = contract::string(n, state, "src", "");
    if !src.is_empty() {
        let loaded = egui::Image::new(src).load_for_size(ui.ctx(), rect.size());
        let status = match &loaded {
            Ok(egui::load::TexturePoll::Ready { .. }) => "load",
            Err(_) => "error",
            _ => "pending",
        };
        let marker = json!([src, status]);
        if memory.properties.get("imageStatus") != Some(&marker) && status != "pending" {
            collector.push(ui, status, None, json!(src));
        }
        memory.properties.insert("imageStatus".into(), marker);
        let fallback = contract::string(n, state, "fallback", "");
        let shown = if loaded.is_err() && !fallback.is_empty() {
            fallback
        } else {
            src
        };
        let circle = avatar && contract::string(n, state, "shape", "circle") == "circle";
        let image = egui::Image::new(shown)
            .fit_to_exact_size(rect.size())
            .maintain_aspect_ratio(contract::string(n, state, "fit", "contain") != "fill")
            .corner_radius(if circle {
                rect.height().min(rect.width()) / 2.
            } else {
                0.
            });
        if loaded.is_err() && fallback.is_empty() {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                contract::string(n, state, "alt", "图片加载失败"),
                FontId::proportional(14.),
                color,
            );
        } else {
            image.paint_at(ui, rect);
        }
        let response = ui.interact(rect, egui::Id::new((&n.id, "image")), egui::Sense::click());
        if response.clicked() && contract::flag(n, state, "preview", !avatar) {
            memory.was_open = true;
            collector.push(ui, "preview", None, json!(src));
        }
        if memory.was_open {
            egui::Window::new(if r.title.is_empty() {
                "图片预览"
            } else {
                &r.title
            })
            .id(egui::Id::new((&n.id, "preview")))
            .open(&mut memory.was_open)
            .show(ui.ctx(), |ui| {
                ui.add(egui::Image::new(src).max_size(Vec2::new(960., 720.)));
            });
        }
    } else {
        let circle = avatar && contract::string(n, state, "shape", "circle") == "circle";
        ui.painter().rect_filled(
            rect,
            if circle { rect.height() / 2. } else { 6. },
            token(ui, Token::Accent).gamma_multiply(0.2),
        );
        let mut style = n.style.clone();
        style.text_align = Some(v02::TextAlign::Center);
        styled_label(
            ui,
            &ui.painter().clone(),
            rect,
            if r.text.is_empty() {
                contract::string(n, state, "icon", contract::string(n, state, "alt", "用户"))
            } else {
                &r.text
            },
            &style,
            color,
        );
    }
}
