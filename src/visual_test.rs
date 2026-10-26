//! Feature-gated packaged-EXE pixel rendering. Never creates a desktop window.
use desktop_app_engine::{application, loader, renderer};
use eframe::egui;
use egui_kittest::Harness;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scenario {
    output: PathBuf,
    dark: bool,
    pixels_per_point: f32,
    steps: Vec<Step>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    name: String,
    #[serde(default = "capture_default")]
    capture: bool,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    target_position: Option<[f32; 2]>,
    #[serde(default)]
    pointer: Option<[f32; 2]>,
    #[serde(default)]
    pressed: Option<bool>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    select_all: bool,
    #[serde(default)]
    keys: Vec<TestKey>,
    #[serde(default)]
    wheel: Option<[f32; 2]>,
    #[serde(default)]
    alt: bool,
    #[serde(default)]
    wait_ms: u64,
}

fn capture_default() -> bool {
    true
}

#[derive(Deserialize)]
enum TestKey {
    ArrowDown,
    ArrowUp,
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
}

impl TestKey {
    fn egui(&self) -> egui::Key {
        match self {
            Self::ArrowDown => egui::Key::ArrowDown,
            Self::ArrowUp => egui::Key::ArrowUp,
            Self::Enter => egui::Key::Enter,
            Self::Escape => egui::Key::Escape,
            Self::Tab => egui::Key::Tab,
            Self::Space => egui::Key::Space,
            Self::Backspace => egui::Key::Backspace,
        }
    }
}

pub fn run(path: &Path) -> Result<(), String> {
    if !application::is_packaged()? {
        return Err("图片验收必须运行实际打包 EXE，读取内置定义".into());
    }
    let scenario: Scenario =
        serde_json::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("图片场景无效: {e}"))?;
    if !scenario.pixels_per_point.is_finite()
        || !(0.5..=4.0).contains(&scenario.pixels_per_point)
        || scenario.steps.is_empty()
        || scenario.steps.len() > 100
    {
        return Err("场景需要 1..100 个步骤，缩放必须在 0.5..4 之间".into());
    }
    let mut names = std::collections::HashSet::new();
    for step in &scenario.steps {
        if step.name.is_empty()
            || !step
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            || !names.insert(&step.name)
            || step
                .pointer
                .is_some_and(|p| p.iter().any(|n| !n.is_finite()))
            || step.wheel.is_some_and(|p| p.iter().any(|n| !n.is_finite()))
            || step.keys.len() > 20
            || step.wait_ms > 5000
        {
            return Err(
                "步骤名称必须唯一且仅含 ASCII 字母、数字、下划线或连字符；坐标必须有限".into(),
            );
        }
    }
    // None deliberately selects the definition stored in this running EXE.
    let description = loader::load_document(None)?;
    let size = [
        description.app().window.width,
        description.app().window.height,
    ];
    let app_name = description.app().name.clone();
    let mut app = renderer::DashboardApp::from_compiled(description)?;
    let mut installed = false;
    let targets = std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new()));
    let drawn_targets = targets.clone();
    let mut setup_error = None;
    let render_state =
        egui_kittest::wgpu::create_render_state(egui_kittest::wgpu::default_wgpu_setup());
    let adapter = render_state.adapter.get_info();
    let mut harness = Harness::builder()
        .with_size(size)
        .with_pixels_per_point(scenario.pixels_per_point)
        // The kittest default is 250 ms/frame: settling a press for four frames
        // would exceed egui's click timeout and turn it into a long press.
        .with_step_dt(1.0 / 60.0)
        .renderer(egui_kittest::wgpu::WgpuTestRenderer::from_render_state(
            render_state,
        ))
        .build(|ctx| {
            if !installed {
                setup_error = renderer::install_chinese_font(ctx).err();
                ctx.set_theme(if scenario.dark {
                    egui::ThemePreference::Dark
                } else {
                    egui::ThemePreference::Light
                });
                ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(1).unwrap());
                installed = true;
            }
            app.draw(ctx);
            *drawn_targets.borrow_mut() = app.visual_targets();
        });
    std::fs::create_dir_all(&scenario.output).map_err(|e| e.to_string())?;
    let mut position = egui::pos2(-100., -100.);
    let mut captures = Vec::new();
    for step in &scenario.steps {
        let modifiers = egui::Modifiers {
            alt: step.alt,
            ..Default::default()
        };
        harness.input_mut().modifiers = modifiers;
        let pointer = if let Some(target) = &step.target {
            let bounds = targets
                .borrow()
                .get(target)
                .copied()
                .ok_or_else(|| format!("步骤 {} 的目标 {target} 不在可见区域", step.name))?;
            let [x, y] = step.target_position.unwrap_or([0.5, 0.5]);
            if !x.is_finite()
                || !y.is_finite()
                || !(0.0..=1.0).contains(&x)
                || !(0.0..=1.0).contains(&y)
            {
                return Err("目标位置必须在 0..1 之间".into());
            }
            Some([
                bounds.left() + bounds.width() * x,
                bounds.top() + bounds.height() * y,
            ])
        } else {
            step.pointer
        };
        if let Some([x, y]) = pointer {
            position = egui::pos2(x, y);
            harness
                .input_mut()
                .events
                .push(egui::Event::PointerMoved(position));
        }
        if let Some(pressed) = step.pressed {
            harness.input_mut().events.push(egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers,
            });
        }
        if let Some([x, y]) = step.wheel {
            harness.input_mut().events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(x, y),
                modifiers,
            });
        }
        harness.step();
        for key in &step.keys {
            harness.press_key(key.egui());
        }
        if step.select_all {
            harness.press_key_modifiers(
                egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
                egui::Key::A,
            );
        }
        if let Some(text) = &step.text {
            harness
                .input_mut()
                .events
                .push(egui::Event::Text(text.clone()));
            harness.step();
        }
        // Settle layout and style animations, including the frame after event effects.
        harness.run_steps(4);
        // Allow real asynchronous plugin workers to finish while continuing to poll
        // and draw the packaged application. No desktop window or system input.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(step.wait_ms);
        while std::time::Instant::now() < deadline {
            harness.run_steps(1);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if !step.capture {
            continue;
        }
        let image = harness.render()?;
        let file = scenario.output.join(format!("{}.png", step.name));
        image
            .save(&file)
            .map_err(|e| format!("写入 {}: {e}", file.display()))?;
        captures.push(
            serde_json::json!({"name":step.name,"width":image.width(),"height":image.height(),"wait_ms":step.wait_ms,
                "targets": targets.borrow().iter().map(|(id, rect)| (id.clone(), [rect.left(),rect.top(),rect.right(),rect.bottom()])).collect::<std::collections::BTreeMap<_,_>>()}),
        );
    }
    drop(harness);
    if let Some(error) = setup_error {
        return Err(error);
    }
    let report = serde_json::json!({
        "executable": std::env::current_exe().map_err(|e| e.to_string())?,
        "definition": "embedded", "app": app_name,
        "build": if cfg!(debug_assertions) { "Debug" } else { "Release" },
        "renderer": "egui_kittest 0.31.1 / wgpu (offscreen; not Glow)",
        "adapter": {"name":adapter.name, "backend":format!("{:?}", adapter.backend),
                    "device_type":format!("{:?}", adapter.device_type),
                    "driver":adapter.driver,"driver_info":adapter.driver_info},
        "theme": if scenario.dark { "dark" } else { "light" },
        "pixels_per_point": scenario.pixels_per_point, "logical_size":size,
        "fonts": "egui defaults plus install_chinese_font system fonts",
        "captures":captures
    });
    std::fs::write(
        scenario.output.join("render-report.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("PNG 已生成（仍需图片验收）: {}", scenario.output.display());
    Ok(())
}
