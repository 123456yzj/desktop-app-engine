use super::{
    Constraint, Limit, Size, flex,
    flex::{dimension, grid_widths, item},
    visible_children, wrap_lines,
};
use crate::{
    component::{NodeId, Resolved, Tree},
    v02::{ColumnSpec, Kind, Style},
};

pub trait TextMeasure {
    fn text(&self, text: &str, width: f32, font_size: f32) -> Size;
    fn styled_text(&self, text: &str, width: f32, style: &Style) -> Size {
        self.text(
            text,
            if style.text_wrap == Some(false) {
                100_000.0
            } else {
                width
            },
            style.font_size.unwrap_or(16.0),
        )
    }
}

pub fn measure(
    tree: &Tree,
    resolved: &[Resolved],
    id: NodeId,
    c: Constraint,
    text: &impl TextMeasure,
) -> Size {
    Measurer {
        tree,
        resolved,
        text,
    }
    .node(id, c)
}

pub(super) struct Measurer<'a, T> {
    pub(super) tree: &'a Tree,
    pub(super) resolved: &'a [Resolved],
    pub(super) text: &'a T,
}

impl<T: TextMeasure> Measurer<'_, T> {
    pub(super) fn at_width(&self, id: NodeId, width: Limit) -> Size {
        self.node(
            id,
            Constraint {
                width,
                height: Limit::Unbounded,
            },
        )
    }

    fn node(&self, id: NodeId, c: Constraint) -> Size {
        if !self.resolved[id].visible {
            return Size::default();
        }
        let node = &self.tree.nodes[id].spec;
        let layout = &node.layout;
        let padding = layout.padding + node.style.border_width;
        let available = match c.width {
            Limit::Bounded(w) => w,
            Limit::Unbounded => 100_000.0,
        };
        let width = layout
            .width
            .as_ref()
            .and_then(|t| t.fixed)
            .unwrap_or(available)
            .min(layout.max_width.unwrap_or(1e8))
            .max(layout.min_width.unwrap_or(0.0));
        let natural = self.natural(id, (width - 2.0 * padding).max(0.0), c.width);
        Size {
            w: dimension(
                layout.width.as_ref(),
                natural.w + 2.0 * padding,
                c.width,
                layout.min_width,
                layout.max_width,
            ),
            h: dimension(
                layout.height.as_ref(),
                natural.h + 2.0 * padding,
                c.height,
                layout.min_height,
                layout.max_height,
            ),
        }
    }

    fn natural(&self, id: NodeId, inner: f32, width: Limit) -> Size {
        let node = &self.tree.nodes[id].spec;
        let resolved = &self.resolved[id];
        let mut effective_style = node.style.clone();
        let size = resolved
            .api
            .get("size")
            .or_else(|| resolved.api.get("@size"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("medium");
        if effective_style.font_size.is_none() {
            effective_style.font_size = Some(match size {
                "small" => 12.,
                "large" => 18.,
                _ => 16.,
            });
        }
        let style = &effective_style;
        let font = style.font_size.unwrap_or(16.0);
        match node.kind {
            Kind::Text | Kind::Badge | Kind::Button => {
                let text_width = if node.kind == Kind::Text {
                    inner
                } else {
                    (inner - 2.0 * style.padding_x.unwrap_or(10.0)).max(0.)
                };
                let mut size = self.text.styled_text(&resolved.text, text_width, style);
                if node.kind != Kind::Text {
                    size.w += 2.0 * style.padding_x.unwrap_or(10.0);
                    size.h += 2.0 * style.padding_y.unwrap_or(6.0);
                }
                if node.kind == Kind::Text {
                    if resolved.api.get("copyable") == Some(&serde_json::json!(true)) {
                        size.h += 28.;
                    }
                    if resolved.api.get("editable") == Some(&serde_json::json!(true)) {
                        size.h = size.h.max(72.);
                        size.w = size.w.max(220.);
                    }
                }
                if node.kind == Kind::Button {
                    if resolved.api.get("loading") == Some(&serde_json::json!(true)) {
                        size.w += 22.;
                    }
                    if resolved
                        .api
                        .get("shape")
                        .and_then(serde_json::Value::as_str)
                        == Some("circle")
                    {
                        size.w = size.h;
                    }
                    if resolved.api.get("block") == Some(&serde_json::json!(true)) {
                        size.w = inner;
                    }
                }
                size
            }
            Kind::Input | Kind::Select | Kind::InputNumber => Size {
                w: 220.0,
                h: (if node.kind == Kind::Input
                    && resolved.api.get("multiline") == Some(&serde_json::json!(true))
                {
                    resolved
                        .api
                        .get("rows")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(3) as f32
                        * (font + 6.)
                        + 12.
                } else {
                    match size {
                        "small" => 24.,
                        "large" => 40.,
                        _ => 34.,
                    }
                })
                .max(if node.kind == Kind::Input {
                    self.text.styled_text("Mg", inner, style).h + 2. * style.padding_y.unwrap_or(2.)
                } else {
                    0.
                }),
            },
            Kind::Checkbox | Kind::Switch => {
                let label = self
                    .text
                    .styled_text(&resolved.text, (inner - 52.0).max(1.0), style);
                Size {
                    w: label.w
                        + if node.kind == Kind::Switch {
                            52.0
                        } else {
                            30.0
                        },
                    h: label.h.max(28.0),
                }
            }
            Kind::Radio => {
                let mut size = Size::default();
                for option in &resolved.options {
                    let label =
                        self.text
                            .styled_text(&option.label, (inner - 30.0).max(1.0), style);
                    if node.vertical {
                        size.w = size.w.max(label.w + 30.0);
                        size.h += label.h.max(28.0) + 8.0;
                    } else {
                        size.w += label.w + 38.0;
                        size.h = size.h.max(label.h.max(28.0));
                    }
                }
                if node.vertical {
                    size.h = (size.h - 8.0).max(0.0);
                }
                size
            }
            Kind::Slider => {
                if node.vertical {
                    Size { w: 40.0, h: 180.0 }
                } else {
                    Size { w: 220.0, h: 34.0 }
                }
            }
            Kind::FormItem => {
                let mut size = self.column(id, inner);
                if !resolved.label.is_empty() {
                    size.w = size
                        .w
                        .max(self.text.styled_text(&resolved.label, inner, style).w + 12.0);
                    size.h += font + 8.0;
                }
                if !node.rules.is_empty() {
                    size.h += font + 8.0;
                }
                size
            }
            Kind::Metric => Size {
                w: self
                    .text
                    .text(&resolved.text, inner, font * 1.875)
                    .w
                    .max(self.text.text(&resolved.label, inner, font).w),
                h: font * 4.125,
            },
            Kind::Progress => Size { w: 160.0, h: 32.0 },
            Kind::BadgeCount => crate::components::egui::widgets::badge_count::measure(
                resolved, style, inner, self.text,
            ),
            Kind::Table => Size {
                w: if matches!(width, Limit::Unbounded) {
                    400.0
                } else {
                    inner
                },
                h: 280.0,
            },
            Kind::Plugin | Kind::LineChart | Kind::BarChart => Size {
                w: if matches!(width, Limit::Unbounded) {
                    600.0
                } else {
                    inner
                },
                h: 240.0,
            },
            Kind::Divider => {
                if node.vertical {
                    Size { w: 16.0, h: 24.0 }
                } else {
                    Size {
                        w: if matches!(width, Limit::Unbounded) {
                            160.0
                        } else {
                            inner
                        },
                        h: if resolved.text.is_empty() {
                            16.0
                        } else {
                            font + 12.0
                        },
                    }
                }
            }
            Kind::Masonry => self.masonry(id, inner),
            Kind::Splitter if !node.vertical => self.splitter(id, inner),
            Kind::Grid => self.grid(id, inner, width),
            kind if crate::v02::widgets::is_widget(kind) => {
                if kind.is_container() && !matches!(kind, Kind::Modal | Kind::Drawer) {
                    let mut size = self.column(id, inner);
                    if self.tree.nodes[id].children.is_empty() {
                        match kind {
                            Kind::Tooltip | Kind::Popover | Kind::Popconfirm | Kind::Spin => {
                                size.w = 240.;
                                size.h = 36.;
                            }
                            Kind::Carousel
                                if resolved
                                    .api
                                    .get("items")
                                    .and_then(serde_json::Value::as_array)
                                    .is_some_and(|a| !a.is_empty()) =>
                            {
                                size.w = 240.;
                                size.h = 140.;
                            }
                            Kind::Watermark | Kind::BorderBeam => {
                                size.w = 240.;
                                size.h = 120.;
                            }
                            _ => {}
                        }
                    }
                    if matches!(kind, Kind::Tabs | Kind::Carousel | Kind::Collapse)
                        && !(kind == Kind::Tabs
                            && resolved.api.get("hideTabBar")
                                == Some(&serde_json::Value::Bool(true)))
                    {
                        size.h += if kind == Kind::Collapse {
                            resolved
                                .api
                                .get("items")
                                .and_then(serde_json::Value::as_array)
                                .map_or(0, Vec::len) as f32
                                * 36.
                        } else {
                            44.
                        };
                    }
                    size
                } else {
                    let (w, h) = match kind {
                        Kind::Modal
                        | Kind::Drawer
                        | Kind::Message
                        | Kind::Notification
                        | Kind::FloatButton
                        | Kind::Tour => (0., 0.),
                        Kind::Icon => (24., 24.),
                        Kind::Avatar => (40., 40.),
                        Kind::Image | Kind::QrCode => (180., 180.),
                        Kind::Calendar => (320., 300.),
                        Kind::Tree | Kind::Transfer | Kind::List | Kind::Listy => (inner, 240.),
                        Kind::Descriptions | Kind::Timeline | Kind::Result => (inner, 160.),
                        Kind::Skeleton => (inner, 120.),
                        Kind::Alert | Kind::Steps => (inner, 64.),
                        Kind::Mentions => (240., 100.),
                        Kind::Rate => (180., 34.),
                        _ => (240., 36.),
                    };
                    Size { w, h }
                }
            }
            _ if self.tree.horizontal(id) => self.row(id, inner),
            _ => self.column(id, inner),
        }
    }

    fn grid(&self, id: NodeId, inner: f32, width: Limit) -> Size {
        let node = &self.tree.nodes[id].spec;
        let children = visible_children(self.tree, self.resolved, id);
        if children.is_empty() {
            return Size::default();
        }
        let widths = if matches!(width, Limit::Unbounded)
            && node.layout.width.as_ref().and_then(|t| t.fixed).is_none()
        {
            node.columns
                .iter()
                .enumerate()
                .map(|(col, track)| {
                    if let ColumnSpec::Track(t) = track
                        && let Some(fixed) = t.fixed
                    {
                        fixed
                    } else {
                        children
                            .iter()
                            .enumerate()
                            .filter(|(j, _)| j % node.columns.len() == col)
                            .map(|(_, &child)| self.at_width(child, Limit::Unbounded).w)
                            .fold(0.0, f32::max)
                    }
                })
                .collect::<Vec<_>>()
        } else {
            grid_widths(node, inner)
        };
        let mut heights = vec![0.0_f32; children.len().div_ceil(widths.len())];
        for (j, &child) in children.iter().enumerate() {
            let size = self.at_width(child, Limit::Bounded(widths[j % widths.len()]));
            heights[j / widths.len()] = heights[j / widths.len()].max(size.h);
        }
        Size {
            w: widths.iter().sum::<f32>()
                + node.layout.column_gap() * widths.len().saturating_sub(1) as f32,
            h: heights.iter().sum::<f32>()
                + node.layout.row_gap() * heights.len().saturating_sub(1) as f32,
        }
    }

    fn row(&self, id: NodeId, inner: f32) -> Size {
        let node = &self.tree.nodes[id].spec;
        let layout = &node.layout;
        let gap = layout.column_gap();
        let children = visible_children(self.tree, self.resolved, id);
        let items: Vec<_> = children
            .iter()
            .map(|&child| {
                item(
                    &self.tree.nodes[child].spec.layout,
                    self.at_width(child, Limit::Bounded(inner)).w,
                    true,
                )
            })
            .collect();
        let bases: Vec<_> = items
            .iter()
            .map(|item| item.base.clamp(item.min, item.max))
            .collect();
        let lines = if layout.wrap {
            wrap_lines(&bases, inner, gap)
        } else if children.is_empty() {
            vec![]
        } else {
            std::iter::once(0..children.len()).collect()
        };
        let mut size = Size::default();
        for line in &lines {
            let widths = flex(&items[line.clone()], inner, gap);
            let height = line
                .clone()
                .zip(&widths)
                .map(|(j, &width)| self.at_width(children[j], Limit::Bounded(width)).h)
                .fold(0.0, f32::max);
            size.h += height;
            size.w = size
                .w
                .max(widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32);
        }
        size.h += layout.row_gap() * lines.len().saturating_sub(1) as f32;
        size
    }

    fn column(&self, id: NodeId, inner: f32) -> Size {
        let node = &self.tree.nodes[id].spec;
        let children = visible_children(self.tree, self.resolved, id);
        let mut size = Size::default();
        for &child in &children {
            let measured = self.at_width(child, Limit::Bounded(inner));
            size.w = size.w.max(measured.w);
            size.h += measured.h;
        }
        let gap = if node.kind == Kind::Splitter {
            node.bar_size.unwrap_or(8.0)
        } else {
            node.layout.row_gap()
        };
        size.h += gap * children.len().saturating_sub(1) as f32;
        if super::has_layout_title(node.kind) && !self.resolved[id].title.is_empty() {
            size.h += node.style.font_size.unwrap_or(16.0) + 12.0;
        }
        size
    }

    fn masonry(&self, id: NodeId, inner: f32) -> Size {
        let node = &self.tree.nodes[id].spec;
        let (_, width) = super::masonry::columns(node, inner);
        let heights: Vec<_> = visible_children(self.tree, self.resolved, id)
            .iter()
            .map(|&child| self.at_width(child, Limit::Bounded(width)).h)
            .collect();
        let positions = super::masonry::positions(node, inner, &heights);
        Size {
            w: inner,
            h: positions.iter().map(|r| r.y + r.h).fold(0.0, f32::max),
        }
    }

    fn splitter(&self, id: NodeId, inner: f32) -> Size {
        let node = &self.tree.nodes[id].spec;
        let children = visible_children(self.tree, self.resolved, id);
        if children.len() != 2 {
            return self.column(id, inner);
        }
        let bar = super::splitter::bar_size(node, inner);
        let sizes = super::splitter::sizes(
            inner - bar,
            self.resolved[id]
                .value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32,
            super::splitter::limits(&self.tree.nodes[children[0]].spec.layout, true),
            super::splitter::limits(&self.tree.nodes[children[1]].spec.layout, true),
        );
        Size {
            w: sizes.iter().sum::<f32>() + bar,
            h: children
                .iter()
                .zip(sizes)
                .map(|(&child, width)| self.at_width(child, Limit::Bounded(width)).h)
                .fold(0.0, f32::max),
        }
    }
}
