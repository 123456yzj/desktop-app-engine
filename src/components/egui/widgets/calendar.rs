use super::*;
component!(shared::date::PROPERTIES, &["change"]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        state,
        memory,
        value,
        ..
    } = c;
    shared::date::calendar(ui, n, state, memory, value);
}
