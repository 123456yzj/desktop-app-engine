use super::*;
component!(shared::toast::PROPERTIES, &["openChange", "close"]; None, &[("openChange", "open")]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::toast::paint(c, "top", false);
}
