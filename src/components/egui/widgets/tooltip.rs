use super::*;
component!(shared::popup::PROPERTIES, &["openChange"]; None, &[("openChange", "open")]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::popup::paint(c, "hover", false);
}
