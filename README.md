# 桌面应用描述引擎

对外提供一个原生打包入口：输入 JSON 内容，输出应用专属的 Windows EXE。统一使用协议 0.2，提供组件树、布局、类型化状态、模块作用域、表格、图表、命令、HTTP 数据源和 JavaScript 插件。

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

[全部组件展示](examples/all-components.json) 是完整的协议 0.2 应用，覆盖全部 84 种组件，按布局、录入、导航、展示、数据和反馈分为六个可切换分区。示例包含表单校验、模块通信、表格分页、图表与弹窗交互；Image 使用无来源占位，Upload 只记录本地选择。Plugin 使用仓库中的 ECharts 适配器，开发加载需在描述文件旁准备 `plugins/`，打包需按 [插件安装说明](docs/15-plugins.md#安装和运行) 准备生产依赖和 Node 运行库。

[组件实验室](examples/component-gallery.json) 提供文档式浏览：左侧按分类逐个列出 84 种组件，右侧展示说明与 173 个用法示例，支持中英文搜索跳转和独立滚动。运行 `python scripts/build-component-gallery.py` 可根据上述示例与用法配置重新生成定义。它使用相同的 ECharts 插件资源；打包方式与其他协议 0.2 应用一致。实际 EXE 的图片检查方法见 [组件文档展示应用测试](docs/testing-policy.md#组件文档展示应用)。

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
| `src/components/` | 全部组件实现、组件契约、输入校验、组件树、数据算法、布局与原生绘制 |
| `src/services/` | 子进程、HTTP、插件 RPC、凭据和日志 |
| `src/packaging.rs`、`src/application.rs` | 单 EXE 导出、内置定义读取和插件资源缓存 |
| `src/renderer/` | 组件宿主的公开导出入口和日志界面 |
| `src/bin/vben-package/` | 业务控制器、远端操作、设置和界面生成 |

数据流为 `protocol::parse` → `compiler::compile_at` → `CompiledDescription` → `Runtime::from_compiled`。`loader` 负责输入选择和资源基准目录，直接返回编译结果；`schemaVersion` 必须为 `"0.2"`。原始描述可重复编译；内部名称、插件注册和事件样例不属于 JSON 协议。运行时通过只读访问器提供状态，写入走 `set`、`set_many` 或 `set_from`，统一维护校验与版本。详情见 [架构](docs/02-architecture.md)。

所有组件源码统一位于 [src/components/](src/components/README.md)。84 种组件各自封装在 `egui/widgets/` 的独立文件，声明扩展属性、事件和受控映射，实现专属绘制、交互或布局策略；数据、布局算法和重复的绘制机制共享。目录说明提供完整文件对应表。

## 应用打包

统一接口接收 JSON 内容和输出 EXE 路径，宿主从 JSON 的 `app.host` 自动解析；省略时使用通用引擎。原生宿主模板 `<host>.exe` 安装在制作端引擎旁，缺失时报错。先构建一次制作端引擎，随后直接调用原生命令：

```cmd
target\release\desktop-app-engine.exe package --output exports\Monitor.exe < C:\Apps\monitor.json
```

引擎先完成结构和语义校验，将定义及所需插件资源嵌入 EXE，再执行包内校验；通过后写入指定输出路径。最终只交付一个 EXE，已有输出路径拒绝覆盖。成功退出 0，标准输出为产物绝对路径；失败退出 2，原因写入标准错误。

打包 EXE 固定使用内置定义，拒绝 `--app` 覆盖。修改 JSON 后重新打包。客户机需要 Windows 系统中文字体、OpenGL 驱动及应用声明的外部程序；插件及其 Node 运行库自动释放到用户资源缓存。凭据和用户设置保存在当前用户目录。详见 [分发决策](docs/06-packaging-decision.md)。

## 验证与协议

**测试必须遵循 [项目测试规范](docs/testing-policy.md)。** 界面样式采用离屏 PNG 图片测试，打包产物在隔离环境中验证实际显示与交互；禁止通过注入输入并断言绘制数据来验收 GUI。下面的构建、单元测试及 `--check` 不代表视觉验收通过。

编写界面可从 [组件使用教程](docs/components/README.md) 开始：按布局容器、基础组件、数据录入、数据展示与扩展分类，提供完整 JSON 示例、属性表、事件说明和常见问题。首次使用建议先读 [01 快速开始](docs/components/01-quick-start.md) 和 [02 公共属性与布局](docs/components/02-common.md)。

```powershell
cargo fmt -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --lib --locked --offline
cargo run --bin desktop-app-engine --locked --offline -- --app .\examples\vben-package.json --check
cargo run --bin vben-package --locked --offline -- --check
```

协议 0.2 当前提供 84 种组件 type，包含新增的导航、日期/树输入、图片/二维码、弹窗提示和配置装饰组件。教程示例可通过 `python scripts/check-component-docs.py --schema` 批量校验（需 Python jsonschema）；现有单元测试覆盖协议与 class 解析、纯布局算法、表单校验、受控状态和运行时事件顺序。打包 EXE 的离屏 PNG 入口和 class 示例图片验证已接入，使用可选 `visual-tests` feature；[运行方法和覆盖范围](docs/testing-policy.md#打包产物离屏入口) 说明 wgpu 与正式 Glow 后端的差异。真实窗口的隔离自动化尚未接入。完整目录见 [组件教程](docs/components/README.md)，新增组件见 [导航](docs/components/33-navigation.md)、[录入与树](docs/components/34-native-inputs.md)、[展示](docs/components/35-native-display.md)、[反馈](docs/components/36-native-feedback.md)、[配置与装饰](docs/components/37-native-context.md)。[覆盖对照](docs/21-component-coverage.md) 逐项记录尚未实现的属性和桌面差异，名称覆盖不代表官网完整 API 兼容。远端 SSH 操作与客户环境需单独验证。

- [描述格式](docs/05-description-format.md) 与 [0.2 JSON Schema](docs/description-0.2.schema.json)
- [命令](docs/10-command-binding.md)、[动态事件](docs/11-dynamic-event-binding.md)、[作用域](docs/12-scoped-state.md)
- [模块](docs/13-module-component.md) 与 [模块通信](docs/14-module-communication.md)
- [插件](docs/15-plugins.md)、[HTTP](docs/16-http-data-sources.md)、[凭据](docs/17-encrypted-credentials.md)
- [下拉选择](docs/18-select.md)、[运行日志](docs/19-runtime-logs.md)、[构建工作台](docs/20-turbo-build.md)
