use super::*;
component!(&[("items", Items), ("separator", Text)], &["click"]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::selection::paint(c, shared::selection::Mode::Breadcrumb);
}
