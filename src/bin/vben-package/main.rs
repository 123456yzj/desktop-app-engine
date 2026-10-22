#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod turbo_build;

use desktop_app_engine::{
    protocol::v02,
    renderer::{self as gui, DashboardApp},
    services::logging,
};
use eframe::egui;
use serde_json::Value;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use turbo_build::{
    Controller,
    backend::{Backend, Connection},
    view,
};

struct App {
    model: Controller,
    template: Value,
    main: DashboardApp,
    modal: Option<DashboardApp>,
    last_refresh: Instant,
}
impl App {
    fn draw_main(&mut self, ctx: &egui::Context) {
        if self.model.auto_scroll {
            self.main.scroll_to_end("logScroll");
        }
        self.main.draw(ctx);
        if self.model.auto_scroll && self.main.user_scrolled("logScroll") {
            self.model.auto_scroll = false;
            self.model.dirty = true;
        }
    }
    fn new(model: Controller, template: Value) -> Result<Self, String> {
        let main = DashboardApp::embedded(view::document(&template, &model)?)?;
        Ok(Self {
            model,
            template,
            main,
            modal: None,
            last_refresh: Instant::now(),
        })
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.model.poll();
        if self.model.dirty
            || !self.model.idle() && self.last_refresh.elapsed() > Duration::from_millis(150)
        {
            let result = view::document(&self.template, &self.model)
                .and_then(|d| self.main.replace_description(d));
            if let Err(error) = result {
                self.model.error = error;
            }
            if self.model.settings_open || self.model.pending.is_some() {
                match view::modal(&self.template, &self.model) {
                    Ok(d) => {
                        if let Some(modal) = &mut self.modal {
                            if let Err(e) = modal.replace_description(d) {
                                self.model.error = e;
                            }
                        } else {
                            match DashboardApp::embedded(d) {
                                Ok(app) => self.modal = Some(app),
                                Err(e) => self.model.error = e,
                            }
                        }
                    }
                    Err(e) => self.model.error = e,
                }
            } else {
                self.modal = None;
            }
            self.model.dirty = false;
            self.last_refresh = Instant::now();
        }
        self.draw_main(ctx);
        let mut events = self.main.take_events();
        if let Some(modal) = &mut self.modal {
            let title = if self.model.settings_open {
                "连接设置"
            } else {
                "确认构建计划"
            };
            let height = if self.model.settings_open {
                240.0
            } else {
                (140.0 + self.model.pending.as_ref().map_or(0, |p| p.projects.len()) as f32 * 58.0)
                    .min(560.0)
            };
            egui::Window::new(title)
                .id(egui::Id::new("build_dialog"))
                .collapsible(false)
                .resizable(false)
                .movable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .fixed_size([620.0, height])
                .show(ctx, |ui| {
                    ui.set_min_size(egui::vec2(620.0, height));
                    modal.draw_in(ctx, ui);
                });
            events.extend(modal.take_events());
        }
        for event in events {
            if event.component_id == "copyLogs" {
                ctx.copy_text(self.model.log_text());
            } else {
                self.model.handle(event);
            }
        }
        if self.model.dirty {
            ctx.request_repaint();
        }
        if !self.model.idle() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}
fn parse_template(text: &str, base: &std::path::Path) -> Result<(v02::Description, Value), String> {
    let description = v02::parse(text)?;
    desktop_app_engine::compiler::compile_at(description.clone(), base)?;
    let template = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok((description, template))
}

fn run() -> Result<(), String> {
    let mut path = None;
    let mut check = false;
    let mut check_plugins = false;
    let mut inspect = false;
    let mut theme = egui::ThemePreference::Light;
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--app") if path.is_none() => {
                path = Some(PathBuf::from(args.next().ok_or("--app 缺少路径")?));
            }
            Some("--check") => check = true,
            Some("--check-plugins") => {
                check = true;
                check_plugins = true;
            }
            Some("--inspect") => inspect = true,
            Some("--theme") => {
                theme = match args.next().as_deref().and_then(std::ffi::OsStr::to_str) {
                    Some("light") => egui::ThemePreference::Light,
                    Some("dark") => egui::ThemePreference::Dark,
                    Some("system") => egui::ThemePreference::System,
                    _ => return Err("--theme 需要 light、dark 或 system".into()),
                };
            }
            _ => return Err("用法：vben-package.exe [--app FILE] [--check|--check-plugins|--inspect] [--theme light|dark|system]".into()),
        }
    }
    if check && inspect {
        return Err("--check 与 --inspect 不能同时使用".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let base = exe.parent().ok_or("无法定位程序目录")?.to_owned();
    let compiled = desktop_app_engine::application::packaged_definition(path.as_deref())?;
    let packaged = compiled.is_some();
    let path = path.or_else(|| {
        if compiled.is_some() {
            return None;
        }
        let p = base.join("app.json");
        p.exists().then_some(p)
    });
    let bytes = if let Some(value) = compiled {
        serde_json::to_vec(&value).map_err(|e| e.to_string())?
    } else if let Some(path) = &path {
        std::fs::read(path).map_err(|e| format!("读取 app.json 失败: {e}"))?
    } else {
        include_bytes!("../../../examples/vben-package.json").to_vec()
    };
    let text = std::str::from_utf8(&bytes)
        .map_err(|e| e.to_string())?
        .trim_start_matches('\u{feff}');
    let resource_base = if packaged {
        desktop_app_engine::application::resource_base(&base)?
    } else {
        path.as_deref()
            .and_then(std::path::Path::parent)
            .unwrap_or(&base)
            .to_owned()
    };
    let (description, template) = parse_template(text, &resource_base)?;
    let mut connection = Connection {
        host: description
            .state
            .get("sshTarget")
            .and_then(Value::as_str)
            .ok_or("缺少 state.sshTarget")?
            .into(),
        root: description
            .state
            .get("repository")
            .and_then(Value::as_str)
            .ok_or("缺少 state.repository")?
            .into(),
    };
    let settings_path = turbo_build::settings::user_path()?;
    if !check && let Some(saved) = turbo_build::settings::load(&settings_path)? {
        connection = saved;
    }
    let session = logging::global()
        .ok_or_else(|| logging::init_error().unwrap_or_else(|| "日志初始化失败".into()))?;
    let backend = Backend {
        base,
        session,
        program: "ssh.exe".into(),
        prefix_args: vec![],
    };
    let mut model = Controller::new(connection, backend, Some(settings_path));
    let compiled = desktop_app_engine::compiler::compile_at(
        view::document(&template, &model)?,
        &resource_base,
    )?;
    if check {
        if check_plugins {
            let runtime = desktop_app_engine::runtime::Runtime::from_compiled(compiled)?;
            for id in runtime.check_plugins()? {
                println!("插件执行与渲染校验通过: {id}");
            }
        }
        println!("多仓库打包应用校验通过");
        return Ok(());
    }
    if inspect {
        model.connection.validate()?;
        model.scan_read_only();
        let deadline = Instant::now() + Duration::from_secs(125);
        while !model.idle() {
            if Instant::now() > deadline {
                return Err("扫描超时".into());
            }
            std::thread::sleep(Duration::from_millis(30));
            model.poll();
        }
        if !model.error.is_empty() {
            return Err(model.error.clone());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(
                &model
                    .root_repo
                    .iter()
                    .chain(model.repos.iter())
                    .map(|r| &r.project)
                    .collect::<Vec<_>>()
            )
            .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if !model.settings_open {
        model.scan();
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([description.app.window.width, description.app.window.height])
            .with_min_inner_size([1120.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "项目构建工作台",
        options,
        Box::new(move |cc| {
            gui::install_chinese_font(&cc.egui_ctx)?;
            cc.egui_ctx.set_theme(theme);
            cc.egui_ctx
                .options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(1).unwrap());
            Ok(Box::new(App::new(model, template)?))
        }),
    )
    .map_err(|e| e.to_string())
}
fn main() {
    let _ = logging::global();
    if let Err(error) = run() {
        logging::event("ERROR", "app.failed", serde_json::json!({"error":error}));
        eprintln!("{error}");
        std::process::exit(2);
    }
    logging::event("INFO", "app.exit", serde_json::json!({"exitCode":0}));
}
