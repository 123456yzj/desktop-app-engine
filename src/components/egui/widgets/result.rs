use super::*;
component!(
    &[
        (
            "status",
            Enum(&["success", "error", "info", "warning", "403", "404", "500"])
        ),
        ("subTitle", Text)
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui, n, r, state, ..
    } = c;
    ui.vertical_centered(|ui| {
        let status = contract::string(n, state, "status", "info");
        ui.label(
            egui::RichText::new(if matches!(status, "403" | "404" | "500") {
                status
            } else {
                shared::icon(status)
            })
            .size(44.),
        );
        ui.heading(&r.title);
        ui.label(&r.text);
        ui.weak(contract::string(n, state, "subTitle", ""));
    });
}
