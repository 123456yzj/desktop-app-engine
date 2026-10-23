use super::*;
component!(
    &[
        ("items", Items),
        ("activeKey", Text),
        ("defaultActiveKey", Text),
        ("tabPlacement", Enum(&["top", "bottom", "start", "end"])),
        ("tabType", Enum(&["line", "card", "editable-card"])),
        ("centered", Bool),
        ("hideAdd", Bool),
        ("tabBarGutter", Number(0., 256.))
    ],
    &["change", "tabClick", "edit", "tabScroll"]; Some("activeKey"), &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::selection::paint(c, shared::selection::Mode::Tabs);
}
