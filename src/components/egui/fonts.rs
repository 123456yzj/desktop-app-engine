use eframe::egui;

pub fn install_chinese_font(ctx: &egui::Context) -> Result<(), String> {
    let windows = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    let mut fonts = egui::FontDefinitions::default();
    let bytes = ["msyh.ttc", "simhei.ttf", "simsun.ttc"]
        .iter()
        .find_map(|name| {
            std::fs::read(std::path::Path::new(&windows).join("Fonts").join(name)).ok()
        })
        .ok_or("未找到中文系统字体（微软雅黑、黑体或宋体），请安装 Windows 中文字体")?;
    fonts
        .font_data
        .insert("chinese".into(), egui::FontData::from_owned(bytes).into());
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("chinese".into());
    }
    // Status, navigation and action glyphs need a symbol fallback as well as CJK.
    if let Ok(bytes) = std::fs::read(std::path::Path::new(&windows).join("Fonts/seguisym.ttf")) {
        fonts
            .font_data
            .insert("symbols".into(), egui::FontData::from_owned(bytes).into());
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .push("symbols".into());
        }
    }
    let mut bold = Vec::new();
    for name in ["segoeuib.ttf", "msyhbd.ttc"] {
        if let Ok(bytes) = std::fs::read(std::path::Path::new(&windows).join("Fonts").join(name)) {
            fonts
                .font_data
                .insert(name.into(), egui::FontData::from_owned(bytes).into());
            bold.push(name.into());
        }
    }
    bold.extend(fonts.families[&egui::FontFamily::Proportional].clone());
    fonts
        .families
        .insert(egui::FontFamily::Name("bold".into()), bold);
    ctx.set_fonts(fonts);
    Ok(())
}
