use super::{
    BoxLayout, FlexItem, LayoutResult, Limit, Rect, Size, TextMeasure, flex,
    flex::{axis, dimension, grid_widths, item},
    measure::Measurer,
    visible_children, wrap_lines,
};
use crate::{
    component::{NodeId, Resolved, Tree},
    v02::{Align, Justify, Kind, Overflow},
};

pub fn arrange(
    tree: &Tree,
    resolved: &[Resolved],
    viewport: Rect,
    scroll: &mut [Size],
    text: &impl TextMeasure,
    scale: f32,
) -> LayoutResult {
    let mut out = LayoutResult {
        boxes: vec![BoxLayout::default(); tree.nodes.len()],
    };
    let mut worker = Arranger {
        measurer: Measurer {
            tree,
            resolved,
            text,
        },
        scroll,
        out: &mut out,
        scale,
    };
    let root = &tree.nodes[0].spec.layout;
    let root_rect = Rect {
        x: viewport.x,
        y: viewport.y,
        w: dimension(
            root.width.as_ref(),
            viewport.w,
            Limit::Bounded(viewport.w),
            root.min_width,
            root.max_width,
        ),
        h: dimension(
            root.height.as_ref(),
            viewport.h,
            Limit::Bounded(viewport.h),
            root.min_height,
            root.max_height,
        ),
    };
    worker.node(0, root_rect, viewport);
    out
}

struct Arranger<'a, T> {
    measurer: Measurer<'a, T>,
    scroll: &'a mut [Size],
    out: &'a mut LayoutResult,
    scale: f32,
}

impl<T: TextMeasure> Arranger<'_, T> {
    fn node(&mut self, id: NodeId, rect: Rect, parent_clip: Rect) {
        if !self.measurer.resolved[id].visible {
            return;
        }
        let node = &self.measurer.tree.nodes[id].spec;
        let mut rect = rect.rounded(self.scale);
        if node.kind == Kind::Affix {
            rect = crate::components::egui::widgets::affix::position(
                rect,
                parent_clip,
                &self.measurer.resolved[id],
            );
        }
        let mut content = rect.inset(node.layout.padding + node.style.border_width);
        if super::has_layout_title(node.kind) && !self.measurer.resolved[id].title.is_empty() {
            let title_height = node.style.font_size.unwrap_or(16.0) + 12.0;
            content.y += title_height;
            content.h = (content.h - title_height).max(0.0);
        }
        if node.kind == Kind::FormItem {
            let label = if self.measurer.resolved[id].label.is_empty() {
                0.0
            } else {
                node.style.font_size.unwrap_or(16.0) + 8.0
            };
            let error = if node.rules.is_empty() {
                0.0
            } else {
                node.style.font_size.unwrap_or(16.0) + 8.0
            };
            content.y += label;
            content.h = (content.h - label - error).max(0.0);
        }
        if matches!(node.kind, Kind::Tabs | Kind::Carousel | Kind::Collapse)
            && !(node.kind == Kind::Tabs
                && self.measurer.resolved[id].api.get("hideTabBar")
                    == Some(&serde_json::Value::Bool(true)))
        {
            let header = if node.kind == Kind::Collapse {
                self.measurer.resolved[id]
                    .api
                    .get("items")
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, Vec::len) as f32
                    * 36.
            } else {
                44.
            };
            let placement = self.measurer.resolved[id]
                .api
                .get("tabPlacement")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("top");
            if node.kind == Kind::Tabs && matches!(placement, "start" | "end") {
                let width = 120_f32.min(content.w);
                if placement == "start" {
                    content.x += width;
                }
                content.w = (content.w - width).max(0.);
            } else {
                if placement != "bottom" {
                    content.y += header;
                }
                content.h = (content.h - header).max(0.);
            }
        }
        if node.layout.overflow == Overflow::Scroll {
            content.w = (content.w - 12.0).max(0.0);
            content.h = (content.h - 12.0).max(0.0);
        }
        let clip = parent_clip.intersect(rect);
        let children = visible_children(self.measurer.tree, self.measurer.resolved, id);
        let mut placements = if children.is_empty() {
            vec![]
        } else {
            match crate::components::catalogue::definition(node.kind).layout {
                crate::components::catalogue::LayoutKind::Grid => self.grid(id, &children, content),
                crate::components::catalogue::LayoutKind::Masonry => {
                    self.masonry(id, &children, content)
                }
                crate::components::catalogue::LayoutKind::Splitter => {
                    self.splitter(id, &children, content)
                }
                _ => self.linear(id, &children, content),
            }
        };
        if self.measurer.resolved[id]
            .api
            .get("@direction")
            .and_then(serde_json::Value::as_str)
            == Some("rtl")
        {
            for (_, rect) in &mut placements {
                rect.x = content.w - rect.x - rect.w;
            }
        }
        let extent = placements
            .iter()
            .fold(Size::default(), |size, (_, rect)| Size {
                w: size.w.max(rect.x + rect.w),
                h: size.h.max(rect.y + rect.h),
            });
        self.scroll[id].w = self.scroll[id]
            .w
            .clamp(0.0, (extent.w - content.w).max(0.0));
        self.scroll[id].h = self.scroll[id]
            .h
            .clamp(0.0, (extent.h - content.h).max(0.0));
        self.out.boxes[id] = BoxLayout {
            rect,
            content,
            clip,
            extent,
        };
        for (child, mut rect) in placements {
            rect.x += content.x - self.scroll[id].w;
            rect.y += content.y - self.scroll[id].h;
            self.node(child, rect, clip.intersect(content));
        }
    }

    fn grid(&self, id: NodeId, children: &[NodeId], content: Rect) -> Vec<(NodeId, Rect)> {
        let node = &self.measurer.tree.nodes[id].spec;
        let layout = &node.layout;
        let widths = grid_widths(node, content.w);
        // Reuse each child's measured size for both row sizing and placement.
        let measured: Vec<_> = children
            .iter()
            .enumerate()
            .map(|(j, &child)| {
                self.measurer
                    .at_width(child, Limit::Bounded(widths[j % widths.len()]))
            })
            .collect();
        let row_items: Vec<_> = children
            .chunks(widths.len())
            .zip(measured.chunks(widths.len()))
            .map(|(chunk, sizes)| {
                let natural = sizes.iter().map(|size| size.h).fold(0.0, f32::max);
                let grow = chunk
                    .iter()
                    .filter_map(|&child| {
                        self.measurer.tree.nodes[child]
                            .spec
                            .layout
                            .height
                            .as_ref()
                            .and_then(|t| t.fr)
                    })
                    .fold(0.0, f32::max);
                let fill_single_row = children.len() <= widths.len()
                    && layout
                        .height
                        .as_ref()
                        .is_some_and(|t| t.fixed.is_some() || t.fr.is_some());
                FlexItem {
                    base: if grow > 0.0 { 0.0 } else { natural },
                    min: 0.0,
                    max: 1e8,
                    grow: if grow == 0.0 && fill_single_row {
                        1.0
                    } else {
                        grow
                    },
                    shrink: 0.0,
                }
            })
            .collect();
        let heights = flex(&row_items, content.h, layout.row_gap());
        let mut placements = Vec::with_capacity(children.len());
        let mut y = 0.0;
        for ((chunk, sizes), height) in children
            .chunks(widths.len())
            .zip(measured.chunks(widths.len()))
            .zip(heights)
        {
            let mut x = 0.0;
            for ((&child, size), &available) in chunk.iter().zip(sizes).zip(&widths) {
                let child_layout = &self.measurer.tree.nodes[child].spec.layout;
                let desired = dimension(
                    child_layout.height.as_ref(),
                    size.h,
                    Limit::Bounded(height),
                    child_layout.min_height,
                    child_layout.max_height,
                );
                let (dy, h) = cross(layout.align, height, desired);
                let width = dimension(
                    child_layout.width.as_ref(),
                    available,
                    Limit::Bounded(available),
                    child_layout.min_width,
                    child_layout.max_width,
                );
                let (dx, _) = cross(layout.align, available, width);
                placements.push((
                    child,
                    Rect {
                        x: x + dx,
                        y: y + dy,
                        w: width,
                        h,
                    },
                ));
                x += available + layout.column_gap();
            }
            y += height + layout.row_gap();
        }
        placements
    }

    fn linear(&self, id: NodeId, children: &[NodeId], content: Rect) -> Vec<(NodeId, Rect)> {
        let node = &self.measurer.tree.nodes[id].spec;
        let layout = &node.layout;
        let horizontal = self.measurer.tree.horizontal(id);
        let main_gap = layout.main_gap(horizontal);
        let natural: Vec<_> = children
            .iter()
            .map(|&child| self.measurer.at_width(child, Limit::Bounded(content.w)))
            .collect();
        let items: Vec<_> = natural
            .iter()
            .enumerate()
            .map(|(j, size)| {
                item(
                    &self.measurer.tree.nodes[children[j]].spec.layout,
                    if horizontal { size.w } else { size.h },
                    horizontal,
                )
            })
            .collect();
        let bases: Vec<_> = items
            .iter()
            .map(|item| item.base.clamp(item.min, item.max))
            .collect();
        let lines = if horizontal && layout.wrap {
            wrap_lines(&bases, content.w, main_gap)
        } else {
            std::iter::once(0..children.len()).collect()
        };
        let mut placements = Vec::with_capacity(children.len());
        let mut line_y = 0.0;
        for line in lines {
            let available = if horizontal {
                content.w
            } else if layout.overflow == Overflow::Scroll {
                content.h.max(
                    natural.iter().map(|size| size.h).sum::<f32>()
                        + main_gap * children.len().saturating_sub(1) as f32,
                )
            } else {
                content.h
            };
            let sizes = flex(&items[line.clone()], available, main_gap);
            let (mut position, gap) = justify(layout.justify, available, &sizes, main_gap);
            let cross_sizes: Vec<_> = line
                .clone()
                .enumerate()
                .map(|(k, j)| {
                    if horizontal {
                        self.measurer
                            .at_width(children[j], Limit::Bounded(sizes[k]))
                            .h
                    } else {
                        natural[j].w
                    }
                })
                .collect();
            let cross_available = if horizontal {
                if layout.wrap {
                    cross_sizes.iter().copied().fold(0.0, f32::max)
                } else {
                    content.h
                }
            } else {
                content.w
            };
            for (k, j) in line.enumerate() {
                let (track, min, max) = axis(
                    &self.measurer.tree.nodes[children[j]].spec.layout,
                    !horizontal,
                );
                let natural = if layout.align == Align::Stretch {
                    cross_available
                } else {
                    cross_sizes[k]
                };
                let desired = dimension(track, natural, Limit::Bounded(cross_available), min, max);
                let (offset, _) = cross(layout.align, cross_available, desired);
                let rect = if horizontal {
                    Rect {
                        x: position,
                        y: line_y + offset,
                        w: sizes[k],
                        h: desired,
                    }
                } else {
                    Rect {
                        x: offset,
                        y: position,
                        w: desired,
                        h: sizes[k],
                    }
                };
                placements.push((children[j], rect));
                position += sizes[k] + gap;
            }
            line_y += cross_available + layout.row_gap();
        }
        placements
    }

    fn masonry(&self, id: NodeId, children: &[NodeId], content: Rect) -> Vec<(NodeId, Rect)> {
        let node = &self.measurer.tree.nodes[id].spec;
        let (_, width) = super::masonry::columns(node, content.w);
        let heights: Vec<_> = children
            .iter()
            .map(|&child| self.measurer.at_width(child, Limit::Bounded(width)).h)
            .collect();
        children
            .iter()
            .copied()
            .zip(super::masonry::positions(node, content.w, &heights))
            .map(|(child, mut rect)| {
                let layout = &self.measurer.tree.nodes[child].spec.layout;
                let desired = dimension(
                    layout.width.as_ref(),
                    if node.layout.align == Align::Stretch {
                        rect.w
                    } else {
                        self.measurer.at_width(child, Limit::Bounded(width)).w
                    },
                    Limit::Bounded(width),
                    layout.min_width,
                    layout.max_width,
                );
                rect.x += cross(node.layout.align, width, desired).0;
                rect.w = desired;
                (child, rect)
            })
            .collect()
    }

    fn splitter(&self, id: NodeId, children: &[NodeId], content: Rect) -> Vec<(NodeId, Rect)> {
        let node = &self.measurer.tree.nodes[id].spec;
        let horizontal = !node.vertical;
        let available = if horizontal { content.w } else { content.h };
        let bar = if children.len() == 2 {
            super::splitter::bar_size(node, available)
        } else {
            0.0
        };
        let sizes = if children.len() == 2 {
            super::splitter::sizes(
                available - bar,
                self.measurer.resolved[id]
                    .value
                    .as_ref()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5) as f32,
                super::splitter::limits(
                    &self.measurer.tree.nodes[children[0]].spec.layout,
                    horizontal,
                ),
                super::splitter::limits(
                    &self.measurer.tree.nodes[children[1]].spec.layout,
                    horizontal,
                ),
            )
        } else {
            [available, 0.0]
        };
        let mut position = 0.0;
        children
            .iter()
            .enumerate()
            .map(|(j, &child)| {
                let rect = if horizontal {
                    Rect {
                        x: position,
                        y: 0.0,
                        w: sizes[j],
                        h: content.h,
                    }
                } else {
                    Rect {
                        x: 0.0,
                        y: position,
                        w: content.w,
                        h: sizes[j],
                    }
                };
                position += sizes[j] + bar;
                (child, rect)
            })
            .collect()
    }
}

/// Place retained children in a native popup's body using the same geometry as the main tree.
#[allow(clippy::too_many_arguments)]
pub(crate) fn arrange_children_at(
    tree: &Tree,
    resolved: &[Resolved],
    id: NodeId,
    body: Rect,
    scroll: &mut [Size],
    text: &impl TextMeasure,
    scale: f32,
    out: &mut LayoutResult,
) {
    let mut worker = Arranger {
        measurer: Measurer {
            tree,
            resolved,
            text,
        },
        scroll,
        out,
        scale,
    };
    let children = visible_children(tree, resolved, id);
    for (child, mut rect) in worker.linear(id, &children, body) {
        rect.x += body.x;
        rect.y += body.y;
        worker.node(child, rect, body);
    }
}

fn cross(align: Align, available: f32, natural: f32) -> (f32, f32) {
    match align {
        Align::Stretch => (0.0, available),
        Align::Start => (0.0, natural.min(available)),
        Align::Center => ((available - natural).max(0.0) / 2.0, natural.min(available)),
        Align::End => ((available - natural).max(0.0), natural.min(available)),
    }
}

fn justify(justify: Justify, available: f32, sizes: &[f32], gap: f32) -> (f32, f32) {
    let free = (available - sizes.iter().sum::<f32>() - gap * sizes.len().saturating_sub(1) as f32)
        .max(0.0);
    match justify {
        Justify::Start => (0.0, gap),
        Justify::Center => (free / 2.0, gap),
        Justify::End => (free, gap),
        Justify::SpaceBetween => (
            0.0,
            gap + if sizes.len() > 1 {
                free / (sizes.len() - 1) as f32
            } else {
                0.0
            },
        ),
        Justify::SpaceAround => {
            let extra = free / sizes.len().max(1) as f32;
            (extra / 2.0, gap + extra)
        }
        Justify::SpaceEvenly => {
            let extra = free / (sizes.len() + 1) as f32;
            (extra, gap + extra)
        }
    }
}

/// Returns unconsumed wheel delta, allowing the next ancestor to consume it.
pub fn scroll_axis(offset: &mut f32, delta: f32, extent: f32, viewport: f32) -> f32 {
    let old = *offset;
    *offset = (*offset + delta).clamp(0.0, (extent - viewport).max(0.0));
    delta - (*offset - old)
}
