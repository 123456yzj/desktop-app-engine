# 引擎架构

工程为单 Cargo crate。通用运行时和业务工作台是两个二进制，业务代码位于 `src/bin/vben-package/`，通过公开库接口使用引擎。

## 描述到运行时

```text
loader：选择输入与协议版本，确定资源基准目录
  → protocol：严格解析原始描述
  → compiler：插件注册、词法作用域、引用与类型校验
  → CompiledDescription：已解析描述与内部元数据
  → runtime：状态、有序事件、动作和任务调度
  → components/layout：解析属性、测量与安排矩形
  → renderer：输入采集、字体、主题和绘制
```

`protocol::v02::parse` 只解析输入，不发现插件、不改写作用域名称。`compiler::compile_at` 接收描述及资源目录，产生 `CompiledDescription`；只有编译器能构造该类型。节点名称映射、插件注册、动作所在模块和事件类型样例保存在独立元数据中，不进入 JSON 契约。

`Runtime::new` 为程序化调用提供解析后编译入口；加载器和渲染器也可直接接收编译结果。原始描述可重复使用，内部编译结果无需再次解析词法作用域。

## 边界与状态

- `protocol` 定义输入类型；`compiler` 解析名称、注册资源并验证完整语义。
- `components` 为库内实现，维护 arena 树、绑定缓存及表格/图表算法；`layout` 计算布局。
- `runtime` 的状态、版本、generation 和编译描述不允许库外直接写入。宿主通过 `state()`、`version()`、`state_from()` 读取，通过 `set()`、`set_many()`、`set_from()` 写入。
- 状态更新先验证候选状态；失败保持旧值和版本。成功更新只增加变动键的版本，同值写入不增加版本。
- 输入按 sequence 提交，旧 generation 和重复事件被丢弃。动作按步骤执行，读取开始时快照；失败停止后续步骤，之前成功写入保留。
- HTTP、插件和命令完成结果通过运行时提交状态；网络、进程、RPC、DPAPI 与日志的具体实现位于 `services`。
- `renderer` 组织宿主交互，文本、主题和图表绘制有独立实现。业务控制器管理仓库、构建批次和用户连接设置。

库只公开分层入口，原平铺短名仅在 crate 内作为别名使用。协议 0.1 的执行和渲染位于各层 legacy 实现，保持独立版本语义。

`src/layout/mod.rs` 保留布局入口与结果类型，通过导出维持 `layout::arrange`、`layout::measure` 等调用路径。`geometry.rs` 定义几何与约束，`flex.rs` 分配轨道空间，`measure.rs` 测量组件自然尺寸，`arrange.rs` 计算位置、裁剪和滚动。网格排列复用同一批子节点测量结果计算行高和单元格位置；文本尺寸由宿主实现 `TextMeasure` 提供。

## 资源与交付

开发运行时优先使用 `--app FILE`，其次为 EXE 同目录 `app.json`，缺少描述时报错。插件相对输入目录发现；打包描述的插件相对 EXE 目录发现。

`build.rs` 根据 `DAE_APP_DESCRIPTION` 生成内嵌定义，每次应用导出都编译对应运行时。通用打包脚本使用显式 `-Runtime` 参数，业务使用说明由独立模板提供。打包 EXE 固定使用内嵌定义，详见 [分发决策](06-packaging-decision.md)。

egui/eframe 0.31.1 提供窗口、输入和基础图元；resvg 0.48.1 处理插件 SVG。中文字体来自 Windows 标准字体目录。版本由 `Cargo.lock` 锁定。
