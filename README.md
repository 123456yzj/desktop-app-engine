# 桌面应用描述引擎

项目仓库：[123456yzj/desktop-app-engine](https://github.com/123456yzj/desktop-app-engine)。固定运行时读取 JSON 描述，生成带接口数据、图表、交互和加密 API 凭据的 Windows 桌面应用。

仓库包含源码、协议文档、示例、插件适配器、测试和文字验证记录。构建产物、导出应用、Node/npm 依赖、截图、原始日志及凭据密文由本地生成，不纳入版本控制。

## 从仓库开始

开发与运行目标为 Windows。准备 Rust/Cargo、Windows C++ 构建工具，以及 PowerShell 7；插件示例和完整测试还需要 Node/npm。首次克隆后运行：

```powershell
git clone https://github.com/123456yzj/desktop-app-engine.git
Set-Location desktop-app-engine
$env:CARGO_TARGET_DIR = "$PWD\target-http"
cargo fetch --locked
.\scripts\install-plugin.ps1 -Package .\plugins\echarts -AppDirectory .\examples -InstallDependencies
cargo test --locked --offline
cargo build --release --locked --offline
.\target-http\release\desktop-app-engine.exe --app .\examples\plugin-dashboard.json --theme dark
```

Python 仅用于演示 API 与可选的 Schema 检查；运行引擎本身不需要 Python。以下实施记录中的绝对目录是本地验证环境，克隆后使用自己的项目路径。

独立项目位置：`D:\project\impact\desktop-app-engine`。本项目负责 JSON 协议、校验、执行、桌面渲染与应用包导出；可视化编辑器应作为另一个项目，通过 JSON 契约对接。原 `app-v1/desktop-app-engine` 保留为迁移前历史副本。

目标：低代码设计界面，生成应用描述，由引擎运行并交付独立 Windows 应用。

第一版使用 Rust + egui/eframe：JSON → 类型化描述与校验 → 状态和动作 → 原生 GUI。

现支持独立协议 0.1 与 0.2。0.2 增加自研 arena 组件树、纯布局、类型化状态和绑定、容器与指标、固定行高虚拟表格、线性折线/分组柱图及可安装 JavaScript 插件。egui/eframe 0.31.1 承担窗口、输入、字体与基础图元绘制；插件 SVG 使用 resvg 0.48.1 栅格化。完整能力及限制见 [协议](docs/05-description-format.md)、[组件实测](docs/09-component-implementation.md) 和 [插件机制](docs/15-plugins.md)。

- [产品需求](docs/01-requirements.md)
- [架构方向](docs/02-architecture.md)
- [分步骤实施方案](docs/03-implementation-plan.md)
- [问题与问答](docs/04-discussion.md)
- [描述格式](docs/05-description-format.md)
- [分发决策](docs/06-packaging-decision.md)
- [独立项目边界与迁移](docs/07-project-independence.md)
- [自研底层组件设计与开发路线](docs/08-component-engine-design.md)

本目录是独立工作位置，代码、配置、测试和构建产物均放在这里。

## 运行

正式交付为应用目录：运行时 EXE、同目录 `app.json` 和 `README.txt`。客户机无需 Rust、编辑器或源码；使用 Windows 中文系统字体及 OpenGL 图形驱动。
描述加载优先级：`--app FILE` > EXE 同目录 `app.json` > 内嵌中文问候示例（开发兜底）。双击应用 EXE 即可读取同目录描述；修改描述无需重新编译。

```powershell
.\target-verify\release\desktop-app-engine.exe
.\target-verify\release\desktop-app-engine.exe --app .\examples\hello.json
.\target-verify\release\desktop-app-engine.exe --theme dark
.\target-verify\release\desktop-app-engine.exe --app .\examples\hello.json --check
.\target-verify\release\desktop-app-engine.exe --app .\examples\dashboard.json --check
.\target-verify\release\desktop-app-engine.exe --app .\examples\dashboard.json --theme dark
.\target-verify\release\desktop-app-engine.exe --app .\examples\command.json
.\target-verify\release\desktop-app-engine.exe --app .\examples\step-button.json
.\target-verify\release\desktop-app-engine.exe --app .\examples\scoped-state.json
.\target-verify\release\desktop-app-engine.exe --app .\examples\modules.json
.\target-verify\release\desktop-app-engine.exe --app .\examples\module-linkage.json
```

`--check` 仅校验，成功退出 0，错误退出 2；错误可重定向 stderr 获取。
0.2 按钮支持 `command` 动作：后台启动程序/脚本，参数可绑定输入状态，结果状态包含执行状态、stdout/stderr、退出码和错误。命令示例使用 Windows PowerShell，输入文字后点击按钮即可显示输出。协议与超时、输出限制见 [命令绑定](docs/05-description-format.md#按钮命令绑定02)；本次实现和验证见 [命令实施记录](docs/10-command-binding.md)。
0.2 事件动作也可绑定状态：`events.click` 支持固定动作名或 state/literal 字符串绑定。步骤按钮示例中，同一个按钮随状态切换“下一步 → 提交 → 已提交”，点击动作同步改变，完成后禁用，支持重置。配置与语义见 [动态事件绑定](docs/11-dynamic-event-binding.md)。
0.2 容器可声明局部 state/actions，形成页面和模块作用域。同名状态与动作在兄弟模块中互相隔离，后代就近解析；输入、按钮、表格、图表和命令结果采用相同规则。完整示例 [scoped-state.json](examples/scoped-state.json)，规则见 [作用域说明](docs/12-scoped-state.md)。
模块化优先使用专门的 `module` 组件，在内部声明 state/actions，并通过 children 组织布局和控件。module 默认纵向布局、不添加卡片外观，支持嵌套、同名局部状态隔离和响应式按钮；例子 [modules.json](examples/modules.json)，说明见 [模块组件](docs/13-module-component.md)。
模块间支持显式 `props` 和 `emit`：父级传入只读响应式 props，子级发出已声明事件，父级通过 on 动作读取 event 载荷并更新状态，兄弟模块自动刷新。示例 [module-linkage.json](examples/module-linkage.json)，协议与验收见 [模块通信](docs/14-module-communication.md)。
插件包通过 manifest 注册组件、props、事件和方法，安装及修改应用 JSON 后即可使用，无需重新编译引擎。ECharts 示例支持 SVG SSR 绘制、响应式 option、柱形点击与 dispatchAction。先安装示例资源，再运行或校验：

```powershell
.\scripts\install-plugin.ps1 -Package .\plugins\echarts -AppDirectory .\examples -InstallDependencies
.\target-verify\release\desktop-app-engine.exe --app .\examples\plugin-chart.json --check-plugins
.\target-verify\release\desktop-app-engine.exe --app .\examples\plugin-chart.json
```

`--check-plugins` 在后台实际启动插件、mount、栅格化 SVG 并 unmount，不打开窗口。完整契约见 [插件机制](docs/15-plugins.md)。安装脚本拒绝覆盖已有插件目录。开发安装需要 Node/npm；交付包携带 Node，客户机无需安装。
HTTP 接口可在顶层 dataSources 配置，支持 GET/POST/PUT/PATCH/DELETE、绑定参数、JSON 响应映射、页面首次加载和定时刷新。响应写入 state 后驱动指标、表格和插件重绘。演示运行：

```powershell
# 终端一：启动本地演示 API。
python .\scripts\http-demo-server.py
# 终端二：加载每 2 秒自动请求的页面（需先安装 ECharts 示例插件）。
.\target-http\release\desktop-app-engine.exe --app .\examples\http-chart.json
```

完整配置及请求生命周期见 [HTTP 数据源](docs/16-http-data-sources.md)。校验模式不发送 HTTP 请求。
API 认证凭据支持 Windows DPAPI 加密保存，配置只引用名称。Authorization、X-API-Key 等常见认证头必须引用凭据；录入使用隐藏输入，密文保存在当前用户本地目录，应用包不携带制作机 Token。用法见 [加密凭据](docs/17-encrypted-credentials.md)：

```powershell
.\scripts\set-credential.ps1 -Store http-auth-demo -Name api-token
.\target-http\release\desktop-app-engine.exe --app .\examples\http-auth.json --check-credentials
```
普通 GUI 模式加载失败会弹出明确错误并退出 2，外部描述错误不会回退到示例。
路径包含空格时加引号。

## 分发

先构建一次 release 运行时，再导出应用包：

```powershell
$env:CARGO_TARGET_DIR = "$PWD\target-verify"
cargo build --release --locked --offline
.\scripts\package.ps1 -Description .\examples\hello.json -Output .\exports -Name HelloApp
.\exports\HelloApp\HelloApp.exe --check
.\scripts\package.ps1 -Description .\examples\dashboard.json -Output .\exports -Name DashboardApp
```

脚本产出 `exports/HelloApp/HelloApp.exe`、`app.json`、`README.txt`，自动执行 `--check`，成功退出 0，失败退出 2。
省略 `-Name` 时采用描述中的 `app.name`；名称必须是有效 Windows 文件名。已有应用目录拒绝覆盖，请换输出位置。
运行时从 `$env:CARGO_TARGET_DIR/release` 读取，未设置时从项目 `target/release` 读取；脚本不调用 Cargo，不要求导出端安装 Rust，只需要已有 release 运行时和 PowerShell。
分发整个应用目录，不单独发送 EXE。详见 [分发决策](docs/06-packaging-decision.md)。

原项目 **`app-v1/desktop-app-engine/dist/desktop-app-engine.exe` 已过期**，未迁入独立项目。它构建于 2026-10-10，`main.rs` 修改于 10-12；历史实测二者 `--check` 输出不同。请在本项目构建新的 release 运行时，勿复制旧 dist。

## 构建与验证

使用标准用户级 Rust 工具链。Windows 环境准备：

1. 从 [Rust 官网](https://rust-lang.org/tools/install/) 下载并运行 `rustup-init.exe`，安装默认 stable MSVC 工具链。
2. 安装 Visual Studio C++ Build Tools 的 x64/x86 MSVC 工具和 Windows SDK，具体步骤见 [官方 Windows 安装说明](https://rust-lang.github.io/rustup/installation/windows-msvc.html)。
3. 重新打开终端；如果使用编辑器集成终端，重启编辑器。确认 `%USERPROFILE%\.cargo\bin` 已在用户 PATH 中。

在项目根目录运行：

```powershell
rustup show
cargo --version
rustc --version
cargo run --locked --offline
cargo run --locked --offline -- --app .\examples\hello.json
cargo run --locked --offline -- --app .\examples\hello.json --check
```

`cargo run` 会编译并启动开发版本，适合查看 `println!` 输出。`--` 后面的参数传给应用。
`--offline` 要求本机已缓存锁定依赖；首次使用先执行 `cargo fetch --locked`。

构建与验证：

插件回归需要先执行上述安装命令准备 `examples/plugins`（生成资源不纳入源码）。首次构建需下载 Cargo.lock 中的完整依赖，之后可离线执行。

```powershell
cargo fmt -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline
cargo build --release --locked --offline
```

常规开发使用 `rustup` 管理的用户级工具链，Cargo 命令位于 `%USERPROFILE%\.cargo\bin`，工具链位于 `%USERPROFILE%\.rustup`。`scripts/cargo.ps1` 仅保留为标准 Cargo 的兼容入口。本项目不包含工具链安装包和缓存，不依赖父项目源码。
`Cargo.lock` 锁定完整依赖；源码入口 `src/main.rs`，协议 `description.rs`，校验 `validation.rs`，加载 `loader.rs`，执行 `engine.rs`，渲染 `gui.rs`。
示例在 `examples/hello.json`，通过 `include_str!` 嵌入；修改嵌入示例后需要重新编译。正式应用修改包内 app.json 即可，无需重新编译。

Dashboard 示例是外部 JSON，包含四个指标、趋势图、分组柱图和明细表。“更新指标”执行确定性本地动作；表头点击稳定排序，拖动列边界调整宽度，点击/方向键选择行；图例切换系列，趋势图支持 hover、滚轮缩放、拖拽平移，双击/Home 恢复视口。HTTP 自动刷新见 http-chart.json，ECharts 复合看板见 plugin-dashboard.json；尚无可变行高、合并单元格或可视化编辑器。

各阶段应用包保存在 exports；插件示例导出使用 `.\scripts\plugin-verify.ps1`，自动安装依赖、捆绑 Node、导出并运行后台插件校验。已有目录不覆盖，重新导出请换 `-Name`。
默认构建产物为 `target/release/desktop-app-engine.exe`，设置 CARGO_TARGET_DIR 后位于对应目录；dist 中历史副本已过期。

历史受限环境使用 GNU 工具链。采用这种方式时，先将 `$toolchainRoot` 设置为实际工具链存放目录，再执行；它不要求位于项目内部：

```powershell
$toolchainRoot = 'D:\project\impact\app-v1\desktop-app-engine\.tools'
$env:RUSTUP_HOME = "$toolchainRoot\rustup"
$env:CARGO_HOME = "$toolchainRoot\cargo"
$env:PATH = "$toolchainRoot\cargo\bin;$toolchainRoot\mingw64\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "$PWD\target-verify"
$env:TMP = "$PWD\tmp-verify"
$env:TEMP = $env:TMP
cargo fmt -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline
cargo test --locked --offline --test description --test engine --test loading
cargo build --release --locked --offline
```

原目录中 `external` 测试写入已有 fixture 受沙箱限制；独立项目验证结果见 [迁移记录](docs/07-project-independence.md)。历史实施输出见 docs/03，其中的旧路径属于原项目。

实际验证记录与剩余限制见分步骤实施方案。`scripts/gui-verify.ps1` 使用 Windows UIAutomation、键盘和鼠标测试中文输入、点击与缩放；`scripts/window-inspect.ps1` 保存窗口截图到 `evidence`。
