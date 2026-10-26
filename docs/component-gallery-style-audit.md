# 组件实验室逐项样式检查记录

本记录区分页面检查、具体交互复验与未测范围。截图生成成功本身不代表样式通过。

## 当前复验：audit-v10

- 实际被测产物：`C:/Users/yzj123/AppData/Local/Temp/opencode/component-gallery-audit-v10/component-gallery.exe`，Debug，包含 `visual-tests`，从 EXE 内置定义加载。
- EXE SHA-256：`23d1c7f51176e7acde842781693bc9ceab5a773ad4c5098b91f4a31bc37217db`。
- 全量证据：`C:/Users/yzj123/AppData/Local/Temp/opencode/gallery-audit-v10`。233 个重放场景完成，最终 `failed` 为空；另有六类 corrected 场景和 FormItem 状态补测完成。84 页各有顶部、中部、底部 PNG，共 252 张基础页图片，覆盖 173 个展示示例。
- 基础页检查使用已查看的 v9 图片与 v10 的 RGB 比较；69 页完全相同，其余 15 页的 v10 联系表已逐页查看。评分差异为半星修复；加载指示器、骨架和边框流光有动画差异；长页中部存在滚动停留位置差异，顶部和底部用于补足内容检查。原始图保留供复核。
- 深度交互包含树选择/级联展开及单选叶项关闭、Tour 三步完成、自动完成与提及过滤选择、按钮焦点、输入清除、Radio 禁用、Slider 拖动与键盘、Transfer 双向/单向移动、Form/FormItem 必填校验、Modal 保存取消、Drawer、Popconfirm、Tabs、Carousel、Table、Dropdown、Collapse。
- corrected 图片已确认：半星初始 3.5→清零→再次选择 3.5，只读评分不变；Tree 两组展开/收起、第一组勾选和两组选择 Ada；Calendar 切月与选日、受限月份的日期不能提交；Steps、Segmented、Menu 两组均切换选中项。
- 补充场景已看图确认：Alert 两组关闭、Splitter 两组拖到 70%、InputNumber 整数和 0.5 步进、ColorPicker 改色与外点关闭、TimePicker 的 PM→AM、DatePicker 两组选择并收起、Pagination 第三页/每页 20 条/简洁分页前进。补充证据分别在 `gallery-v10-supplement`、`gallery-v10-picker-corrected`。
- Message 成功/警告提示与 Notification 成功/警告通知的自动消失已通过持续绘制后的 PNG 确认，Notification 显式关闭也已看图；最终证据为 `gallery-v10-toast-timed`（分别 6、8 张 capture）。先前仅等待墙钟时间的场景未使提示消失，由此最终场景替代。
- 16 个同版本 focused 场景完成：1.75 倍缩放与深色宿主各覆盖 button/form/select/treeSelect/cascader/tour/autoComplete/mentions。沿用已查看的 v9 图片并比较；仅放大 Button 的加载指示器存在局部差异，v10 图片已查看。显式浅色页面不构成完整深色主题设计验收。
- 最新 `cargo test --locked --features visual-tests`：39 项逻辑测试通过；`cargo clippy --all-targets --features visual-tests --locked -- -D warnings`、`cargo fmt --check` 通过。上述检查不替代视觉验收。

通用状态场景中的 Tab 可能把焦点移到引擎底部工具栏，Enter 随后打开日志窗口。此类图片只保留为实际行为记录，不能证明示例按钮的键盘激活通过；按钮焦点以深度场景中实际可见的外圈为依据。控件矩形只用于定位，不作为验收断言。

首次 v10 重放因临时盘空间不足中断，保留了日志；清理本轮隔离运行时缓存后从完整 PNG/report 续跑。旧 Form 必填示例定位失败已用滚动到底部的场景重测。脚本现于进程结束后清理每个场景的独立缓存，并原子写入进度 JSON。失败与被替代场景不用于计算通过数。

以下 v1/v2 对象记录保留历史线索；最终交付以 v10 的 EXE 哈希和逐组件证据索引为准。

## 对象与证据

- audit-v1：实际打包 Debug EXE，SHA-256 `d95583d65c9b2f1bb0933f33fc5fd4bf00bdb5ee23922774ba44952a776c92be`。
- v1 证据目录：`C:/Users/yzj123/AppData/Local/Temp/opencode/gallery-style-audit-v1`。
- 全部 84 页已生成顶部、中部、底部共 252 张 PNG；对应联系表已逐页查看。部分首屏检查来自本轮前序工作。
- 页面证据：`pages/<组件类型>/{top,middle,end}.png`；联系表 `review.png`。
- 状态证据：`states/<组件类型>Usage<编号>/`；每个目录保留输入场景、原始图片及渲染报告。
- v2 证据目录：`C:/Users/yzj123/AppData/Local/Temp/opencode/gallery-style-audit-v2`，包含修复后的新 EXE 回归，检查中。
- 后端为 egui_kittest / wgpu、Dx12 CPU，1440×960、浅色、1 倍缩放。普通窗口使用 Glow，当前记录不覆盖原生窗口、系统输入或物理 DPI。

## 已发现问题

| 编号 | 问题与图片线索 | 当前处理 | 复验 |
| --- | --- | --- | --- |
| S01 | 用户截图中的文字按钮灰底、示例卡片分隔线过重 | link 透明背景，示例分隔线显式浅色；统一组件外观 | v10 Button 与各示例分隔已看图 |
| S02 | `button` 加载按钮只显示部分“处理中” | 测量宽度加入指示器空间 | v10 普通/1.75 倍完整显示文字与指示器 |
| S03 | `pagination` 完整分页区域不足 | 分页示例改为整行展示 | v10 页码、条数选择和简洁分页已看图 |
| S04 | `switch` 滑块为深色，未选轨道与页面难以区分 | 白色滑块、灰色未选轨道 | v10 开关状态图已查看 |
| S05 | `switch` 示例声明 loading，但没有指示器，仍可切换 | loading 绘制指示器并禁止点击 | v10 loading 指示器可见，点击后值保持 |
| S06 | `badgeCount` 数量标记和标签文字距离过大 | 文字在标记后靠起始位置对齐 | v10 与已查看 v9 图片一致 |
| S07 | `badge` 标签铺满卡片宽度 | 示例指定紧凑宽高，保留关闭区域 | 已查看选中、取消和关闭图，v10 对应图片一致 |
| S08 | 日期弹层使用默认控件外观，与组件内日历不一致 | 日历内容入口应用统一组件外观 | v10 两组日期弹层、选值收起已看图 |
| S09 | 文字按钮缺少可见键盘焦点 | 有焦点时绘制外圈 | v10 点击焦点与 Tab 切到危险按钮的外圈可见 |
| S10 | 树选择/级联选择点击分支后弹层关闭，展开子项被过短视口裁剪 | 分支交互保留弹层，内容最小高度 180 | v9 深度图已确认展开子项完整显示 |
| S11 | 树选择/级联选择单选叶项后弹层仍展开 | 单选提交显式关闭 popup | v9 `select-leaf.png` 已确认显示所选值并收起 |
| S12 | 评分为 3.5 时第四颗星仍整颗填色 | 按每颗星的小数部分裁剪覆盖颜色 | v10 初始、清零、重选半星及只读 PNG 已查看 |

## 逐组件检查清单

下表保留逐组件检查范围；“已看”包含历史基础图检查及上文 v10 全量复验。状态栏是检查关注点，不代表每一分支都已穷举通过。交付目录另附按组件列出的 v10 原图、状态/深度场景索引和具体限制。

| 组件 | 基础页 | 状态检查与关注点 |
| --- | --- | --- |
| button | 已看，S01/S02/S09 | 悬停、按下、键盘焦点、禁用、加载、危险按钮 |
| floatButton | 已看 | 原始全屏图、圆形/方形、悬停、点击 |
| icon | 已看 | 完成图标、旋转图标 |
| text | 已看 | 可编辑文字输入和焦点 |
| divider | 已看，S01 | 普通分隔、标题分隔 |
| flex | 已看 | 横向、纵向布局 |
| grid | 已看 | 两种列布局 |
| layout | 已看 | 宽侧栏与标准布局 |
| header | 已看 | 标题区域与布局关系 |
| sider | 已看 | 侧栏与布局关系 |
| content | 已看 | 内容区域与布局关系 |
| footer | 已看 | 底部区域与布局关系 |
| masonry | 已看 | 不同高度条目排列 |
| space | 已看 | 间距与方向 |
| splitter | 已看 | 分隔条拖动，两个示例 |
| row | 已看 | 两种横向排列 |
| column | 已看 | 两种纵向排列 |
| anchor | 已看 | 水平/垂直，点击反馈；当前无自动滚动 |
| breadcrumb | 已看 | 两种分隔符、点击 |
| dropdown | 已看 | 点击/悬停菜单展开，菜单项选择 |
| menu | 已看 | 水平/分组，选中与禁用项 |
| pagination | 已看，S03 | 页码、跳页、条数、简洁分页 |
| steps | 已看 | 水平/垂直，切换步骤 |
| tabs | 已看 | 顶部/侧边，切换面板 |
| autoComplete | 已看 | 候选展开、过滤、无匹配、选择 |
| cascader | 已看 | 展开到叶节点、单选/多选 |
| checkbox | 已看 | 单个/组，选中/取消 |
| colorPicker | 已看 | 调色板、预设色、修改值 |
| datePicker | 已看，S08 | 弹层、日期选择、范围、格式 |
| form | 已看 | 清空必填、错误提示、提交 |
| formItem | 已看 | 标签对齐、错误提示、提交 |
| input | 已看 | 编辑、清空、密码、多行字数 |
| inputNumber | 已看 | 编辑、步进、精度、范围 |
| mentions | 已看 | 两种前缀、候选选择 |
| radio | 已看 | 普通/按钮式，切换 |
| rate | 已看 | 半星选择、只读 |
| select | 已看 | 单选、多选、标签、禁用、键盘 |
| slider | 已看 | 单值/范围，拖动与键盘 |
| switch | 已看，S04/S05 | 开/关、加载禁止修改 |
| timePicker | 已看 | 数字编辑、12 小时切换、清空 |
| transfer | 已看 | 勾选条目、双向/单向移动 |
| treeSelect | 已看 | 展开、单选/多选 |
| upload | 已看 | 悬停；系统文件对话框未测 |
| avatar | 已看 | 圆形与方形文字头像 |
| badgeCount | 已看，S06 | 数量上限、状态圆点 |
| badge | 已看，S07 | 选中/取消、关闭 |
| calendar | 已看 | 日期选择、月份切换、受限日期 |
| card | 已看 | 正常内容、加载占位 |
| panel | 已看 | 正常内容、加载占位 |
| carousel | 已看 | 箭头、指示点、第二张内容 |
| collapse | 已看 | 展开、收起 |
| descriptions | 已看 | 双列、单列；需确认长标签留白 |
| empty | 已看 | 两种空状态文字 |
| image | 已看 | 缺少来源、失败回退；有效图片预览未测 |
| list | 已看 | 正常条目、加载 |
| listy | 已看 | 内部滚动，底部半行裁剪属于滚动视口 |
| popover | 已看 | 点击/悬停，弹层内容与关闭 |
| qrCode | 已看 | 正常码、过期遮罩；扫码解码未测 |
| segmented | 已看 | 水平/垂直，切换选中 |
| metric | 已看 | 整数、百分比前后缀 |
| table | 已看 | 排序、选行、分页、空数据 |
| timeline | 已看 | 正序、倒序、等待项 |
| tooltip | 已看 | 提示显示、移出关闭 |
| tour | 已看 | 每个步骤、前后切换、关闭；当前无目标高亮 |
| tree | 已看 | 展开、选择、勾选 |
| alert | 已看 | 成功/警告、关闭 |
| drawer | 已看 | 左/右展开、关闭、Escape |
| message | 已看 | 成功/警告、自动关闭 |
| modal | 已看 | 两种宽度、表单编辑、保存/取消 |
| notification | 已看 | 成功/警告、关闭 |
| popconfirm | 已看 | 展开、确认/取消 |
| progress | 已看 | 线性、圆环、仪表盘 |
| result | 已看 | 成功与 404 |
| skeleton | 已看 | 头像正文与简洁正文 |
| spin | 已看 | 开关加载，遮罩与内部禁用 |
| watermark | 已看 | 两种重复水印，文字可读性 |
| affix | 已看 | 点击；完整滚动吸附尚未实现 |
| app | 已看，受 S04 影响 | 作用域正常/统一禁用 |
| borderBeam | 已看 | 两种动画时间与边框 |
| configProvider | 已看，受 S04 影响 | 作用域正常/统一禁用 |
| module | 已看 | 局部输入、发送到父级 |
| lineChart | 已看 | 悬停、缩放、平移、隐藏图例 |
| barChart | 已看 | 悬停、隐藏图例 |
| plugin | 已看 | 实际 ECharts 加载、柱/折线、点击反馈 |

## 验收范围与保留项

页面与上述已确认交互的结论限于实际 Debug EXE 的内置定义、egui_kittest/wgpu/Dx12 CPU 离屏渲染。普通窗口使用 Glow；原生窗口隔离自动化、系统输入、物理 DPI、Release 构建未验收。未穷举 173 个示例的全部输入组合、主题和缩放。

已知展示能力限制：Anchor 没有自动滚动，Affix 完整滚动吸附尚未实现，Tour 没有目标高亮。Upload 的系统文件对话框、有效 Image 来源及预览、QRCode 扫码解码未验收。多选树/级联的全部分支、所有键盘焦点路径、图表完整缩放平移边界也未穷举。这些项目不写为通过。

图片内容滚出视口后的裁剪属于滚动结果；判断长内容是否完整须结合顶部、中部和底部。浮动按钮、左右抽屉和顶部提示使用全屏原图核实，右侧内容裁剪联系表不足以覆盖这些区域。
