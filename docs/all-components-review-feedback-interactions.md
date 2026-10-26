# 反馈组件交互 PNG 逐图验收

## 范围与方法

- 图片根目录：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-interactions`；下列路径均相对此目录（interactions 根）。
- 实际使用 `read` 分别打开并目视检查 18 张原始 PNG：Popconfirm 12 张、Notification 6 张；未使用联系表、生成报告或文件名代替看图。
- 检查整图文字、裁剪、重叠、弹层内容及通知位置，并在同组图片之间比较操作前后的可见变化。pass/fail 是该图综合视觉验收结论，交互成功但存在遮挡的图仍记 fail。
- 普通滚动区域底部 Skeleton 被视口截断不计错误；dark 宿主内固定浅色页面不计错误。顶部 Message 是浮层，不因其位于页头区域而直接判错。
- 本次仅验收已有图片，未运行 GUI、构建或测试程序；图片所对应的 EXE、构建类型及渲染后端未独立核实。本报告结论限于现有截图可见内容，不能证明系统输入链路或不可见的事件分发。
- 普通与 dark 图片位置约以 1360×900 原图像素表示；scaled 原图经报告与 Pillow 核对为 2380×1575，Read 缩小预览为 2000×1324，下文 scaled 位置采用工具预览坐标。

## 逐图结果

| 相对 interactions 路径 | 结论 | 可见依据与位置观察 |
| --- | --- | --- |
| `popconfirm-interact/opened.png` | pass | 主区右上方触发按钮下方约 x773–1013、y370–438 出现白色确认气泡，显示“确定执行演示操作吗？”及“取消”“确定”；内容完整、非空，未见文字重复或异常遮挡；整图结构完整，底部 Skeleton 为普通 scroll 边界截断。 |
| `popconfirm-interact/confirmed.png` | pass | 与 opened 相比确认气泡消失；顶部中央出现绿色“Message · 操作成功”，底部状态变为“Popconfirm 操作已确认。”，共同提供确认发生的可见依据；未见空白残留弹层、文字重复或异常裁剪。 |
| `popconfirm-interact/reopened.png` | pass | 与 confirmed 相比，触发按钮下方同一位置重新出现完整问题及两个按钮，证明气泡重开；顶部成功 Message 与底部“操作已确认”仍保留，为上一步反馈，未与气泡重叠，未见重复气泡或异常裁剪。 |
| `popconfirm-interact/cancelled.png` | pass | 与 reopened 相比气泡及其按钮已消失，支持取消后收起；顶部成功 Message、底部“操作已确认”沿用之前确认状态，未出现额外确认反馈。截图可验证收起，不能独立区分取消与其他关闭途径；整图未见异常重叠或文字重复。 |
| `popconfirm-interact-scaled/opened.png` | pass | 放大图约 x1137–1490、y545–645 的气泡紧接触发按钮下方，问题、取消和确定均完整可读；没有空白弹层或文字重影，侧栏、主区、页脚完整，Skeleton 仅在滚动视口边界截断。 |
| `popconfirm-interact-scaled/confirmed.png` | pass | 与同组 opened 相比气泡消失，顶部中央显示绿色“Message · 操作成功”，页脚显示“Popconfirm 操作已确认。”；确认后的变化清楚，未见异常裁剪、重复文字或残留空白气泡。 |
| `popconfirm-interact-scaled/reopened.png` | pass | 与 confirmed 相比，约 x1137–1490、y545–645 重新显示问题及取消/确定两个按钮；已有顶部 Message 和页脚确认状态仍在，三者互不遮挡；未见重复弹层、空白或异常裁剪。 |
| `popconfirm-interact-scaled/cancelled.png` | pass | 与 reopened 相比气泡收起，原位置恢复为空白页面区域；顶部 Message 和页脚原确认状态保持，未新增反馈。可见结果支持取消收起，但截图不单独证明关闭事件来源；全图无异常文字重复或重叠。 |
| `popconfirm-interact-dark/opened.png` | pass | 深色外框内浅色页面完整；约 x773–1013、y370–438 气泡显示完整问题与取消/确定按钮，非空且不越出画面；未见文字重复、异常重叠或裁剪，固定浅色不计错误。 |
| `popconfirm-interact-dark/confirmed.png` | pass | 与 opened 相比气泡消失，顶部中央绿色成功 Message 和页脚“Popconfirm 操作已确认。”出现；确认变化可见，深色宿主边框内没有残留空白弹层或文字重复。 |
| `popconfirm-interact-dark/reopened.png` | pass | 与 confirmed 相比，按钮下方再次出现完整确认气泡及两个操作按钮；顶部旧成功提示和页脚旧确认状态保留，未挡住气泡；整图未见异常裁剪或重复文字。 |
| `popconfirm-interact-dark/cancelled.png` | pass | 与 reopened 相比确认气泡消失，顶部旧成功提示、页脚旧确认状态保留，没有新增可见反馈；支持取消后收起，截图本身不独立区分其他关闭途径；未见异常遮挡、空白弹层或文字重复。 |
| `notification-close/opened.png` | fail | 右下角约 x1078–1344、y732–812 显示通知卡片，标题“Notification 通知”、关闭 × 和“所有组件均可在对应分区中体验。”完整，说明通知打开且没有空白/重复文本；但卡片覆盖约 x1272–1320、y771–819 的蓝色“指南”悬浮按钮上部，问号被遮住，仅露出下缘和下方标签，存在浮层位置冲突。 |
| `notification-close/closed.png` | pass | 与 opened 相比右下角通知卡片、标题、正文及 × 全部消失，蓝色“？”指南按钮完整恢复可见，关闭结果清楚；没有通知残影、空白框、文字重复或异常裁剪。 |
| `notification-close-scaled/opened.png` | fail | 右下角约 x1586–1976、y1078–1194 的通知标题、关闭 × 和正文均完整可读，未出画面；但卡片覆盖约 x1871–1941、y1135–1205 的蓝色指南按钮大部分，问号不可见，仅余蓝色下缘及“指南”标签；属于位置重叠，非普通 scroll 截断。 |
| `notification-close-scaled/closed.png` | pass | 与 opened 相比通知完全消失，右下角蓝色问号按钮重新完整显示，支持通知关闭实际产生可见结果；正文区、页脚与侧栏无异常重叠、重复文字或残留空白通知框。 |
| `notification-close-dark/opened.png` | fail | 深色宿主内右下角约 x1078–1344、y732–812 的浅色通知完整显示标题、关闭 × 与正文，非空且无文字重复；但与普通图相同，遮住蓝色指南按钮上部和问号，只露底部蓝色弧边及下方标签，通知位置与固定按钮冲突；固定浅色本身不是问题。 |
| `notification-close-dark/closed.png` | pass | 与 opened 相比通知卡片完全消失，右下角蓝色问号按钮完整可见；关闭状态的可见变化成立，无残留空白框、重复文字或异常裁剪，浅色页面在深色宿主内正常保留。 |

## 统计

| 分组 | 实际查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| popconfirm-interact | 4 | 4 | 0 | 0 |
| popconfirm-interact-scaled | 4 | 4 | 0 | 0 |
| popconfirm-interact-dark | 4 | 4 | 0 | 0 |
| notification-close | 2 | 1 | 1 | 0 |
| notification-close-scaled | 2 | 1 | 1 | 0 |
| notification-close-dark | 2 | 1 | 1 | 0 |
| **总计** | **18** | **15** | **3** | **0** |

## 问题聚类与交互比较

### N1：右下角 Notification 覆盖固定“指南”按钮（3 张 fail）

- 涉及 `notification-close/opened.png`、`notification-close-scaled/opened.png`、`notification-close-dark/opened.png`。
- 三张均有可见的通知卡片与蓝色固定按钮重叠；通知本身文字和关闭按钮完整，但指南问号被挡住，不能同时完整识别两个组件。
- 三组 closed 图中通知消失且指南按钮恢复完整，表明通知关闭的可见结果通过；fail 来自 opened 状态位置冲突。

### Popconfirm 状态链（无 fail）

- 三组均可见：opened 有内容气泡 → confirmed 气泡收起且出现成功 Message 与确认页脚 → reopened 气泡再次出现 → cancelled 气泡再次收起。
- cancelled 中仍可见上一轮确认的成功 Message 和页脚，不能据此判定取消又触发确认；图片没有新增确认的可见证据。取消收起的视觉结果成立，是否确由取消按钮而非其他关闭途径产生，超出静态截图可独立证明的范围。
- 全部 18 图未见异常重复文字或空白弹层；滚动区域底部截断及 dark 宿主中的固定浅色不列为问题。
