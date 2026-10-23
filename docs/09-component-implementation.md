# 组件实现

所有组件源码统一位于 [src/components/](../src/components/README.md)；该目录说明列出实际文件位置，本页记录已实现能力。

0.2 的 84 种组件各自封装在 `src/components/egui/widgets/<组件名>.rs`，声明扩展属性、事件、主属性与二级受控映射，实现专属绘制、交互或布局策略。组件树与绑定缓存位于 `src/components/tree.rs`，布局算法位于 `src/components/layout/`，按几何类型、空间分配、尺寸测量和位置排列拆分。表格与图表算法分别位于 `src/components/table.rs` 和 `src/components/chart.rs`，egui 组件绘制宿主位于 `src/components/egui/mod.rs`。

容器支持纵横布局、网格、尺寸约束、滚动和裁剪，以及 Flex、Space、Divider、Layout、Splitter 和 Masonry。文字、输入、按钮、下拉、指标、状态标签和进度采用统一的状态绑定与可见性/禁用规则。

基础表单提供 Checkbox、Radio、Switch、InputNumber、Slider、Form/FormItem。协议节点定义位于 `src/protocol/v02/`，组件输入与表单字段规则位于 `src/components/input_validation.rs`，编译器校验可写绑定和结构。表单提交、错误刷新及原子重置由 `src/runtime/forms.rs` 调度；具体绘制分别位于 `widgets/checkbox.rs`、`radio.rs`、`switch.rs`、`input_number.rs`、`slider.rs`、`form_item.rs`，`form_controls.rs` 复用控件样式、禁用处理与变更收集。

表格使用固定行高与可见行范围，支持稳定排序、拖动列宽、行选择和方向键焦点。图表提供线性坐标轴、折线与分组柱图、图例显隐、悬停、缩放和平移；数据和采样缓存按绑定版本更新。

各组件通过 `component!` 声明契约，`widgets.rs::definition` 穷尽注册全部组件，`catalogue.rs` 定义共享元数据类型。`src/components/contracts.rs` 读取这些声明，负责公共属性规范化、值及跨字段校验；协议层通过内部别名使用，Rust 工具入口为 `v02::util::component_api`。解析器将扩展属性规范化为 Binding，作用域解析与依赖缓存跟踪这些绑定；主属性变更先提交 change，再提交关联业务事件。二级 open/pageSize/openKeys/checkedKeys/expandedKeys/current/checked 按组件文件声明的固定事件映射写回。

组件文件共用 `widgets/shared/` 下的菜单、选择、弹层、对话框、图片、日期等机制。布局组件声明布局策略，测量和排列使用共享算法；LineChart/BarChart 各自绘制系列，复用 `egui/charts.rs` 的坐标轴、图例和视口交互。宿主协调输入本地值、表单提交、滚动和事件队列。模态框通过 egui Modal 阻挡背景，受控弹层通过状态开关，Spin/Watermark/BorderBeam 在子内容之后绘制。ConfigProvider/App 的主题、大小、禁用、语言和方向向下传递，临时解析结果与缓存基础结果分离。Table 的 pageSize 启用排序后分页，current 可受控，pageChange 先更新状态再调用动作。

模块作用域在编译器中解析，布局和绘制接收已解析树。业务宿主通过公开运行时或渲染器使用组件能力，不直接构造内部树。

运行 `cargo test --lib --locked --offline` 检查布局几何、状态更新、非法配置、表单校验/重置及 egui 鼠标/键盘交互。教程还包含亮暗主题、多尺寸与缩放的无窗口绘制检查。构建引擎后运行 `python scripts/check-component-docs.py --schema` 检查完整 JSON 示例、Schema 和文档链接（需安装 Python jsonschema）。实际 Windows 窗口的视觉效果与系统输入仍需人工验收。完整组件契约见 [描述格式](05-description-format.md)、[组件教程](components/README.md) 和 [覆盖对照](21-component-coverage.md)，设计方向见 [组件设计](08-component-engine-design.md)。

当前尚无可变行高、合并单元格、页面路由或可视化编辑器。
