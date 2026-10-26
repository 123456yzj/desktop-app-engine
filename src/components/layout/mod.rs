//! Shared component geometry and layout. Text measurement is injected; no egui dependency.
mod arrange;
mod flex;
mod geometry;
mod masonry;
mod measure;
pub(crate) mod splitter;
#[cfg(test)]
mod tests;

pub(crate) use arrange::arrange_children_at;
pub use arrange::{arrange, scroll_axis};
pub use flex::{FlexItem, flex, wrap_lines};
pub use geometry::{Constraint, Limit, Rect, Size};
pub use measure::{TextMeasure, measure};

use crate::component::{NodeId, Resolved, Tree};

/// These components render titles inside their own result, popup or preview.
pub(crate) fn has_layout_title(kind: crate::protocol::Kind) -> bool {
    use crate::protocol::Kind;
    !matches!(
        kind,
        Kind::Modal
            | Kind::Drawer
            | Kind::Message
            | Kind::Notification
            | Kind::Tour
            | Kind::Tooltip
            | Kind::Popover
            | Kind::Popconfirm
            | Kind::Result
            | Kind::Avatar
            | Kind::Image
    )
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

fn visible_children(tree: &Tree, resolved: &[Resolved], id: NodeId) -> Vec<NodeId> {
    tree.nodes[id]
        .children
        .iter()
        .copied()
        .filter(|&child| resolved[child].visible)
        .collect()
}
