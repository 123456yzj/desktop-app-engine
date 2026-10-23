use super::*;
component!(
    &[
        ("src", Text),
        ("alt", Text),
        ("fallback", Text),
        ("preview", Bool),
        ("fit", Enum(&["contain", "cover", "fill"]))
    ],
    &["load", "error", "preview"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::image::paint(c, false);
}
