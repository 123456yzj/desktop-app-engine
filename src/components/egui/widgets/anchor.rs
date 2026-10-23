use super::*;
component!(
    &[
        ("items", Items),
        ("direction", Enum(&["vertical", "horizontal"])),
        ("offsetTop", Number(0., 10000.)),
        ("targetOffset", Number(0., 10000.))
    ],
    &["click"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::selection::paint(c, shared::selection::Mode::Anchor);
}
