# 项目测试规范

本规范适用于组件样式、GUI 交互、打包产物及相关自动化测试。项目级代理入口见 [AGENTS.md](../AGENTS.md)。

## 允许的测试方案

| 测试对象 | 要求采用的方式 | 验证范围 |
| --- | --- | --- |
| 协议、class 解析、状态运行时、输入校验、纯布局算法 | 普通单元测试 | 数据契约、合并规则、业务行为与算法结果 |
| 界面样式与 GUI 交互状态 | 离屏渲染 PNG、图片快照对比及必要的视觉检查 | 实际像素效果，以及普通、悬停、按下、禁用、焦点等适用状态 |
| 最终打包应用 | 隔离环境中启动实际 EXE；或产物自身已实现的离屏测试入口 | 内置定义加载后的实际显示、交互及所用渲染链路 |

**禁止使用仅运行 egui 上下文、注入模拟输入并检查绘制数据或内部 UI 状态的第一种方案。** 不得新增或恢复此前清理的这类测试，也不得用“产生了图形”“绘制参数符合预期”或“成功完成三角化”代替图片测试。离屏测试可由测试框架驱动输入，但必须实际完成像素渲染并验证结果图片。

纯逻辑单元测试可直接构造运行时事件验证业务规则；其结果只说明相应逻辑通过，不说明 GUI 的命中、焦点、显示或交互通过。

## 离屏图片测试要求

- 将实际渲染结果输出为可检查的 PNG，不能仅检查 `Shape`、`Galley`、颜色参数或布局结构体。
- 固定或记录字体、主题、窗口尺寸、缩放比例和渲染后端，使结果可复现。按变更范围覆盖明暗主题、缩放及适用交互状态。
- 基准图必须先检查视觉效果。不得把自动生成或更新基准图后的测试成功当作正确性证据；差异应检查并说明。
- 图片失败时保留实际图和差异图。测试未运行、图片未生成或结果未检查时，不得报告视觉验收通过。
- 报告离屏后端与应用后端的差异。通过其他后端的图片测试不能证明最终应用的 OpenGL、窗口系统或 DPI 行为通过。

## 打包产物测试要求

- 使用实际生成的 EXE，从 EXE 内置定义加载页面；记录构建类型（Debug/Release）和产物位置。
- 开发程序或测试程序读取源 JSON 的结果不能替代打包产物的显示与交互验证。
- `--check`、`--check-plugins`、JSON Schema 校验、编译和静态检查只验证各自范围，不能证明界面样式正确。
- 打包产物的离屏入口只有在已实现、已执行且确认读取内置定义后，才可作为相应测试证据。不得将计划中的能力写成已具备能力。
- 实际窗口测试使用独立虚拟机或专用桌面会话等隔离环境，不占用用户当前桌面的鼠标、键盘和前台窗口。交互受干扰或截图不符合目标状态时，该项不得判定通过。

## 执行与报告

用户仅询问测试状态、方法或方案时，应直接回答，不因问题本身启动测试。执行测试应属于用户已授权的工作范围。

报告应明确测试对象、构建类型、测试方式、实际结果、图片证据与未测范围。逻辑测试通过、描述可加载和视觉验收通过必须分别陈述；未完成或失败的测试如实报告。

当前已接入协议与 class 解析、状态运行时、输入校验和纯布局算法单元测试，以及下述打包产物离屏 PNG 验证。真实窗口的隔离自动化尚未接入。

## 打包产物离屏入口

通过可选 Cargo feature `visual-tests` 编译；默认构建不包含离屏测试依赖和入口。该构建仍使用 Glow 运行普通窗口，`--visual-test SCENARIO.json` 则使用 `egui_kittest 0.31.1` 的 wgpu 后端渲染内存纹理并读回 PNG，不创建桌面窗口或发送系统输入。它仅允许实际打包 EXE 读取内置定义，拒绝与 `--app`、校验或凭据命令混用。

```powershell
cargo build --bin desktop-app-engine --features visual-tests --locked
python scripts/test-packaged-classes.py --engine target/debug/desktop-app-engine.exe --output C:/Temp/class-first
```

脚本需要 Pillow；`--output` 必须是不存在的新目录。它打包 `examples/utility-classes.json`，在项目目录外启动新 EXE，覆盖浅色/深色和 1/1.75 倍缩放，每组输出 12 张完整 PNG，检查实际像素颜色、圆角/显式直角、点击状态变化、禁用状态、输入编辑及溢出提示。禁用按钮绑定一个可见错误动作，用于确认点击没有被分发。示例使用显式白色页面及配套文字色，因此在两种主题下均保留固定配色。

先检查这些图片，再明确指定已检查的目录进行逐像素快照比较：

```powershell
python scripts/test-packaged-classes.py --engine target/debug/desktop-app-engine.exe --output C:/Temp/class-repeat --baseline C:/Temp/class-first
```

脚本不会创建或更新基准图。快照不同时保留实际 PNG 和 `*.diff.png` 并失败。每组的 `render-report.json` 记录实际 EXE、内置定义来源、Debug/Release、主题、尺寸、缩放、适配器及后端；总目录保留 `fonts.json`（实际系统字体文件 SHA-256）、进程输出和 `results.json`。字体、平台或渲染器改变时必须重新检查图片，不能直接认可新基准。

Release 产物用 `cargo build --release --bin desktop-app-engine --features visual-tests --locked` 构建，再将脚本 `--engine` 指向 `target/release/desktop-app-engine.exe`。离屏通过的范围仍然是该构建的 wgpu 渲染与应用交互；Glow 驱动、原生窗口、系统输入、实际显示器 DPI 和全部组件均须另外验收。

### Select 图片与交互回归

`python scripts/test-packaged-select.py --engine target/debug/desktop-app-engine.exe --output C:/Temp/select-acceptance` 打包专用定义并运行其 EXE 内置定义，覆盖浅色 1/1.75 倍和深色 1 倍。每组输出 43 张 PNG，检查完整悬停边框、搜索、禁用项、单选提交收起、Escape/Tab/外部点击关闭、清空、多选上限、标签创建和加载态；仍须查看关键图片。测试定义显式使用白色控件，深色场景验证白色控件在深色宿主中的显示。

离屏步骤支持可选 `keys` 数组（ArrowDown、ArrowUp、Enter、Escape、Tab、Space、Backspace，每步最多 20 个）及 `wheel: [x, y]` 点单位滚轮输入。键盘、滚轮只注入离屏上下文，交互结果实际渲染为 PNG。

步骤可设置 `alt: true`，用于验证原生图表的 Alt + 滚轮缩放；默认无 Alt，普通滚轮滚动页面。

可选 `wait_ms`（0..5000）在截图前继续轮询和绘制应用，等待真实插件进程等异步任务；默认不等待。它不向桌面发送输入。

### 组件文档展示应用

`python scripts/build-component-gallery.py` 从现有组件示例与用法配置生成 `examples/component-gallery.json`，包含 84 个独立组件页面。打包后运行 `python scripts/test-packaged-gallery.py --exe <实际打包的EXE> --output <新目录>`，通过应用自身的搜索跳转依次访问全部页面并生成 PNG 与联系表。`--pages button,form,select` 可缩小范围；`--scale 1.75` 和 `--dark` 可切换缩放及宿主主题。页面采用显式浅色配置。生成图片后仍须检查相应页面与交互，脚本仅生成证据并检查图片存在及非空，不自动宣布视觉验收通过。
