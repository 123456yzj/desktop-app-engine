//! Finite point geometry and layout. Text measurement is injected; no egui dependency.
use crate::{
    component::{NodeId, Resolved, Tree},
    v02::{Align, ColumnSpec, Justify, Kind, Overflow, Track},
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub fn inset(self, p: f32) -> Self {
        Self {
            x: self.x + p,
            y: self.y + p,
            w: (self.w - 2.0 * p).max(0.0),
            h: (self.h - 2.0 * p).max(0.0),
        }
    }
    pub fn intersect(self, b: Self) -> Self {
        let x = self.x.max(b.x);
        let y = self.y.max(b.y);
        Self {
            x,
            y,
            w: (self.x + self.w).min(b.x + b.w).sub(x).max(0.0),
            h: (self.y + self.h).min(b.y + b.h).sub(y).max(0.0),
        }
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        self.w > 0.0
            && self.h > 0.0
            && x >= self.x
            && x <= self.x + self.w
            && y >= self.y
            && y <= self.y + self.h
    }
}
use std::ops::Sub;
#[derive(Clone, Copy, Debug)]
pub enum Limit {
    Bounded(f32),
    Unbounded,
}
#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    pub width: Limit,
    pub height: Limit,
}
pub trait TextMeasure {
    fn text(&self, text: &str, width: f32, font_size: f32) -> Size;
    fn styled_text(&self, text: &str, width: f32, style: &crate::v02::Style) -> Size {
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
#[derive(Clone, Copy, Debug)]
pub struct FlexItem {
    pub base: f32,
    pub min: f32,
    pub max: f32,
    pub grow: f32,
    pub shrink: f32,
}
/// Iterative min/max freezing for positive and negative space.
pub fn flex(items: &[FlexItem], available: f32, gap: f32) -> Vec<f32> {
    let mut sizes: Vec<_> = items.iter().map(|i| i.base.clamp(i.min, i.max)).collect();
    let target = (available - gap * (items.len().saturating_sub(1)) as f32).max(0.0);
    let mut frozen = vec![false; items.len()];
    for _ in 0..=items.len() {
        let free = target - sizes.iter().sum::<f32>();
        if free.abs() < 0.001 {
            break;
        }
        let weights: Vec<_> = items
            .iter()
            .enumerate()
            .map(|(j, i)| {
                if frozen[j] {
                    0.0
                } else if free > 0.0 {
                    i.grow
                } else {
                    i.shrink * i.base
                }
            })
            .collect();
        let sum = weights.iter().sum::<f32>();
        if sum <= 0.0 {
            break;
        }
        let mut clamped = false;
        for (j, i) in items.iter().enumerate() {
            if frozen[j] {
                continue;
            }
            let proposed = sizes[j] + free * weights[j] / sum;
            sizes[j] = proposed.clamp(i.min, i.max);
            if (sizes[j] - proposed).abs() > 0.001 {
                frozen[j] = true;
                clamped = true;
            }
        }
        if !clamped {
            break;
        }
    }
    sizes
}
pub fn wrap_lines(bases: &[f32], available: f32, gap: f32) -> Vec<std::ops::Range<usize>> {
    let mut result = vec![];
    let mut start = 0;
    let mut used = 0.0;
    for (i, b) in bases.iter().enumerate() {
        if i > start && used + gap + b > available {
            result.push(start..i);
            start = i;
            used = 0.0;
        }
        used += if i == start { *b } else { gap + b };
    }
    if start < bases.len() {
        result.push(start..bases.len());
    }
    result
}
#[derive(Clone, Copy, Debug, Default)]
pub struct BoxLayout {
    pub rect: Rect,
    pub content: Rect,
    pub clip: Rect,
    pub extent: Size,
}
pub struct LayoutResult {
    pub boxes: Vec<BoxLayout>,
}
fn dimension(
    track: Option<&Track>,
    natural: f32,
    available: Limit,
    min: Option<f32>,
    max: Option<f32>,
) -> f32 {
    let n = match track {
        Some(t) if t.fixed.is_some() => t.fixed.unwrap(),
        Some(t) if t.fr.is_some() => match available {
            Limit::Bounded(n) => n,
            Limit::Unbounded => natural,
        },
        _ => natural,
    };
    n.clamp(min.unwrap_or(0.0), max.unwrap_or(1e8))
}
fn item(tree: &Tree, id: usize, natural: f32, horizontal: bool) -> FlexItem {
    let l = &tree.nodes[id].spec.layout;
    let (track, min, max) = if horizontal {
        (&l.width, l.min_width, l.max_width)
    } else {
        (&l.height, l.min_height, l.max_height)
    };
    FlexItem {
        base: track.as_ref().and_then(|t| t.fixed).unwrap_or_else(|| {
            if track.as_ref().is_some_and(|t| t.fr.is_some()) {
                min.unwrap_or(0.0)
            } else {
                natural
            }
        }),
        min: min.unwrap_or(0.0),
        max: max.unwrap_or(1e8),
        grow: track.as_ref().and_then(|t| t.fr).unwrap_or(0.0),
        shrink: l
            .shrink
            .unwrap_or(if track.as_ref().is_some_and(|t| t.fixed.is_some()) {
                0.0
            } else {
                1.0
            }),
    }
}
pub fn measure(
    tree: &Tree,
    resolved: &[Resolved],
    id: NodeId,
    c: Constraint,
    text: &impl TextMeasure,
) -> Size {
    if !resolved[id].visible {
        return Size::default();
    }
    let n = &tree.nodes[id];
    let s = &n.spec;
    let l = &s.layout;
    let p = l.padding + s.style.border_width;
    let w = match c.width {
        Limit::Bounded(w) => w,
        Limit::Unbounded => 100_000.0,
    };
    let w = l
        .width
        .as_ref()
        .and_then(|t| t.fixed)
        .unwrap_or(w)
        .min(l.max_width.unwrap_or(1e8))
        .max(l.min_width.unwrap_or(0.0));
    let inner = (w - 2.0 * p).max(0.0);
    let font = s.style.font_size.unwrap_or(16.0);
    let r = &resolved[id];
    let children: Vec<_> = n
        .children
        .iter()
        .copied()
        .filter(|&i| resolved[i].visible)
        .collect();
    let mut natural = match s.kind {
        Kind::Text | Kind::Badge | Kind::Button => {
            let mut size = text.styled_text(&r.text, inner, &s.style);
            if s.kind != Kind::Text {
                size.w += 2.0 * s.style.padding_x.unwrap_or(10.0);
                size.h += 2.0 * s.style.padding_y.unwrap_or(6.0);
            }
            size
        }
        Kind::Input | Kind::Select => Size { w: 220.0, h: 34.0 },
        Kind::Metric => Size {
            w: text
                .text(&r.text, inner, font * 1.875)
                .w
                .max(text.text(&r.label, inner, font).w),
            h: font * 4.125,
        },
        Kind::Progress => Size { w: 160.0, h: 32.0 },
        Kind::Table => Size {
            w: if matches!(c.width, Limit::Unbounded) {
                400.0
            } else {
                inner
            },
            h: 280.0,
        },
        Kind::Plugin | Kind::LineChart | Kind::BarChart => Size {
            w: if matches!(c.width, Limit::Unbounded) {
                600.0
            } else {
                inner
            },
            h: 240.0,
        },
        Kind::Grid if children.is_empty() => Size::default(),
        Kind::Grid => {
            let widths = if matches!(c.width, Limit::Unbounded)
                && l.width.as_ref().and_then(|t| t.fixed).is_none()
            {
                s.columns
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
                                .filter(|(j, _)| j % s.columns.len() == col)
                                .map(|(_, child)| {
                                    measure(
                                        tree,
                                        resolved,
                                        *child,
                                        Constraint {
                                            width: Limit::Unbounded,
                                            height: Limit::Unbounded,
                                        },
                                        text,
                                    )
                                    .w
                                })
                                .fold(0.0, f32::max)
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                grid_widths(s, inner)
            };
            let mut heights = vec![0.0_f32; children.len().div_ceil(widths.len())];
            for (j, &child) in children.iter().enumerate() {
                let m = measure(
                    tree,
                    resolved,
                    child,
                    Constraint {
                        width: Limit::Bounded(widths[j % widths.len()]),
                        height: Limit::Unbounded,
                    },
                    text,
                );
                heights[j / widths.len()] = heights[j / widths.len()].max(m.h);
            }
            Size {
                w: widths.iter().sum::<f32>() + l.gap * widths.len().saturating_sub(1) as f32,
                h: heights.iter().sum::<f32>() + l.gap * heights.len().saturating_sub(1) as f32,
            }
        }
        Kind::Row => {
            let measured: Vec<_> = children
                .iter()
                .map(|&child| {
                    measure(
                        tree,
                        resolved,
                        child,
                        Constraint {
                            width: Limit::Bounded(inner),
                            height: Limit::Unbounded,
                        },
                        text,
                    )
                })
                .collect();
            let bases: Vec<_> = measured
                .iter()
                .enumerate()
                .map(|(j, s)| item(tree, children[j], s.w, true).base)
                .collect();
            let lines = if l.wrap {
                wrap_lines(&bases, inner, l.gap)
            } else if children.is_empty() {
                vec![]
            } else {
                std::iter::once(0..children.len()).collect()
            };
            let mut height = 0.0_f32;
            let mut width = 0.0_f32;
            for line in &lines {
                let items: Vec<_> = line
                    .clone()
                    .map(|j| item(tree, children[j], measured[j].w, true))
                    .collect();
                let widths = flex(&items, inner, l.gap);
                let mut row_h = 0.0_f32;
                for (j, w) in line.clone().zip(&widths) {
                    row_h = row_h.max(
                        measure(
                            tree,
                            resolved,
                            children[j],
                            Constraint {
                                width: Limit::Bounded(*w),
                                height: Limit::Unbounded,
                            },
                            text,
                        )
                        .h,
                    );
                }
                height += row_h;
                width = width.max(
                    widths.iter().sum::<f32>() + l.gap * widths.len().saturating_sub(1) as f32,
                );
            }
            Size {
                w: width,
                h: height + l.gap * lines.len().saturating_sub(1) as f32,
            }
        }
        _ => {
            let mut size = Size::default();
            for &child in &children {
                let m = measure(
                    tree,
                    resolved,
                    child,
                    Constraint {
                        width: Limit::Bounded(inner),
                        height: Limit::Unbounded,
                    },
                    text,
                );
                size.w = size.w.max(m.w);
                size.h += m.h;
            }
            size.h += l.gap * children.len().saturating_sub(1) as f32;
            if !r.title.is_empty() {
                size.h += font + 12.0;
            }
            size
        }
    };
    natural.w += 2.0 * p;
    natural.h += 2.0 * p;
    Size {
        w: dimension(
            l.width.as_ref(),
            natural.w,
            c.width,
            l.min_width,
            l.max_width,
        ),
        h: dimension(
            l.height.as_ref(),
            natural.h,
            c.height,
            l.min_height,
            l.max_height,
        ),
    }
}
fn grid_widths(s: &crate::v02::Node, available: f32) -> Vec<f32> {
    let items: Vec<_> = s
        .columns
        .iter()
        .filter_map(|c| {
            if let ColumnSpec::Track(t) = c {
                Some(FlexItem {
                    base: t.fixed.unwrap_or(0.0),
                    min: 0.0,
                    max: 1e8,
                    grow: t.fr.unwrap_or(0.0),
                    shrink: 0.0,
                })
            } else {
                None
            }
        })
        .collect();
    flex(&items, available, s.layout.gap)
}
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
        tree,
        resolved,
        scroll,
        text,
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
    tree: &'a Tree,
    resolved: &'a [Resolved],
    scroll: &'a mut [Size],
    text: &'a T,
    out: &'a mut LayoutResult,
    scale: f32,
}
impl<T: TextMeasure> Arranger<'_, T> {
    fn node(&mut self, id: usize, rect: Rect, parent_clip: Rect) {
        if !self.resolved[id].visible {
            return;
        }
        let round = |n: f32| (n * self.scale).round() / self.scale;
        let rect = Rect {
            x: round(rect.x),
            y: round(rect.y),
            w: round(rect.x + rect.w) - round(rect.x),
            h: round(rect.y + rect.h) - round(rect.y),
        };
        let n = &self.tree.nodes[id];
        let s = &n.spec;
        let l = &s.layout;
        let mut content = rect.inset(l.padding + s.style.border_width);
        if !self.resolved[id].title.is_empty() {
            let title_height = s.style.font_size.unwrap_or(16.0) + 12.0;
            content.y += title_height;
            content.h = (content.h - title_height).max(0.0);
        }
        if l.overflow == Overflow::Scroll {
            content.w = (content.w - 12.0).max(0.0);
            content.h = (content.h - 12.0).max(0.0);
        }
        let clip = parent_clip.intersect(rect);
        let children: Vec<_> = n
            .children
            .iter()
            .copied()
            .filter(|&i| self.resolved[i].visible)
            .collect();
        let mut placements = Vec::new();
        if s.kind == Kind::Grid {
            let widths = grid_widths(s, content.w);
            let row_items: Vec<_> = children
                .chunks(widths.len())
                .map(|chunk| {
                    let natural = chunk
                        .iter()
                        .enumerate()
                        .map(|(j, &c)| {
                            measure(
                                self.tree,
                                self.resolved,
                                c,
                                Constraint {
                                    width: Limit::Bounded(widths[j]),
                                    height: Limit::Unbounded,
                                },
                                self.text,
                            )
                            .h
                        })
                        .fold(0.0, f32::max);
                    let grow = chunk
                        .iter()
                        .filter_map(|&c| {
                            self.tree.nodes[c]
                                .spec
                                .layout
                                .height
                                .as_ref()
                                .and_then(|t| t.fr)
                        })
                        .fold(0.0, f32::max);
                    FlexItem {
                        base: if grow > 0.0 { 0.0 } else { natural },
                        min: 0.0,
                        max: 1e8,
                        grow: if grow == 0.0
                            && children.len() <= widths.len()
                            && l.height
                                .as_ref()
                                .is_some_and(|t| t.fixed.is_some() || t.fr.is_some())
                        {
                            1.0
                        } else {
                            grow
                        },
                        shrink: 0.0,
                    }
                })
                .collect();
            let heights = flex(&row_items, content.h, l.gap);
            let mut y = 0.0;
            for (row, chunk) in children.chunks(widths.len()).enumerate() {
                let measured: Vec<_> = chunk
                    .iter()
                    .enumerate()
                    .map(|(j, &c)| {
                        measure(
                            self.tree,
                            self.resolved,
                            c,
                            Constraint {
                                width: Limit::Bounded(widths[j]),
                                height: Limit::Unbounded,
                            },
                            self.text,
                        )
                    })
                    .collect();
                let height = heights[row];
                let mut x = 0.0;
                for (j, &c) in chunk.iter().enumerate() {
                    let desired = dimension(
                        self.tree.nodes[c].spec.layout.height.as_ref(),
                        measured[j].h,
                        Limit::Bounded(height),
                        self.tree.nodes[c].spec.layout.min_height,
                        self.tree.nodes[c].spec.layout.max_height,
                    );
                    let (dy, h) = cross(l.align, height, desired);
                    let child_layout = &self.tree.nodes[c].spec.layout;
                    let width = dimension(
                        child_layout.width.as_ref(),
                        widths[j],
                        Limit::Bounded(widths[j]),
                        child_layout.min_width,
                        child_layout.max_width,
                    );
                    let (dx, _) = cross(l.align, widths[j], width);
                    placements.push((
                        c,
                        Rect {
                            x: x + dx,
                            y: y + dy,
                            w: width,
                            h,
                        },
                    ));
                    x += widths[j] + l.gap;
                }
                y += height + l.gap;
            }
        } else if !children.is_empty() {
            let horizontal = s.kind == Kind::Row;
            let natural: Vec<_> = children
                .iter()
                .map(|&c| {
                    measure(
                        self.tree,
                        self.resolved,
                        c,
                        Constraint {
                            width: Limit::Bounded(content.w),
                            height: Limit::Unbounded,
                        },
                        self.text,
                    )
                })
                .collect();
            let bases: Vec<_> = natural
                .iter()
                .map(|m| if horizontal { m.w } else { m.h })
                .collect();
            let lines = if horizontal && l.wrap {
                wrap_lines(&bases, content.w, l.gap)
            } else {
                std::iter::once(0..children.len()).collect()
            };
            let mut line_y = 0.0;
            for line in lines {
                let items: Vec<_> = line
                    .clone()
                    .map(|j| item(self.tree, children[j], bases[j], horizontal))
                    .collect();
                let available = if horizontal {
                    content.w
                } else if l.overflow == Overflow::Scroll {
                    content.h.max(
                        bases.iter().sum::<f32>() + l.gap * children.len().saturating_sub(1) as f32,
                    )
                } else {
                    content.h
                };
                let sizes = flex(&items, available, l.gap);
                let free = (available
                    - sizes.iter().sum::<f32>()
                    - l.gap * sizes.len().saturating_sub(1) as f32)
                    .max(0.0);
                let (mut position, gap) = match l.justify {
                    Justify::Start => (0.0, l.gap),
                    Justify::Center => (free / 2.0, l.gap),
                    Justify::End => (free, l.gap),
                    Justify::SpaceBetween => (
                        0.0,
                        l.gap
                            + if sizes.len() > 1 {
                                free / (sizes.len() - 1) as f32
                            } else {
                                0.0
                            },
                    ),
                };
                let row_heights: Vec<_> = line
                    .clone()
                    .enumerate()
                    .map(|(k, j)| {
                        if horizontal {
                            measure(
                                self.tree,
                                self.resolved,
                                children[j],
                                Constraint {
                                    width: Limit::Bounded(sizes[k]),
                                    height: Limit::Unbounded,
                                },
                                self.text,
                            )
                            .h
                        } else {
                            natural[j].w
                        }
                    })
                    .collect();
                let cross_available = if horizontal {
                    if l.wrap {
                        row_heights.iter().copied().fold(0.0, f32::max)
                    } else {
                        content.h
                    }
                } else {
                    content.w
                };
                for (k, j) in line.enumerate() {
                    let child_layout = &self.tree.nodes[children[j]].spec.layout;
                    let (track, min, max) = if horizontal {
                        (
                            &child_layout.height,
                            child_layout.min_height,
                            child_layout.max_height,
                        )
                    } else {
                        (
                            &child_layout.width,
                            child_layout.min_width,
                            child_layout.max_width,
                        )
                    };
                    let desired = dimension(
                        track.as_ref(),
                        if l.align == Align::Stretch {
                            cross_available
                        } else {
                            row_heights[k]
                        },
                        Limit::Bounded(cross_available),
                        min,
                        max,
                    );
                    let (offset, _) = cross(l.align, cross_available, desired);
                    let cross_size = desired;
                    let r = if horizontal {
                        Rect {
                            x: position,
                            y: line_y + offset,
                            w: sizes[k],
                            h: cross_size,
                        }
                    } else {
                        Rect {
                            x: offset,
                            y: position,
                            w: cross_size,
                            h: sizes[k],
                        }
                    };
                    placements.push((children[j], r));
                    position += sizes[k] + gap;
                }
                line_y += cross_available + l.gap;
            }
        }
        let extent = placements.iter().fold(Size::default(), |a, (_, r)| Size {
            w: a.w.max(r.x + r.w),
            h: a.h.max(r.y + r.h),
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
        for (child, mut r) in placements {
            r.x += content.x - self.scroll[id].w;
            r.y += content.y - self.scroll[id].h;
            self.node(child, r, clip.intersect(content));
        }
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

/// Returns unconsumed wheel delta, allowing the next ancestor to consume it.
pub fn scroll_axis(offset: &mut f32, delta: f32, extent: f32, viewport: f32) -> f32 {
    let old = *offset;
    *offset = (*offset + delta).clamp(0.0, (extent - viewport).max(0.0));
    delta - (*offset - old)
}
