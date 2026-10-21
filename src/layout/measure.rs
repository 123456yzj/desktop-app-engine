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
        let style = &node.style;
        let font = style.font_size.unwrap_or(16.0);
        match node.kind {
            Kind::Text | Kind::Badge | Kind::Button => {
                let mut size = self.text.styled_text(&resolved.text, inner, style);
                if node.kind != Kind::Text {
                    size.w += 2.0 * style.padding_x.unwrap_or(10.0);
                    size.h += 2.0 * style.padding_y.unwrap_or(6.0);
                }
                size
            }
            Kind::Input | Kind::Select => Size { w: 220.0, h: 34.0 },
            Kind::Metric => Size {
                w: self
                    .text
                    .text(&resolved.text, inner, font * 1.875)
                    .w
                    .max(self.text.text(&resolved.label, inner, font).w),
                h: font * 4.125,
            },
            Kind::Progress => Size { w: 160.0, h: 32.0 },
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
            Kind::Grid => self.grid(id, inner, width),
            Kind::Row => self.row(id, inner),
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
            w: widths.iter().sum::<f32>() + node.layout.gap * widths.len().saturating_sub(1) as f32,
            h: heights.iter().sum::<f32>()
                + node.layout.gap * heights.len().saturating_sub(1) as f32,
        }
    }

    fn row(&self, id: NodeId, inner: f32) -> Size {
        let layout = &self.tree.nodes[id].spec.layout;
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
        let bases: Vec<_> = items.iter().map(|item| item.base).collect();
        let lines = if layout.wrap {
            wrap_lines(&bases, inner, layout.gap)
        } else if children.is_empty() {
            vec![]
        } else {
            std::iter::once(0..children.len()).collect()
        };
        let mut size = Size::default();
        for line in &lines {
            let widths = flex(&items[line.clone()], inner, layout.gap);
            let height = line
                .clone()
                .zip(&widths)
                .map(|(j, &width)| self.at_width(children[j], Limit::Bounded(width)).h)
                .fold(0.0, f32::max);
            size.h += height;
            size.w = size.w.max(
                widths.iter().sum::<f32>() + layout.gap * widths.len().saturating_sub(1) as f32,
            );
        }
        size.h += layout.gap * lines.len().saturating_sub(1) as f32;
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
        size.h += node.layout.gap * children.len().saturating_sub(1) as f32;
        if !self.resolved[id].title.is_empty() {
            size.h += node.style.font_size.unwrap_or(16.0) + 12.0;
        }
        size
    }
}
