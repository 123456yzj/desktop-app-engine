use super::*;
component!(
    shared::tree_selection::PROPERTIES,
    &["change", "openChange", "clear"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::tree_selection::paint(c, false);
}
