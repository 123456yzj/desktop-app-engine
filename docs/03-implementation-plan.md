# 分步骤实施方案

每步完成后勾选，并记录实际验证结果。

本文件的打包验收和权限阻塞记录来自原 `app-v1/desktop-app-engine` 目录。独立项目最新验证见 [迁移记录](07-project-independence.md)：新目录首次完整测试 13 项全部通过，包括 external；以下旧目录命令输出保留作为历史证据。

## 阶段 0：独立目录与方案

- [x] 在父目录创建 `desktop-app-engine`。
- [x] 分别记录需求、架构、实施步骤和问答。
- [x] 检查文档与当前需求一致。

## 阶段 1：环境与 GUI 验证

- [x] 检查 Rust、Windows 链接器及构建工具。
- [x] 核实并选择 GUI 框架，记录选择理由。
- [x] 建立独立 Cargo 项目，运行最小窗口。
- [ ] 验证中文显示、亮暗主题和窗口关闭。

完成条件：本机能构建并启动 Windows 桌面窗口。

## 阶段 2：描述协议

- [x] 定义协议 V0.1 的 Rust 类型和示例 JSON。
- [x] 实现版本、字段、状态类型、组件 ID 和引用校验。
- [x] 测试合法描述，以及未知组件、重复 ID、缺失引用等错误。

完成条件：合法描述可以加载，非法描述有清晰错误。

## 阶段 3：渲染与绑定

- [x] 支持行／列布局、文本、输入框和按钮。
- [x] 支持固定属性和状态引用。
- [x] 输入框更新状态，绑定文本随状态变化。
- [ ] 验证嵌套布局、长文本、窗口缩放及亮暗显示。

完成条件：界面结构来自描述，输入和显示能够关联。

## 阶段 4：事件与动作

- [x] 按钮点击引用动作，动作支持文本拼接与状态赋值。
- [x] 实现输入快照、顺序执行和失败反馈。
- [x] 测试输入名字后点击确认、修改名字后再次执行及无效动作。

完成条件：“输入 → 点击 → 结果更新”完整运行。

## 阶段 5：EXE 交付

- [x] 将示例描述嵌入程序，支持指定外部描述用于开发测试。
- [x] 构建 release EXE，提供可复现的构建命令。
- [x] 在无源码独立目录启动 EXE 的 --check，验证不依赖源文件或工作目录。
- [x] 运行格式检查、Clippy、原有 11 项测试、新增加载测试及发布构建。
- [ ] 完整 cargo test 全绿：external 写已有 fixture 受沙箱权限阻塞，尚未完成。
- [x] 记录产物位置、验证证据和未验证事项。
- [x] 采用固定运行时 + app.json，提供打包脚本并实跑冒烟校验。

完成条件：交付固定运行时与应用描述包。分发链路已验证；完整测试中的 external 权限阻塞待解除。GUI 历史遗留验收见阶段 1、3，未因本次 --check 验证自动勾选。

## 后续阶段

依次增加 SSH 与数据源、导航与多页面、可视化编辑器；应用包导出脚本已完成，编辑器的一键导出集成后续实施。
每阶段单独补充范围和验收，再实施。

## 当前验证状态

2026-10-12：本次使用工作区 .tools GNU 1.99.0 与 mingw64；以下命令均从项目根目录执行。
用户级 MSVC 安装不参与本次验收。未修改 Cargo.lock、依赖或协议 0.1；未修改主题及系统中文字体代码。

```powershell
$env:RUSTUP_HOME = "$PWD\.tools\rustup"
$env:CARGO_HOME = "$PWD\.tools\cargo"
$env:PATH = "$PWD\.tools\cargo\bin;$PWD\.tools\mingw64\bin;$env:PATH"
$env:CARGO_TARGET_DIR = "$PWD\target-verify"
$env:TMP = "$PWD\tmp-verify"
$env:TEMP = $env:TMP
```

### 验收 1～4：检查、测试与发布构建

```text
> cargo fmt -- --check
（无诊断输出）
FMT_EXIT=0

> cargo clippy --all-targets --locked --offline -- -D warnings
Checking desktop-app-engine v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
CLIPPY_EXIT=0

> cargo test --locked --offline
description: test result: ok. 5 passed; 0 failed
engine: test result: ok. 6 passed; 0 failed
external_descriptions_report_errors_and_generate_stress_fixture ... FAILED
tests\external.rs:11:53:
called `Result::unwrap()` on an `Err` value:
Os { code: 5, kind: PermissionDenied, message: "拒绝访问。" }
TEST_EXIT=101

> cargo test --locked --offline --test description --test engine --test loading
description: test result: ok. 5 passed; 0 failed
engine: test result: ok. 6 passed; 0 failed
loading: packaged_loading_priority_and_errors ... ok
loading: test result: ok. 1 passed; 0 failed
FOCUSED_TEST_EXIT=0

> cargo build --release --locked --offline
Finished `release` profile [optimized] target(s) in 0.28s
BUILD_EXIT=0
```

首次 release 构建实际编译耗时 13.05s，最终复查命中缓存。产物：`target-verify/release/desktop-app-engine.exe`，5,515,776 字节。
external 在写已有 `evidence/fixtures/invalid.json` 时失败，属于沙箱禁止覆盖已有文件的环境限制；未删除、改写或忽略该测试。
新增 loading 测试通过真实子进程验证同目录 app.json、包外工作目录、显式描述覆盖、错误退出 2 和内嵌兜底。

### 验收 5：两条 release 加载路径

为可靠等待 Windows GUI 子系统 EXE，实际用 Start-Process 重定向输出并等待进程；以下是执行命令。

```powershell
$explicitProcess = Start-Process -FilePath "$PWD\target-verify\release\desktop-app-engine.exe" -ArgumentList @('--app',"$PWD\examples\hello.json",'--check') -RedirectStandardOutput "$PWD\tmp-verify\release-explicit.out" -RedirectStandardError "$PWD\tmp-verify\release-explicit.err" -PassThru
$explicitProcess.WaitForExit()
"EXPLICIT_EXIT=$($explicitProcess.ExitCode)"
```

```text
启动参数：["--app", "D:\\project\\impact\\app-v1\\desktop-app-engine\\examples\\hello.json", "--check"]
描述校验通过: 问候示例 · 桌面描述引擎
EXPLICIT_EXIT=0
```

同目录加载仅传 `--check`，不传 `--app`，具体独立目录命令及输出见验收 6。

### 验收 6：打包与独立目录运行

```powershell
.\scripts\package.ps1 -Description '.\examples\hello.json' -Output '.\tmp-verify\exports-verified' -Name 'HelloApp'
"PACKAGE_EXIT=$LASTEXITCODE"
Get-ChildItem -LiteralPath '.\tmp-verify\exports-verified\HelloApp' | Format-Table Name,Length
```

```text
启动参数：["--check"]
描述校验通过: 问候示例 · 桌面描述引擎
应用包校验通过（退出码 0）：D:\project\impact\app-v1\desktop-app-engine\tmp-verify\exports-verified\HelloApp
PACKAGE_EXIT=0
Name          Length
app.json        1272
HelloApp.exe 5515776
README.txt       553
```

```powershell
$independent = New-Item -ItemType Directory -Path '.\tmp-verify\independent-app'
Copy-Item -LiteralPath '.\tmp-verify\exports-verified\HelloApp\HelloApp.exe','.\tmp-verify\exports-verified\HelloApp\app.json','.\tmp-verify\exports-verified\HelloApp\README.txt' -Destination $independent.FullName
Get-ChildItem -LiteralPath $independent.FullName | Format-Table Name,Length
$independentProcess = Start-Process -FilePath (Join-Path $independent.FullName 'HelloApp.exe') -ArgumentList '--check' -WorkingDirectory $env:SystemRoot -RedirectStandardOutput "$PWD\tmp-verify\release-independent.out" -RedirectStandardError "$PWD\tmp-verify\release-independent.err" -PassThru
$independentProcess.WaitForExit()
"INDEPENDENT_EXIT=$($independentProcess.ExitCode)"
```

```text
Name          Length
app.json        1272
HelloApp.exe 5515776
README.txt       553
启动参数：["--check"]
描述校验通过: 问候示例 · 桌面描述引擎
INDEPENDENT_EXIT=0
```

目录内只有上述三个文件，没有 Rust、源码或制作工具。工作目录为 C:\Windows，加载的是 EXE 同目录 app.json。
另外用 Get-FileHash 比较 release 运行时与包内 EXE 的 SHA256，结果 `RUNTIME_PACKAGE_IDENTICAL=True`，确认仅复制运行时。
打包等待逻辑采用重定向输出加 WaitForExit，避免自动化终端等待 GUI 子系统进程时阻塞；超时 30 秒返回失败。

### 验收 7：文档与旧 dist

docs/06 记录 A/B 比较并采用 A；README 更新运行、分发和 GNU 离线验收；docs/02 更新决策；本文件仅按实测勾选。
docs/01、04 同步应用包交付范围。旧 dist 未覆盖，禁止作为新打包源。

实际执行 `Get-Item -LiteralPath '.\dist\desktop-app-engine.exe','.\src\main.rs' | Format-Table Name,LastWriteTime`：

```text
desktop-app-engine.exe 2026/10/10 17:51:19
main.rs                2026/10/12 14:38:22
```

实际启动旧 dist 的 `--check`，退出码 0；`tmp-verify/old-dist.out` 只有“描述校验通过: 问候示例 · 桌面描述引擎”。新 EXE 的输出另外包含“启动参数”，证明旧产物与当前 main.rs 不一致。

本次未生成截图，也未重新验收暗色/长文本/缩放 GUI。历史有效证据仅 `evidence/window-39128.png`、`evidence/window-36932.png`；其他指定空白废图不作为证据。

## 历史验证状态（2026-10-10，不代表本次结果）

2026-10-10：PATH、用户默认安装目录及 D 盘未找到 Rust/GCC。
已在本项目 `.tools` 安装 Rust 1.99.0 GNU、rustfmt、Clippy，以及 WinLibs GCC 16.2.0 / MinGW-w64 UCRT。
工具版本命令已运行成功；不修改全局 PATH。`scripts/cargo.ps1` 为当前进程配置工具路径。
GitHub 直连下载过慢，改用本机已有代理完成 105 MiB 工具包下载。
最小窗口已编译、启动并通过 UIAutomation 读取及截图检查中文、亮色主题；通过 WindowPattern.Close 关闭。
证据：`evidence/window-39128.png`。
协议 5 项测试、动作与绑定 6 项测试全部通过（非法样例使用多次断言）。
Clippy `--all-targets --locked -- -D warnings` 已通过。
第一轮 release 构建完成，原生 GUI 通过键盘输入/鼠标点击与 UIAutomation 检查“小明→小红”两次结果。
480×360 缩放复测发现按钮挤出可视区，已调整输入框预留宽度及垂直滚动，等待最终复测。
最终 EXE 独立目录验证、暗色与长文本尚待完成。

## 阶段 0 Review

已检查文档范围与导航目标：方向、架构、步骤和问答分别保存；当前实现与后续能力区分明确。
该 Review 为阶段 0 历史记录；现已选择 egui/eframe，打包结论与当前实测以本文件的 2026-10-12 记录和 docs/06 为准。
