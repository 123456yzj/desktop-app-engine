use super::*;
component!(shared::list::PROPERTIES, &[]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    shared::list::paint(c);
}
