//! Finite point geometry and layout. Text measurement is injected; no egui dependency.
mod arrange;
mod flex;
mod geometry;
mod measure;

pub use arrange::{arrange, scroll_axis};
pub use flex::{FlexItem, flex, wrap_lines};
pub use geometry::{Constraint, Limit, Rect, Size};
pub use measure::{TextMeasure, measure};

use crate::component::{NodeId, Resolved, Tree};

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
