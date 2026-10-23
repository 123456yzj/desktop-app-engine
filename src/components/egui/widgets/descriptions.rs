use super::*;
component!(
    &[
        ("items", Items),
        ("column", Integer(1, 24)),
        ("bordered", Bool),
        ("colon", Bool),
        ("direction", Enum(&["vertical", "horizontal"]))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext { ui, n, state, .. } = c;
    let columns = contract::number(n, state, "column", 3.) as usize;
    egui::Grid::new("descriptions")
        .num_columns(columns * 2)
        .striped(contract::flag(n, state, "bordered", false))
        .show(ui, |ui| {
            for (index, item) in contract::items(n, state).iter().enumerate() {
                let content = item["content"]
                    .as_str()
                    .or_else(|| item["text"].as_str())
                    .unwrap_or("");
                if contract::string(n, state, "direction", "horizontal") == "vertical" {
                    ui.vertical(|ui| {
                        ui.weak(item_label(item));
                        ui.label(content);
                    });
                } else {
                    ui.weak(format!(
                        "{}{}",
                        item_label(item),
                        if contract::flag(n, state, "colon", true) {
                            ":"
                        } else {
                            ""
                        }
                    ));
                    ui.label(content);
                }
                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
        });
}
