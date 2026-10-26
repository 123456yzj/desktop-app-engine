//! Native host: retained frame traversal, shared styles, state and event collection.
mod charts;
mod fonts;
mod form_controls;
mod text;
mod theme;
pub(in crate::components) mod widgets;
use crate::{
    binding::Binding,
    chart::{self, Point, Transform},
    component::Resolved,
    interaction::{Runtime, UiEvent},
    layout::{self, Rect, Size},
    protocol::{self, Kind, Overflow, Token, Tone},
    table::TableState,
};
use eframe::egui::{self, Color32, FontId, Pos2, Stroke, StrokeKind, Vec2};
pub use fonts::install_chinese_font;
use std::collections::{BTreeMap, HashSet};
use text::{EguiMeasure, decorated_label, label, label_line, style_font, styled_label};
use theme::{control_theme, token};

fn er(r: Rect) -> egui::Rect {
    egui::Rect::from_min_size(Pos2::new(r.x, r.y), Vec2::new(r.w, r.h))
}
fn plot_rect(rect: egui::Rect, legend: bool) -> egui::Rect {
    egui::Rect::from_min_max(
        rect.min + Vec2::new(64., 12.),
        rect.max - Vec2::new(16., if legend { 50. } else { 28. }),
    )
}
fn point_pos(p: Point) -> Pos2 {
    Pos2::new(p.x.clamp(-1e7, 1e7) as f32, p.y.clamp(-1e7, 1e7) as f32)
}
fn series_color(ui: &egui::Ui, i: usize) -> Color32 {
    token(
        ui,
        [Token::Accent, Token::Success, Token::Warning, Token::Danger][i % 4],
    )
}
#[derive(Default)]
struct ChartState {
    version: u64,
    points: Vec<Vec<Option<Point>>>,
    sampled: Vec<Vec<usize>>,
    sample_width: usize,
    sample_range: Option<(u64, u64, u64, u64)>,
    viewport: Option<(chart::Range, chart::Range)>,
    manual: bool,
    hidden: HashSet<usize>,
}
pub struct DashboardApp {
    // Positioning metadata for offscreen input; acceptance still uses rendered PNGs.
    #[cfg(feature = "visual-tests")]
    visual_targets: BTreeMap<String, egui::Rect>,
    embedded: bool,
    host_events: Vec<UiEvent>,
    user_scrolled: HashSet<String>,
    logs: crate::gui_logs::Viewer,
    runtime: Runtime,
    scroll: Vec<Size>,
    inputs: BTreeMap<String, String>,
    tables: BTreeMap<String, TableState>,
    charts: BTreeMap<String, ChartState>,
    splitters: BTreeMap<String, f32>,
    control_values: BTreeMap<String, serde_json::Value>,
    number_edits: BTreeMap<String, form_controls::NumberEdit>,
    widget_memory: BTreeMap<String, widgets::Memory>,
    plugin_textures: BTreeMap<String, egui::TextureHandle>,
    sequence: u64,
    resolved: Vec<Resolved>,
    resolved_version: u64,
}
impl DashboardApp {
    pub fn set_command_base(&mut self, base: std::path::PathBuf) {
        self.runtime.set_command_base(base);
    }
    pub fn new(d: protocol::Description) -> Result<Self, String> {
        Self::from_compiled(crate::compiler::compile(d)?)
    }
    pub fn from_compiled(
        description: crate::compiler::CompiledDescription,
    ) -> Result<Self, String> {
        let runtime = Runtime::from_compiled(description)?;
        let count = runtime.tree.nodes.len();
        Ok(Self {
            #[cfg(feature = "visual-tests")]
            visual_targets: BTreeMap::new(),
            embedded: false,
            host_events: vec![],
            user_scrolled: HashSet::new(),
            logs: crate::gui_logs::Viewer::default(),
            runtime,
            scroll: vec![Size::default(); count],
            inputs: BTreeMap::new(),
            tables: BTreeMap::new(),
            charts: BTreeMap::new(),
            splitters: BTreeMap::new(),
            control_values: BTreeMap::new(),
            number_edits: BTreeMap::new(),
            widget_memory: BTreeMap::new(),
            plugin_textures: BTreeMap::new(),
            sequence: 0,
            resolved: vec![],
            resolved_version: 0,
        })
    }
    /// Embed the renderer in a native controller without the separate diagnostics window.
    pub fn embedded(description: protocol::Description) -> Result<Self, String> {
        let mut app = Self::new(description)?;
        app.embedded = true;
        Ok(app)
    }
    pub fn take_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.host_events)
    }
    pub fn state(&self) -> &crate::binding::State {
        self.runtime.state()
    }
    /// Native hosts own their background jobs. Retain scroll positions by component ID.
    pub fn replace_description(
        &mut self,
        description: protocol::Description,
    ) -> Result<(), String> {
        if self.runtime.commands_running()
            || self.runtime.http_active()
            || self.runtime.has_plugins()
        {
            return Err("动态视图替换不允许丢弃运行中的命令、数据源或插件".into());
        }
        let positions: BTreeMap<_, _> = self
            .runtime
            .tree
            .nodes
            .iter()
            .zip(&self.scroll)
            .map(|(n, p)| (n.spec.id.clone(), *p))
            .collect();
        let mut runtime = Runtime::new(description)?;
        runtime.set_command_base(self.runtime.command_base().to_owned());
        runtime.advance_generation(self.runtime.generation());
        self.scroll = runtime
            .tree
            .nodes
            .iter()
            .map(|n| positions.get(&n.spec.id).copied().unwrap_or_default())
            .collect();
        self.runtime = runtime;
        self.splitters.clear();
        self.control_values.clear();
        self.number_edits.clear();
        self.widget_memory.clear();
        self.resolved.clear();
        self.resolved_version = 0;
        Ok(())
    }
    pub fn scroll_to_end(&mut self, id: &str) {
        if let Some((index, _)) = self
            .runtime
            .tree
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.spec.id == id)
        {
            self.scroll[index].h = 1_000_000.;
        }
    }
    /// Whether wheel or scrollbar input moved this container in the most recent draw.
    pub fn user_scrolled(&self, id: &str) -> bool {
        self.user_scrolled.contains(id)
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        self.render(ctx);
    }
    #[cfg(feature = "visual-tests")]
    pub fn visual_targets(&self) -> BTreeMap<String, egui::Rect> {
        self.visual_targets.clone()
    }
    pub fn draw_in(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.render_inner(ctx, Some(ui));
    }
}
impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.render(ctx);
    }
}
impl DashboardApp {
    fn render(&mut self, ctx: &egui::Context) {
        self.render_inner(ctx, None);
    }
    fn render_inner(&mut self, ctx: &egui::Context, host_ui: Option<&mut egui::Ui>) {
        egui_extras::install_image_loaders(ctx);
        self.user_scrolled.clear();
        self.runtime.poll_commands();
        self.runtime.poll_http();
        self.runtime.poll_plugins();
        if !self.embedded {
            self.logs.show(ctx, self.runtime.error.as_deref());
        }
        for (id, image) in std::mem::take(&mut self.runtime.plugin_images) {
            let image = egui::ColorImage::from_rgba_premultiplied(image.size, &image.rgba);
            self.plugin_textures.insert(
                id.clone(),
                ctx.load_texture(id, image, egui::TextureOptions::LINEAR),
            );
        }
        if self.resolved_version != self.runtime.version() {
            let result = if self.resolved.is_empty() {
                self.runtime.tree.resolve(self.runtime.state())
            } else {
                self.runtime
                    .tree
                    .resolve_changed(
                        self.runtime.state(),
                        self.runtime.key_versions(),
                        self.resolved_version,
                        &mut self.resolved,
                    )
                    .map(|_| std::mem::take(&mut self.resolved))
            };
            match result {
                Ok(r) => {
                    self.resolved = r;
                    self.resolved_version = self.runtime.version();
                }
                Err(e) => self.runtime.error = Some(e),
            }
        }
        let mut events = vec![];
        for (id, node) in self.runtime.tree.nodes.iter().enumerate() {
            if node.spec.kind == Kind::Splitter
                && !matches!(node.spec.value, Some(Binding::State { .. }))
                && let Some(&ratio) = self.splitters.get(&node.spec.id)
            {
                self.resolved[id].value = Some(serde_json::json!(ratio));
            }
        }
        let base_resolved = self.resolved.clone();
        widgets::prepare(
            &self.runtime.tree,
            self.runtime.state(),
            &mut self.resolved,
            &self.control_values,
            &self.widget_memory,
        );
        let embedded = self.embedded;
        let mut paint = |ui: &mut egui::Ui| {
            if let Some(error) = &self.runtime.error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            let v = ui.available_rect_before_wrap();
            let viewport = Rect {
                x: v.min.x,
                y: v.min.y,
                w: v.width(),
                h: v.height(),
            };
            let measurer = EguiMeasure(ui.painter().clone());
            let mut boxes = layout::arrange(
                &self.runtime.tree,
                &self.resolved,
                viewport,
                &mut self.scroll,
                &measurer,
                ctx.pixels_per_point(),
            );
            let pointer = ctx.pointer_hover_pos();
            let mut wheel = ui.input(|i| i.smooth_scroll_delta);
            // Descendants consume first; remaining delta bubbles through ancestor containers.
            if let Some(p) = pointer {
                for id in (0..self.runtime.tree.nodes.len()).rev() {
                    let node = &self.runtime.tree.nodes[id].spec;
                    let b = boxes.boxes[id];
                    if !self.resolved[id].enabled
                        || !b.clip.contains(p.x, p.y)
                        || !b.rect.contains(p.x, p.y)
                    {
                        continue;
                    }
                    if node.kind == Kind::Table {
                        let t = self.tables.entry(node.id.clone()).or_default();
                        let count = node
                            .rows
                            .as_ref()
                            .and_then(|b| b.resolve(self.runtime.state()).ok())
                            .and_then(|v| v.as_array())
                            .map_or(0, Vec::len);
                        let mut dy = -wheel.y;
                        dy = layout::scroll_axis(
                            &mut t.scroll_y,
                            dy,
                            count as f32 * node.row_height.unwrap_or(32.),
                            (b.content.h - 44.).max(0.),
                        );
                        wheel.y = -dy;
                        let mut dx = -wheel.x;
                        dx = layout::scroll_axis(
                            &mut t.scroll_x,
                            dx,
                            t.widths.iter().sum(),
                            b.content.w,
                        );
                        wheel.x = -dx;
                    }
                    if matches!(node.kind, Kind::LineChart | Kind::BarChart)
                        && node.zoom == Some(true)
                        && ui.input(|i| i.modifiers.alt)
                    {
                        let plot = plot_rect(er(b.content), node.legend.unwrap_or(true));
                        let c = self.charts.entry(node.id.clone()).or_default();
                        if plot.contains(p)
                            && wheel.y != 0.
                            && let Some((x, y)) = c.viewport
                        {
                            let t = Transform {
                                x,
                                y,
                                left: plot.left() as f64,
                                top: plot.top() as f64,
                                width: plot.width() as f64,
                                height: plot.height() as f64,
                            };
                            let anchor = t.data(Point {
                                x: p.x as f64,
                                y: p.y as f64,
                            });
                            let factor = (-wheel.y as f64 * 0.005).exp().clamp(0.5, 2.);
                            c.viewport = Some((
                                chart::zoom_range(x, anchor.x, factor),
                                chart::zoom_range(y, anchor.y, factor),
                            ));
                            c.manual = true;
                            wheel = Vec2::ZERO;
                        }
                    }
                    if node.layout.overflow == Overflow::Scroll {
                        let before = self.scroll[id];
                        wheel.y = -layout::scroll_axis(
                            &mut self.scroll[id].h,
                            -wheel.y,
                            b.extent.h,
                            b.content.h,
                        );
                        wheel.x = -layout::scroll_axis(
                            &mut self.scroll[id].w,
                            -wheel.x,
                            b.extent.w,
                            b.content.w,
                        );
                        if self.scroll[id].w != before.w || self.scroll[id].h != before.h {
                            self.user_scrolled.insert(node.id.clone());
                        }
                    }
                }
                boxes = layout::arrange(
                    &self.runtime.tree,
                    &self.resolved,
                    viewport,
                    &mut self.scroll,
                    &measurer,
                    ctx.pixels_per_point(),
                );
            }
            let visible_plugins: Vec<(String, u32, u32)> = self
                .runtime
                .tree
                .nodes
                .iter()
                .enumerate()
                .filter(|(id, n)| {
                    n.spec.kind == Kind::Plugin
                        && self.resolved[*id].visible
                        && boxes.boxes[*id].clip.w > 0.
                        && boxes.boxes[*id].clip.h > 0.
                })
                .map(|(id, n)| {
                    let b = boxes.boxes[id].content;
                    (
                        n.spec.id.clone(),
                        b.w.clamp(1., 4096.) as u32,
                        b.h.clamp(1., 4096.) as u32,
                    )
                })
                .collect();
            self.plugin_textures
                .retain(|id, _| visible_plugins.iter().any(|(visible, _, _)| visible == id));
            if let Err(e) = self.runtime.sync_plugins(visible_plugins) {
                self.runtime.error = Some(e);
            }
            let mut popup_layers = vec![None; self.runtime.tree.nodes.len()];
            #[cfg(feature = "visual-tests")]
            self.visual_targets.clear();
            for (id, node) in self.runtime.tree.nodes.iter().enumerate() {
                let mut spec = node.spec.clone();
                if let Some(locale) = self.resolved[id].api.get("@locale") {
                    spec.api.insert(
                        "@locale".into(),
                        Binding::Literal {
                            value: locale.clone(),
                        },
                    );
                }
                let control_size = self.resolved[id]
                    .api
                    .get("size")
                    .or_else(|| self.resolved[id].api.get("@size"))
                    .and_then(serde_json::Value::as_str);
                if let Some(size) = control_size
                    && spec.style.font_size.is_none()
                {
                    spec.style.font_size = Some(if size == "small" {
                        12.
                    } else if size == "large" {
                        18.
                    } else {
                        16.
                    });
                }
                let n = &spec;
                let b = boxes.boxes[id];
                let r = &self.resolved[id];
                let floating = matches!(
                    n.kind,
                    Kind::Modal
                        | Kind::Drawer
                        | Kind::Message
                        | Kind::Notification
                        | Kind::FloatButton
                        | Kind::Tour
                );
                if !r.visible || !floating && (b.clip.w <= 0. || b.clip.h <= 0.) {
                    continue;
                }
                #[cfg(feature = "visual-tests")]
                if !floating {
                    let visible = er(b.rect).intersect(er(b.clip));
                    if visible.is_positive() {
                        self.visual_targets.insert(n.id.clone(), visible);
                    }
                }
                if let Some(parent) = node.parent {
                    popup_layers[id] = popup_layers[parent];
                }
                let mut popup_ui;
                let ui = if let Some(layer) = popup_layers[id] {
                    popup_ui = egui::Ui::new(
                        ctx.clone(),
                        egui::Id::new((&n.id, "popupChild")),
                        egui::UiBuilder::new().layer_id(layer).max_rect(er(b.clip)),
                    );
                    &mut popup_ui
                } else {
                    &mut *ui
                };
                let previous_clip = ui.clip_rect();
                let previous_style = ui.style().clone();
                if let Some(theme) = r.api.get("@theme").and_then(serde_json::Value::as_str) {
                    ui.set_style(egui::Style {
                        visuals: if theme == "dark" {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        },
                        ..ui.style().as_ref().clone()
                    });
                }
                theme::component_visuals(ui);
                ui.set_clip_rect(er(b.clip));
                let painter = ui.painter().with_clip_rect(er(b.clip));
                let rect = er(b.rect);
                let content = er(b.content);
                let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
                let font = n.style.font_size.unwrap_or(16.);
                // Components with their own frame consume style.background/border themselves.
                let own_frame = matches!(
                    n.kind,
                    Kind::Panel
                        | Kind::Card
                        | Kind::Button
                        | Kind::Input
                        | Kind::Select
                        | Kind::InputNumber
                );
                if let Some(bg) = n.style.background
                    && !own_frame
                {
                    painter.rect_filled(rect, n.style.radius, token(ui, bg));
                }
                match n.kind {
                    Kind::Panel => {
                        widgets::panel::paint(ui, n, rect, content, self.runtime.state())
                    }
                    Kind::Card => widgets::card::paint(ui, n, rect, content, self.runtime.state()),
                    Kind::Space => {
                        widgets::space::paint(ui, n, r, &node.children, &self.resolved, &boxes)
                    }
                    _ => {}
                }
                if n.style.border_width > 0. && !own_frame && !matches!(n.kind, Kind::Divider) {
                    painter.rect_stroke(
                        rect,
                        n.style.radius,
                        Stroke::new(
                            n.style.border_width,
                            token(ui, n.style.border_color.unwrap_or(Token::Border)),
                        ),
                        StrokeKind::Inside,
                    );
                }
                if layout::has_layout_title(n.kind) && !r.title.is_empty() {
                    label(
                        ui,
                        &painter,
                        egui::Rect::from_min_size(
                            rect.min + Vec2::splat(n.layout.padding),
                            Vec2::new(content.width(), font + 12.),
                        ),
                        &r.title,
                        font,
                        color,
                    );
                }
                let mut collector = Collector {
                    events: &mut events,
                    base: self.sequence,
                    generation: self.runtime.generation(),
                    component: &n.id,
                };
                match n.kind {
                    Kind::Checkbox
                    | Kind::Switch
                    | Kind::Radio
                    | Kind::InputNumber
                    | Kind::Slider => {
                        let mut value = r.value.clone().unwrap();
                        if matches!(n.value, Some(Binding::Literal { .. })) {
                            value = self.control_values.get(&n.id).cloned().unwrap_or(value);
                        }
                        let (changed, submit) = form_controls::control(
                            ui,
                            n,
                            r,
                            content,
                            &mut value,
                            self.number_edits.entry(n.id.clone()).or_default(),
                        );
                        if changed {
                            self.control_values.insert(n.id.clone(), value.clone());
                            collector.push(ui, "change", None, value);
                        }
                        if submit {
                            collector.push(ui, "submit", None, serde_json::Value::Null);
                            submit_form(ui, &self.runtime.tree, id, "submit", &mut collector);
                        }
                    }
                    Kind::FormItem => widgets::form_item::paint(
                        ui,
                        &painter,
                        n,
                        r,
                        rect,
                        self.runtime.form_error(&n.id),
                    ),
                    Kind::Divider => widgets::divider::paint(ui, &painter, n, content, &r.text),
                    Kind::Splitter => {
                        let children: Vec<_> = node
                            .children
                            .iter()
                            .copied()
                            .filter(|&child| self.resolved[child].visible)
                            .collect();
                        if children.len() == 2 {
                            let panels = [
                                &self.runtime.tree.nodes[children[0]].spec.layout,
                                &self.runtime.tree.nodes[children[1]].spec.layout,
                            ];
                            if let Some(ratio) = widgets::splitter::paint(
                                ui,
                                &painter,
                                n,
                                content,
                                er(boxes.boxes[children[0]].rect),
                                panels,
                                r.enabled,
                            ) {
                                self.splitters.insert(n.id.clone(), ratio);
                                collector.push(ui, "change", None, serde_json::json!(ratio));
                                ctx.request_repaint();
                            }
                        }
                    }
                    Kind::Plugin => {
                        if let Err(e) = widgets::plugin::paint(
                            ui,
                            &painter,
                            n,
                            content,
                            r.enabled,
                            self.plugin_textures.get(&n.id),
                            &self.runtime.plugins,
                        ) {
                            self.runtime.error = Some(e);
                        }
                    }
                    Kind::Text => {
                        let mut display = r.clone();
                        if !n.api.is_empty() && !matches!(n.text, Some(Binding::State { .. })) {
                            display.text = self
                                .inputs
                                .get(&n.id)
                                .cloned()
                                .unwrap_or_else(|| r.text.clone());
                        }
                        if let Some(text) = widgets::text::paint(
                            ui,
                            n,
                            &display,
                            content,
                            self.runtime.state(),
                            &mut collector,
                        ) {
                            self.inputs.insert(n.id.clone(), text);
                        }
                    }
                    Kind::Metric => widgets::metric::paint(ui, n, r, content, self.runtime.state()),
                    Kind::Progress => {
                        widgets::progress::paint(ui, n, r, content, self.runtime.state())
                    }
                    Kind::Button => {
                        if widgets::button::paint(
                            ui,
                            n,
                            r,
                            rect,
                            content,
                            self.runtime.state(),
                            &mut collector,
                        ) && n.html_type != protocol::ButtonType::Button
                        {
                            submit_form(
                                ui,
                                &self.runtime.tree,
                                id,
                                if n.html_type == protocol::ButtonType::Submit {
                                    "submit"
                                } else {
                                    "reset"
                                },
                                &mut collector,
                            );
                        }
                    }
                    Kind::Input => {
                        let initial = r
                            .value
                            .as_ref()
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let mut text = if matches!(n.value, Some(Binding::Literal { .. })) {
                            self.inputs
                                .entry(n.id.clone())
                                .or_insert_with(|| initial.into())
                                .clone()
                        } else {
                            initial.into()
                        };
                        let before = text.clone();
                        let submit = widgets::input::paint(
                            ui,
                            n,
                            r,
                            content,
                            self.runtime.state(),
                            &mut text,
                            &mut collector,
                        );
                        if before != text {
                            self.inputs.insert(n.id.clone(), text);
                        }
                        if submit {
                            submit_form(ui, &self.runtime.tree, id, "submit", &mut collector);
                        }
                    }
                    Kind::Table => widgets::table::paint(
                        ui,
                        &painter,
                        n,
                        content,
                        r.enabled,
                        self.runtime.state(),
                        self.runtime
                            .tree
                            .binding_version(id, self.runtime.key_versions()),
                        self.tables.entry(n.id.clone()).or_default(),
                        &mut collector,
                    ),
                    Kind::LineChart => widgets::line_chart::paint(
                        ui,
                        &painter,
                        n,
                        content,
                        r.enabled,
                        self.runtime.state(),
                        self.runtime
                            .tree
                            .binding_version(id, self.runtime.key_versions()),
                        self.charts.entry(n.id.clone()).or_default(),
                    ),
                    Kind::BarChart => widgets::bar_chart::paint(
                        ui,
                        &painter,
                        n,
                        content,
                        r.enabled,
                        self.runtime.state(),
                        self.runtime
                            .tree
                            .binding_version(id, self.runtime.key_versions()),
                        self.charts.entry(n.id.clone()).or_default(),
                    ),
                    Kind::Watermark | Kind::BorderBeam | Kind::Spin => {}
                    kind if kind == Kind::Badge
                        || kind == Kind::Select
                        || protocol::widgets::is_widget(kind) =>
                    {
                        let mut value = r.value.clone().unwrap_or(serde_json::Value::Null);
                        if kind == Kind::Select && !matches!(n.value, Some(Binding::State { .. })) {
                            value = self.control_values.get(&n.id).cloned().unwrap_or(value);
                        }
                        let memory = self.widget_memory.entry(n.id.clone()).or_default();
                        if matches!(kind, Kind::Modal | Kind::Drawer) {
                            let width =
                                (protocol::widgets::number(n, self.runtime.state(), "width", 520.)
                                    as f32)
                                    .min(v.width() - 32.)
                                    - 24.;
                            memory.body_height = self.runtime.tree.nodes[id]
                                .children
                                .iter()
                                .filter(|&&child| self.resolved[child].visible)
                                .map(|&child| {
                                    layout::measure(
                                        &self.runtime.tree,
                                        &self.resolved,
                                        child,
                                        layout::Constraint {
                                            width: layout::Limit::Bounded(width),
                                            height: layout::Limit::Unbounded,
                                        },
                                        &measurer,
                                    )
                                    .h + n.layout.gap
                                })
                                .sum::<f32>()
                                .max(40.);
                        }
                        let widget_rect = widgets::header_rect(n, r, rect, content);
                        widgets::paint(
                            ui,
                            n,
                            r,
                            widget_rect,
                            self.runtime.state(),
                            memory,
                            &mut value,
                            &mut collector,
                        );
                        if !matches!(n.value, Some(Binding::State { .. })) {
                            self.control_values.insert(n.id.clone(), value);
                        }
                        if matches!(kind, Kind::Modal | Kind::Drawer)
                            && let (Some(body), Some(layer)) = (memory.body, memory.layer)
                        {
                            popup_layers[id] = Some(layer);
                            layout::arrange_children_at(
                                &self.runtime.tree,
                                &self.resolved,
                                id,
                                Rect {
                                    x: body.min.x,
                                    y: body.min.y,
                                    w: body.width(),
                                    h: body.height(),
                                },
                                &mut self.scroll,
                                &measurer,
                                ctx.pixels_per_point(),
                                &mut boxes,
                            );
                        }
                    }
                    _ => {}
                }
                if n.layout.overflow == Overflow::Scroll {
                    let before = self.scroll[id];
                    scrollbar(
                        ui,
                        &painter,
                        (&n.id, "vertical"),
                        egui::Rect::from_min_size(
                            Pos2::new(content.right(), content.top()),
                            Vec2::new(12., content.height()),
                        ),
                        &mut self.scroll[id].h,
                        b.extent.h,
                        content.height(),
                        false,
                        r.enabled,
                    );
                    scrollbar(
                        ui,
                        &painter,
                        (&n.id, "horizontal"),
                        egui::Rect::from_min_size(
                            Pos2::new(content.left(), content.bottom()),
                            Vec2::new(content.width(), 12.),
                        ),
                        &mut self.scroll[id].w,
                        b.extent.w,
                        content.width(),
                        true,
                        r.enabled,
                    );
                    if self.scroll[id].w != before.w || self.scroll[id].h != before.h {
                        self.user_scrolled.insert(n.id.clone());
                    }
                }
                ui.set_clip_rect(previous_clip);
                ui.set_style(previous_style);
            }
            // Decorations paint after their retained children so backgrounds cannot cover them.
            for (id, node) in self.runtime.tree.nodes.iter().enumerate() {
                if !matches!(
                    node.spec.kind,
                    Kind::Watermark | Kind::BorderBeam | Kind::Spin
                ) || !self.resolved[id].visible
                {
                    continue;
                }
                let b = boxes.boxes[id];
                if b.clip.w <= 0. || b.clip.h <= 0. {
                    continue;
                }
                let layer = popup_layers[id].unwrap_or_else(|| ui.layer_id());
                let mut overlay = egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new((&node.spec.id, "decoration")),
                    egui::UiBuilder::new().layer_id(layer).max_rect(er(b.rect)),
                );
                overlay.set_clip_rect(er(b.clip));
                let mut collector = Collector {
                    events: &mut events,
                    base: self.sequence,
                    generation: self.runtime.generation(),
                    component: &node.spec.id,
                };
                widgets::paint(
                    &mut overlay,
                    &node.spec,
                    &self.resolved[id],
                    er(b.rect),
                    self.runtime.state(),
                    self.widget_memory.entry(node.spec.id.clone()).or_default(),
                    &mut serde_json::Value::Null,
                    &mut collector,
                );
            }
        };
        if let Some(ui) = host_ui {
            paint(ui);
        } else {
            let panel = egui::CentralPanel::default();
            let panel = if embedded {
                panel.frame(egui::Frame::NONE)
            } else {
                panel
            };
            panel.show(ctx, paint);
        }
        self.resolved = base_resolved;
        self.sequence = (self.sequence + ctx.input(|i| i.events.len() as u64 + 2) * 4)
            .max(events.last().map_or(0, |e| e.sequence + 1));
        if !events.is_empty() {
            if self.embedded {
                self.host_events.extend(events.iter().cloned());
            }
            self.runtime.commit(events);
            ctx.request_repaint();
        }
        if self.runtime.commands_running()
            || self.runtime.http_active()
            || self.runtime.has_plugins()
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
    }
}
fn submit_form(
    ui: &egui::Ui,
    tree: &crate::component::Tree,
    id: usize,
    kind: &str,
    collector: &mut Collector<'_>,
) {
    if let Some(form) = tree.ancestor(id, Kind::Form) {
        collector.push_for(ui, &tree.nodes[form].spec.id, kind);
    }
}
struct Collector<'a> {
    events: &'a mut Vec<UiEvent>,
    base: u64,
    generation: u64,
    component: &'a str,
}
impl Collector<'_> {
    fn push_for(&mut self, ui: &egui::Ui, component: &str, kind: &str) {
        self.push(ui, kind, None, serde_json::Value::Null);
        let event = self.events.last_mut().unwrap();
        event.component_id = component.to_owned();
        event.sequence += 1;
    }
    fn push(&mut self, ui: &egui::Ui, kind: &str, key: Option<String>, payload: serde_json::Value) {
        let index = ui.input(|i| {
            i.events
                .iter()
                .rposition(|e| {
                    if kind == "change" {
                        matches!(
                            e,
                            egui::Event::Text(_)
                                | egui::Event::Paste(_)
                                | egui::Event::Key { .. }
                                | egui::Event::Ime(_)
                        )
                    } else {
                        matches!(
                            e,
                            egui::Event::PointerButton { pressed: false, .. }
                                | egui::Event::Key { pressed: false, .. }
                        )
                    }
                })
                .unwrap_or(i.events.len())
        });
        let sequence = (self.base + index as u64 * 4 + u64::from(kind == "submit"))
            .max(self.events.last().map_or(self.base, |e| e.sequence + 1));
        self.events.push(UiEvent {
            component_id: self.component.into(),
            target_key: key,
            kind: kind.into(),
            payload,
            sequence,
            generation: self.generation,
        });
    }
}
#[allow(clippy::too_many_arguments)]
fn scrollbar(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    id: impl std::hash::Hash,
    track: egui::Rect,
    offset: &mut f32,
    extent: f32,
    viewport: f32,
    horizontal: bool,
    enabled: bool,
) {
    if extent <= viewport || viewport <= 0. {
        return;
    }
    p.rect_filled(track, 3., token(ui, Token::SurfaceRaised));
    let length = if horizontal {
        track.width()
    } else {
        track.height()
    };
    let handle = (length * viewport / extent).max(18.).min(length);
    let travel = (length - handle).max(1.);
    let position = *offset / (extent - viewport) * travel;
    let thumb = if horizontal {
        egui::Rect::from_min_size(track.min + Vec2::new(position, 1.), Vec2::new(handle, 10.))
    } else {
        egui::Rect::from_min_size(track.min + Vec2::new(1., position), Vec2::new(10., handle))
    };
    let response = ui.interact(
        track,
        egui::Id::new(id),
        if enabled {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        },
    );
    if (response.dragged() || response.clicked())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let point = if horizontal {
            pointer.x - track.left()
        } else {
            pointer.y - track.top()
        };
        *offset =
            ((point - handle / 2.) / travel * (extent - viewport)).clamp(0., extent - viewport);
        ui.ctx().request_repaint();
    }
    p.rect_filled(thumb, 4., token(ui, Token::TextMuted));
}
