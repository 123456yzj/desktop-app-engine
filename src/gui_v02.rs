//! egui is a paint/input/text host. All component rectangles come from the engine.
use desktop_app_engine::{
    binding::Binding,
    chart::{self, Point, Transform},
    component::Resolved,
    interaction::{Runtime, UiEvent},
    layout::{self, Rect, Size, TextMeasure},
    table::{self, TableState},
    v02::{self, Kind, Overflow, Token, Tone},
};
use eframe::egui::{self, Color32, FontId, Pos2, Stroke, StrokeKind, Vec2};
use std::collections::{BTreeMap, HashSet};

struct EguiMeasure(egui::Painter);
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
        Size {
            w: g.size().x,
            h: g.size().y,
        }
    }
    fn text(&self, text: &str, width: f32, font_size: f32) -> Size {
        let g = self.0.layout(
            text.to_owned(),
            FontId::proportional(font_size),
            Color32::WHITE,
            width.max(1.0),
        );
        Size {
            w: g.size().x,
            h: g.size().y,
        }
    }
}
fn er(r: Rect) -> egui::Rect {
    egui::Rect::from_min_size(Pos2::new(r.x, r.y), Vec2::new(r.w, r.h))
}
fn plot_rect(rect: egui::Rect, legend: bool) -> egui::Rect {
    egui::Rect::from_min_max(
        rect.min + Vec2::new(64.0, 12.0),
        rect.max - Vec2::new(16.0, if legend { 50.0 } else { 28.0 }),
    )
}
fn point_pos(p: Point) -> Pos2 {
    Pos2::new(p.x.clamp(-1e7, 1e7) as f32, p.y.clamp(-1e7, 1e7) as f32)
}
fn token(ui: &egui::Ui, t: Token) -> Color32 {
    let v = ui.visuals();
    match t {
        Token::Surface => v.panel_fill,
        Token::SurfaceRaised => v.faint_bg_color,
        Token::TextPrimary => v.text_color(),
        Token::TextMuted => v.weak_text_color(),
        Token::Border => v.widgets.noninteractive.bg_stroke.color,
        Token::Accent => v.selection.bg_fill,
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
fn style_font(style: &v02::Style, ctx: &egui::Context) -> FontId {
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
fn styled_label(
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
    let pos = match style.text_align {
        Some(v02::TextAlign::Center) => {
            r.min
                + Vec2::new(
                    (r.width() - size.x).max(0.0) / 2.0,
                    (r.height() - size.y).max(0.0) / 2.0,
                )
        }
        Some(v02::TextAlign::End) => r.min + Vec2::new((r.width() - size.x).max(0.0), 0.0),
        _ => r.min,
    };
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
fn control_theme(ui: &mut egui::Ui, style: &v02::Style, height: f32) {
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
fn series_color(ui: &egui::Ui, i: usize) -> Color32 {
    token(
        ui,
        [Token::Accent, Token::Success, Token::Warning, Token::Danger][i % 4],
    )
}
fn label(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    r: egui::Rect,
    text: &str,
    font: f32,
    color: Color32,
) {
    text_label(ui, painter, r, text, font, color, true);
}
fn label_line(
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
    embedded: bool,
    host_events: Vec<UiEvent>,
    open_selects: HashSet<String>,
    user_scrolled: HashSet<String>,
    logs: crate::gui_logs::Viewer,
    runtime: Runtime,
    scroll: Vec<Size>,
    inputs: BTreeMap<String, String>,
    tables: BTreeMap<String, TableState>,
    charts: BTreeMap<String, ChartState>,
    plugin_textures: BTreeMap<String, egui::TextureHandle>,
    sequence: u64,
    resolved: Vec<Resolved>,
    resolved_version: u64,
}
impl DashboardApp {
    pub fn set_command_base(&mut self, base: std::path::PathBuf) {
        self.runtime.command_base = base;
    }
    pub fn new(d: v02::Description) -> Result<Self, String> {
        let runtime = Runtime::new(d)?;
        let count = runtime.tree.nodes.len();
        Ok(Self {
            embedded: false,
            host_events: vec![],
            open_selects: HashSet::new(),
            user_scrolled: HashSet::new(),
            logs: crate::gui_logs::Viewer::default(),
            runtime,
            scroll: vec![Size::default(); count],
            inputs: BTreeMap::new(),
            tables: BTreeMap::new(),
            charts: BTreeMap::new(),
            plugin_textures: BTreeMap::new(),
            sequence: 0,
            resolved: vec![],
            resolved_version: 0,
        })
    }
    /// Embed the renderer in a native controller without the separate diagnostics window.
    pub fn embedded(description: v02::Description) -> Result<Self, String> {
        let mut app = Self::new(description)?;
        app.embedded = true;
        Ok(app)
    }
    pub fn take_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.host_events)
    }
    pub fn state(&self) -> &desktop_app_engine::binding::State {
        &self.runtime.state
    }
    /// Native hosts own their background jobs. Retain scroll positions by component ID.
    pub fn replace_description(&mut self, description: v02::Description) -> Result<(), String> {
        if self.runtime.commands_running()
            || self.runtime.http_active()
            || !self.runtime.description.plugins.is_empty()
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
        runtime.command_base = self.runtime.command_base.clone();
        runtime.generation = self.runtime.generation + 1;
        self.scroll = runtime
            .tree
            .nodes
            .iter()
            .map(|n| positions.get(&n.spec.id).copied().unwrap_or_default())
            .collect();
        self.runtime = runtime;
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
            self.scroll[index].h = 1_000_000.0;
        }
    }
    /// Whether wheel or scrollbar input moved this container in the most recent draw.
    pub fn user_scrolled(&self, id: &str) -> bool {
        self.user_scrolled.contains(id)
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        self.render(ctx);
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
        if self.resolved_version != self.runtime.version {
            let result = if self.resolved.is_empty() {
                self.runtime.tree.resolve(&self.runtime.state)
            } else {
                self.runtime
                    .tree
                    .resolve_changed(
                        &self.runtime.state,
                        &self.runtime.key_versions,
                        self.resolved_version,
                        &mut self.resolved,
                    )
                    .map(|_| std::mem::take(&mut self.resolved))
            };
            match result {
                Ok(r) => {
                    self.resolved = r;
                    self.resolved_version = self.runtime.version;
                }
                Err(e) => self.runtime.error = Some(e),
            }
        }
        let mut events = vec![];
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
                            .and_then(|b| b.resolve(&self.runtime.state).ok())
                            .and_then(|v| v.as_array())
                            .map_or(0, Vec::len);
                        let mut dy = -wheel.y;
                        dy = layout::scroll_axis(
                            &mut t.scroll_y,
                            dy,
                            count as f32 * node.row_height.unwrap_or(32.0),
                            (b.content.h - 44.0).max(0.0),
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
                    {
                        let plot = plot_rect(er(b.content), node.legend.unwrap_or(true));
                        let c = self.charts.entry(node.id.clone()).or_default();
                        if plot.contains(p)
                            && wheel.y != 0.0
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
                            let factor = (-wheel.y as f64 * 0.005).exp().clamp(0.5, 2.0);
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
                        && boxes.boxes[*id].clip.w > 0.0
                        && boxes.boxes[*id].clip.h > 0.0
                })
                .map(|(id, n)| {
                    let b = boxes.boxes[id].content;
                    (
                        n.spec.id.clone(),
                        b.w.clamp(1.0, 4096.0) as u32,
                        b.h.clamp(1.0, 4096.0) as u32,
                    )
                })
                .collect();
            self.plugin_textures
                .retain(|id, _| visible_plugins.iter().any(|(visible, _, _)| visible == id));
            if let Err(e) = self.runtime.sync_plugins(visible_plugins) {
                self.runtime.error = Some(e);
            }
            for (id, node) in self.runtime.tree.nodes.iter().enumerate() {
                let n = &node.spec;
                let b = boxes.boxes[id];
                let r = &self.resolved[id];
                if !r.visible || b.clip.w <= 0.0 || b.clip.h <= 0.0 {
                    continue;
                }
                let previous_clip = ui.clip_rect();
                let previous_style = ui.style().clone();
                ui.set_clip_rect(er(b.clip));
                let painter = ui.painter().with_clip_rect(er(b.clip));
                let rect = er(b.rect);
                let content = er(b.content);
                let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
                let font = n.style.font_size.unwrap_or(16.0);
                if let Some(bg) = n.style.background {
                    painter.rect_filled(rect, n.style.radius, token(ui, bg));
                }
                if matches!(n.kind, Kind::Panel | Kind::Card) {
                    painter.rect_filled(
                        rect,
                        n.style.radius.max(6.0),
                        token(ui, n.style.background.unwrap_or(Token::SurfaceRaised)),
                    );
                }
                if n.style.border_width > 0.0
                    && !matches!(n.kind, Kind::Button | Kind::Input | Kind::Select)
                {
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
                if !r.title.is_empty() {
                    label(
                        ui,
                        &painter,
                        egui::Rect::from_min_size(
                            rect.min + Vec2::splat(n.layout.padding),
                            Vec2::new(content.width(), font + 12.0),
                        ),
                        &r.title,
                        font,
                        color,
                    );
                }
                let mut collector = Collector {
                    events: &mut events,
                    base: self.sequence,
                    generation: self.runtime.generation,
                    component: &n.id,
                };
                match n.kind {
                    Kind::Plugin => {
                        if let Err(e) = paint_plugin(
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
                    Kind::Text => styled_label(ui, &painter, content, &r.text, &n.style, color),
                    Kind::Metric => {
                        label(
                            ui,
                            &painter,
                            content,
                            &r.label,
                            font,
                            token(ui, Token::TextMuted),
                        );
                        label(
                            ui,
                            &painter,
                            egui::Rect::from_min_max(
                                content.min + Vec2::new(0.0, font + 12.0),
                                content.max,
                            ),
                            &r.text,
                            font * 1.875,
                            color,
                        );
                    }
                    Kind::Badge => {
                        let tone = match n.tone.unwrap_or_default() {
                            Tone::Accent => Token::Accent,
                            Tone::Success => Token::Success,
                            Tone::Warning => Token::Warning,
                            Tone::Danger => Token::Danger,
                            Tone::Muted => Token::TextMuted,
                        };
                        painter.rect_filled(content, 6.0, token(ui, tone).gamma_multiply(0.15));
                        let mut style = n.style.clone();
                        style.text_wrap = Some(false);
                        style.text_align = Some(v02::TextAlign::Center);
                        styled_label(
                            ui,
                            &painter,
                            content.shrink2(Vec2::new(
                                n.style.padding_x.unwrap_or(10.0),
                                n.style.padding_y.unwrap_or(2.0),
                            )),
                            &r.text,
                            &style,
                            token(ui, tone),
                        );
                    }
                    Kind::Progress => {
                        let value = r.value.as_ref().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let bar = egui::Rect::from_min_size(
                            content.min + Vec2::new(0.0, content.height() - 8.0),
                            Vec2::new(content.width(), 8.0),
                        );
                        painter.rect_filled(bar, 4.0, token(ui, Token::Border));
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                bar.min,
                                Vec2::new(bar.width() * value, bar.height()),
                            ),
                            4.0,
                            token(ui, Token::Accent),
                        );
                        label(ui, &painter, content, &r.text, font, color);
                    }
                    Kind::Button => {
                        let response = ui.interact(
                            content,
                            egui::Id::new((&n.id, "button")),
                            if r.enabled {
                                egui::Sense::click()
                            } else {
                                egui::Sense::hover()
                            },
                        );
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::Button, r.enabled, &r.text)
                        });
                        let fill = if !r.enabled {
                            ui.visuals().widgets.noninteractive.bg_fill
                        } else if let Some(background) = n.style.background {
                            let base = token(ui, background);
                            if response.is_pointer_button_down_on() {
                                base.gamma_multiply(0.8)
                            } else if response.hovered() {
                                base.gamma_multiply(0.9)
                            } else {
                                base
                            }
                        } else if response.is_pointer_button_down_on() {
                            ui.visuals().widgets.active.bg_fill
                        } else if response.hovered() {
                            ui.visuals().widgets.hovered.bg_fill
                        } else {
                            ui.visuals().widgets.inactive.bg_fill
                        };
                        let state = if !r.enabled {
                            n.style.disabled.as_ref()
                        } else if response.is_pointer_button_down_on() {
                            n.style.pressed.as_ref()
                        } else if response.hovered() {
                            n.style.hover.as_ref()
                        } else {
                            None
                        };
                        let fill = state
                            .and_then(|s| s.background)
                            .map(|c| token(ui, c))
                            .unwrap_or(fill);
                        let border = state
                            .and_then(|s| s.border_color)
                            .or(n.style.border_color)
                            .unwrap_or(Token::Border);
                        painter.rect_filled(rect, n.style.radius.max(4.0), fill);
                        painter.rect_stroke(
                            rect,
                            n.style.radius.max(4.0),
                            Stroke::new(
                                if n.style.border_color.is_some() {
                                    n.style.border_width
                                } else {
                                    1.0
                                },
                                if response.has_focus() {
                                    token(ui, Token::Focus)
                                } else {
                                    token(ui, border)
                                },
                            ),
                            StrokeKind::Inside,
                        );
                        styled_label(
                            ui,
                            &painter,
                            rect.shrink2(Vec2::new(
                                n.style.padding_x.unwrap_or(10.0),
                                n.style.padding_y.unwrap_or(6.0),
                            )),
                            &r.text,
                            &n.style,
                            if let Some(c) = state.and_then(|s| s.color) {
                                token(ui, c)
                            } else if r.enabled {
                                color
                            } else {
                                token(ui, Token::TextMuted)
                            },
                        );
                        if response.clicked() {
                            collector.push(ui, "click", None, serde_json::Value::Null);
                        }
                    }
                    Kind::Input => {
                        control_theme(ui, &n.style, content.height());
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
                        let response = ui
                            .add_enabled_ui(r.enabled, |ui| {
                                ui.put(
                                    content,
                                    egui::TextEdit::singleline(&mut text)
                                        .text_color(color)
                                        .font(style_font(&n.style, ui.ctx()))
                                        .id(egui::Id::new((&n.id, "input")))
                                        .hint_text(&r.placeholder)
                                        .char_limit(crate::MAX_INPUT_CHARS),
                                )
                            })
                            .inner;
                        if response.changed() {
                            self.inputs.insert(n.id.clone(), text.clone());
                            collector.push(ui, "change", None, serde_json::Value::String(text));
                        }
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            collector.push(ui, "submit", None, serde_json::Value::Null);
                        }
                    }
                    Kind::Select => {
                        control_theme(ui, &n.style, content.height());
                        let options =
                            v02::select_options(n, &self.runtime.state).unwrap_or_default();
                        let initial = r
                            .value
                            .as_ref()
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let mut selected = if matches!(n.value, Some(Binding::Literal { .. })) {
                            self.inputs
                                .get(&n.id)
                                .map(String::as_str)
                                .unwrap_or(initial)
                                .to_owned()
                        } else {
                            initial.to_owned()
                        };
                        let before = selected.clone();
                        let display = options
                            .iter()
                            .find(|o| o.value == selected)
                            .map(|o| o.label.as_str())
                            .unwrap_or(if selected.is_empty() {
                                &r.placeholder
                            } else {
                                &selected
                            })
                            .to_owned();
                        let mut open = false;
                        ui.add_enabled_ui(r.enabled, |ui| {
                            ui.push_id((&n.id, "select"), |ui| {
                                let row_height = (ui.text_style_height(&egui::TextStyle::Button)
                                    + 2.0 * ui.spacing().button_padding.y)
                                    .max(ui.spacing().interact_size.y);
                                let row_spacing = ui.spacing().item_spacing.y;
                                let response = ui.put(content, egui::Button::new(""));
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::ComboBox,
                                        r.enabled,
                                        &display,
                                    )
                                });
                                let visuals = ui.style().interact(&response);
                                let text_rect = egui::Rect::from_min_max(
                                    content.min + Vec2::new(ui.spacing().button_padding.x, 0.0),
                                    content.max - Vec2::new(24.0, 0.0),
                                );
                                let galley = ui.painter().layout_no_wrap(
                                    display.clone(),
                                    style_font(&n.style, ui.ctx()),
                                    visuals.text_color(),
                                );
                                ui.painter()
                                    .with_clip_rect(text_rect.intersect(ui.clip_rect()))
                                    .galley(
                                        Pos2::new(
                                            text_rect.left(),
                                            text_rect.center().y - galley.size().y / 2.0,
                                        ),
                                        galley,
                                        visuals.text_color(),
                                    );
                                let center = Pos2::new(content.right() - 13.0, content.center().y);
                                ui.painter().add(egui::Shape::convex_polygon(
                                    vec![
                                        center + Vec2::new(-4.0, -2.0),
                                        center + Vec2::new(4.0, -2.0),
                                        center + Vec2::new(0.0, 3.0),
                                    ],
                                    visuals.text_color(),
                                    Stroke::NONE,
                                ));
                                let popup_id = ui.make_persistent_id("popup");
                                if response.clicked() {
                                    ui.memory_mut(|m| m.toggle_popup(popup_id));
                                }
                                let screen = ui.ctx().screen_rect();
                                let below =
                                    (screen.bottom() - response.rect.bottom() - 12.0).max(0.0);
                                let above = (response.rect.top() - screen.top() - 12.0).max(0.0);
                                let count = options.len().clamp(1, 10) as f32;
                                let desired = count * row_height + (count - 1.0) * row_spacing;
                                let (direction, available) = if below < desired && above > below {
                                    (egui::AboveOrBelow::Above, above)
                                } else {
                                    (egui::AboveOrBelow::Below, below)
                                };
                                let height = desired.min(available);
                                let popup_style = ui.style().clone();
                                let result = egui::popup::popup_above_or_below_widget(
                                    ui,
                                    popup_id,
                                    &response,
                                    direction,
                                    egui::PopupCloseBehavior::CloseOnClickOutside,
                                    |ui| {
                                        ui.set_style(popup_style);
                                        ui.spacing_mut().interact_size.y = row_height;
                                        ui.spacing_mut().item_spacing.y = row_spacing;
                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                        // The popup Area remembers its previous size. Override the
                                        // scroll minimum as well as its maximum so a loading row
                                        // cannot permanently constrain a later full option list.
                                        egui::ScrollArea::vertical()
                                            .min_scrolled_height(height)
                                            .max_height(height)
                                            .show(ui, |ui| {
                                                for option in &options {
                                                    ui.add_enabled_ui(!option.disabled, |ui| {
                                                        if ui
                                                            .selectable_value(
                                                                &mut selected,
                                                                option.value.clone(),
                                                                &option.label,
                                                            )
                                                            .clicked()
                                                        {
                                                            ui.memory_mut(|m| m.close_popup());
                                                        }
                                                    });
                                                }
                                            });
                                    },
                                );
                                open = result.is_some();
                            });
                        });
                        if open && self.open_selects.insert(n.id.clone()) {
                            collector.push(ui, "open", None, serde_json::Value::Null);
                        } else if !open {
                            self.open_selects.remove(&n.id);
                        }
                        if selected != before {
                            self.inputs.insert(n.id.clone(), selected.clone());
                            collector.push(ui, "change", None, serde_json::Value::String(selected));
                        }
                    }
                    Kind::Table => paint_table(
                        ui,
                        &painter,
                        n,
                        content,
                        r.enabled,
                        &self.runtime.state,
                        self.runtime
                            .tree
                            .binding_version(id, &self.runtime.key_versions),
                        self.tables.entry(n.id.clone()).or_default(),
                        &mut collector,
                    ),
                    Kind::LineChart | Kind::BarChart => paint_chart(
                        ui,
                        &painter,
                        n,
                        content,
                        r.enabled,
                        &self.runtime.state,
                        self.runtime
                            .tree
                            .binding_version(id, &self.runtime.key_versions),
                        self.charts.entry(n.id.clone()).or_default(),
                    ),
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
                            Vec2::new(12.0, content.height()),
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
                            Vec2::new(content.width(), 12.0),
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
        self.sequence += ctx.input(|i| i.events.len() as u64 + 2) * 4;
        if !events.is_empty() {
            if self.embedded {
                self.host_events.extend(events.iter().cloned());
            }
            self.runtime.commit(events);
            ctx.request_repaint();
        }
        if self.runtime.commands_running()
            || self.runtime.http_active()
            || !self.runtime.description.plugins.is_empty()
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
    }
}
struct Collector<'a> {
    events: &'a mut Vec<UiEvent>,
    base: u64,
    generation: u64,
    component: &'a str,
}
fn paint_plugin(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    n: &v02::Node,
    content: egui::Rect,
    enabled: bool,
    texture: Option<&egui::TextureHandle>,
    host: &desktop_app_engine::plugin::Host,
) -> Result<(), String> {
    if let Some(texture) = texture {
        painter.image(
            texture.id(),
            content,
            egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
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
        host.send(&n.id, serde_json::json!({"op":"input","event":{"kind":"click","x":p.x-content.left(),"y":p.y-content.top()}}), None, None)?;
    }
    Ok(())
}
impl Collector<'_> {
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
        self.events.push(UiEvent {
            component_id: self.component.into(),
            target_key: key,
            kind: kind.into(),
            payload,
            sequence: self.base + (index as u64) * 4 + u64::from(kind == "submit"),
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
    if extent <= viewport || viewport <= 0.0 {
        return;
    }
    p.rect_filled(track, 3.0, token(ui, Token::SurfaceRaised));
    let length = if horizontal {
        track.width()
    } else {
        track.height()
    };
    let handle = (length * viewport / extent).max(18.0).min(length);
    let travel = (length - handle).max(1.0);
    let position = *offset / (extent - viewport) * travel;
    let thumb = if horizontal {
        egui::Rect::from_min_size(
            track.min + Vec2::new(position, 1.0),
            Vec2::new(handle, 10.0),
        )
    } else {
        egui::Rect::from_min_size(
            track.min + Vec2::new(1.0, position),
            Vec2::new(10.0, handle),
        )
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
            ((point - handle / 2.0) / travel * (extent - viewport)).clamp(0.0, extent - viewport);
        ui.ctx().request_repaint();
    }
    p.rect_filled(thumb, 4.0, token(ui, Token::TextMuted));
}

#[allow(clippy::too_many_arguments)]
fn paint_table(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    n: &v02::Node,
    rect: egui::Rect,
    enabled: bool,
    state: &desktop_app_engine::binding::State,
    version: u64,
    t: &mut TableState,
    events: &mut Collector<'_>,
) {
    let rows = n
        .rows
        .as_ref()
        .unwrap()
        .resolve(state)
        .unwrap()
        .as_array()
        .unwrap();
    let columns: Vec<_> = table::columns(n).collect();
    let key = n.row_key.as_ref().unwrap();
    let h = n.row_height.unwrap_or(32.0);
    let font = n.style.font_size.unwrap_or(14.0);
    let text_color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    if t.version != version || t.widths.len() != columns.len() {
        t.order = if let Some((field, descending)) = &t.sort {
            table::sorted_indices(rows, field, *descending)
        } else {
            (0..rows.len()).collect()
        };
        t.widths = table::column_widths(&columns, rows, rect.width() - 12.0, &t.overrides, |s| {
            p.layout_no_wrap(s.into(), FontId::proportional(font), Color32::WHITE)
                .size()
                .x
        });
        t.version = version;
    }
    // fr columns follow viewport changes; auto widths stay sampled until data changes.
    let fixed = t
        .widths
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            columns[*i].width.fr.is_none() || t.overrides.get(*i).is_some_and(Option::is_some)
        })
        .map(|(_, w)| w)
        .sum::<f32>();
    let fr = columns
        .iter()
        .enumerate()
        .filter(|(i, c)| c.width.fr.is_some() && t.overrides.get(*i).is_none_or(Option::is_none))
        .map(|(_, c)| c.width.fr.unwrap())
        .sum::<f32>();
    if fr > 0.0 {
        for (i, c) in columns.iter().enumerate() {
            if let Some(weight) = c.width.fr
                && t.overrides.get(i).is_none_or(Option::is_none)
            {
                t.widths[i] = ((rect.width() - 12.0 - fixed) * weight / fr).max(48.0);
            }
        }
    }
    let total = t.widths.iter().sum::<f32>();
    let header =
        egui::Rect::from_min_size(rect.min, Vec2::new((rect.width() - 12.0).max(0.0), 32.0));
    let body = egui::Rect::from_min_max(header.left_bottom(), rect.max - Vec2::splat(12.0));
    t.scroll_y = table::clamp_scroll(t.scroll_y, rows.len(), h, body.height());
    t.scroll_x = t.scroll_x.clamp(0.0, (total - body.width()).max(0.0));
    let head_p = p.with_clip_rect(header.intersect(ui.clip_rect()));
    let body_p = p.with_clip_rect(body.intersect(ui.clip_rect()));
    head_p.rect_filled(header, 0.0, token(ui, Token::SurfaceRaised));
    let mut x = header.left() - t.scroll_x;
    let mut sort_change = None;
    for (i, c) in columns.iter().enumerate() {
        let cell =
            egui::Rect::from_min_size(Pos2::new(x, header.top()), Vec2::new(t.widths[i], 32.0));
        let label_text = format!(
            "{}{}",
            c.title,
            if t.sort.as_ref().is_some_and(|(f, _)| f == &c.field) {
                if t.sort.as_ref().unwrap().1 {
                    " ▼"
                } else {
                    " ▲"
                }
            } else {
                ""
            }
        );
        label_line(
            ui,
            &head_p,
            cell.shrink2(Vec2::new(8.0, 6.0)),
            &label_text,
            font,
            text_color,
        );
        let response = ui.interact(
            cell.intersect(header).intersect(ui.clip_rect()),
            egui::Id::new((&n.id, &c.id, "sort")),
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &label_text)
        });
        let edge = egui::Rect::from_center_size(
            Pos2::new(cell.right(), cell.center().y),
            Vec2::new(6.0, 32.0),
        );
        let drag = ui.interact(
            edge.intersect(header).intersect(ui.clip_rect()),
            egui::Id::new((&n.id, &c.id, "resize")),
            if enabled {
                egui::Sense::drag()
            } else {
                egui::Sense::hover()
            },
        );
        if drag.dragged() {
            let w = (t.widths[i] + ui.input(|i| i.pointer.delta().x)).clamp(48.0, 4000.0);
            t.overrides.resize(columns.len(), None);
            t.overrides[i] = Some(w);
            t.widths[i] = w;
            ui.ctx().request_repaint();
        }
        if response.clicked() && !drag.dragged() {
            let desc = t.sort.as_ref().is_some_and(|(f, d)| f == &c.field && !*d);
            sort_change = Some((c.field.clone(), desc, c.id.clone()));
        }
        head_p.line_segment(
            [cell.right_top(), cell.right_bottom()],
            Stroke::new(1.0_f32, token(ui, Token::Border)),
        );
        x += t.widths[i];
    }
    if let Some((field, desc, id)) = sort_change {
        t.sort = Some((field.clone(), desc));
        t.order = table::sorted_indices(rows, &field, desc);
        events.push(
            ui,
            "sortChange",
            Some(id),
            serde_json::json!({"field":field,"descending":desc}),
        );
    }
    let range = table::visible_rows(rows.len(), h, t.scroll_y, body.height());
    t.painted_rows = 0;
    if rows.is_empty() {
        label(
            ui,
            &body_p,
            body.shrink(8.0),
            "暂无数据",
            font,
            token(ui, Token::TextMuted),
        );
    }
    let mut keyboard = None;
    for index in range {
        let row = &rows[t.order[index]];
        let row_key = row[key].as_str().unwrap();
        let row_rect = egui::Rect::from_min_size(
            Pos2::new(body.left(), body.top() + index as f32 * h - t.scroll_y),
            Vec2::new(body.width(), h),
        );
        if !row_rect
            .intersect(body)
            .intersect(ui.clip_rect())
            .is_positive()
        {
            continue;
        }
        t.painted_rows += 1;
        let response = ui.interact(
            row_rect.intersect(body).intersect(ui.clip_rect()),
            egui::Id::new((&n.id, row_key)),
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        let selected = t.selected.as_deref() == Some(row_key);
        if selected || response.hovered() || index % 2 == 1 {
            body_p.rect_filled(
                row_rect,
                0.0,
                if selected {
                    ui.visuals().selection.bg_fill.gamma_multiply(0.4)
                } else {
                    token(ui, Token::SurfaceRaised)
                },
            );
        }
        if response.has_focus() {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                keyboard = Some((index + 1).min(rows.len() - 1));
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                keyboard = Some(index.saturating_sub(1));
            }
        }
        if response.clicked() {
            response.request_focus();
            if t.select(row_key.to_owned()) {
                events.push(
                    ui,
                    "rowSelect",
                    Some(row_key.into()),
                    serde_json::json!(row_key),
                );
            }
        }
        let mut x = body.left() - t.scroll_x;
        let mut summary = String::new();
        for (i, c) in columns.iter().enumerate() {
            let cell =
                egui::Rect::from_min_size(Pos2::new(x, row_rect.top()), Vec2::new(t.widths[i], h));
            let text = c.format.apply(&row[&c.field]).unwrap();
            summary.push_str(&format!("{}: {}; ", c.title, text));
            let cp = body_p.with_clip_rect(cell.intersect(body_p.clip_rect()));
            label_line(
                ui,
                &cp,
                cell.shrink2(Vec2::new(8.0, 6.0)),
                &text,
                font,
                text_color,
            );
            x += t.widths[i];
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                enabled,
                selected,
                &summary,
            )
        });
    }
    if let Some(index) = keyboard {
        let row_key = rows[t.order[index]][key].as_str().unwrap();
        if t.select(row_key.into()) {
            events.push(
                ui,
                "rowSelect",
                Some(row_key.into()),
                serde_json::json!(row_key),
            );
        }
        let top = index as f32 * h;
        if top < t.scroll_y {
            t.scroll_y = top;
        } else if top + h > t.scroll_y + body.height() {
            t.scroll_y = top + h - body.height();
        }
        ui.memory_mut(|m| m.request_focus(egui::Id::new((&n.id, row_key))));
        ui.ctx().request_repaint();
    }
    scrollbar(
        ui,
        p,
        (&n.id, "table-y"),
        egui::Rect::from_min_size(body.right_top(), Vec2::new(12.0, body.height())),
        &mut t.scroll_y,
        rows.len() as f32 * h,
        body.height(),
        false,
        enabled,
    );
    scrollbar(
        ui,
        p,
        (&n.id, "table-x"),
        egui::Rect::from_min_size(body.left_bottom(), Vec2::new(body.width(), 12.0)),
        &mut t.scroll_x,
        total,
        body.width(),
        true,
        enabled,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_chart(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    n: &v02::Node,
    rect: egui::Rect,
    enabled: bool,
    state: &desktop_app_engine::binding::State,
    version: u64,
    c: &mut ChartState,
) {
    let series = n.series.as_ref().unwrap();
    if c.version != version {
        c.points = series
            .iter()
            .map(|s| chart::series_points(s, state).unwrap())
            .collect();
        c.version = version;
        c.sample_width = 0;
    }
    let legend = n.legend.unwrap_or(true);
    let plot = plot_rect(rect, legend);
    if plot.width() <= 0.0 || plot.height() <= 0.0 {
        return;
    }
    let x = chart::axis_range(
        c.points
            .iter()
            .enumerate()
            .filter(|(i, _)| !c.hidden.contains(i))
            .flat_map(|(_, s)| s.iter().flatten().map(|p| p.x)),
        n.x_axis.as_ref(),
        false,
    );
    let y = chart::axis_range(
        c.points
            .iter()
            .enumerate()
            .filter(|(i, _)| !c.hidden.contains(i))
            .flat_map(|(_, s)| s.iter().flatten().map(|p| p.y)),
        n.y_axis.as_ref(),
        n.kind == Kind::BarChart,
    );
    if !c.manual {
        c.viewport = Some((x, y));
    }
    let response = ui.interact(
        plot.intersect(ui.clip_rect()),
        egui::Id::new((&n.id, "chart-input")),
        if enabled {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        },
    );
    if response.double_clicked()
        || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Home))
    {
        c.manual = false;
        c.viewport = Some((x, y));
    }
    if n.pan == Some(true)
        && response.dragged()
        && let Some((x, y)) = c.viewport
    {
        let delta = ui.input(|i| i.pointer.delta());
        c.viewport = Some((
            chart::pan_range(x, -delta.x as f64 / plot.width() as f64 * (x.max - x.min)),
            chart::pan_range(y, delta.y as f64 / plot.height() as f64 * (y.max - y.min)),
        ));
        c.manual = true;
        ui.ctx().request_repaint();
    }
    let (x, y) = c.viewport.unwrap_or((x, y));
    let transform = Transform {
        x,
        y,
        left: plot.left() as f64,
        top: plot.top() as f64,
        width: plot.width() as f64,
        height: plot.height() as f64,
    };
    let range_key = (
        x.min.to_bits(),
        x.max.to_bits(),
        y.min.to_bits(),
        y.max.to_bits(),
    );
    if c.sample_width != plot.width() as usize || c.sample_range != Some(range_key) {
        c.sampled = c
            .points
            .iter()
            .zip(series)
            .map(|(p, s)| {
                if s.monotonic_x {
                    chart::downsample(p, x, plot.width() as usize)
                } else {
                    (0..p.len()).collect()
                }
            })
            .collect();
        c.sample_width = plot.width() as usize;
        c.sample_range = Some(range_key);
    }
    let font = n.style.font_size.unwrap_or(14.0);
    let color = token(ui, n.style.color.unwrap_or(Token::TextMuted));
    let border = Stroke::new(1.0_f32, token(ui, Token::Border));
    for tick in chart::ticks(x, plot.width() as f64) {
        let pos = transform.screen(Point { x: tick, y: y.min });
        let text = chart::tick_text(tick);
        p.line_segment(
            [
                Pos2::new(pos.x as f32, plot.top()),
                Pos2::new(pos.x as f32, plot.bottom()),
            ],
            border,
        );
        label(
            ui,
            p,
            egui::Rect::from_min_size(
                Pos2::new(pos.x as f32 - 22.0, plot.bottom() + 5.0),
                Vec2::new(70.0, 20.0),
            ),
            &text,
            font * 0.857,
            color,
        );
    }
    for tick in chart::ticks(y, plot.height() as f64) {
        let pos = transform.screen(Point { x: x.min, y: tick });
        let text = chart::tick_text(tick);
        p.line_segment(
            [
                Pos2::new(plot.left(), pos.y as f32),
                Pos2::new(plot.right(), pos.y as f32),
            ],
            border,
        );
        label(
            ui,
            p,
            egui::Rect::from_min_size(
                Pos2::new(rect.left(), pos.y as f32 - 7.0),
                Vec2::new(60.0, 20.0),
            ),
            &text,
            font * 0.857,
            color,
        );
    }
    let pp = p.with_clip_rect(plot.intersect(p.clip_rect()));
    let mut hover = None;
    let pointer = ui
        .input(|i| i.pointer.hover_pos())
        .filter(|p| plot.contains(*p) && ui.clip_rect().contains(*p));
    let visible_series = series.len().saturating_sub(c.hidden.len()).max(1);
    let mut group = 0;
    for (i, s) in series.iter().enumerate() {
        if c.hidden.contains(&i) {
            continue;
        }
        let color = series_color(ui, i);
        if n.kind == Kind::LineChart {
            let mut previous = None;
            for &index in &c.sampled[i] {
                let Some(point) = c.points[i][index] else {
                    previous = None;
                    continue;
                };
                let point = transform.screen(point);
                let point = point_pos(point);
                if let Some(last) = previous {
                    pp.line_segment([last, point], Stroke::new(2.0_f32, color));
                } else {
                    pp.circle_filled(point, 3.0, color);
                }
                previous = Some(point);
            }
            if enabled
                && let Some(pointer) = pointer
                && let Some(index) = chart::hit(
                    &c.points[i],
                    transform,
                    Point {
                        x: pointer.x as f64,
                        y: pointer.y as f64,
                    },
                    s.monotonic_x,
                )
            {
                hover = Some((i, index));
            }
        } else {
            let spacing = c.points[i]
                .iter()
                .flatten()
                .map(|p| p.x)
                .collect::<Vec<_>>()
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .filter(|s| *s > 0.0)
                .fold(x.max - x.min, f64::min);
            let width = (spacing / (x.max - x.min) * transform.width * 0.75).min(100.0)
                / visible_series as f64;
            for (j, point) in c.points[i].iter().enumerate() {
                if let Some(point) = point {
                    let screen = transform.screen(*point);
                    let zero = transform.screen(Point { x: point.x, y: 0.0 });
                    let left =
                        screen.x - width * visible_series as f64 / 2.0 + width * group as f64;
                    let bar = egui::Rect::from_min_max(
                        point_pos(Point {
                            x: left,
                            y: screen.y.min(zero.y),
                        }),
                        point_pos(Point {
                            x: left + width - 1.0,
                            y: screen.y.max(zero.y),
                        }),
                    );
                    pp.rect_filled(bar, 1.0, color);
                    if enabled && pointer.is_some_and(|p| bar.contains(p)) {
                        hover = Some((i, j));
                    }
                }
            }
        }
        group += 1;
    }
    let chart_response = response;
    chart_response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Label,
            enabled,
            format!(
                "图表 {}，系列 {}，{} 个数据点",
                n.id,
                series
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("、"),
                c.points.iter().map(Vec::len).sum::<usize>()
            ),
        )
    });
    if let Some((i, j)) = hover {
        let point = c.points[i][j].unwrap();
        let screen = transform.screen(point);
        pp.circle_stroke(
            point_pos(screen),
            5.0,
            Stroke::new(2.0_f32, series_color(ui, i)),
        );
        let text = format!("{}\nX: {}\nY: {}", series[i].name, point.x, point.y);
        let tooltip = egui::Rect::from_min_size(
            plot.right_top() + Vec2::new(-150.0, 8.0),
            Vec2::new(142.0, 68.0),
        );
        p.rect_filled(tooltip, 4.0, token(ui, Token::SurfaceRaised));
        label(
            ui,
            p,
            tooltip.shrink(6.0),
            &text,
            font,
            token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
        );
        chart_response.on_hover_text(text);
    }
    if c.points.iter().all(|p| p.iter().all(Option::is_none)) {
        label(ui, p, plot.shrink(12.0), "暂无数据", 16.0, color);
    }
    if legend {
        let mut x = plot.left();
        for (i, s) in series.iter().enumerate() {
            let width = p
                .layout_no_wrap(s.name.clone(), FontId::proportional(font), color)
                .size()
                .x
                + 34.0;
            let r = egui::Rect::from_min_size(
                Pos2::new(x, rect.bottom() - 22.0),
                Vec2::new(width, 22.0),
            );
            p.circle_filled(
                r.left_center() + Vec2::new(6.0, 0.0),
                4.0,
                if c.hidden.contains(&i) {
                    color
                } else {
                    series_color(ui, i)
                },
            );
            label(
                ui,
                p,
                egui::Rect::from_min_max(r.min + Vec2::new(16.0, 0.0), r.max),
                &s.name,
                font,
                color,
            );
            let response = ui.interact(
                r,
                egui::Id::new((&n.id, &s.id, "legend")),
                if enabled {
                    egui::Sense::click()
                } else {
                    egui::Sense::hover()
                },
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    enabled,
                    !c.hidden.contains(&i),
                    &s.name,
                )
            });
            if response.clicked() {
                if !c.hidden.remove(&i) {
                    c.hidden.insert(i);
                }
                c.sample_width = 0;
                ui.ctx().request_repaint();
            }
            x += width;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn workspace_frame(
        app: &mut DashboardApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        width: f32,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(width, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.render(ctx),
        )
    }
    fn text_position(output: &egui::FullOutput, text: &str) -> Option<Pos2> {
        output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(t) if t.galley.job.text == text => {
                Some(t.pos + t.galley.size() * 0.5)
            }
            _ => None,
        })
    }
    fn workspace_click(app: &mut DashboardApp, ctx: &egui::Context, pos: Pos2) -> egui::FullOutput {
        workspace_frame(
            app,
            ctx,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            1120.0,
        );
        workspace_frame(
            app,
            ctx,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            1120.0,
        )
    }
    #[test]
    fn workspace_actual_render_dropdown_and_task_locking() {
        let description = v02::parse(include_str!("../examples/vben-package-legacy.json")).unwrap();
        let mut app = DashboardApp::new(description).unwrap();
        let ctx = egui::Context::default();
        for width in [1120.0, 960.0] {
            let output = workspace_frame(&mut app, &ctx, vec![], width);
            assert!(text_position(&output, "当前所在分支").is_some());
            assert!(text_position(&output, "开始打包").is_some());
        }
        let mut view = app.runtime.state["view"].clone();
        view["currentBranch"] = serde_json::json!("dev");
        view["branches"] = serde_json::json!("dev\nfeature/tms");
        view["ready"] = serde_json::json!(true);
        view["hasBranches"] = serde_json::json!(true);
        app.runtime.set("view", view).unwrap();
        let output = workspace_frame(&mut app, &ctx, vec![], 1120.0);
        let pos = text_position(&output, "读取分支后，在此选择")
            .expect("Select missing from actual render");
        workspace_click(&mut app, &ctx, pos);
        let output = workspace_frame(&mut app, &ctx, vec![], 1120.0);
        let pos = text_position(&output, "feature/tms").expect("Select popup missing");
        workspace_click(&mut app, &ctx, pos);
        assert_eq!(app.runtime.state["branch"], "feature/tms");
        assert_eq!(app.runtime.state["switchAvailable"], true);
        assert_eq!(app.runtime.state["view"]["currentBranch"], "dev");
        app.runtime.set("idle", serde_json::json!(false)).unwrap();
        app.runtime.set("busy", serde_json::json!(true)).unwrap();
        let resolved = app.runtime.tree.resolve(&app.runtime.state).unwrap();
        for id in [
            "sshTarget",
            "repository",
            "branch",
            "refreshButton",
            "switchButton",
            "pullButton",
            "buildButton",
        ] {
            let index = app
                .runtime
                .tree
                .nodes
                .iter()
                .position(|n| n.spec.id == id)
                .unwrap();
            assert!(!resolved[index].enabled, "{id} must be locked during a job");
        }
        workspace_frame(&mut app, &ctx, vec![], 1120.0);
        let session = desktop_app_engine::logging::global().unwrap();
        let task = session
            .task(
                "headless live task",
                "test.exe",
                &std::env::current_dir().unwrap(),
                1000,
            )
            .unwrap();
        task.write(false, b"headless live output");
        app.runtime.invoke("openLogs").unwrap();
        workspace_frame(&mut app, &ctx, vec![], 1120.0);
        let output = workspace_frame(&mut app, &ctx, vec![], 1120.0);
        assert!(text_position(&output, "日志与任务").is_some());
        assert!(text_position(&output, "headless live output").is_some());
        task.write(false, b" updated");
        let output = workspace_frame(&mut app, &ctx, vec![], 1120.0);
        assert!(text_position(&output, "headless live output updated").is_some());
        task.finish("succeeded", 0, "", true);
        session
            .viewer_open
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
    #[test]
    fn plugin_texture_and_egui_click_reach_actual_echarts_worker() {
        let d = v02::parse_at(
            include_str!("../examples/plugin-chart.json"),
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples"),
        )
        .unwrap();
        let mut runtime = Runtime::new(d).unwrap();
        runtime
            .sync_plugins(vec![("cpuChart".into(), 800, 400)])
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while runtime.plugin_images.is_empty()
            && runtime.error.is_none()
            && std::time::Instant::now() < deadline
        {
            runtime.poll_plugins();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(runtime.error.is_none(), "{:?}", runtime.error);
        let image = runtime.plugin_images.remove("cpuChart").unwrap();
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "chart",
            egui::ColorImage::from_rgba_premultiplied(image.size, &image.rgba),
            egui::TextureOptions::LINEAR,
        );
        let n = runtime
            .tree
            .nodes
            .iter()
            .find(|n| n.spec.id == "cpuChart")
            .unwrap()
            .spec
            .clone();
        let rect = egui::Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(800.0, 400.0));
        let pointer = Pos2::new(420.0, 270.0);
        let mut found = false;
        for input in [
            vec![],
            vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::PointerButton {
                    pos: pointer,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            vec![egui::Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        ] {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(1000.0, 600.0),
                    )),
                    events: input,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let p = ui.painter().clone();
                        paint_plugin(ui, &p, &n, rect, true, Some(&texture), &runtime.plugins)
                            .unwrap();
                    });
                },
            );
            found |= output.shapes.iter().any(
                |s| matches!(&s.shape,egui::epaint::Shape::Mesh(m) if m.texture_id==texture.id()),
            );
        }
        assert!(found, "plugin image was not painted");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while runtime.state["selected"] != "B"
            && runtime.error.is_none()
            && std::time::Instant::now() < deadline
        {
            runtime.poll_plugins();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(runtime.error.is_none(), "{:?}", runtime.error);
        assert_eq!(runtime.state["selected"], "B");
    }
    #[test]
    fn chart_hover_uses_original_vertex_in_input_frame() {
        let d = v02::parse(include_str!("../examples/dashboard.json")).unwrap();
        let n = &d.root.children[4].0.children[0].0;
        let rect = egui::Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(800.0, 400.0));
        let plot = plot_rect(rect, true);
        let pointer = Pos2::new(
            plot.left() + plot.width() * 0.6,
            plot.top() + plot.height() * 0.34,
        );
        let ctx = egui::Context::default();
        let mut c = ChartState::default();
        let mut found = false;
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(1920.0, 1080.0),
                    )),
                    events: vec![egui::Event::PointerMoved(pointer)],
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let p = ui.painter().clone();
                        paint_chart(ui, &p, n, rect, true, &d.state, 1, &mut c);
                    });
                },
            );
            found |= output.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="CPU\nX: 3\nY: 0.66"));
        }
        assert!(found, "hover feedback missing at original vertex");
    }
    #[test]
    fn ten_thousand_rows_use_actual_visible_paint_path() {
        let mut source: serde_json::Value =
            serde_json::from_str(include_str!("../examples/dashboard.json")).unwrap();
        source["state"]["servers"]=serde_json::Value::Array((0..10_000).map(|i|serde_json::json!({"id":format!("row-{i}"),"name":format!("节点 {i}"),"region":"区域","load":(i%100) as f64/100.0,"requests":i})).collect());
        let d = v02::parse(&source.to_string()).unwrap();
        let n = &d.root.children[5].0.children[0].0;
        let ctx = egui::Context::default();
        let mut t = TableState::default();
        let mut times = vec![];
        let mut shapes = 0;
        for frame in 0..101 {
            let start = std::time::Instant::now();
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(1920.0, 1080.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let p = ui.painter().clone();
                        let mut events = vec![];
                        let mut collector = Collector {
                            events: &mut events,
                            base: frame,
                            generation: 1,
                            component: &n.id,
                        };
                        paint_table(
                            ui,
                            &p,
                            n,
                            egui::Rect::from_min_size(
                                Pos2::new(10.0, 10.0),
                                Vec2::new(800.0, 320.0),
                            ),
                            true,
                            &d.state,
                            1,
                            &mut t,
                            &mut collector,
                        );
                        assert!(events.is_empty());
                    });
                },
            );
            if frame > 0 {
                times.push(start.elapsed().as_micros());
            }
            shapes = output.shapes.len();
            assert!(t.painted_rows <= 10);
            assert!(shapes < 250, "unexpected full-table painting: {shapes}");
        }
        times.sort_unstable();
        println!(
            "TABLE total=10000 painted={} shapes={} viewport=800x320 screen=1920x1080 frames=100 p95_us={} profile={}",
            t.painted_rows,
            shapes,
            times[94],
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        );
        t.scroll_y = 999999.0;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let p = ui.painter().clone();
                let mut e = vec![];
                let mut c = Collector {
                    events: &mut e,
                    base: 1000,
                    generation: 1,
                    component: &n.id,
                };
                paint_table(
                    ui,
                    &p,
                    n,
                    egui::Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(800.0, 320.0)),
                    true,
                    &d.state,
                    1,
                    &mut t,
                    &mut c,
                );
            });
        });
        assert!(t.painted_rows <= 10);
        assert!(t.scroll_y < 320000.0);
    }
}
