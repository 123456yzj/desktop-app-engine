# 与 Antdv Next 的组件覆盖对照

对照来源：[Flex 页面及其组件目录](https://www.antdv-next.cn/components/flex-cn)，页面版本 v1.7.0，核对日期 2026-10-22。参考目录共 71 项。本引擎使用原生桌面绘制和 JSON 协议，下面按用户用途对照；“基础支持”不表示具有参考库的全部属性、交互与样式。

本次按“布局组件优先”补齐六类组件，并提供 Layout 的四个区域节点。协议 0.2 从 17 种 type 增至 27 种，教程从 20 篇增至 26 篇。其他类别按下方缺口继续扩展。

## 布局组件

| 参考组件 | 对应实现与教程 | 已支持 | 后续差异 |
| --- | --- | --- | --- |
| Divider | [divider](components/23-divider.md) | 横纵线、标题位置、实线/虚线/点线、主题颜色 | 没有 size 预设、DOM 样式接口 |
| Flex | [flex](components/21-flex.md)，兼容 row/column | 横纵方向、六种 justify、四种 align、数值 gap、独立行列间距、横向换行、轨道伸缩、嵌套 | 没有 CSS 单位、baseline、反向换行、纵向换列；DOM component 不适用 |
| Grid | [grid](components/05-grid.md) | fixed/fr 列轨道、独立行列间距、按内容测量行高、剩余高度分配 | 没有 24 栅格 span/offset/order 和响应断点对象 |
| Layout | [layout/header/sider/content/footer](components/24-layout.md) | 页头、侧栏、内容、页脚、自动识别侧栏方向、区域嵌套 | 没有侧栏折叠按钮和 breakpoint；可用 visible 绑定隐藏 |
| Masonry | [masonry](components/26-masonry.md) | 最短列排布、内容高度测量、列数上限、最小列宽响应、动态重排、滚动 | 没有指定 item 所属列、列跨度、itemRender、layoutChange 事件 |
| Space | [space](components/22-space.md) | 默认 8 point 间距、横纵方向、对齐、横向换行、独立行列间距 | 没有自动 separator 和 size 预设；可显式插入 Divider |
| Splitter | [splitter](components/25-splitter.md) | 双面板、横纵方向、拖动、键盘、双击复位、受控比例、min/max、禁用、嵌套多区 | 没有直接多面板数组、折叠按钮、销毁隐藏内容、resizeStart/resizeEnd |

这些组件通过严格协议解析、编译校验、布局测量、原生绘制和事件提交接入。通用尺寸、颜色、visible/enabled 和局部 state/actions 见 [公共属性](components/02-common.md)。

## 其他已有对应能力

| 参考组件 | 本项目对应 | 覆盖范围 |
| --- | --- | --- |
| Button | button | 基础按钮、状态外观、点击动作 |
| Typography | text | 文本、字号、字重、换行与对齐；未提供整套标题/段落/编辑/复制功能 |
| Input | input | 单行字符串编辑与 change/submit |
| Select | select | 单选、动态选项、禁用选项、open/change |
| Card | card/panel | 标题、内容、背景、布局与滚动 |
| Statistic | metric | 指标值、数值/百分比格式 |
| Table | table | 表格、排序、行选择、列宽调整与可见行绘制 |
| Tag | badge | 语义颜色状态标签；当前 badge 名称对应标签用途 |
| Progress | progress | 0..1 进度与说明文字 |

参考站点的 **Badge 徽标数**（数字、圆点、角标）与本项目的 badge 状态标签用途不同，仍列为缺失。原生 Text 仅部分对应 Typography。LineChart、BarChart、Module 和 Plugin 是本项目额外提供的能力。

## 尚未补齐的参考组件

以下 55 项尚无独立对应实现，保留为后续组件任务，不在本次布局优先范围内：

| 类别 | 缺失组件 |
| --- | --- |
| 通用 | FloatButton 悬浮按钮、Icon 图标 |
| 导航 | Anchor 锚点、Breadcrumb 面包屑、Dropdown 下拉菜单、Menu 导航菜单、Pagination 分页、Steps 步骤条、Tabs 标签页 |
| 数据录入 | AutoComplete 自动完成、Cascader 级联选择、Checkbox 多选框、ColorPicker 颜色选择器、DatePicker 日期选择器、Form 表单、InputNumber 数字输入框、Mentions 提及、Radio 单选框、Rate 评分、Slider 滑动输入条、Switch 开关、TimePicker 时间选择器、Transfer 穿梭框、TreeSelect 树选择、Upload 上传 |
| 数据展示 | Avatar 头像、Badge 徽标数、Calendar 日历、Carousel 走马灯、Collapse 折叠面板、Descriptions 描述列表、Empty 空状态、Image 图片、Listy 虚拟列表、Popover 气泡卡片、QRCode 二维码、Segmented 分段控制器、Timeline 时间轴、Tooltip 文字提示、Tour 漫游导览、Tree 树形控件 |
| 反馈 | Alert 警告提示、Drawer 抽屉、Message 全局提示、Modal 对话框、Notification 通知提醒框、Popconfirm 气泡确认框、Result 结果、Skeleton 骨架屏、Spin 加载中、Watermark 水印 |
| 其他 | Affix 固钉、App 应用配置、BorderBeam 边框流光、ConfigProvider 全局化配置 |

这里的 App/ConfigProvider 指参考库的组件接口，本引擎已有的顶层 app、主题和模块状态不算实现这两个组件。基础能力仍需按对应教程中的 API 判断，完整性不以名称相同作为标准。

## 验证

运行 `cargo test --lib --locked --offline` 验证布局几何、内容重排、比例约束、状态更新顺序和 egui 鼠标/键盘输入。构建后运行 `python scripts/check-component-docs.py`，检查所有教程完整 JSON 示例及本地链接。安装 Python jsonschema 后可加 `--schema` 同时校验 [description-0.2.schema.json](description-0.2.schema.json)；实际运行仍需通过引擎 `--check`。
