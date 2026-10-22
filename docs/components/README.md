# 组件总览

本教程面向使用 JSON 描述构建桌面应用的开发者，适用于协议 **0.2**。按“何时使用 → 代码演示 → API → 常见问题”组织，每个组件单独查阅。

教程使用 `01`～`26` 两位编号，文件名、页面标题和下方目录保持一致。`21`～`26` 是补充的布局组件教程；当前共有 27 种 type（含 Layout 的四种区域节点）。与参考站点的覆盖情况见 [组件对照](../21-component-coverage.md)。

## 开始使用

1. 阅读 [01 快速开始](01-quick-start.md)，运行第一个带输入与按钮的界面。
2. 阅读 [02 公共属性与布局](02-common.md)，理解绑定、尺寸、样式、显示与禁用。
3. 从下面选择组件，将示例保存为 JSON 后运行。
4. 组合页面时参考 [20 常见布局与组合](20-recipes.md)。

教程的 `json` 代码块均为完整应用，可直接保存为文件运行；示例中的修改建议采用行内配置写法。插件示例另需安装配套资源。各页说明预期效果，实际界面由原生引擎运行显示。

## 布局与容器

| 编号 | 组件 | type | 何时使用 |
| --- | --- | --- | --- |
| 03 | [Row 横向容器](03-row.md) | `row` | 工具栏、横向排列、比例分栏、自动换行 |
| 04 | [Column 纵向容器](04-column.md) | `column` | 页面主体、纵向表单、固定区与剩余空间分配 |
| 05 | [Grid 网格](05-grid.md) | `grid` | 等宽卡片、多列指标、固定列与弹性列 |
| 06 | [Panel 面板](06-panel.md) | `panel` | 带标题的区域、可滚动内容区 |
| 07 | [Card 卡片](07-card.md) | `card` | 聚合指标、说明和一组操作 |
| 08 | [Module 模块](08-module.md) | `module` | 封装局部状态、动作与父子通信 |
| 21 | [Flex 弹性布局](21-flex.md) | `flex` | 可选方向、六种主轴对齐、独立行列间距与换行 |
| 22 | [Space 间距](22-space.md) | `space` | 默认 8 point 间距、居中对齐的操作组 |
| 23 | [Divider 分割线](23-divider.md) | `divider` | 水平/纵向、标题、实线/虚线/点线 |
| 24 | [Layout 页面布局](24-layout.md) | `layout`、`header`、`sider`、`content`、`footer` | 页头、侧栏、内容、页脚及剩余空间分配 |
| 25 | [Splitter 面板分割](25-splitter.md) | `splitter` | 拖动、键盘调整、比例绑定与双面板约束 |
| 26 | [Masonry 瀑布流](26-masonry.md) | `masonry` | 不等高卡片、最短列排布与自适应列数 |

## 基础组件

| 编号 | 组件 | type | 何时使用 |
| --- | --- | --- | --- |
| 09 | [Text 文本](09-text.md) | `text` | 标题、说明、动态文字、等宽日志 |
| 10 | [Button 按钮](10-button.md) | `button` | 触发状态更新或业务动作 |

## 数据录入

| 编号 | 组件 | type | 何时使用 |
| --- | --- | --- | --- |
| 11 | [Input 输入框](11-input.md) | `input` | 编辑单行字符串、输入后联动或提交 |
| 12 | [Select 下拉选择](12-select.md) | `select` | 单选字符串值、动态选项、禁用部分选项 |

## 数据展示与反馈

| 编号 | 组件 | type | 何时使用 |
| --- | --- | --- | --- |
| 13 | [Metric 指标](13-metric.md) | `metric` | 显示数量、数值、百分比 |
| 14 | [Badge 状态标签](14-badge.md) | `badge` | 显示状态文字与语义颜色 |
| 15 | [Progress 进度条](15-progress.md) | `progress` | 显示 0 到 1 的完成进度 |
| 16 | [Table 表格](16-table.md) | `table` | 对象数组、排序、行选择与列宽调整 |
| 17 | [LineChart 折线图](17-line-chart.md) | `lineChart` | 展示数值趋势、断点、多系列与缩放 |
| 18 | [BarChart 柱状图](18-bar-chart.md) | `barChart` | 比较数值分组、多个系列 |

## 扩展

| 编号 | 组件 | type | 何时使用 |
| --- | --- | --- | --- |
| 19 | [Plugin 插件组件](19-plugin.md) | `plugin` | 使用已安装的 SVG 插件，例如 ECharts 适配器 |

## 如何选择容器

| 需求 | 选择 |
| --- | --- |
| 一行放置多个控件 | Row，配合 `gap` 和 `align` |
| 控件从上往下排列 | Column |
| 窄窗口下换行 | Row 的 `wrap: true` |
| 多列卡片按行排列 | Grid 的 `columns` |
| 一组内容需要标题与背景 | Panel 或 Card；两者当前布局与默认外观规则相同 |
| 内容超过区域后滚动 | 容器的 `overflow: "scroll"`，同时限制区域高度 |
| 一组控件需要独立状态与事件接口 | Module，内部再组合布局容器 |
| 一种容器配置横向或纵向，比较主轴分布 | Flex |
| 一组按钮需要默认间距与居中对齐 | Space |
| 段落或行内操作之间需要分隔线 | Divider |
| 页头、侧栏、主体与页脚 | Layout 及其区域节点 |
| 用户拖动调整分栏尺寸 | Splitter；多区域可嵌套 |
| 不同高度卡片紧凑排布 | Masonry |

## 进一步阅读

- [完整协议参考](../05-description-format.md) 与 [JSON Schema](../description-0.2.schema.json)
- [执行外部命令](../10-command-binding.md)、[HTTP 数据源](../16-http-data-sources.md)
- [模块作用域](../12-scoped-state.md)、[模块通信](../14-module-communication.md)
- [打包与分发](../06-packaging-decision.md)

维护教程时可运行 [示例检查脚本](../../scripts/check-component-docs.py)，批量执行引擎 `--check` 并检查本教程的本地文档链接；方法见 [01 快速开始](01-quick-start.md)。
