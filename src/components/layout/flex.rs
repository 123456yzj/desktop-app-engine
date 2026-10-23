use super::Limit;
use crate::v02::{ColumnSpec, Layout, Node, Track};

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
    let target = (available - gap * items.len().saturating_sub(1) as f32).max(0.0);
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

pub(super) fn dimension(
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

pub(super) fn axis(
    layout: &Layout,
    horizontal: bool,
) -> (Option<&Track>, Option<f32>, Option<f32>) {
    if horizontal {
        (layout.width.as_ref(), layout.min_width, layout.max_width)
    } else {
        (layout.height.as_ref(), layout.min_height, layout.max_height)
    }
}

pub(super) fn item(layout: &Layout, natural: f32, horizontal: bool) -> FlexItem {
    let (track, min, max) = axis(layout, horizontal);
    FlexItem {
        base: track.and_then(|t| t.fixed).unwrap_or_else(|| {
            if track.is_some_and(|t| t.fr.is_some()) {
                min.unwrap_or(0.0)
            } else {
                natural
            }
        }),
        min: min.unwrap_or(0.0),
        max: max.unwrap_or(1e8),
        grow: track.and_then(|t| t.fr).unwrap_or(0.0),
        shrink: layout
            .shrink
            .unwrap_or(if track.is_some_and(|t| t.fixed.is_some()) {
                0.0
            } else {
                1.0
            }),
    }
}

pub(super) fn grid_widths(node: &Node, available: f32) -> Vec<f32> {
    let items: Vec<_> = node
        .columns
        .iter()
        .filter_map(|column| {
            if let ColumnSpec::Track(track) = column {
                Some(FlexItem {
                    base: track.fixed.unwrap_or(0.0),
                    min: 0.0,
                    max: 1e8,
                    grow: track.fr.unwrap_or(0.0),
                    shrink: 0.0,
                })
            } else {
                None
            }
        })
        .collect();
    flex(&items, available, node.layout.column_gap())
}
