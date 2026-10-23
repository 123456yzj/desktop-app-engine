use super::*;
component!(
    &[
        ("items", Items),
        ("current", Integer(0, 10000)),
        ("initial", Integer(0, 10000)),
        ("direction", Enum(&["vertical", "horizontal"])),
        ("status", Enum(&["wait", "process", "finish", "error"])),
        ("percent", Number(0., 100.)),
        ("progressDot", Bool)
    ],
    &["change"]; Some("current"), &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::selection::paint(c, shared::selection::Mode::Steps);
}
