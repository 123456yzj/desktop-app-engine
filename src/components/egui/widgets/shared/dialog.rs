use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("open", Bool),
    ("closable", Bool),
    ("maskClosable", Bool),
    ("keyboard", Bool),
    ("width", Number(160., 8192.)),
    ("height", Number(100., 8192.)),
    ("placement", Enum(&["left", "right", "top", "bottom"])),
    ("okText", Text),
    ("cancelText", Text),
    ("confirmLoading", Bool),
    ("footer", Bool),
    ("centered", Bool),
];

pub(in crate::components::egui::widgets) fn paint(c: WidgetContext<'_, '_>, drawer: bool) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        collector,
        ..
    } = c;
    if !local(n, state, memory, "open", json!(false))
        .as_bool()
        .unwrap_or(false)
    {
        memory.was_open = false;
        memory.body = None;
        memory.layer = None;
        return;
    }
    let screen = ui.ctx().screen_rect();
    let mut width =
        (contract::number(n, state, "width", 520.) as f32).min((screen.width() - 32.).max(40.));
    let mut height = (contract::number(
        n,
        state,
        "height",
        if drawer {
            320.
        } else {
            (memory.body_height + 104.) as f64
        },
    ) as f32)
        .min((screen.height() - 32.).max(40.));
    let placement = contract::string(n, state, "placement", "right");
    if drawer {
        if matches!(placement, "left" | "right") {
            height = (screen.height() - 32.).max(40.);
        } else {
            width = (screen.width() - 32.).max(40.);
        }
    }
    let position = if !drawer {
        screen.center() - Vec2::new(width, height) / 2.
    } else {
        match placement {
            "left" | "top" => screen.left_top(),
            "bottom" => Pos2::new(screen.left(), screen.bottom() - height),
            _ => Pos2::new(screen.right() - width, screen.top()),
        }
    };
    let mut close = false;
    let mut ok = false;
    let id = egui::Id::new((&n.id, "overlay"));
    let area = if !drawer && contract::flag(n, state, "centered", true) {
        egui::Modal::default_area(id)
    } else {
        egui::Area::new(id)
            .kind(egui::UiKind::Modal)
            .order(egui::Order::Foreground)
            .sense(egui::Sense::hover())
            .fixed_pos(position)
            .interactable(true)
    };
    let style = ui.style().clone();
    let frame = egui::Frame::popup(&style).inner_margin(12.);
    let response = egui::Modal::new(id)
        .area(area)
        .frame(frame)
        .show(ui.ctx(), |ui| {
            ui.set_style(style);
            ui.set_width((width - 24.).max(16.));
            ui.horizontal(|ui| {
                ui.heading(&r.title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if contract::flag(n, state, "closable", true) && ui.button("×").clicked() {
                        close = true;
                    }
                });
            });
            if !r.text.is_empty() {
                ui.label(&r.text);
            }
            let body = egui::Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), (height - 96.).max(40.)),
            );
            memory.body = Some(body);
            memory.layer = Some(ui.layer_id());
            ui.allocate_space(body.size());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if contract::flag(n, state, "footer", !drawer) {
                    if ui
                        .add_enabled(
                            !contract::flag(n, state, "confirmLoading", false),
                            egui::Button::new(
                                egui::RichText::new(contract::string(n, state, "okText", "确定"))
                                    .color(Color32::WHITE),
                            )
                            .fill(token(ui, Token::Accent)),
                        )
                        .clicked()
                    {
                        ok = true;
                    }
                    if ui
                        .button(contract::string(n, state, "cancelText", "取消"))
                        .clicked()
                    {
                        close = true;
                    }
                }
                if drawer
                    && contract::flag(n, state, "closable", true)
                    && ui.button("关闭").clicked()
                {
                    close = true;
                }
            });
        });
    if response.is_top_modal
        && !response.any_popup_open
        && contract::flag(n, state, "keyboard", true)
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        close = true;
    }
    if memory.was_open
        && contract::flag(n, state, "maskClosable", drawer)
        && response.backdrop_response.clicked()
    {
        close = true;
    }
    memory.was_open = true;
    if ok {
        collector.push(ui, "ok", None, Value::Null);
    }
    if close {
        emit(ui, n, memory, collector, "openChange", json!(false));
        collector.push(ui, "cancel", None, Value::Null);
        collector.push(ui, "close", None, Value::Null);
    }
}
