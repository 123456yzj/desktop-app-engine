use super::*;
component!(
    &[
        ("src", Text),
        ("alt", Text),
        ("shape", Enum(&["circle", "square"])),
        ("icon", Text)
    ],
    &["load", "error", "preview"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::image::paint(c, true);
}
