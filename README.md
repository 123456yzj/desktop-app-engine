# 桌面应用描述引擎

对外提供一个原生打包入口：输入 JSON 内容，输出应用专属的 Windows EXE。支持协议 0.1 和 0.2；0.2 提供组件树、布局、类型化状态、模块作用域、表格、图表、命令、HTTP 数据源和 JavaScript 插件。

```cmd
desktop-app-engine.exe package --output MyApp.exe < app.json
```

命令通过标准输入接收 UTF-8 JSON；上例的文件重定向语法用于 `cmd.exe`。调用程序也可直接将 JSON 字节写入子进程标准输入。打包时无需 Rust、Cargo、PowerShell 或项目源码，只需制作端引擎及 JSON 声明的已安装原生宿主、插件资源。

## 开发运行

开发目标为 Windows，需要 Rust/Cargo、MSVC C++ 构建工具和 Windows SDK。首次构建先缓存锁定依赖，之后可离线构建：

```powershell
cargo fetch --locked
cargo check --all-targets --locked --offline
cargo build --release --bin desktop-app-engine --locked --offline
```

制作端引擎无参数运行时显示打包用法。内部运行时调试通过 `--app` 显式指定输入；开发加载器也支持 EXE 同目录的 `app.json`。外部输入错误时直接报告原因。以下 `C:\Apps\monitor.json` 表示自行制作的应用描述路径：

```powershell
cargo run --bin desktop-app-engine --locked --offline -- --app C:\Apps\monitor.json --check
cargo run --bin desktop-app-engine --locked --offline -- --app C:\Apps\monitor.json --theme dark
```

`--check` 执行完整结构与语义校验，成功退出 0，错误退出 2；不发起 HTTP 请求或执行业务命令。`--check-plugins` 额外执行插件 mount、SVG 栅格化和 unmount。`--check-credentials` 检查引用凭据是否可解密，不发送网络请求。

仓库保留的业务模板为 [examples/vben-package.json](examples/vben-package.json)，配套业务运行时是多仓库构建工作台：

```powershell
cargo run --bin vben-package --locked --offline -- --check
cargo run --bin vben-package --locked --offline
```

工作台启动后的扫描和构建会使用用户配置的 SSH 连接，操作规则见 [构建工作台](docs/20-turbo-build.md)。

工作台 JSON 通过 `app.host: "vben-package"` 声明所需原生宿主。先构建一次宿主模板，随后仍由统一命令导出完整应用：

```powershell
cargo build --release --bin vben-package --locked --offline
```

```cmd
target\release\desktop-app-engine.exe package --output exports\vben-package.exe < examples\vben-package.json
```

## 分层与入口

工程保留一个 Cargo crate，对外命令是 `desktop-app-engine.exe package`。仓库中的 `vben-package` 是使用库接口的内部业务开发宿主：

| 目录 | 职责 |
| --- | --- |
| `src/protocol/` | 原始描述、绑定和严格结构解析；公共 app/window 类型 |
| `src/compiler/` | 插件发现、作用域解析、语义校验和私有编译元数据 |
| `src/runtime/` | 受校验的状态提交、有序事件、动作及异步任务调度 |
| `src/components/`、`src/layout/` | 内部组件树、表格/图表纯算法、布局 |
| `src/services/` | 子进程、HTTP、插件 RPC、凭据和日志 |
| `src/packaging.rs`、`src/application.rs` | 单 EXE 导出、内置定义读取和插件资源缓存 |
| `src/renderer/` | egui 宿主、主题、文本测量、绘制和日志界面 |
| `src/bin/vben-package/` | 业务控制器、远端操作、设置和界面生成 |

0.2 数据流为 `protocol::v02::parse` → `compiler::compile_at` → `CompiledDescription` → `Runtime::from_compiled`。`loader` 负责版本分派和输入目录。原始描述可重复编译；内部名称、插件注册和事件样例不属于 JSON 协议。运行时通过只读访问器提供状态，写入走 `set`、`set_many` 或 `set_from`，统一维护校验与版本。详情见 [架构](docs/02-architecture.md)。

## 应用打包

统一接口接收 JSON 内容和输出 EXE 路径，宿主从 JSON 的 `app.host` 自动解析；省略时使用通用引擎。原生宿主模板 `<host>.exe` 安装在制作端引擎旁，缺失时报错。先构建一次制作端引擎，随后直接调用原生命令：

```cmd
target\release\desktop-app-engine.exe package --output exports\Monitor.exe < C:\Apps\monitor.json
```

引擎先完成结构和语义校验，将定义及所需插件资源嵌入 EXE，再执行包内校验；通过后写入指定输出路径。最终只交付一个 EXE，已有输出路径拒绝覆盖。成功退出 0，标准输出为产物绝对路径；失败退出 2，原因写入标准错误。

打包 EXE 固定使用内置定义，拒绝 `--app` 覆盖。修改 JSON 后重新打包。客户机需要 Windows 系统中文字体、OpenGL 驱动及应用声明的外部程序；插件及其 Node 运行库自动释放到用户资源缓存。凭据和用户设置保存在当前用户目录。详见 [分发决策](docs/06-packaging-decision.md)。

## 验证与协议

```powershell
cargo fmt -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo run --bin desktop-app-engine --locked --offline -- --app .\examples\vben-package.json --check
cargo run --bin vben-package --locked --offline -- --check
```

当前仓库不保留自动化测试代码，使用构建检查和描述校验命令验证。界面交互、远端 SSH 操作与客户环境需单独验证。

- [描述格式](docs/05-description-format.md) 与 [0.2 JSON Schema](docs/description-0.2.schema.json)
- [命令](docs/10-command-binding.md)、[动态事件](docs/11-dynamic-event-binding.md)、[作用域](docs/12-scoped-state.md)
- [模块](docs/13-module-component.md) 与 [模块通信](docs/14-module-communication.md)
- [插件](docs/15-plugins.md)、[HTTP](docs/16-http-data-sources.md)、[凭据](docs/17-encrypted-credentials.md)
- [下拉选择](docs/18-select.md)、[运行日志](docs/19-runtime-logs.md)、[构建工作台](docs/20-turbo-build.md)
