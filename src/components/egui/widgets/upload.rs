use super::*;
component!(
    &[
        ("accept", Text),
        ("multiple", Bool),
        ("maxCount", Integer(1, 10000)),
        ("maxSize", Integer(1, 10000000000)),
        ("directory", Bool),
        ("showUploadList", Bool),
        ("listType", Enum(&["text", "picture", "picture-card"]))
    ],
    &["change", "remove", "preview", "reject"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        value,
        collector,
        ..
    } = c;
    let mut files = value.as_array().cloned().unwrap_or_default();
    if ui
        .button(if r.text.is_empty() {
            "选择文件"
        } else {
            &r.text
        })
        .clicked()
    {
        let multiple = contract::flag(n, state, "multiple", false);
        let dialog = rfd::FileDialog::new();
        let selected = if contract::flag(n, state, "directory", false) {
            dialog.pick_folder().map(|p| vec![p])
        } else if multiple {
            dialog.pick_files()
        } else {
            dialog.pick_file().map(|p| vec![p])
        };
        if let Some(paths) = selected {
            if !multiple {
                files.clear();
            }
            let max =
                contract::number(n, state, "maxCount", if multiple { 10000. } else { 1. }) as usize;
            for path in paths {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                let accept = contract::string(n, state, "accept", "");
                let allowed = accept.is_empty()
                    || accept
                        .split(',')
                        .any(|suffix| name.to_lowercase().ends_with(&suffix.trim().to_lowercase()));
                if files.len() >= max
                    || size as f64 > contract::number(n, state, "maxSize", 10000000000.)
                    || !allowed
                {
                    collector.push(ui, "reject", None, json!({"name":name,"size":size}));
                    continue;
                }
                let path = path.to_string_lossy().into_owned();
                if !files.iter().any(|f| f["path"] == path) {
                    files.push(json!({"uid":path,"path":path,"name":name,"size":size,"status":"selected","percent":0}));
                }
            }
        }
    }
    if contract::flag(n, state, "showUploadList", true) {
        let mut removed = None;
        for (index, file) in files.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.link(file["name"].as_str().unwrap_or("文件")).clicked() {
                    collector.push(ui, "preview", None, file.clone());
                }
                if ui.small_button("×").clicked() {
                    removed = Some(index);
                }
            });
        }
        if let Some(index) = removed {
            collector.push(ui, "remove", None, files.remove(index));
        }
    }
    *value = json!(files);
}
