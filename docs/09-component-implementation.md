# 静态 Dashboard 实施记录

日期：2026-10-13。工作目录：`D:\project\impact\desktop-app-engine`。

## 方案与环境

完整阅读 README、docs/02、05、06、07、08 及 src/tests 全部代码。当前基线只有协议 0.1；保留其解析、执行和渲染路径，新增独立 0.2。新增纯算法 binding/component/layout/interaction/table/chart 模块及 GUI 适配，不增加依赖，不复制第三方源码。

0.2 直接 JSON 初始值递归记录类型；空数组的元素类型由组件消费校验。set/concat 保持快照。布局尺寸为 point，width/height 使用 `{fixed}`、`{fr}` 或 `{auto:true}`；缺省为 auto。所有字段必须有行为，不支持的配置拒绝。

GNU 离线工具链：Cargo 1.99.0 (5f94df478)、rustc 1.99.0 (b940084d7)、MinGW GCC 16.2.0，x86_64-pc-windows-gnu。工具链位于历史目录 .tools，构建和临时目录为独立项目 target-verify、tmp-verify。CodeGraph 未加载项目，使用实际文件读取。

基线 `cargo test --locked --offline`：原有 13 项全部通过，退出码 0（含 external，无权限错误）。

## 阶段状态

- P0/P1 协议和接口：首轮完成。fmt、Clippy（-D warnings）、test（原 13 + 新 3 项）、release build 均退出 0。发布构建 13.95s。CLI 的 0.2 GUI 分派将在布局切片接通。
- P2/P3：首轮 fmt、Clippy、test（原 13 + 绑定 3 + 布局 3）、release build 均退出 0。首轮 release 15.16s。0.2 CLI 接通。GUI 检查后修正 fr 基准挤压和可访问性裁剪，等待复验。
- P4/P5：首轮算法 5 项通过，全部测试 24 项通过；fmt、Clippy、release build 均退出 0，release 14.19s。共享列宽、固定表头、稳定排序、固定行高虚拟化、坐标变换、刻度、分组柱图和降采样接入。补充缓存、键盘、GUI 与性能验证中。
- P6：Dashboard、schema、文档已接通；原生 GUI 和独立分发验证见下文。

每阶段验证及最终 GUI、性能、分发实测在完成后追加；未勾选项目不能视为通过。

最终回归中 `cargo build --release --locked --offline` 曾退出 101：`failed to remove file target-verify/release/desktop-app-engine.exe`，`拒绝访问 (os error 5)`。Get-Process 查明本次验收启动的暗主题进程 49968 尚在占用 EXE（CloseMainWindow 返回 true 但进程未退出）。结束该验收进程后重跑，不删除测试、不修改权限。失败和重跑输出保存在 evidence/component-verification.txt。

## GUI 中间检查

原生 release 窗口已启动。最初 CopyFromScreen 截到了遮挡它的 VS Code，`dashboard-light-initial` 随后已由实际 Dashboard 前台截图覆盖；`dashboard-focus-debug*` 是调试截图，不能作验收证据。通过临时置顶、点击标题后解除置顶获得前台，未修改其他窗口。AccessKit 的 FindFirst 条件查询未检出自绘标签，改用 FindAll 后按 Name 检查。

已实际点击更新并观察 42.0% → 57.0%，滚动页面后点击负载列出现 ▲，截图可见 21%、26%、31% 顺序。脚本随后因测试脚本类型名笔误中止，选择/键盘/缩放不据此标为通过。后续修正脚本并重新跑。

补充协议：图表可显式 `zoom:true`、`pan:true`（默认 false）；双击或聚焦后 Home 恢复自动范围。视口只保存在组件本地。绑定使用每个状态键版本，指标更新不重建无关表格和系列缓存。

## 最终验证与产物

环境：Windows 10.0.26200，Intel Core i5-13500H，12 核/16 逻辑处理器，GNU release、thin LTO。构建未新增依赖，Cargo.toml/Cargo.lock 保持原有依赖；不复制第三方代码，算法依据设计基线独立实现，无新增第三方移植许可证条目。

| 命令 | 实际输出摘要 | 退出码 |
| --- | --- | --- |
| `cargo fmt -- --check` | 无格式诊断 | 0 |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | Finished dev profile | 0 |
| `cargo test --locked --offline` | 30 passed，0 failed，0 ignored；原有 13 项全通过 | 0 |
| `cargo build --release --locked --offline` | 最终代码构建 21.25s，缓存复验 0.40s | 0 |
| `desktop-app-engine.exe --app examples/hello.json --check` | 描述校验通过: 问候示例 · 桌面描述引擎 | 0 |
| `desktop-app-engine.exe --app examples/dashboard.json --check` | 描述校验通过: 运行状态看板 | 0 |
| `python scripts/schema-verify.py` | Draft 2020-12、Dashboard、4 个非法结构案例通过 | 0 |
| `cargo test --release --locked --offline ten_thousand_rows_use_actual_visible_paint_path -- --nocapture` | TABLE total=10000 painted=9 shapes=52 viewport=800x320 screen=1920x1080 frames=100 p95_us=93 profile=release | 0 |
| `cargo test --release --locked --offline sampling_preserves_spikes_order_and_gaps -- --nocapture` | SAMPLE points=100000 output=2004 width=1000 elapsed_us=495 profile=release | 0 |
| `scripts/package.ps1 -Description examples/dashboard.json -Output exports -Name DashboardAppRelease` | 应用包校验通过 | 0 |
| `scripts/standalone-verify.ps1` | STANDALONE_EXIT=0，独立目录仅 3 个分发文件 | 0 |

完整可复查命令输出在 `evidence/component-verification.txt`（追加保留失败）。合并调用曾在 release 测试编译期间或末尾触及工具 60 秒时限，未声称整个调用成功；随后完整重跑成功。最终性能数值取最终日志，不用首次更小的数值替代；多轮测量有机器负载波动，完整结果均保留。

自动测试构成：description 5、engine 6、external 1、loading 1；新增 layout 4、v02 4、table_chart 6、dashboard_loading 1、GUI 实际绘制路径/原始顶点 hover 2，共 30。新增 CLI 集成测试会复制运行时到无源码临时目录，检查非法字段、绑定类型、引用、结构 null 均退出 2，显式 --app 优先，同目录错误不回退。原有测试未删除或忽略。

性能测试调用真实表格 painter 和 egui Context，含排版和图元生成；不含 OS 窗口、OpenGL 提交和中文系统字体安装。可见行绘制数量随视口变化，不随 10000 行增长；首次数据校验/索引建立和排序仍为 O(N)/O(N log N)。降采样输入含 +1000/-500 相邻尖峰及 null 断点，断言首末/极值/断点保留、输出索引有序。缓存依赖系列数据版本、绘图区宽度和轴范围。完整 Dashboard 原生交互帧 CPU P95 <=16ms 尚未采集，不能用上述表格数据替代。

正式产物：

```text
target-verify/release/desktop-app-engine.exe
exports/DashboardAppRelease/DashboardAppRelease.exe    6422016 bytes
exports/DashboardAppRelease/app.json                       6997 bytes
exports/DashboardAppRelease/README.txt                      586 bytes
tmp-verify/DashboardStandaloneRelease/          # 同样仅 3 文件，无源码独立目录
```

运行时与最终导出 EXE SHA256：`1B785C2AC491FA4E790D2698F4CB04CD306F5E700164DAE517EEC221CA169DB5`。
示例与导出 JSON SHA256：`7418D672281D0F1EBF9B69D2762E8AE1133AAC0029F6F57EAEFD29D7CDA00187`。打包只复制，未调用 Cargo。

DashboardApp、DashboardAppFinal 为中间验收包，保留过程产物；最终使用 DashboardAppRelease。独立目录最终再次通过 --check，之后从该目录实际启动 dark 窗口（进程 41144），完整 dashboard-verify 脚本退出 0。

## GUI 实测与限制

- [x] light/dark 原生窗口中文文字、四指标、折线与分组柱图；截图逐张查看确有文字/控件。
- [x] 点击“更新指标”：42.0% → 57.0%，note 更新；重复绘制不触发更新。
- [x] 页面滚动、表体滚动（固定表头）、负载升序；排序截图显示 21%、26%、31% 顺序。
- [x] 列边界拖拽 848 → 928 物理像素，表头/表体列线对齐；选中身份由 rowKey 保存。
- [x] 点击行后方向键切换选中，截图高亮重庆节点 26.0%；重复选择事件由单元测试验证。
- [x] 图表 hover 原始点 CPU X=3/Y=0.66（AccessKit 文本断言 + 截图）；图例隐藏 CPU、滚轮缩放、拖拽平移、双击恢复自动范围。
- [x] 窗口从 1780×1260 调整至 1150×850 物理像素；内容重排、裁剪、滚动保留。
- [x] 独立包 Ctrl+- 缩放后实际文字、图表和表格仍显示；窄表列使用单行裁剪，保持固定行高。
- [ ] 完整窗口帧 CPU P95、多显示器跨 DPI 和独立中文 IME 候选窗的专项复验尚未采集。

有效证据：`evidence/dashboard-light-{initial,hover,zoom,pan,legend,updated,sorted,column-resized,selected,table-scrolled,resized}.png` 和 `dashboard-dark-{initial,hover,zoom,legend,updated,sorted,selected,resized}.png`。`*-failed*`/`*-debug*`/`*-inspect*` 为中间排查文件，不作为通过证据。

最终独立包证据为 `evidence/dashboard-release-dark-{initial,zoom,pan,legend,updated,sorted,column-resized,selected,table-scrolled,resized,ui-scaled}.png`；已打开检查 hover/selected/ui-scaled 文件均有实际文字/控件。最终 hover 的 AccessKit 原始点断言通过，但截图时光标离开，hover 浮层可见证据使用此前已经检查的 `dashboard-final-dark-hover.png` 和 `dashboard-light-hover.png`，不将无浮层截图当 hover 证据。

GUI 脚本曾因鼠标被环境移动而失败（目标 606,775，实际 1462,2159）；增加有限次数定位并验证目标文字后通过，不以“输入已发送”代替 hover 成功。测试还新增无窗口 PointerMoved → 原始顶点文字断言，修正线段 hover 返回相邻末端的问题。

## 实现取舍与后续

0.1 保留原即时渲染，未迁移至 0.2 arena，避免同时改写旧动作和新图表。0.2 使用静态枚举、模块函数和窄的 egui::Painter 适配，未引入动态组件注册或独立绘制命令 IR；几何、布局、绑定、表格、图表纯算法均不依赖 egui::Ui。文本测量注入 TextMeasure，measure/arrange 不修改业务状态；paint 的交互反馈只修改本地状态，业务事件由 commit 统一提交。

布局暂每帧全树计算，解析属性和数据缓存按状态键版本更新；未宣称布局增量失效已完成。固定列 Grid 无 span/固定行轨道/autoColumns；这些字段不接受。轴标签使用固定预留槽和宽度控制刻度密度，未实现任意字体极端标签的两轮轴边距收敛；字段未承诺自动边距。表格可访问性通过选中行摘要和可见单元格文本提供，未构建完整 AccessKit Table/Cell 角色树。

拖拽停靠、可变行高、合并单元格、复杂图表、异步数据源 P7 均在约定范围之外。下一步建议为边界字体/DPI/IME 和完整窗口性能采样补专项验收，再考虑布局缓存与 P7 独立数据源契约。

## 改动文件清单

- 修改：src/lib.rs、src/loader.rs、src/main.rs（模块与版本分派，原 0.1 API 保留）。
- 新增：src/v02.rs、src/binding.rs、src/component.rs、src/layout.rs、src/interaction.rs、src/table.rs、src/chart.rs、src/gui_v02.rs。
- 新增：tests/v02.rs、tests/layout.rs、tests/table_chart.rs、tests/dashboard_loading.rs，gui_v02 内部有实际 painter/input 测试；原 13 项测试文件保持。
- 新增：examples/dashboard.json、docs/description-0.2.schema.json、docs/09-component-implementation.md。
- 更新：README.md、docs/02-architecture.md、docs/05-description-format.md、docs/06-packaging-decision.md、docs/08-component-engine-design.md。
- 新增验证脚本：scripts/dashboard-verify.ps1、scripts/schema-verify.py、scripts/verify-components.ps1、scripts/standalone-verify.ps1。继续使用原 scripts/package.ps1。
- 生成：evidence 中验证日志与截图、exports 应用包、tmp-verify 独立包、target-verify 构建产物。未提交 Git。
