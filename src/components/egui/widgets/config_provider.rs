use super::*;
component!(shared::config::PROPERTIES, &[]);
pub(super) fn prepare(r: &mut Resolved) {
    shared::config::prepare(r);
}
