# 架构方向

## 路线

低代码编辑器 → 应用描述 → Rust 引擎 → Windows 应用。

0.1 保留原 GUI 路径。0.2 布局、容器、表格、图表等组件由引擎自研，egui/eframe 保留为窗口、输入、字体与基础绘制宿主；自研范围与实施规则见 [底层组件设计](08-component-engine-design.md)，实际实现和验收见 [实施记录](09-component-implementation.md)。

## 描述契约

- `schemaVersion`：协议版本。
- `app`：名称、窗口及外观。
- `state`：有类型的初始状态。
- `root`：嵌套组件树、布局和属性。
- `actions`：由事件触发的执行步骤。
- 后续扩展连接、命令、数据源、页面与导航。

关联使用稳定 ID 和显式引用。属性区分固定值与绑定值。
表达式只计算数据；外部操作必须通过动作或数据源触发。

## 引擎职责

1. 加载描述，校验版本、类型、ID 和引用。
2. 将组件描述映射为 GUI 控件。
3. 管理状态变化并更新绑定属性。
4. 接收事件，执行动作并显示错误。

## 第一版规则

- 状态先支持字符串。
- 布局先支持行和列，叶子组件支持文本、输入框和按钮。
- 动作先支持状态赋值与文本拼接。
- 动作按顺序执行；触发时读取输入快照，失败后停止后续步骤。
- 未知字段、未知类型、重复 ID 和无效引用必须明确报错。
- 使用系统亮暗主题，避免固定颜色导致不可读。

## 打包

正式选择“固定运行时 + 应用描述包”：一次构建 release 运行时，每次导出只复制运行时、app.json 和 README.txt；编辑界面无需 Rust 编译，客户机无需 Rust、源码或制作工具。
加载顺序为 `--app FILE` > EXE 同目录 `app.json` > 内嵌开发示例。外部描述出错立即返回，不回退；同目录描述以 EXE 路径定位，不依赖工作目录。
内嵌描述编译进 EXE 的方式保留为开发期快捷方式。协议 0.1 无外部资源字段，本次不扩展资源契约。

0.2 路径：loader 版本分派 → v02 严格契约 → component arena → binding resolve → layout measure/arrange → gui_v02 图元和宿主交互 → interaction 有序 commit。table/chart 为无窗口可测纯算法，GUI 适配负责绘制其结果。业务状态、滚动/选择/视口本地状态和派生缓存分开存储；状态键版本限制绑定和数据缓存重建范围。可安装组件从 app.json 旁的 plugins 目录读取 manifest，在后台 Node 进程执行，SVG 经 resvg 后交给 egui 纹理绘制；协议见 [插件机制](15-plugins.md)。
打包入口为 `scripts/package.ps1`，产物自动执行 `--check`；完整比较与边界见 [分发决策](06-packaging-decision.md)。

GUI 框架选择 egui/eframe 0.31.1，固定版本并提交 Cargo.lock。

| 能力 | egui/eframe | Slint |
| --- | --- | --- |
| 动态树 | Rust 递归调用控件，直接映射 JSON | 可运行时解释 .slint，需要生成源码或组件模型 |
| 嵌套布局 | horizontal/vertical 容器 | HorizontalBox/VerticalBox 等声明布局 |
| 中文 | 注册字体字节，Windows 可读取系统微软雅黑 | 系统字体及注册字体能力 |
| Windows | 原生 EXE，glow 或 wgpu 后端 | 原生 EXE，需选择平台后端及渲染器 |

本期选择直接映射更简单的 egui，使用 glow 后端，避免额外描述转换层。
中文从 Windows 标准字体目录读取微软雅黑，不依赖项目字体文件。
参考：egui 官方 custom_font 示例、eframe 0.31.1 NativeOptions 文档、Slint Rust interpreter 文档。
