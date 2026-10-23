use super::*;
component!(
    &[
        ("allowClear", Bool),
        ("filterOption", Bool),
        ("backfill", Bool),
        ("defaultActiveFirstOption", Bool)
    ],
    &["change", "select", "search", "clear", "submit"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::suggestions::paint(c, false);
}
