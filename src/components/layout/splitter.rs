use super::{FlexItem, flex};
use crate::protocol::{Layout, Node};

pub(crate) fn limits(layout: &Layout, horizontal: bool) -> (f32, f32) {
    if horizontal {
        (
            layout.min_width.unwrap_or(0.0),
            layout.max_width.unwrap_or(1e8),
        )
    } else {
        (
            layout.min_height.unwrap_or(0.0),
            layout.max_height.unwrap_or(1e8),
        )
    }
}

pub(crate) fn bar_size(node: &Node, available: f32) -> f32 {
    node.bar_size.unwrap_or(8.0).min(available)
}

/// Use the same constraints for painting, pointer input, keyboard input and layout.
pub(crate) fn sizes(available: f32, ratio: f32, first: (f32, f32), second: (f32, f32)) -> [f32; 2] {
    let low = first.0.max(available - second.1);
    let high = first.1.min(available - second.0);
    if low <= high {
        let a = (available * ratio).clamp(low, high);
        [a, available - a]
    } else {
        // Infeasible constraints retain minima; normal clipping/scrolling handles overflow.
        let items = [(ratio, first), (1.0 - ratio, second)].map(|(weight, (min, max))| FlexItem {
            base: available * weight,
            min,
            max,
            grow: 1.0,
            shrink: 1.0,
        });
        let sizes = flex(&items, available, 0.0);
        [sizes[0], sizes[1]]
    }
}
