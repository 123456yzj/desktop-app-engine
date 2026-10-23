use desktop_app_engine::{
    loader, renderer as gui,
    renderer::DashboardApp,
    runtime::Runtime,
    services::{credentials, logging},
};
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
                    credentials::valid_name(&name)?;
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
            let d = loader::load_document(options.path.as_deref())?;
            d.credential_namespace()
                .ok_or("app.json 未配置 credentialStore")?
                .to_owned()
        };
        let store = credentials::Store::new(&namespace)?;
        match action {
            CredentialAction::Set(name) => {
                use std::io::{IsTerminal, Read};
                if std::io::stdin().is_terminal() {
                    return Err("凭据仅从重定向的标准输入读取，请由隐藏输入界面传入".into());
                }
                let mut bytes = zeroize::Zeroizing::new(Vec::new());
                std::io::stdin()
                    .take(credentials::MAX_SECRET as u64 + 3)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "读取凭据输入失败")?;
                if bytes.len() > credentials::MAX_SECRET + 2 {
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
    logging::event(
        "INFO",
        "app.loaded",
        serde_json::json!({"name":description.app().name}),
    );
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
        if options.check_credentials || options.check_plugins {
            let runtime = Runtime::from_compiled(description.clone())?;
            if options.check_credentials {
                for name in runtime.check_credentials()? {
                    println!("凭据可解密: {name}");
                }
            }
            if options.check_plugins {
                for id in runtime.check_plugins()? {
                    println!("插件执行与渲染校验通过: {id}");
                }
            }
        }
        println!("描述校验通过: {}", description.app().name);
        return Ok(());
    }
    if let Some(host) = &description.app().host
        && host != "desktop-app-engine"
    {
        return Err(format!(
            "此应用需要原生宿主 {host}，请通过 package 命令导出后运行，或使用对应宿主调试"
        ));
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
            let mut app = DashboardApp::from_compiled(description)?;
            if let Some(base) = command_base {
                app.set_command_base(base);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| format!("GUI 启动失败: {e}"))
}

fn main() {
    if matches!(desktop_app_engine::application::is_packaged(), Ok(false)) {
        let first = std::env::args_os().nth(1);
        if first.is_none()
            || first
                .as_deref()
                .is_some_and(|arg| arg == "package" || arg == "--help" || arg == "-h")
        {
            if let Err(error) = package_command() {
                eprintln!("打包失败：{error}");
                std::process::exit(2);
            }
            return;
        }
    }
    let _ = logging::global();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        logging::event(
            "ERROR",
            "app.panic",
            serde_json::json!({"message":info.to_string(),"backtrace":std::backtrace::Backtrace::force_capture().to_string()}),
        );
        previous(info);
    }));
    if let Err(error) = run() {
        logging::event("ERROR", "app.failed", serde_json::json!({"error":error}));
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
    logging::event("INFO", "app.exit", serde_json::json!({"exitCode":0}));
}

fn package_command() -> Result<(), String> {
    use std::io::{IsTerminal, Read};
    const HELP: &str = "用法：desktop-app-engine.exe package --output <应用.exe>\n\n从标准输入读取 UTF-8 JSON 内容，输出独立 EXE。已有文件拒绝覆盖。\n成功退出 0，标准输出为 EXE 的绝对路径；失败退出 2，原因写入标准错误。\n\n示例（cmd.exe）：\n  desktop-app-engine.exe package --output MyApp.exe < app.json\n";
    let mut args = std::env::args_os().skip(1);
    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        None | Some("--help" | "-h") => {
            print!("{HELP}");
            return Ok(());
        }
        Some("package") => {}
        _ => return Err(HELP.into()),
    }
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => {
                print!("{HELP}");
                return Ok(());
            }
            Some("--output") if output.is_none() => {
                output = Some(PathBuf::from(args.next().ok_or("--output 缺少 EXE 路径")?));
            }
            _ => return Err(format!("未知或重复参数：{}\n{HELP}", arg.to_string_lossy())),
        }
    }
    let output = output.ok_or("必须通过 --output 指定输出 EXE")?;
    if std::io::stdin().is_terminal() {
        return Err("请通过标准输入传入 UTF-8 JSON 内容".into());
    }
    let mut json = String::new();
    std::io::stdin()
        .read_to_string(&mut json)
        .map_err(|e| format!("无法读取 UTF-8 JSON 输入：{e}"))?;
    let output = desktop_app_engine::packaging::package(&json, &output)?;
    println!("{}", output.display());
    Ok(())
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
