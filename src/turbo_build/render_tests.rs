//! Rasterize the actual egui meshes and font atlas without creating an OS window.
use super::*;
use eframe::egui::{
    self, TextureId,
    epaint::{ImageData, Primitive},
};
use std::collections::HashMap;

fn screenshot(model: &Controller, template: &Value, width: usize, height: usize, name: &str) {
    let ctx = egui::Context::default();
    desktop_app_engine::gui::install_chinese_font(&ctx).unwrap();
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.style_mut(|style| style.animation_time = 0.0);
    let mut app = desktop_app_engine::gui_v02::DashboardApp::embedded(
        view::document(template, model).unwrap(),
    )
    .unwrap();
    let mut textures: HashMap<TextureId, (usize, usize, Vec<egui::Color32>)> = HashMap::new();
    let mut output = None;
    let open_branches = name.contains("branches");
    let mut click_pos = egui::Pos2::ZERO;
    for frame_index in 0..if open_branches { 6 } else { 2 } {
        let mut events = vec![];
        if open_branches && matches!(frame_index, 2 | 3) {
            if frame_index == 2 {
                click_pos = text_position(
                    output.as_ref().unwrap(),
                    if name.contains("root-branches") {
                        "feat-copilot"
                    } else {
                        "dev"
                    },
                )
                .unwrap();
            }
            events = vec![
                egui::Event::PointerMoved(click_pos),
                egui::Event::PointerButton {
                    pos: click_pos,
                    button: egui::PointerButton::Primary,
                    pressed: frame_index == 2,
                    modifiers: egui::Modifiers::NONE,
                },
            ];
        }
        let frame = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width as f32, height as f32),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        for (id, delta) in &frame.textures_delta.set {
            let size = delta.image.size();
            let pixels: Vec<egui::Color32> = match &delta.image {
                ImageData::Color(c) => c.pixels.clone(),
                ImageData::Font(f) => f.srgba_pixels(None).collect(),
            };
            if let Some([x, y]) = delta.pos {
                let (tw, _, data) = textures.get_mut(id).unwrap();
                for row in 0..size[1] {
                    data[(y + row) * *tw + x..(y + row) * *tw + x + size[0]]
                        .copy_from_slice(&pixels[row * size[0]..(row + 1) * size[0]]);
                }
            } else {
                textures.insert(*id, (size[0], size[1], pixels));
            }
        }
        output = Some(frame);
    }
    let output = output.unwrap();
    let mut rgba = vec![255u8; width * height * 4];
    for primitive in ctx.tessellate(output.shapes, 1.0) {
        let Primitive::Mesh(mesh) = primitive.primitive else {
            continue;
        };
        let (tw, th, tex) = textures.get(&mesh.texture_id).unwrap();
        assert!(matches!(mesh.texture_id, TextureId::Managed(_)));
        for tri in mesh.indices.as_chunks::<3>().0 {
            let a = mesh.vertices[tri[0] as usize];
            let b = mesh.vertices[tri[1] as usize];
            let c = mesh.vertices[tri[2] as usize];
            let edge = |p: egui::Pos2, q: egui::Pos2, x: f32, y: f32| {
                (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x)
            };
            let area = edge(a.pos, b.pos, c.pos.x, c.pos.y);
            if area.abs() < 0.0001 {
                continue;
            }
            let clip = primitive.clip_rect;
            let x0 = a
                .pos
                .x
                .min(b.pos.x)
                .min(c.pos.x)
                .max(clip.min.x)
                .floor()
                .max(0.0) as usize;
            let y0 = a
                .pos
                .y
                .min(b.pos.y)
                .min(c.pos.y)
                .max(clip.min.y)
                .floor()
                .max(0.0) as usize;
            let x1 = a
                .pos
                .x
                .max(b.pos.x)
                .max(c.pos.x)
                .min(clip.max.x)
                .ceil()
                .min(width as f32) as usize;
            let y1 = a
                .pos
                .y
                .max(b.pos.y)
                .max(c.pos.y)
                .min(clip.max.y)
                .ceil()
                .min(height as f32) as usize;
            for y in y0..y1 {
                for x in x0..x1 {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let u = edge(b.pos, c.pos, px, py) / area;
                    let v = edge(c.pos, a.pos, px, py) / area;
                    let w = 1.0 - u - v;
                    if u < 0.0 || v < 0.0 || w < 0.0 {
                        continue;
                    }
                    let uv = a.uv.to_vec2() * u + b.uv.to_vec2() * v + c.uv.to_vec2() * w;
                    let tx = (uv.x * *tw as f32 - 0.5).clamp(0.0, (*tw - 1) as f32);
                    let ty = (uv.y * *th as f32 - 0.5).clamp(0.0, (*th - 1) as f32);
                    let ix = tx.floor() as usize;
                    let iy = ty.floor() as usize;
                    let fx = tx - ix as f32;
                    let fy = ty - iy as f32;
                    let samples = [
                        (tex[iy * *tw + ix], (1.0 - fx) * (1.0 - fy)),
                        (tex[iy * *tw + (ix + 1).min(*tw - 1)], fx * (1.0 - fy)),
                        (tex[(iy + 1).min(*th - 1) * *tw + ix], (1.0 - fx) * fy),
                        (
                            tex[(iy + 1).min(*th - 1) * *tw + (ix + 1).min(*tw - 1)],
                            fx * fy,
                        ),
                    ];
                    let mut src = [0f32; 4];
                    for k in 0..4 {
                        let color =
                            a.color[k] as f32 * u + b.color[k] as f32 * v + c.color[k] as f32 * w;
                        src[k] = samples
                            .iter()
                            .map(|(c, weight)| c[k] as f32 * weight)
                            .sum::<f32>()
                            * color
                            / 255.0;
                    }
                    let i = (y * width + x) * 4;
                    for k in 0..3 {
                        rgba[i + k] = (src[k] + rgba[i + k] as f32 * (1.0 - src[3] / 255.0))
                            .clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
    }
    // The log panel must be dark through the bottom of the viewport, not stop at natural height.
    let i = ((height - 100) * width + (width - 60)) * 4;
    for (k, expected) in [32i16, 42, 57].iter().enumerate() {
        assert!(
            (rgba[i + k] as i16 - expected).abs() <= 1,
            "log background at bottom must match HTML"
        );
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tmp-verify")
        .join(name);
    resvg::tiny_skia::Pixmap::from_vec(
        rgba,
        resvg::tiny_skia::IntSize::from_wh(width as u32, height as u32).unwrap(),
    )
    .unwrap()
    .save_png(path)
    .unwrap();
}

#[test]
fn workbench_visual_snapshots_from_actual_renderer() {
    let (_, mut model, template) = fixture();
    model.scan();
    wait(&mut model);
    model.connection.host = "demo@example.com".into();
    model.connection.root = "/home/wwwroot/web-framework/current/".into();
    model.root_repo.as_mut().unwrap().project.branch = "feat-copilot".into();
    model
        .root_repo
        .as_mut()
        .unwrap()
        .branches
        .push("feat-copilot".into());
    model.repos[1].project.branch = "ref-FeeItemConfig".into();
    model.repos[3].project.branch = "feat/optimize-waybill-dashboard".into();
    select(&mut model, &["web-bms", "web-tms"]);
    screenshot(&model, &template, 1520, 900, "turbo-native-styled.png");
    screenshot(
        &model,
        &template,
        1120,
        760,
        "turbo-native-styled-compact.png",
    );
    model.repos[0].branches = (0..20).map(|n| format!("feature/branch-{n:02}")).collect();
    screenshot(&model, &template, 1520, 900, "turbo-branches-ten-rows.png");
    screenshot(
        &model,
        &template,
        1120,
        650,
        "turbo-workspace-layout-minimum.png",
    );
    model
        .root_repo
        .as_mut()
        .unwrap()
        .branches
        .extend((0..20).map(|n| format!("feature/root-{n:02}")));
    screenshot(
        &model,
        &template,
        1120,
        760,
        "turbo-workspace-layout-root-branches.png",
    );
}
