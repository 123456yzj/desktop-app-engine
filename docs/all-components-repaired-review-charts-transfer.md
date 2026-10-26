# 修复后 Chart / Transfer / Tag 最终批原始 PNG 逐图视觉验收

## 范围、证据与方法

- 图片根：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final`。下表路径全部相对此根。**本报告现为最终批结论，取代此前 `all-components-repaired-captures-1` 批次在本报告中的结论。**
- 按九组 `render-report.json` 的 captures 清单，对最终批重新使用 **Read 逐张打开并查看全部 36 张原始 PNG**：Chart 12、Transfer 15、Tag 9。未凭文件字节相同跳过图片，未使用 sheet 替代原图；按同组交互顺序对照状态。下表每行均对应最终批原图的复核结论。
- 参考 `docs/testing-policy.md` 和旧报告 `docs/all-components-review-charts-transfer.md`。本次结论来自新图片，旧报告只用于确定回归重点。
- 九组 render-report 均记录：Debug；EXE 为 `C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe`；`definition=embedded`；`egui_kittest 0.31.1 / wgpu (offscreen; not Glow)`；Dx12，Microsoft Basic Render Driver，Cpu。artifact.json 记录 EXE SHA-256 为 `7a962d8e3d9128a2d788cf219b229933ae6f12bd3479c31b594fc7f4fd650ad5`。这些是既有采集记录，本次未重新启动 EXE 或执行测试。
- 普通和 dark 原图为 **1360×900 / 1.0 pixels_per_point**；scaled 原图为 **2380×1575 / 1.75 pixels_per_point**，逻辑尺寸均为 1360×900。Read 对 scaled 显示约 2000×1324 的缩小预览，不能把预览软化当作原图尺寸或截字；下文不以预览坐标冒充物理像素坐标。
- 普通/scaled 为 light，dark 为 dark 宿主；主体固定浅色、宿主边缘和日志栏深色，不作为错误。Image 明示占位，不作为加载失败。
- 配置上下文检索：`examples/all-components.json:41` 的 `tagChecked` 初始为 true，`:263` 的 `checkableTag.checked` 绑定该状态；因此 normal → checked 实际为取消选择。`:227` 的 Transfer options 是普通字符串 read/write/admin，无 disabled 项配置。
- `pass` 表示该图可见布局及结合序列的目标状态通过；`fail` 表示明确可见缺陷；`uncertain` 表示关键证据不足。视口边缘随滚动裁去内容与组件内部截字分别判断。

## 逐图记录

| 原始 PNG 相对路径 | 结论 | 观察 |
| --- | --- | --- |
| `chart/before.png` | pass | 表格 Alpha/Beta/Gamma、负载和在线值、1/2 分页完整；折线标题及 100/50/0 纵轴可读，走势清晰；横轴及后续内容在页面下方视口外，未据此判内部截字。 |
| `chart/zoom.png` | pass | 相对 before 表格与标题位置保持，纵轴改为 60/40、曲线峰值右移，图内缩放结果明确；底部曲线与横轴仍受页面视口裁切。 |
| `chart/page-scroll.png` | pass | 表格已滚走；折线横轴 1.4、1.6、1.8、2、2.2、2.4、2.6、2.8、3、3.2、3.4 均为完整单行，未见长浮点尾数、换行残迹或截断；请求量图例完整。原生六柱及 0–5 标签完整，首尾留白；下方插件内容在视口底边裁切。 |
| `chart/lower.png` | pass | 页面进一步下滚；ECharts 四柱及 Layout/Input/Display/Feedback、0–25 刻度完整可读，首尾柱未横向截断；原生柱图上部在顶部视口外，与上一图的完整证据一致。 |
| `chart-scaled/before.png` | pass | 放大布局的三行表格、分页、折线标题及 100/50/0 刻度清楚，无盖字；折线后续区域在视口下方。 |
| `chart-scaled/zoom.png` | pass | 表格位置不变，纵轴变为 60/40、峰值向右移动，与 before 有明确缩放差异；底部曲线出视口不属于内部标签缺陷。 |
| `chart-scaled/page-scroll.png` | pass | 缩小预览仍可辨认完整单行 1.4–3.4 横轴短刻度，整数 2/3 无浮点长尾；未见旧版下行残迹或截字，图例分离。六柱和 0–5 标签全露、首尾留白，插件下部受页面视口裁切。 |
| `chart-scaled/lower.png` | pass | ECharts 标题、四类标签及四柱完整，首末柱有余量；插件文字在预览中比原生文字软，但可辨认且无截断。原生柱图上部属于页面滚动裁切。 |
| `chart-dark/before.png` | pass | 深色宿主内浅色表格、分页、折线标题及 100/50/0 刻度清楚，文字无明显互盖；横轴尚在下方视口外。 |
| `chart-dark/zoom.png` | pass | 纵轴 60/40、峰值右移，表格及标题位置稳定，图内缩放可见；底部裁切与页面视口边界一致。 |
| `chart-dark/page-scroll.png` | pass | 折线横轴 1.4–3.4 短刻度完整单行，2/3 为整数，无长浮点尾数、换行或残字；请求量图例完整。六柱、0–5 标签及首尾左右余量清楚，插件下部在视口外。 |
| `chart-dark/lower.png` | pass | 四个插件柱及 Layout/Input/Display/Feedback 完整，纵轴与标题可读；顶部原生柱图因页面继续滚动而裁去上部，没有首尾横向裁边。 |
| `transfer/normal.png` | pass | 左可选权限为 write/admin，右已选权限为 read，三项均空心圆；搜索框、双框标题及两枚箭头完整，未重叠。上方旧输入控件被滚动视口顶边裁去。 |
| `transfer/selected.png` | pass | 左 write 变为蓝字蓝勾，admin 和右 read 未选；双框、搜索框与箭头间距正常，选中变化可辨。 |
| `transfer/moved-right.png` | pass | write 从左框消失并出现于右框第二行；左仅 admin，右 read/write 均未勾，右箭头蓝框；右移发生且无重复项。 |
| `transfer/selected-right.png` | pass | 右框第二行 write 蓝字带勾，read 与左 admin 保持未选，右侧选择结果清楚且文字完整。 |
| `transfer/moved-left.png` | pass | write 回到左框第一行、admin 第二行，右框仅 read；勾选清除，左箭头蓝框，左移恢复归属且无重复项。 |
| `transfer-scaled/normal.png` | pass | 放大后左 write/admin、右 read 均完整未选；搜索框与左右标题清楚，箭头居于两框之间，无挤压。顶部内容为滚动视口裁切。 |
| `transfer-scaled/selected.png` | pass | 左 write 蓝字蓝勾，其他项仍空心圆；放大后的勾选与行间距清楚，无截字。 |
| `transfer-scaled/moved-right.png` | pass | 左仅 admin，右为 read/write，write 未勾；右箭头蓝框，右移后的项目数量和归属变化明确。 |
| `transfer-scaled/selected-right.png` | pass | 右第二行 write 蓝字带勾，read 未选、左 admin 保留；选择发生在正确侧，框与按钮完整。 |
| `transfer-scaled/moved-left.png` | pass | 左恢复 write/admin，右仅 read，write 勾清除，左箭头蓝框；双向往返序列成立，未见重复或文字裁切。 |
| `transfer-dark/normal.png` | pass | 深色宿主中的浅色双框完整，左 write/admin、右 read 均空心圆；搜索、标题、箭头清楚。顶部旧控件在视口边缘被裁。 |
| `transfer-dark/selected.png` | pass | 左 write 蓝字带勾，admin 和右 read 不变，选中差异清楚；双框无重叠。 |
| `transfer-dark/moved-right.png` | pass | 左仅 admin，右 read/write，write 勾选清除，右箭头蓝框；项目实际右移且未重复。 |
| `transfer-dark/selected-right.png` | pass | 右 write 第二行蓝字带勾，read 未勾、左 admin 保留，右侧选中可见且文字完整。 |
| `transfer-dark/moved-left.png` | pass | 左 write/admin、右仅 read，write 不再勾选，左箭头蓝框；对照前图确认左移成功，归属恢复。 |
| `tag/normal.png` | pass | 初始选中 Tag 为较明显浅蓝底，“可选 Tag”和 × 完整，与头像、99+、未读消息分离；描述表和二维码完整。底部 List 后续文字在视口边缘被裁。 |
| `tag/checked.png` | pass | 相对 normal，Tag 底色明显变淡，文字与 × 保留；结合初始 true，实际为取消选择，不能按文件名判作选中。其余布局稳定，底部 List 仍为视口裁切。 |
| `tag/closed.png` | pass | 原 Tag 区域的文字、底色与 × 全部消失，头像和消息保留；关闭结果明确，表格和图片区没有新增错位。 |
| `tag-scaled/normal.png` | pass | 放大后的蓝底 Tag、文字、× 均完整，与消息徽标分离；描述表及二维码清楚，无互盖；底部 List 后续内容在页面视口外。 |
| `tag-scaled/checked.png` | pass | Tag 底色由 normal 的浅蓝变为更淡色，文字及 × 未消失；这是初始 true 后的取消选择，放大布局保持完整。 |
| `tag-scaled/closed.png` | pass | Tag 连同关闭 × 消失，Ada、99+ 和未读消息继续显示；描述表、图片占位和二维码位置稳定，无新增重叠。 |
| `tag-dark/normal.png` | pass | 深色宿主内蓝底 Tag 与 × 完整可读，头像和消息间距正常，主体浅色配色一致；表格、二维码完整，List 后续行在底部视口外。 |
| `tag-dark/checked.png` | pass | 相对 normal 的 Tag 底色变淡、文字与 × 保留，取消选择变化可见；结合初始 true 判定，未凭 checked 文件名反推状态。 |
| `tag-dark/closed.png` | pass | Tag 的底色、文字与 × 均不再出现，头像、徽标、描述表及图片区保留，关闭状态明确且布局稳定。 |

## 统计

| 分组 | Read 原图数 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| chart | 4 | 4 | 0 | 0 |
| chart-scaled | 4 | 4 | 0 | 0 |
| chart-dark | 4 | 4 | 0 | 0 |
| transfer | 5 | 5 | 0 | 0 |
| transfer-scaled | 5 | 5 | 0 | 0 |
| transfer-dark | 5 | 5 | 0 | 0 |
| tag | 3 | 3 | 0 | 0 |
| tag-scaled | 3 | 3 | 0 | 0 |
| tag-dark | 3 | 3 | 0 | 0 |
| **合计** | **36** | **36** | **0** | **0** |

## 回归重点与问题结论

1. **Chart 旧 F1 在本次三张 page-scroll 原图中均通过**：横轴为 1.4、1.6、1.8、2、2.2、2.4、2.6、2.8、3、3.2、3.4 的短刻度，完整单行，无长浮点尾数或上下行残迹。旧报告的三张 fail 不延用至本批图片。
2. **Transfer 三组双向移动均可见**：左 write/admin + 右 read → 左 write 蓝勾 → 左 admin + 右 read/write → 右 write 蓝勾 → 左 write/admin + 右 read。两次移动均改变实际项目归属，勾选在移动后清除，未见项目重复或丢失。
3. **Tag 三组序列均为选中 → 取消选择 → 关闭**：normal 浅蓝底，checked 底色变淡但文字与 × 保留，closed 整个 Tag 消失。初始 true 与绑定上下文支持此语义。
4. **本负责范围新增明确视觉问题 0 项，遗留 fail 0 张，uncertain 0 张**。页面视口边缘的 List、旧输入控件及图表上下部分裁切已在逐图记录说明；Chart 内部横轴无旧截字现象。

## 结论边界

本次完成对最终批既有 Debug 内置定义产物离屏图片的人工视觉复核，36/36 张通过所列可见验收点。未进行逐像素基准快照比较、重新采集、Release 验收或 Glow 原生窗口/系统输入/真实显示器 DPI 测试；静态 PNG 不独立证明 Alt 按键事件本身。ECharts 点击后状态栏更新、Transfer 搜索和禁用分支、Tag 再次选回等未出现于本组序列的行为不在本结论覆盖内；其他组件的 toast 修复也不计入本负责范围的已验证项。
