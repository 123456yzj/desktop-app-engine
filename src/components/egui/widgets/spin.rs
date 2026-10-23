use super::*;
component!(
    &[
        ("spinning", Bool),
        ("tip", Text),
        ("delay", Integer(0, 3600000)),
        ("percent", Number(0., 100.))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        rect,
        state,
        memory,
        ..
    } = c;
    if contract::flag(n, state, "spinning", true) {
        let now = ui.input(|i| i.time);
        let start = *memory.started.get_or_insert(now);
        if now - start >= contract::number(n, state, "delay", 0.) / 1000. {
            ui.painter()
                .rect_filled(rect, 0., token(ui, Token::Surface).gamma_multiply(0.7));
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(contract::string(n, state, "tip", ""));
                if let Some(percent) =
                    contract::property(n, state, "percent").and_then(Value::as_f64)
                {
                    ui.label(format!("{percent:.0}%"));
                }
            });
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(30));
    } else {
        memory.started = None;
    }
}
