use super::*;
component!(
    &[
        ("prefix", Text),
        ("split", Text),
        ("rows", Integer(1, 100)),
        ("allowClear", Bool)
    ],
    &["change", "select", "search", "clear", "submit"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::suggestions::paint(c, true);
}
