use super::*;
component!(
    shared::dialog::PROPERTIES,
    &["openChange", "ok", "cancel", "close"]; None, &[("openChange", "open")]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::dialog::paint(c, false);
}
