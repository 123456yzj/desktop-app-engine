# Tag / Transfer / Chart 原始 PNG 逐图视觉验收

## 范围与方法

- 图片根：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-final`；下表路径均相对该根。
- 实际使用 Read 逐张打开并真实查看 **36 张原始 PNG**：Tag 9、Transfer 15、Chart 12。未使用联系表或已有报告替代看图。
- 方法：观察完整画面，并对照同组前后状态；普通及 dark 原图为 1360×900，scaled PNG 物理尺寸为 **2380×1575**（主代理已通过 Pillow 与 render-report 复核）。Read 将 scaled 图片缩小预览为 2000×1324；下述坐标均为 **工具预览近似坐标，并非原图物理像素坐标**。
- 本次对象是用户提供的既有截图；未启动 GUI、未运行生成程序或测试。修订时检索 `examples/all-components.json` 确认状态与配置上下文：第 41 行 `tagChecked` 初始为 true，第 263 行 Tag 的 checked 绑定该状态；第 227 行 Transfer options 为普通字符串 read/write/admin，无 disabled 配置。EXE 路径、Debug/Release、实际渲染后端与输入修饰键不能由这些截图确认，因此本报告不补充此类产物运行结论。
- `pass`：该图所述可见内容/状态对照通过；`fail`：有明确可见缺陷；`uncertain`：该图对应的关键验收点证据不足。单图 pass 不代表未出现的禁用操作或系统输入也通过。
- dark 中宿主边缘和日志栏为深色、应用主体固定浅色，按要求不算错误；Image 占位也不算失败。
- 全图检查：标题、导航、侧栏、页脚及浮动指南未见明显相互盖字；滚动容器边界处截去上下文内容单独说明，不直接当作布局缺陷。图表内部刻度截字与页面视口裁切作区别记录。

## 逐图记录

| 相对 final 路径 | 结论 | 实际可见依据 / 位置 |
| --- | --- | --- |
| `tag/normal.png` | pass | x462–565、y291–321 有浅蓝底“可选 Tag”和关闭 ×；头像、消息、描述表、占位及二维码分区清楚，无明显重叠。底部 List 的下一行被内容视口 y757 裁切。 |
| `tag/checked.png` | pass | 与 normal 对照，Tag 文字和 × 仍在，底色明显变淡；结合 tagChecked 初始 true 及 checked 绑定，实际是从选中切换为取消选中，不要求文件名 checked 对应 true。下方视口裁切与 normal 相同。 |
| `tag/closed.png` | pass | 原 x462–565 的 Tag 连同 × 消失，头像和消息保留；与前图相比关闭结果可见，表格和图片区未错位覆盖。底部 List 仍是视口裁切。 |
| `tag-scaled/normal.png` | pass | x680–829、y429–474 的蓝底 Tag、文字与 × 完整；放大后的描述表、占位、二维码及侧栏可读，无互相挤压。List 后续行在 y1113 视口边缘被裁。 |
| `tag-scaled/checked.png` | pass | 预览 x680–829 Tag 底色相对 normal 变淡，文字与 × 未变；结合初始 true 和状态绑定，可见由选中切换为取消选中。主体其余区域清楚，底部仍为视口裁切。 |
| `tag-scaled/closed.png` | pass | Tag 所在位置留空，× 同时消失，消息徽标仍在；放大布局无新增重叠，表格和二维码完整。底部 List 后续内容不在视口内。 |
| `tag-dark/normal.png` | pass | x462–565、y291–321 可见蓝底 Tag 和 ×；浅色主体在深色宿主边缘内保持可读，描述表和二维码无明显截字；List 后续行被底部视口裁切。 |
| `tag-dark/checked.png` | pass | Tag 底色由 normal 的浅蓝变成更淡色，文字与 × 保留；结合初始 true 和状态绑定，实际为取消选中，与普通及 scaled 组一致。其余主体无明显重叠。 |
| `tag-dark/closed.png` | pass | Tag 和 × 消失，Ada、99+、未读消息仍可见；关闭前后差异明确，主体浅色不是失败；底部 List 仍只露首行和下一行上缘。 |
| `transfer/normal.png` | pass | x269–750 左框为 write/admin，x814–1295 右框为 read，均空心圆；两枚箭头文字较深，布局完整。配置中三项均为普通字符串，admin 是未选普通项；空选箭头 disabled 不属于本次声明要求。顶部旧控件被滚动视口 y190 裁掉。 |
| `transfer/selected.png` | pass | 左框 write 在约 (286,581) 出现蓝色勾和蓝字，admin 未勾，右框 read 未变；选中变化明确，标题、搜索框及左右框无重叠。 |
| `transfer/moved-right.png` | pass | 左框仅余 admin，右框从 read 增为 read/write，write 无勾；右箭头呈蓝色描边。实际项目归属变化证明右移可见，未选普通项 admin 保留。 |
| `transfer/selected-right.png` | pass | 右框第二行 write 蓝字带勾，read 空心圆，左框仍仅 admin；右侧选择状态清楚，未见截字。 |
| `transfer/moved-left.png` | pass | 左框恢复 write/admin，右框仅 read，write 勾消失；左箭头蓝框。与 selected-right 对比可见 write 返回左侧，项目未重复。 |
| `transfer-scaled/normal.png` | pass | 预览左框 x396–1103 为 write/admin，右框 x1198–1904 为 read，圆圈均空；放大布局清楚。三项配置均为普通字符串，admin 未选，无需显示禁用差异；顶部有视口裁切。 |
| `transfer-scaled/selected.png` | pass | 左侧 write 在约 (421,855) 为蓝勾蓝字，admin 和右侧 read 未勾；放大后行间距、框边及按钮均完整。 |
| `transfer-scaled/moved-right.png` | pass | write 从左框消失并成为右框第二项，左框仅 admin，右框 read/write 均未勾；蓝框右箭头清楚，未见框间重叠。 |
| `transfer-scaled/selected-right.png` | pass | 右侧 write 在约 (1222,908) 蓝勾蓝字，read 未勾，左侧 admin 保留；右项选择有真实可见变化。 |
| `transfer-scaled/moved-left.png` | pass | write 回到左框第一行，admin 为第二行，右侧只余 read；左箭头蓝框，项目归属恢复，文字未截断。 |
| `transfer-dark/normal.png` | pass | 左框 write/admin、右框 read 均空心圆；深色宿主中的浅色双框完整可读。配置均为普通字符串，admin 是未选普通项，无禁用标记不构成问题；顶部旧控件属于滚动边界裁切。 |
| `transfer-dark/selected.png` | pass | 左 write 蓝勾蓝字，admin 和右 read 未选；搜索框、两侧标题与箭头间距正常。 |
| `transfer-dark/moved-right.png` | pass | 左框仅 admin，右框 read/write；write 选择勾清除，右箭头蓝框。项目右移清楚，无重复显示。 |
| `transfer-dark/selected-right.png` | pass | 右 write 第二行变蓝并带勾，read 与左 admin 保持未勾；选中位置与项目名称可辨，无盖字。 |
| `transfer-dark/moved-left.png` | pass | 左框 write/admin、右框仅 read，write 不再带勾；左箭头蓝框。与前图比较可见返回左侧。 |
| `chart/before.png` | pass | 上方表格 Alpha/Beta/Gamma 和 1/2 分页可读；折线图 y568 起可见 100/50 刻度与六个节点的走势。下缘 y757 截掉图表后续区域，不能据此单图检查横轴/柱图。 |
| `chart/zoom.png` | pass | 表格和图标题位置未变，折线纵轴从 100/50 变成 60/40，峰值横向位置由约 x900 移至 x1045，曲线范围明显变化；支持图内缩放结果，Alt 按键本身不可从静态图证明。 |
| `chart/page-scroll.png` | fail | 页面确实下滚：表格离开、滚动条下移，完整六柱出现，首柱 0 和末柱 5 左右均留白。缺陷在折线横轴约 x350–1255、y326–344，多处仅剩“1.”“2.”“3.”及下一行残留笔画；不是视口边缘，刻度不完整。 |
| `chart/lower.png` | pass | 继续下滚后，原生六柱上部在视口顶部被裁，首尾横向边缘仍在；下方 ECharts 完整显示 Layout/Input/Display/Feedback 四柱，首末柱完整、标签清楚，约值 20/18/24/22。 |
| `chart-scaled/before.png` | pass | 放大表格三行、1/2 分页及折线标题完整；折线有 100/50 刻度且可读，底部后续内容在 y1113 被滚动视口裁掉，未见标题盖图。 |
| `chart-scaled/zoom.png` | pass | 表格/标题保持位置，纵轴变为 60/40、峰值移至约 x1537，曲线范围相较 before 放大；缩放图像变化明确，修饰键来源无法由图确认。 |
| `chart-scaled/page-scroll.png` | fail | 表格已滚走、滚动条下降，原生六柱全露，0 与 5 首尾柱不被横向截断。折线横轴约 x517–1843、y481–507 的多个数字仅剩“1.”“2.”“3.”及下行残迹，放大后仍明显截字。 |
| `chart-scaled/lower.png` | pass | ECharts 四柱和 Layout 至 Feedback 标签全部可见，首末柱完整；相比 page-scroll 内容继续上移。插件图字体比原生文字软一些但仍能辨认，无明显重叠；顶部原生柱图是视口裁切。 |
| `chart-dark/before.png` | pass | 浅色主体中的表格、三行数据、分页与折线图清楚；100/50 刻度可见，曲线在视口下缘外还有内容；深色宿主边框不影响读取。 |
| `chart-dark/zoom.png` | pass | 表格位置不动，纵轴由 100/50 改为 60/40，折线峰值移至约 x1045，图内缩放变化与普通组一致；静态图不能直接证明 Alt 键事件。 |
| `chart-dark/page-scroll.png` | fail | 可见下滚后的折线底部、完整原生六柱及插件标题；0/5 首尾柱完整。折线横轴 x350–1255、y326–344 多个数字被截成“1.”“2.”“3.”与下行残留，内部标签区存在可见缺陷。 |
| `chart-dark/lower.png` | pass | 继续滚动后 ECharts 四柱完整，Layout/Input/Display/Feedback 均可读，首末柱没有裁边；原生六柱仅上缘被页面视口截去，左右边界仍完整。 |

## 统计

| 分组 | 真实查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| tag | 3 | 3 | 0 | 0 |
| tag-scaled | 3 | 3 | 0 | 0 |
| tag-dark | 3 | 3 | 0 | 0 |
| transfer | 5 | 5 | 0 | 0 |
| transfer-scaled | 5 | 5 | 0 | 0 |
| transfer-dark | 5 | 5 | 0 | 0 |
| chart | 4 | 3 | 1 | 0 |
| chart-scaled | 4 | 3 | 1 | 0 |
| chart-dark | 4 | 3 | 1 | 0 |
| **合计** | **36** | **33** | **3** | **0** |

## 问题聚类与序列结论

### F1：缩放后的折线横轴数字截字（3 张 fail）

- `chart/page-scroll.png`
- `chart-scaled/page-scroll.png`
- `chart-dark/page-scroll.png`

三张均在图表内部横轴区出现数字残缺/疑似换行后的下半部分被裁；其下方图例仍完整，所以不能解释为页面视口裁切。可见事实足以判 fail；仅凭图片不能定位格式化或布局方面的源码原因。各组 zoom 图的横轴尚在页面视口外，不能用它们断言缺陷不存在。

### 修订说明：Tag 的真实切换为取消选中（3 张改为 pass）

- `tag/checked.png`
- `tag-scaled/checked.png`
- `tag-dark/checked.png`

normal → checked 三组均由较蓝背景变淡，文字和 × 保持。检索 `examples/all-components.json:41` 和 `:263` 确认初始 tagChecked=true 且 checked 绑定该状态，结合已实际查看的前后变化，真实语义为 **选中 → 取消选中**；无需额外勾选图标，也不能凭 checked 文件名要求结果为 true。原 uncertain 属于缺少配置上下文导致的误判，现三张均改为 pass。checked → closed 三组均能直接看到 Tag 消失，关闭结果通过。

### 修订说明：Transfer 普通项初始显示通过（3 张改为 pass）

- `transfer/normal.png`
- `transfer-scaled/normal.png`
- `transfer-dark/normal.png`

检索 `examples/all-components.json:227` 确认 options 为 `["read", "write", "admin"]`，均是普通字符串，无 disabled 配置。三组初始图均完整显示左 write/admin、右 read，空心圆表示未选；admin 与其他普通项字色接近符合该配置。无选择时箭头 disabled 也不是本次声明验收要求，因此不应成为 normal 图片 uncertain 的依据，现三张均改为 pass。**未来 disabled 配置分支仍未覆盖**，此项仅记录覆盖范围，不作为本次已配置场景的问题。

### 已发生的预期变化与无法证实的部分

- Transfer 三组均为：左 write/admin + 右 read → 左 write 蓝勾 → 左仅 admin + 右 read/write → 右 write 蓝勾 → 左 write/admin + 右仅 read。右移和左移均有真实可见项目变化，无预期移动完全未发生的序列。
- Chart 三组 before → zoom：页面位置不变，图内刻度、网格和曲线范围改变；zoom → page-scroll → lower：页面内容与滚动条逐步移动，说明图内缩放与页面滚动有不同的可见结果。是否实际使用 Alt 不能由 PNG 单独验证。
- Chart 三组 page-scroll 的原生柱图均为完整六柱，0 首柱和 5 尾柱有左右留白；lower 的插件柱图均完整显示四类，首尾未横向截断。lower 顶部原生柱图的纵向缺失来自页面滚动视口，前一张已提供完整柱图证据。
- Tag 底部 List、Transfer 顶部旧控件、Chart 前两图下部及 lower 顶部均有页面视口边缘裁切；未将这些与内部横轴截字混为同一问题。
- 未见“所有交互状态图完全不变”的序列；Tag 根据初始状态、绑定及可见样式变化确认取消选中，Transfer 根据普通项配置及可见项目归属确认选中与双向移动。ECharts 点击更新状态栏不在这四张图可证明的交互结果中。

## 总结

36 张原图已全部真实查看；本次修订沿用这些实际看图记录并补充检索配置上下文。结论为 **33 pass / 3 fail / 0 uncertain**：原 6 张 uncertain 均因配置上下文澄清改为 pass，3 张折线横轴截字 fail 保留，整体仍不能判定全项通过。scaled 物理尺寸已更正为 2380×1575，位置说明统一为工具预览坐标。本次仅修改此报告。
