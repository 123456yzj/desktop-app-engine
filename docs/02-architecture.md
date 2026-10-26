# 引擎架构

工程为单 Cargo crate，对外提供 `desktop-app-engine.exe package`，接收 JSON 内容并导出一个应用 EXE。内部业务开发宿主位于 `src/bin/vben-package/`，通过库接口使用引擎。

所有组件源码统一位于 [src/components/](../src/components/README.md)，组件契约、数据算法、布局和原生绘制均在该目录内组织。本页说明它们与协议、编译和运行时的关系。

## 描述到运行时

```text
loader：选择输入，确定资源基准目录
  → protocol：严格解析原始描述
  → compiler：插件注册、词法作用域、引用与类型校验
  → CompiledDescription：已解析描述与内部元数据
  → runtime：状态、有序事件、动作和任务调度
  → components/layout：解析属性、测量与安排矩形
  → components/egui：输入采集、字体、主题和绘制
```

`protocol::parse` 只解析输入，首先要求 `schemaVersion` 为 `"0.2"`，不发现插件、不改写作用域名称。`compiler::compile_at` 接收描述及资源目录，产生 `CompiledDescription`；只有编译器能构造该类型。加载器直接返回编译结果。节点名称映射、插件注册、动作所在模块和事件类型样例保存在独立元数据中，不进入 JSON 契约。

`Runtime::new` 为程序化调用提供解析后编译入口；加载器和渲染器也可直接接收编译结果。原始描述可重复使用，内部编译结果无需再次解析词法作用域。

## 边界与状态

- `protocol` 定义输入类型；`compiler` 解析名称、注册资源并验证完整语义。
- `components` 为库内组件实现，84 种组件各有一个 `egui/widgets/` 文件，声明契约并封装专属行为或布局策略。arena 树、绑定缓存、值校验、表格/图表算法和布局算法共享；公共 `layout` 入口导出其中的布局能力。
- `runtime` 的状态、版本、generation 和编译描述不允许库外直接写入。宿主通过 `state()`、`version()`、`state_from()` 读取，通过 `set()`、`set_many()`、`set_from()` 写入。
- 状态更新先验证候选状态；失败保持旧值和版本。成功更新只增加变动键的版本，同值写入不增加版本。
- 输入按 sequence 提交，旧 generation 和重复事件被丢弃。动作按步骤执行，读取开始时快照；失败停止后续步骤，之前成功写入保留。
- HTTP、插件和命令完成结果通过运行时提交状态；网络、进程、RPC、DPAPI 与日志的具体实现位于 `services`。
- `renderer` 导出组件宿主接口，并提供运行日志界面；宿主交互、文本、主题与图表绘制实现在 `components/egui/`。业务控制器管理仓库、构建批次和用户连接设置。

库只公开分层入口，原平铺短名仅在 crate 内作为别名使用。协议、执行器和组件绘制统一使用当前实现；通用字符串限制位于 `protocol/validation.rs`，中文字体安装位于 `components/egui/fonts.rs`。

`src/components/layout/mod.rs` 定义布局入口与结果类型，由库导出为 `layout`，维持 `layout::arrange`、`layout::measure` 等调用路径。`geometry.rs` 定义几何与约束，`flex.rs` 分配轨道空间，`measure.rs` 测量组件自然尺寸，`arrange.rs` 计算位置、裁剪和滚动。网格排列复用同一批子节点测量结果计算行高和单元格位置；文本尺寸由宿主实现 `TextMeasure` 提供。

`components/catalogue.rs` 定义属性、事件和布局策略的共享元数据类型，注册入口转发到 `egui/widgets.rs` 的穷尽分派。`contracts.rs` 使用组件声明进行规范化和事件属性查询。组件绘制复用 `widgets/shared/` 的菜单、弹层、图片等机制；宿主保留帧流程、本地状态、滚动、表单与事件协调。

## 资源与交付

开发运行时优先使用 `--app FILE`，其次为 EXE 同目录 `app.json`，缺少描述时报错。开发插件相对输入目录发现；原生打包命令使用引擎已安装的插件目录，Cargo 开发运行时可使用仓库插件目录。

`packaging::package` 接收 JSON 内容和输出路径，校验后按 `app.host` 选择制作端引擎旁的原生宿主模板，缺省使用通用引擎。随后将插件资源、应用定义及版本化索引追加到 EXE，并将 PE 子系统设为 Windows GUI。每次应用导出不调用编译工具。`application` 从 EXE 尾部读取索引，校验资源范围和路径；插件按资源内容标识释放到当前用户缓存，加载器及原生宿主以缓存目录作为资源基准。打包 EXE 固定使用内置定义，详见 [分发决策](06-packaging-decision.md)。

egui/eframe 0.31.1 提供窗口、输入和基础图元；resvg 0.48.1 处理插件 SVG。中文字体来自 Windows 标准字体目录。版本由 `Cargo.lock` 锁定。
