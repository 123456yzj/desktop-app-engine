use super::Rect;
use crate::protocol::Node;

pub(super) fn columns(node: &Node, available: f32) -> (usize, f32) {
    let maximum = node.column_count.unwrap_or(3);
    let gap = node.layout.column_gap();
    let count = node.min_column_width.map_or(maximum, |min| {
        (((available + gap) / (min + gap)).floor() as usize).clamp(1, maximum)
    });
    (
        count,
        (available - gap * count.saturating_sub(1) as f32).max(0.0) / count as f32,
    )
}

/// Stable shortest-column placement. Hidden children are removed before this call.
pub(super) fn positions(node: &Node, available: f32, heights: &[f32]) -> Vec<Rect> {
    let (count, width) = columns(node, available);
    let mut bottoms = vec![0.0_f32; count];
    heights
        .iter()
        .map(|&height| {
            let column = (0..count)
                .min_by(|&a, &b| bottoms[a].total_cmp(&bottoms[b]))
                .unwrap();
            let y = bottoms[column];
            bottoms[column] = y + height + node.layout.row_gap();
            Rect {
                x: column as f32 * (width + node.layout.column_gap()),
                y,
                w: width,
                h: height,
            }
        })
        .collect()
}
