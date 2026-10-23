# 与 Ant Design 官方组件目录的覆盖对照

对照来源：[Ant Design 组件总览](https://ant-design.antgroup.com/components/overview)，核对日期 2026-10-22，页面版本 6.6.5。目录按 **73 项**统计，包含已弃用的 List 以及工具条目 Util。本引擎以 Rust/egui 原生桌面行为适配，ReactNode 对应组件树，回调对应事件动作，受控值对应 state Binding。

协议 0.2 由原 34 种 type 扩充到 **84 种**，新增 50 种组件；官网原有 22 类对应能力和新增 50 类均有独立或等效入口。Util 提供 Rust 检查函数，不计入 JSON type。名称覆盖不代表官网全部属性、样式、回调和方法都已经实现；下表及教程的“当前限制”是验收范围的一部分。

## 新增组件对应

| 分类 | 官网组件 → JSON type | 教程 |
| --- | --- | --- |
| 通用 | FloatButton→floatButton，Icon→icon | [展示](components/35-native-display.md) |
| 导航 | Anchor→anchor，Breadcrumb→breadcrumb，Dropdown→dropdown，Menu→menu，Pagination→pagination，Steps→steps，Tabs→tabs | [导航](components/33-navigation.md) |
| 数据录入 | AutoComplete→autoComplete，Cascader→cascader，ColorPicker→colorPicker，DatePicker→datePicker，Mentions→mentions，Rate→rate，TimePicker→timePicker，Transfer→transfer，TreeSelect→treeSelect，Upload→upload | [录入与树](components/34-native-inputs.md) |
| 数据展示 | Avatar→avatar，Badge→badgeCount，Calendar→calendar，Carousel→carousel，Collapse→collapse，Descriptions→descriptions，Empty→empty，Image→image，List→list，Listy→listy，Popover→popover，QRCode→qrCode，Segmented→segmented，Timeline→timeline，Tooltip→tooltip，Tour→tour，Tree→tree | [展示](components/35-native-display.md)、[导航](components/33-navigation.md)、[录入](components/34-native-inputs.md)、[反馈](components/36-native-feedback.md) |
| 反馈 | Alert→alert，Drawer→drawer，Message→message，Modal→modal，Notification→notification，Popconfirm→popconfirm，Result→result，Skeleton→skeleton，Spin→spin，Watermark→watermark | [反馈](components/36-native-feedback.md)、[配置与装饰](components/37-native-context.md) |
| 其他 | Affix→affix，App→app，BorderBeam→borderBeam，ConfigProvider→configProvider | [配置与装饰](components/37-native-context.md) |
| 工具 | Util→`v02::util::component_api(Kind)`，无 JSON 节点 | [桌面工具](components/37-native-context.md) |

`badge` 为既有状态标签，对应 **Tag**；`badgeCount` 对应官网 **Badge 徽标数**。`text` 对应 Typography，`metric` 对应 Statistic。顶层应用元信息 `app` 与 `type: "app"` 配置容器是不同字段。

## 已有能力与本轮扩展

| 官网组件 | 对应实现 | 本轮新增或保留的实际行为 |
| --- | --- | --- |
| Button | button | loading、图标、外形、variant、danger、block、href |
| Typography | text | 编辑、复制、截断、代码/粗体/斜体/删除线等 |
| Input | input | 清除、密码、多行、长度、只读、前后缀、计数、状态、焦点事件 |
| Select | select | 单选/多选/tags、搜索、清除、loading、maxCount |
| Checkbox | checkbox | 单项 boolean 或 options 组的字符串数组 |
| Radio | radio | 普通/按钮组选项、键盘方向选择 |
| Switch | switch | loading、开启/关闭文字 |
| InputNumber | inputNumber | 数字约束、精度、只读、隐藏按钮、前后缀 |
| Slider | slider | 单值/双滑块、反向、marks、鼠标与键盘步进 |
| Form | form/formItem | 统一提交/重置、数组必填、Rust regex pattern、新输入控件 |
| Card | card/panel | loading、边框、悬停、extra 字符串 |
| Statistic | metric | 前后缀、loading、数值/百分比格式 |
| Table | table | 全数据稳定排序、分页、虚拟绘制、行选择、列宽调整 |
| Tag | badge | 可关闭、可勾选、受控 checked、图标 |
| Progress | progress | 百分比、line/circle/dashboard、状态、线宽与文字 |
| Divider、Flex、Grid、Layout、Masonry、Space、Splitter | 同名 type；Layout 含 header/sider/content/footer | 原有几何、约束、滚动、分栏交互；Space 新增 separator |

完整演示、字段说明和组件限制直接维护在 [组件总览](components/README.md) 对应的各组件教程中。LineChart、BarChart、Module、Plugin、Row、Column 是引擎的额外能力。

## 尚未完成的 API 与桌面差异

| 范围 | 当前限制 |
| --- | --- |
| 共通接口 | 没有 React ref、hooks、DOM portal、CSS classNames/styles 或任意 JS render/callback；使用组件树、布局 Token、事件动作和状态绑定 |
| 导航 | Anchor 尚未自动滚动；Menu 为内嵌子菜单；Steps 的状态/百分比/进度点、Tabs 部分样式/居中/滚动事件、Pagination align、Segmented block 尚未完整实现 |
| 内容切换 | Collapse 头部集中排列，bordered/ghost 未完整实现；Carousel 尚无滑动动画或触摸手势 |
| 自动完成与树 | 补全键盘/backfill、默认激活、树选择全展开与 showLine、异步加载、字段名映射、完整父子禁用语义仍有缺口 |
| 日期时间 | 尚无 RangePicker、时区和禁用日期回调；部分 showNow/fullscreen 选项未生效，时间 step 目前控制拖动速度 |
| Upload | 仅系统选择器、文件记录和移除/预览/拒绝事件；无实际上传、上传进度、取消、拖放，accept 暂按后缀 |
| Image/Icon | cover 当前与 contain 等效；预览无组/旋转/下载；图标为内置符号集，未覆盖全部官网图标 |
| 列表与描述 | 固定行高虚拟列表，无任意行子树/可变行高；Descriptions 边框使用行底纹，无单元格合并 |
| 提示与导览 | 无消息队列、静态 Promise API 或 useApp；同位置消息可重叠；Tour 未高亮目标；气泡触发子树交互尚需完善 |
| 配置与装饰 | 国际化只覆盖部分文案，RTL 基础支持；Affix 尚非完整滚动监听与 change；App 暂为配置作用域 |
| 原有组件 | Grid 无 24 栅格 span/断点；Splitter 直接双面板；Button dashed 使用普通边框；Form 无异步 validator/Form.List；Table 无筛选、合并和展开行 |

属性被解析和校验只表示存在协议契约；未生效项在对应教程标注，不能据此视为官网 API 兼容完成。Rust 宿主可通过 `component_api` 查询扩展属性的准确枚举和范围；Schema 负责结构形状，绑定解析、成员关系和跨字段约束仍以引擎校验为准。

## 验证

`cargo test --lib --locked --offline` 覆盖几何、状态原子更新、数组与日期校验、主次受控属性、事件顺序、模态背景拦截、键盘与鼠标交互。50 个新增组件另有亮/暗主题、三组视口与缩放比例的无输入绘制检查；通过绘制检查不代表全部交互和 API 已验收。

构建后运行 `python scripts/check-component-docs.py --schema` 检查所有教程完整 JSON、Schema 和本地链接。真实 Windows 窗口、系统文件对话框、图片网络加载与视觉效果仍需人工验收。
