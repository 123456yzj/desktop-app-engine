# 组件源码目录

协议 0.2 的 84 种组件各有一个独立文件，统一位于 `egui/widgets/`。每个文件声明本组件的扩展属性、允许事件、主属性和二级受控事件映射，并封装专属绘制、交互或布局策略。数据算法、校验和重复的绘制机制在本目录内共享。

## 目录结构

```text
src/components/
├── mod.rs                    # 组件模块入口
├── catalogue.rs              # 组件定义类型、布局策略与注册入口
├── contracts.rs              # 通用属性规范化、类型及值约束校验
├── input_validation.rs       # 输入值与表单字段校验
├── tree.rs                   # 保留组件树、解析属性与绑定依赖
├── table.rs                  # 表格数据、排序、列宽和分页状态
├── chart.rs                  # 图表数据、坐标与采样算法
├── layout/                   # 所有组件的测量、排列、裁剪与滚动
│   ├── mod.rs
│   ├── geometry.rs
│   ├── flex.rs
│   ├── measure.rs
│   ├── arrange.rs
│   ├── masonry.rs
│   ├── splitter.rs
│   └── tests.rs
└── egui/                     # 原生组件绘制、输入和本地状态
    ├── mod.rs                # 帧流程、绑定缓存、滚动、状态及事件协调
    ├── fonts.rs              # 系统中文与粗体字体安装
    ├── form_controls.rs      # 表单控件的共用样式、禁用及变更收集
    ├── charts.rs             # 图表坐标轴、图例、缩放及系列绘制接口
    ├── text.rs               # 文字测量与绘制辅助
    ├── theme.rs              # 主题与控件样式辅助
    ├── widgets.rs            # 共用上下文、本地状态及绘制分派
    └── widgets/
        ├── button.rs        # Button 的契约、绘制与点击交互
        ├── input_number.rs  # InputNumber 的契约与数值编辑
        ├── modal.rs         # Modal 的契约与模态交互
        ├── row.rs           # Row 的契约与横向布局策略
        ├── …                # 共 84 个组件文件，完整对应表见下文
        └── shared/          # 菜单、弹层、对话框、图片、日期等共用机制
```

`widgets.rs::definition` 对全部 `Kind` 穷尽分派；新增组件时必须添加独立文件并注册。组件通过 `component!` 声明契约，`contracts.rs` 读取这些声明进行规范化和事件属性查询。公共协议字段仍由 `protocol` 定义，专属值校验和跨字段规则由共享校验模块处理。

`protocol/classes.rs` 使用 `hypen-tailwind-parse` 将静态 JSON `class` 严格映射到原生 style/layout，在应用默认值前合并显式字段；未知类与不支持属性报错。测量、排列与绘制消费相同的最终节点。支持范围见 [公共属性教程](../../docs/components/02-common.md#json-class-工具类)，可运行 [utility-classes.json](../../examples/utility-classes.json)。

布局组件在各自文件选择 `LayoutKind`，共同使用 `layout/` 的测量和排列算法。LineChart、BarChart 分别实现系列绘制，共用坐标轴、图例和交互框架。相近控件也共用机制，例如 Modal/Drawer 的对话框、List/Listy 的虚拟列表、Image/Avatar 的图片加载；组件文件保留各自配置和差异行为。

## 组件定位

| 组件类别 | 独立实现文件（均相对于 `egui/widgets/`） |
| --- | --- |
| 行列与空间布局 | [row.rs](egui/widgets/row.rs)、[column.rs](egui/widgets/column.rs)、[grid.rs](egui/widgets/grid.rs)、[flex.rs](egui/widgets/flex.rs)、[space.rs](egui/widgets/space.rs)、[divider.rs](egui/widgets/divider.rs)、[splitter.rs](egui/widgets/splitter.rs)、[masonry.rs](egui/widgets/masonry.rs) |
| 页面与容器 | [layout.rs](egui/widgets/layout.rs)、[header.rs](egui/widgets/header.rs)、[sider.rs](egui/widgets/sider.rs)、[content.rs](egui/widgets/content.rs)、[footer.rs](egui/widgets/footer.rs)、[panel.rs](egui/widgets/panel.rs)、[card.rs](egui/widgets/card.rs)、[module.rs](egui/widgets/module.rs) |
| 基础组件 | [text.rs](egui/widgets/text.rs)、[button.rs](egui/widgets/button.rs)、[input.rs](egui/widgets/input.rs)、[select.rs](egui/widgets/select.rs)、[metric.rs](egui/widgets/metric.rs)、[badge.rs](egui/widgets/badge.rs)、[progress.rs](egui/widgets/progress.rs)、[plugin.rs](egui/widgets/plugin.rs) |
| 表单控件 | [checkbox.rs](egui/widgets/checkbox.rs)、[radio.rs](egui/widgets/radio.rs)、[switch.rs](egui/widgets/switch.rs)、[input_number.rs](egui/widgets/input_number.rs)、[slider.rs](egui/widgets/slider.rs)、[form.rs](egui/widgets/form.rs)、[form_item.rs](egui/widgets/form_item.rs) |
| 表格与图表 | [table.rs](egui/widgets/table.rs)、[line_chart.rs](egui/widgets/line_chart.rs)、[bar_chart.rs](egui/widgets/bar_chart.rs) |
| 导航与内容切换 | [anchor.rs](egui/widgets/anchor.rs)、[breadcrumb.rs](egui/widgets/breadcrumb.rs)、[dropdown.rs](egui/widgets/dropdown.rs)、[menu.rs](egui/widgets/menu.rs)、[pagination.rs](egui/widgets/pagination.rs)、[steps.rs](egui/widgets/steps.rs)、[tabs.rs](egui/widgets/tabs.rs)、[segmented.rs](egui/widgets/segmented.rs)、[collapse.rs](egui/widgets/collapse.rs) |
| 扩展录入与树 | [auto_complete.rs](egui/widgets/auto_complete.rs)、[cascader.rs](egui/widgets/cascader.rs)、[color_picker.rs](egui/widgets/color_picker.rs)、[date_picker.rs](egui/widgets/date_picker.rs)、[mentions.rs](egui/widgets/mentions.rs)、[rate.rs](egui/widgets/rate.rs)、[time_picker.rs](egui/widgets/time_picker.rs)、[transfer.rs](egui/widgets/transfer.rs)、[tree_select.rs](egui/widgets/tree_select.rs)、[upload.rs](egui/widgets/upload.rs)、[calendar.rs](egui/widgets/calendar.rs)、[tree.rs](egui/widgets/tree.rs) |
| 图片、图标与列表 | [float_button.rs](egui/widgets/float_button.rs)、[icon.rs](egui/widgets/icon.rs)、[avatar.rs](egui/widgets/avatar.rs)、[badge_count.rs](egui/widgets/badge_count.rs)、[image.rs](egui/widgets/image.rs)、[qr_code.rs](egui/widgets/qr_code.rs)、[descriptions.rs](egui/widgets/descriptions.rs)、[list.rs](egui/widgets/list.rs)、[listy.rs](egui/widgets/listy.rs)、[timeline.rs](egui/widgets/timeline.rs)、[carousel.rs](egui/widgets/carousel.rs) |
| 反馈与装饰 | [empty.rs](egui/widgets/empty.rs)、[alert.rs](egui/widgets/alert.rs)、[result.rs](egui/widgets/result.rs)、[skeleton.rs](egui/widgets/skeleton.rs)、[spin.rs](egui/widgets/spin.rs)、[border_beam.rs](egui/widgets/border_beam.rs)、[watermark.rs](egui/widgets/watermark.rs) |
| 弹窗、气泡与导览 | [modal.rs](egui/widgets/modal.rs)、[drawer.rs](egui/widgets/drawer.rs)、[message.rs](egui/widgets/message.rs)、[notification.rs](egui/widgets/notification.rs)、[popover.rs](egui/widgets/popover.rs)、[tooltip.rs](egui/widgets/tooltip.rs)、[popconfirm.rs](egui/widgets/popconfirm.rs)、[tour.rs](egui/widgets/tour.rs) |
| 配置与定位 | [config_provider.rs](egui/widgets/config_provider.rs)、[app.rs](egui/widgets/app.rs)、[affix.rs](egui/widgets/affix.rs) |

## 与引擎其他模块的衔接

- `protocol` 定义应用描述与协议节点，引用本目录的组件契约和输入校验规则。
- `compiler` 处理作用域、引用、插件注册与完整语义校验。
- `runtime` 负责通用状态提交、有序事件、动作与表单提交调度。
- `renderer` 导出组件宿主接口，并提供运行日志界面；公共 `layout` 入口导出本目录的布局实现。
- `services` 提供命令、网络、插件等外部能力，业务宿主通过引擎公开接口使用组件。

`catalogue` 的元数据类型不包含 egui 类型，注册入口当前转发到 `egui/widgets.rs`；它是内部静态注册表。布局中的 Affix 定位也调用对应组件文件。宿主仍协调 Input、Select、Form 等控件的本地状态、表单提交与事件收集，共享校验仍负责具体值约束。

组件使用 API 和可运行示例直接维护在 [各组件教程](../../docs/components/README.md)；实际能力和限制见 [覆盖对照](../../docs/21-component-coverage.md)。

## 验证

必须遵循 [项目测试规范](../../docs/testing-policy.md)，禁止新增或恢复仅注入 egui 输入并检查绘制数据或内部 UI 状态的界面测试。

现有单元测试覆盖协议与 class 解析、状态运行时、输入校验和纯布局算法。组件教程可通过 `python scripts/check-component-docs.py --schema` 校验描述及 Schema，完整工程检查见 [项目 README](../../README.md)。

已接入可选 `visual-tests` feature 的打包 EXE 离屏入口，以及 `scripts/test-packaged-classes.py` 的实际 PNG 像素检查和显式基准图比较，运行方法见 [测试规范](../../docs/testing-policy.md#打包产物离屏入口)。当前覆盖 utility-classes 示例的明暗主题、缩放和交互，使用 wgpu；真实 Glow 窗口的隔离自动化尚未接入。单元测试与描述校验通过不代表视觉效果通过。
