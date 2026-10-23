use super::*;
component!(&[("block", Bool), ("vertical", Bool)], &["change"]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::selection::paint(c, shared::selection::Mode::Segmented);
}
