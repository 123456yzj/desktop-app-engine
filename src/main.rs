#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
mod gui_v02;
const MAX_INPUT_CHARS: usize = desktop_app_engine::validation::MAX_STRING_BYTES / 4;

use desktop_app_engine::{engine::Engine, loader};
use eframe::egui;
use std::path::PathBuf;

struct Options {
    credential_store: Option<String>,
    credential_action: Option<CredentialAction>,
    check_credentials: bool,
    path: Option<PathBuf>,
    theme: egui::ThemePreference,
    check: bool,
    check_plugins: bool,
}
enum CredentialAction {
    Set(String),
    Delete(String),
    List,
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        credential_store: None,
        credential_action: None,
        check_credentials: false,
        path: None,
        theme: egui::ThemePreference::System,
        check: false,
        check_plugins: false,
    };
    let mut args = std::env::args_os().skip(1);

    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--credential-store") => {
                if options.credential_store.is_some() {
                    return Err("--credential-store 只能指定一次".into());
                }
                options.credential_store = Some(
                    args.next()
                        .and_then(|s| s.into_string().ok())
                        .ok_or("--credential-store 缺少名称")?,
                );
            }
            Some("--credential-set" | "--credential-delete" | "--credential-list") => {
                if options.credential_action.is_some() {
                    return Err("每次只能执行一个凭据管理命令".into());
                }
                options.credential_action = Some(if arg == "--credential-list" {
                    CredentialAction::List
                } else {
                    let name = args
                        .next()
                        .and_then(|s| s.into_string().ok())
                        .ok_or("凭据管理命令缺少名称")?;
                    desktop_app_engine::credentials::valid_name(&name)?;
                    if arg == "--credential-set" {
                        CredentialAction::Set(name)
                    } else {
                        CredentialAction::Delete(name)
                    }
                });
            }
            Some("--check-credentials") => {
                options.check = true;
                options.check_credentials = true;
            }
            Some("--app") => {
                if options.path.is_some() {
                    return Err("--app 只能指定一次".into());
                }
                options.path = Some(PathBuf::from(
                    args.next().ok_or("--app 缺少 JSON 文件路径")?,
                ));
            }
            Some("--theme") => {
                options.theme = match args.next().as_deref().and_then(std::ffi::OsStr::to_str) {
                    Some("system") => egui::ThemePreference::System,
                    Some("light") => egui::ThemePreference::Light,
                    Some("dark") => egui::ThemePreference::Dark,
                    _ => return Err("--theme 需要 system、light 或 dark".into()),
                };
            }
            Some("--check") => options.check = true,
            Some("--check-plugins") => {
                options.check = true;
                options.check_plugins = true;
            }
            _ => {
                return Err("未知参数。用法: desktop-app-engine.exe [--app FILE.json] [--theme system|light|dark] [--check|--check-plugins|--check-credentials]；凭据命令: --credential-store STORE --credential-set NAME|--credential-delete NAME|--credential-list".into());
            }
        }
    }
    Ok(options)
}

fn run() -> Result<(), String> {
    let options = options()?;
    if let Some(action) = &options.credential_action {
        if options.check {
            return Err("凭据管理命令不能与校验命令混用".into());
        }
        let namespace = if let Some(namespace) = &options.credential_store {
            namespace.clone()
        } else {
            let loader::Document::V02(d) = loader::load_document(options.path.as_deref())? else {
                return Err("凭据管理需要 --credential-store 或 0.2 app.json".into());
            };
            d.credential_store
                .ok_or("app.json 未配置 credentialStore")?
        };
        let store = desktop_app_engine::credentials::Store::new(&namespace)?;
        match action {
            CredentialAction::Set(name) => {
                use std::io::{IsTerminal, Read};
                if std::io::stdin().is_terminal() {
                    return Err(
                        "凭据仅从标准输入读取，请使用 scripts/set-credential.ps1 的隐藏输入".into(),
                    );
                }
                let mut bytes = zeroize::Zeroizing::new(Vec::new());
                std::io::stdin()
                    .take(desktop_app_engine::credentials::MAX_SECRET as u64 + 3)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "读取凭据输入失败")?;
                if bytes.len() > desktop_app_engine::credentials::MAX_SECRET + 2 {
                    return Err("凭据输入超限".into());
                }
                let text = std::str::from_utf8(&bytes).map_err(|_| "凭据输入必须是 UTF-8")?;
                let secret = if let Some(line) = text.strip_suffix('\n') {
                    line.strip_suffix('\r').unwrap_or(line)
                } else {
                    text
                };
                store.put(name, secret)?;
                println!("凭据已加密保存: {name}");
            }
            CredentialAction::Delete(name) => {
                store.delete(name)?;
                println!("凭据已删除: {name}");
            }
            CredentialAction::List => {
                for name in store.names()? {
                    println!("{name}");
                }
            }
        }
        return Ok(());
    }
    if options.credential_store.is_some() {
        return Err(
            "--credential-store 仅用于凭据管理命令；接口使用 app.json 的 credentialStore".into(),
        );
    }
    let description = loader::load_document(options.path.as_deref())?;
    let command_base = options
        .path
        .as_ref()
        .map(|path| {
            std::fs::canonicalize(path)
                .map(|p| p.parent().unwrap().to_owned())
                .map_err(|e| format!("无法定位描述目录: {e}"))
        })
        .transpose()?;
    if options.check {
        if options.check_credentials
            && let loader::Document::V02(d) = &description
        {
            let store = d
                .credential_store
                .as_deref()
                .map(desktop_app_engine::credentials::Store::new)
                .transpose()?;
            let mut checked = std::collections::BTreeSet::new();
            for source in d.data_sources.values() {
                for header in source.headers.values() {
                    if let desktop_app_engine::http::HeaderArgument::Credential(
                        desktop_app_engine::http::CredentialHeader::Credential { name, .. },
                    ) = header
                        && checked.insert(name)
                    {
                        let secret = store.as_ref().ok_or("缺少 credentialStore")?.get(name)?;
                        let _ = reqwest::header::HeaderValue::from_str(&secret)
                            .map_err(|_| "凭据无法作为 HTTP 请求头")?;
                        println!("凭据可解密: {name}");
                    }
                }
            }
        }
        if options.check_plugins
            && let loader::Document::V02(d) = &description
        {
            let runtime = desktop_app_engine::interaction::Runtime::new((**d).clone())?;
            for node in &runtime.tree.nodes {
                if let Some(r) = &node.spec.registration {
                    let mut rpc = desktop_app_engine::plugin::Rpc::start(r)?;
                    let result = rpc.request(&serde_json::json!({"op":"mount","component":r.component,"props":desktop_app_engine::v02::plugin_props(&node.spec,&runtime.state)?,"width":800,"height":400}))?;
                    if let Some(svg) = result.get("svg").and_then(serde_json::Value::as_str) {
                        desktop_app_engine::plugin::rasterize(svg)?;
                    }
                    rpc.request(&serde_json::json!({"op":"unmount","component":r.component}))?;
                    println!("插件执行与渲染校验通过: {}", node.spec.id);
                }
            }
        }
        println!("描述校验通过: {}", description.app().name);
        return Ok(());
    }
    let title = description.app().name.clone();
    let window = &description.app().window;
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window.width, window.height])
            .with_min_inner_size([160.0, 160.0]),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        native_options,
        Box::new(move |cc| {
            gui::install_chinese_font(&cc.egui_ctx)?;
            cc.egui_ctx.set_theme(options.theme);
            // One render pass per frame: event effects never run in a discarded layout pass.
            cc.egui_ctx
                .options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(1).unwrap());
            match description {
                loader::Document::V01(d) => Ok(Box::new(gui::DesktopApp::new(Engine::new(d)?))),
                loader::Document::V02(d) => {
                    let mut app = gui_v02::DashboardApp::new(*d)?;
                    if let Some(base) = command_base {
                        app.set_command_base(base);
                    }
                    Ok(Box::new(app))
                }
            }
        }),
    )
    .map_err(|e| format!("GUI 启动失败: {e}"))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        if !std::env::args_os().any(|arg| {
            arg == "--check"
                || arg == "--check-plugins"
                || arg
                    .to_str()
                    .is_some_and(|s| s.starts_with("--credential-") || s == "--check-credentials")
        }) {
            show_error(&error);
        }
        std::process::exit(2);
    }
}

fn show_error(error: &str) {
    let message: Vec<u16> = error.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "桌面描述引擎：加载失败"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // Both buffers are NUL-terminated and remain alive throughout the synchronous call.
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            windows_sys::Win32::UI::WindowsAndMessaging::MB_OK
                | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
        );
    }
}
