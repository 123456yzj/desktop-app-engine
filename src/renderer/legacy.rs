use crate::{
    description::{Component, Property},
    runtime::Engine,
    validation::MAX_STRING_BYTES,
};
use eframe::egui;
use std::collections::BTreeMap;

pub fn install_chinese_font(ctx: &egui::Context) -> Result<(), String> {
    let windows = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    let mut fonts = egui::FontDefinitions::default();
    let bytes = ["msyh.ttc", "simhei.ttf", "simsun.ttc"]
        .iter()
        .find_map(|name| {
            std::fs::read(std::path::Path::new(&windows).join("Fonts").join(name)).ok()
        })
        .ok_or("未找到中文系统字体（微软雅黑、黑体或宋体），请安装 Windows 中文字体")?;
    fonts
        .font_data
        .insert("chinese".into(), egui::FontData::from_owned(bytes).into());
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("chinese".into());
    }
    let mut bold = Vec::new();
    for name in ["segoeuib.ttf", "msyhbd.ttc"] {
        if let Ok(bytes) = std::fs::read(std::path::Path::new(&windows).join("Fonts").join(name)) {
            fonts
                .font_data
                .insert(name.into(), egui::FontData::from_owned(bytes).into());
            bold.push(name.into());
        }
    }
    bold.extend(fonts.families[&egui::FontFamily::Proportional].clone());
    fonts
        .families
        .insert(egui::FontFamily::Name("bold".into()), bold);
    ctx.set_fonts(fonts);
    Ok(())
}

pub struct DesktopApp {
    logs: crate::gui_logs::Viewer,
    engine: Engine,
    root: Component,
    local_inputs: BTreeMap<String, String>,
    render_error: Option<String>,
}

impl DesktopApp {
    pub fn new(engine: Engine) -> Self {
        Self {
            logs: crate::gui_logs::Viewer::default(),
            root: engine.description().root.clone(),
            engine,
            local_inputs: BTreeMap::new(),
            render_error: None,
        }
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.logs
            .show(ctx, self.engine.error().or(self.render_error.as_deref()));
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = self.engine.error().or(self.render_error.as_deref()) {
                ui.colored_label(ui.visuals().error_fg_color, error);
                ui.separator();
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Err(error) =
                        render(ui, &self.root, &mut self.engine, &mut self.local_inputs)
                    {
                        self.render_error = Some(error);
                    }
                });
        });
        // Actions consume explicit click events. A repaint with no event has no effects.
        self.engine.process_pending();
    }
}

fn render(
    ui: &mut egui::Ui,
    node: &Component,
    engine: &mut Engine,
    inputs: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    ui.push_id(node.id(), |ui| -> Result<(), String> {
        match node {
            Component::Column { children, .. } => {
                ui.vertical(|ui| {
                    for child in children {
                        render(ui, child, engine, inputs)?;
                    }
                    Ok::<(), String>(())
                })
                .inner?;
            }
            Component::Row { children, .. } => {
                ui.horizontal_wrapped(|ui| {
                    for child in children {
                        render(ui, child, engine, inputs)?;
                    }
                    Ok::<(), String>(())
                })
                .inner?;
            }
            Component::Text { text, .. } => {
                ui.add(egui::Label::new(engine.resolve(text)?).wrap());
            }
            Component::Input {
                id,
                value,
                placeholder,
            } => {
                let hint = placeholder
                    .as_ref()
                    .map(|p| engine.resolve(p))
                    .transpose()?
                    .unwrap_or_default();
                let mut text = match value {
                    Property::State { .. } => engine.resolve(value)?,
                    Property::Literal { value } => inputs
                        .entry(id.clone())
                        .or_insert_with(|| value.clone())
                        .clone(),
                };
                let response = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .id(egui::Id::new(("input", id)))
                        .hint_text(hint)
                        .desired_width(220.0_f32.min((ui.available_width() - 64.0).max(60.0)))
                        .char_limit(MAX_STRING_BYTES / 4),
                );
                if response.changed() {
                    match value {
                        Property::State { key } => engine.set_state(key, text)?,
                        Property::Literal { .. } => {
                            inputs.insert(id.clone(), text);
                        }
                    }
                    ui.ctx().request_repaint();
                }
            }
            Component::Button { text, on_click, .. } => {
                if ui.button(engine.resolve(text)?).clicked() {
                    engine.trigger(on_click)?;
                    ui.ctx().request_repaint();
                }
            }
        }
        Ok(())
    })
    .inner
}
