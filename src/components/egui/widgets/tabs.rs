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
        ("hideTabBar", Bool),
        ("tabBarGutter", Number(0., 256.))
    ],
    &["change", "tabClick", "edit", "tabScroll"]; Some("activeKey"), &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    if contract::flag(c.n, c.state, "hideTabBar", false) {
        return;
    }
    shared::selection::paint(c, shared::selection::Mode::Tabs);
}
