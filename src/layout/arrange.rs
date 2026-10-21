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
        let rect = rect.rounded(self.scale);
        let node = &self.measurer.tree.nodes[id].spec;
        let mut content = rect.inset(node.layout.padding + node.style.border_width);
        if !self.measurer.resolved[id].title.is_empty() {
            let title_height = node.style.font_size.unwrap_or(16.0) + 12.0;
            content.y += title_height;
            content.h = (content.h - title_height).max(0.0);
        }
        if node.layout.overflow == Overflow::Scroll {
            content.w = (content.w - 12.0).max(0.0);
            content.h = (content.h - 12.0).max(0.0);
        }
        let clip = parent_clip.intersect(rect);
        let children = visible_children(self.measurer.tree, self.measurer.resolved, id);
        let placements = if node.kind == Kind::Grid {
            self.grid(id, &children, content)
        } else if children.is_empty() {
            vec![]
        } else {
            self.linear(id, &children, content)
        };
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
        let heights = flex(&row_items, content.h, layout.gap);
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
                x += available + layout.gap;
            }
            y += height + layout.gap;
        }
        placements
    }

    fn linear(&self, id: NodeId, children: &[NodeId], content: Rect) -> Vec<(NodeId, Rect)> {
        let node = &self.measurer.tree.nodes[id].spec;
        let layout = &node.layout;
        let horizontal = node.kind == Kind::Row;
        let natural: Vec<_> = children
            .iter()
            .map(|&child| self.measurer.at_width(child, Limit::Bounded(content.w)))
            .collect();
        let bases: Vec<_> = natural
            .iter()
            .map(|size| if horizontal { size.w } else { size.h })
            .collect();
        let lines = if horizontal && layout.wrap {
            wrap_lines(&bases, content.w, layout.gap)
        } else {
            std::iter::once(0..children.len()).collect()
        };
        let mut placements = Vec::with_capacity(children.len());
        let mut line_y = 0.0;
        for line in lines {
            let items: Vec<_> = line
                .clone()
                .map(|j| {
                    item(
                        &self.measurer.tree.nodes[children[j]].spec.layout,
                        bases[j],
                        horizontal,
                    )
                })
                .collect();
            let available = if horizontal {
                content.w
            } else if layout.overflow == Overflow::Scroll {
                content.h.max(
                    bases.iter().sum::<f32>()
                        + layout.gap * children.len().saturating_sub(1) as f32,
                )
            } else {
                content.h
            };
            let sizes = flex(&items, available, layout.gap);
            let (mut position, gap) = justify(layout.justify, available, &sizes, layout.gap);
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
            line_y += cross_available + layout.gap;
        }
        placements
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
    }
}

/// Returns unconsumed wheel delta, allowing the next ancestor to consume it.
pub fn scroll_axis(offset: &mut f32, delta: f32, extent: f32, viewport: f32) -> f32 {
    let old = *offset;
    *offset = (*offset + delta).clamp(0.0, (extent - viewport).max(0.0));
    delta - (*offset - old)
}
