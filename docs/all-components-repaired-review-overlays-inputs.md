# 修复后弹层与录入组件逐图视觉验收（最终批）

## 对象与方法

- 最终批图片根目录：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final`。下表路径均相对此目录。
- 指定的 **30/30 张原始 PNG 均逐张使用 read 打开并查看全图**，读取失败 0 张；未用 sheet 代替原图。
- 依据 `docs/testing-policy.md`，参考旧报告 `docs/all-components-review-overlays-inputs.md` 复核通知碰撞与日期文字对比问题。
- 本次为已有截图静态视觉复核，没有重新启动产物或执行交互。已读取/检索最终批 `notification{,-scaled,-dark}/render-report.json` 及 `date-dark/render-report.json` 的相关记录核对来源与位置：Debug，definition 为 embedded，产物为 `C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe`，渲染器为 `egui_kittest 0.31.1 / wgpu` 离屏，普通通知记录为 Dx12 / Microsoft Basic Render Driver。来源信息来自已有记录，不是本次重新运行所得。
- 普通组及 scaled 组记录为 light 宿主；`-dark` 仅表示 dark 宿主，页面与示例仍保留固定浅色，不能据此宣称组件全面深色适配通过。scaled 记录为 1.75 pixels_per_point、2380×1575，read 显示缩小预览；普通及 dark 为 1360×900。缩放截图不构成实际显示器 DPI 验证。
- 结论仅覆盖这些图片呈现的 Debug embedded wgpu 离屏视觉状态；不声称 Release、Glow、原生窗口、系统输入或实际 DPI 通过，不根据 opened/settled 文件名推断关闭、提交或计时行为。
- `pass`：本图未发现明确视觉缺陷；`fail`：存在可见缺陷；`uncertain`：无法读取或无法据图判断。正常滚动视口裁剪、下拉覆盖背景表单不单独判错。

## 统计

| 类别 | 已查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| Notification | 6 | 6 | 0 | 0 |
| Modal | 6 | 6 | 0 | 0 |
| Popconfirm | 6 | 6 | 0 | 0 |
| Inputs | 3 | 3 | 0 | 0 |
| Cascader | 3 | 3 | 0 | 0 |
| Date | 3 | 3 | 0 | 0 |
| Autocomplete | 3 | 3 | 0 | 0 |
| **合计** | **30** | **30** | **0** | **0** |

## 逐图记录

| 相对路径 | 结论 | 具体观察 |
| --- | --- | --- |
| `notification/opened.png` | pass | 右下通知标题、关闭按钮及正文完整；卡片底边约 y763，与约 y771 起的指南圆钮分离，顶部圆弧完整。灰色阴影延伸至间隙附近，未见遮住圆钮轮廓、问号或指南标签。 |
| `notification/settled.png` | pass | 通知边框及正文下留白完整，卡片与指南圆钮保持可见间隔；圆钮顶端未再削平，阴影未造成可见遮挡，问号及标签清楚。 |
| `notification-scaled/opened.png` | pass | 放大预览中通知标题、正文与关闭按钮清楚；卡片底边约 y1122、圆钮顶约 y1134，主体间隙可辨。阴影在间隙附近渐淡，蓝色顶部圆弧连续，问号和标签完整（坐标为 read 预览近似值）。 |
| `notification-scaled/settled.png` | pass | 放大通知文字无裁剪，卡片底边与指南圆钮分离；圆形顶部完整，无此前横切轮廓现象，阴影未遮住按钮轮廓或文字。 |
| `notification-dark/opened.png` | pass | dark 宿主中的浅色通知内容完整；卡片与指南圆钮有间隔，顶部圆弧和问号清楚，通知下方软阴影未形成可见按钮遮挡。 |
| `notification-dark/settled.png` | pass | 通知标题、正文和关闭按钮可读，卡片底边与指南圆钮顶部保持分离；阴影邻近间隙但未削平圆弧，问号及指南标签完整。 |
| `modal/opened.png` | pass | 白色编辑资料框居中，标题、共用姓名状态说明和 Ada 输入完整；关闭按钮与标题分离，取消/蓝色确定右对齐，遮罩与阴影层次清楚，无内部重叠。 |
| `modal/settled.png` | pass | 仍显示完整 Ada 编辑框，说明未换行截断；输入到操作行间距合理，取消与确定均未贴边，背景处于遮罩下，没有穿透弹框。 |
| `modal-scaled/opened.png` | pass | 放大弹框标题、说明、Ada 和关闭按钮清楚，输入左右内边距足够；底部两按钮完整且互不挤压，四周留有空间。 |
| `modal-scaled/settled.png` | pass | 放大后的主体边界、阴影完整，说明一行可读，长输入未溢出；取消与蓝色确定保持主次和间隔，无裁剪。 |
| `modal-dark/opened.png` | pass | dark 宿主外围可见，中央浅色弹框与灰色遮罩区分明确；标题、Ada、说明和两项操作均可读，关闭按钮无叠压。 |
| `modal-dark/settled.png` | pass | 固定浅色编辑框仍居中，正文和输入完整；底部按钮文字居中，蓝色确定突出，未见低对比、越界或背景内容穿透。 |
| `popconfirm/opened.png` | pass | 白色确认浮层紧接触发器下方，与其左右对齐；问句完整，取消和确定分列在下一行，边框与阴影完整，未碰旁边 Alert。 |
| `popconfirm/settled.png` | pass | 问句及取消/确定均清楚，按钮之间有间隔；浮层高度紧凑、底部有留白，未见文字重叠或越出容器。 |
| `popconfirm-scaled/opened.png` | pass | 放大确认问句完整，按钮行与问句分开；框宽匹配触发器，左右及底边完整，无异常空白高柱或裁字。 |
| `popconfirm-scaled/settled.png` | pass | 放大浮层的取消、确定和问句均完整，边框阴影连续；浮层位于内容区内部，未与相邻提示叠压。 |
| `popconfirm-dark/opened.png` | pass | dark 宿主中浅色确认层的深色问句与按钮文字可读，触发器蓝色状态清楚；浮层未遮住旁边 Alert，边缘完整。 |
| `popconfirm-dark/settled.png` | pass | 浅色浮层仍紧贴触发器，问句与两按钮完整，行间距清楚；未见内容越界、混色或其他控件穿透。 |
| `inputs/scrolled.png` | pass | Radio、启用开关、Slider 与 Rust 选择框清楚；姓名/地区双列对齐，日期、12:30:00、此刻、清除及 #1677ff 完整，无跨列重叠；底部欢迎输入框下半部为滚动视口正常裁剪。 |
| `inputs-scaled/scrolled.png` | pass | 放大后选择状态、分组标题及双列表单清楚，日期与时间各按钮间隔可见，色块与色值分离；底部文本框在视口处截取，未见控件内部截字。 |
| `inputs-dark/scrolled.png` | pass | dark 宿主下固定浅色表单保持可读，桌面选项、开关、Rust、日期时间和色值均完整；双列无挤压，欢迎文本框下部停在滚动区边界。 |
| `cascader/opened.png` | pass | 地区框蓝色展开态明确，白色浮层紧接下方并左右对齐；空搜索框和折叠的亚洲根节点完整，浮层紧凑，无多余纵向空白，覆盖下方时间行属正常下拉层叠。 |
| `cascader-scaled/opened.png` | pass | 放大搜索框、三角箭头和亚洲文字清楚，浮层宽度匹配地区框；底边与阴影完整，没有空白高柱或内容裁剪，当前仅显示折叠根节点。 |
| `cascader-dark/opened.png` | pass | dark 宿主中浅色级联层文字对比正常，搜索框及亚洲行完整；展开输入与浮层对齐，当前高度符合两行内容，未见背景穿透。 |
| `date/opened.png` | pass | 浅色日历年/月、前后箭头、七列星期与六周网格完整；当月深色数字、跨月灰字及蓝色选中 22 可区分，今天按钮完整。底边贴近画面下缘，未见明确文字裁剪。 |
| `date-scaled/opened.png` | pass | 放大日历年/月与网格清楚，各日期无叠字，选中 22 高亮明显；今天按钮文字完整，日历贴近画面底边但未见明确内容缺失。 |
| `date-dark/opened.png` | pass | 白色 popup 上星期及当月未选中日期已呈深色，跨月日期为较浅灰色；年/月、箭头、选中 22 和今天按钮均清楚，旧报告近白文字低对比问题在本图中已修复（D1）。底边近画面边缘，文字完整。 |
| `autocomplete/opened.png` | pass | 输入蓝色焦点边框清楚，列表 Ada、Alan、Grace 三行完整且分隔明确；浮层轻微左内缩/右外伸但仍邻接输入，未侵入地区列，覆盖下方表单属正常下拉。 |
| `autocomplete-scaled/opened.png` | pass | 放大联想列表三项清楚，无空白行或叠字；外框相对输入略有横向偏差，仍未进入地区列，底边与阴影完整，无明显脱离输入。 |
| `autocomplete-dark/opened.png` | pass | dark 宿主下浅色列表使用深色 Ada/Alan/Grace 文字，焦点框清楚；三行完整，轻微左右外框差未造成跨列或裁字，浮层紧接输入。 |

## 问题与修复复核

### N1：通知与指南圆钮碰撞已修复（最终批 6 张 pass）

最终批六张 notification 原图均显示卡片与指南圆钮分离、蓝色顶部圆弧完整，问号与指南标签清楚；通知软阴影在间隙附近可见，但未见其遮去按钮轮廓或文字，不将正常软阴影本身判为碰撞。三组 render-report 的 opened/settled 均记录 notification 底边 763、指南区域顶边 771，主体间距为 8 逻辑点；该数值用于辅助核对，视觉通过依据仍是逐张原图。未执行逐像素阴影覆盖检测，未据此判断按钮点击能力。

历史：`all-components-repaired-captures-1` 批通知六图曾因卡片底边仍切入指南圆钮顶部判为 fail，最终批复核均转为 pass。

### D1：date-dark 浅色 popup 文字对比已修复

最终批 `date-dark/opened.png` 中星期、当月未选日期保持可读深色，跨月灰字、蓝色选中态与年/月及今天控件均清楚，文字对比修复保持通过。此结论仅针对 dark 宿主里的当前浅色 popup 截图，不扩展为全面深色主题或动态日期操作验收。

## 证据边界

- 日期三图均接近画面下边缘，但网格与今天按钮文字完整；本次未发现明确裁字。
- 联想三图轻微外框横向差异保留为观察，未见错列或侵入右列的明确后果。
- 级联三图仅覆盖折叠亚洲根节点；Modal、Popconfirm 的 opened/settled 图仅覆盖可见展开状态。
- 最终批未发现明确视觉问题，30 张均为 pass；逐图记录中的正常裁剪、轻微外框差异不另计缺陷。
- 未执行键盘、点击、滚动、关闭、提交、子级展开或动态计时验证；未进行图片快照自动差异比较。本次仅更新本报告文件。
