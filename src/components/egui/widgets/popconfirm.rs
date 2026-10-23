use super::*;
component!(
    shared::popup::PROPERTIES,
    &["openChange", "confirm", "cancel"]; None, &[("openChange", "open")]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::popup::paint(c, "click", true);
}
