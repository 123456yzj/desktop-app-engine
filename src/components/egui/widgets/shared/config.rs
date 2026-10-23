use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("componentSize", crate::components::contracts::SIZE),
    ("componentDisabled", Bool),
    ("theme", Enum(&["light", "dark"])),
    ("direction", Enum(&["ltr", "rtl"])),
    ("locale", Enum(&["zh-CN", "en-US"])),
];
pub(in crate::components::egui::widgets) fn prepare(r: &mut Resolved) {
    for (name, key) in [
        ("theme", "@theme"),
        ("componentSize", "@size"),
        ("locale", "@locale"),
        ("direction", "@direction"),
    ] {
        if let Some(value) = r.api.get(name).cloned() {
            r.api.insert(key.into(), value);
        }
    }
}
