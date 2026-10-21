# 组件实现

0.2 的组件树与绑定缓存位于 `src/components/tree.rs`，纯布局位于 `src/layout.rs`。表格与图表算法分别位于 `src/components/table.rs` 和 `src/components/chart.rs`，egui 绘制宿主位于 `src/renderer/egui/`。

容器支持纵横布局、网格、尺寸约束、滚动和裁剪。文字、输入、按钮、下拉、指标、徽标和进度采用统一的状态绑定与可见性/禁用规则。

表格使用固定行高与可见行范围，支持稳定排序、拖动列宽、行选择和方向键焦点。图表提供线性坐标轴、折线与分组柱图、图例显隐、悬停、缩放和平移；数据和采样缓存按绑定版本更新。

模块作用域在编译器中解析，布局和绘制接收已解析树。业务宿主通过公开运行时或渲染器使用组件能力，不直接构造内部树。

当前回归在 `src/runtime/tests.rs` 检查状态、作用域和事件，在 `src/renderer/egui/tests.rs` 检查表格、折线图和柱图的绘制及数据刷新，在业务测试中校验生成描述及两种窗口尺寸的无窗口绘制。运行 `cargo test --locked --offline`。完整组件契约见 [描述格式](05-description-format.md)，设计方向见 [组件设计](08-component-engine-design.md)。

当前尚无可变行高、合并单元格、页面路由或可视化编辑器。
