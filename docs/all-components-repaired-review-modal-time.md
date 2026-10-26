# 最终批 Modal 与时间编辑：逐张原始 PNG 视觉验收

## 对象与方法

- 验收日期：2026-10-26。最终批图片根目录：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final/`；下表路径均相对此目录。**本报告最终批结论取代 `all-components-repaired-captures-1` 的上一批结论。**
- 对最终根目录六组场景，重新使用 `read` 逐张打开全部 **36 张原始 PNG**（Modal 24 张、时间 12 张），均成功显示；未凭文件字节相同跳过任何图，未用 sheet 或旧批图片代替。逐张检查目标字段、焦点、弹窗/遮罩、按钮、文字、toast 边界以及整页截字、重叠和异常空白，并比较同组序列；下表均为本次最终批复核记录。
- `artifact.json` 与六份 `render-report.json` 指向 `C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe`，**Debug / embedded**。artifact 记录 SHA-256 为 `7a962d8e3d9128a2d788cf219b229933ae6f12bd3479c31b594fc7f4fd650ad5`（本次未重新计算）。六组均记录 `egui_kittest 0.31.1 / wgpu (offscreen; not Glow)`、Dx12 / Cpu；普通 Modal 报告记录适配器 Microsoft Basic Render Driver，驱动 `10.0.26100.9711`。图片页头为“组件实验室”，显示 84 种组件 / 6 个分区。
- 普通组 light、1x、1360×900；scaled 组 light、1.75x、2380×1575；dark 组 dark、1x、1360×900；逻辑尺寸均为 1360×900。scaled 的 `read` 预览为 2000×1324，不将预览尺寸误作原图尺寸。dark 中固定浅色页面与控件符合本轮约定。
- 本轮已重新读取最终根目录六份对应场景 JSON，并核对六份 render-report 的产物、构建、内置定义、尺寸、主题及缩放记录。三组时间均全选后仅输入 `text:"5"`，随后 Enter，再点击外部 `[1150,470]`；三组 Modal 均输入 `Grace`，确定 `[902,513]`、取消 `[843,513]`、右上 × `[913,387]` 为不同逻辑坐标，均为按下后释放。计划只辅助识别动作，结论仍以实际图片字段和序列为依据。
- 参考旧报告 `docs/all-components-review-modal-time.md` 与 `docs/testing-policy.md`。旧报告的直接输入 `05` 不能证明补零；本轮独立核实的是单字符 `5 → 05`。
- `pass` 表示本图目标状态及序列可见结果符合要求，未见明确视觉缺陷；`fail` 表示可确认错误；`uncertain` 表示要求内证据不足。本次为已有产物图片审阅，未重新启动 EXE 或执行构建/测试。

## 逐图记录（36 张，每图一行）

| 原始 PNG 相对路径 | 状态 | 具体观察 |
| --- | --- | --- |
| `modal-interact/opened.png` | pass | 中央白色“Modal 编辑资料”弹窗完整，输入为 `Ada`，说明、右上 ×、取消和确定均清楚；灰遮罩覆盖背景，未见弹窗截字或按钮重叠。 |
| `modal-interact/focused.png` | pass | `Ada` 出现选中底色和末尾光标，输入框蓝边，聚焦可见；标题、说明与底部按钮完整，整页未见异常错位。 |
| `modal-interact/edited.png` | pass | 同一输入框由 `Ada` 改为 `Grace`，蓝边及末尾光标可见；文字未溢出，弹窗边界和按钮保持完整。 |
| `modal-interact/confirmed.png` | pass | 弹窗与遮罩消失，顶部中央新增白底绿字“Message · 操作成功”；消息框紧凑，文字完整且留有内边距，无越界或异常放大。结合下一张重开 `Grace` 支持确定结果；正文 Result、Skeleton 与侧栏清楚。 |
| `modal-interact/reopened.png` | pass | 弹窗重新出现，输入实际显示 `Grace`，证明确定后重开保留编辑值；说明与关闭/取消/确定按钮完整。旧成功 toast 在顶部遮罩后变暗，框与文字完整且不与弹窗重叠。 |
| `modal-interact/cancelled.png` | pass | 对照前图与取消坐标 `[843,513]`，取消后弹窗、遮罩均消失，背景恢复；顶部成功 toast 保持紧凑白框、完整绿字，沿用前次确定，未当作本次取消的新增反馈。 |
| `modal-interact/reopened-again.png` | pass | 取消后再次打开，完整弹窗与遮罩恢复，字段仍为 `Grace`；× 与两按钮可辨。顶部旧 toast 在遮罩后变暗，边界完整；无明确截字、异常空白或非预期重叠。 |
| `modal-interact/closed.png` | pass | 对照前图右上 × 与 `[913,387]` 输入，最终弹窗和遮罩消失，× 关闭结果成立；顶部旧 toast 白框与绿字完整、大小正常，原页面及底部区域正常可辨。 |
| `modal-interact-scaled/opened.png` | pass | 1.75x 弹窗完整居中，`Ada`、说明、右上 × 和取消/确定均在边界内，遮罩正常；无明确截字或挤压。 |
| `modal-interact-scaled/focused.png` | pass | 放大图中 `Ada` 选中底色、光标及蓝色焦点框清晰；标题与说明未受焦点框影响，按钮间距正常。 |
| `modal-interact-scaled/edited.png` | pass | 放大后的实际字段为 `Grace`，光标位于末尾，编辑状态蓝边完整；说明和按钮无溢出、非预期重叠。 |
| `modal-interact-scaled/confirmed.png` | pass | 放大图中弹窗及遮罩消失，顶部中央出现白底绿字成功 toast；框随缩放完整，四周有留白，文字未截断、越界或撑出异常大框。下一张 `Grace` 保留支持确定成功，正文及 Skeleton 无明确布局异常。 |
| `modal-interact-scaled/reopened.png` | pass | 重开弹窗完整，输入 `Grace` 保留；顶部旧成功 toast 位于遮罩后变暗，框及文字完整，与弹窗分离；弹窗标题、说明、× 与底部按钮清晰。 |
| `modal-interact-scaled/cancelled.png` | pass | 取消 `[843,513]` 后弹窗、遮罩消失，与前图取消按钮位置及后续重开相符；顶部沿用 toast 为紧凑白框、完整绿字，按钮、Result、侧栏恢复，无明确截字或异常重叠。 |
| `modal-interact-scaled/reopened-again.png` | pass | 再次出现完整放大弹窗及 `Grace`；右上 × 与右下取消/确定布局完整。顶部沿用 toast 在遮罩后边界完整，未见异常空白或文本越界。 |
| `modal-interact-scaled/closed.png` | pass | × `[913,387]` 后弹窗与遮罩消失；顶部旧 toast 白框、绿字及内边距完整，无异常放大。放大页面可辨，Skeleton 下缘为滚动视口裁切，未见明确视觉缺陷。 |
| `modal-interact-dark/opened.png` | pass | 深色宿主外沿内保留固定浅色页面，中央白色弹窗与灰遮罩完整，字段 `Ada`；×、说明及两按钮清楚。 |
| `modal-interact-dark/focused.png` | pass | `Ada` 选中、末尾光标、输入蓝边均可见；白色弹窗与遮罩背景层次清楚，未见截字或异常重叠。 |
| `modal-interact-dark/edited.png` | pass | 输入实际变成 `Grace`，光标在末尾，蓝框完整；弹窗说明及按钮未挤压，整页无明确布局异常。 |
| `modal-interact-dark/confirmed.png` | pass | 弹窗与遮罩消失，顶部中央新成功 toast 以白底绿字完整显示；框紧凑、内边距可见，未见截字或异常放大。下一张重开 `Grace` 支持确定结果，浅色正文与黑色宿主边界正常。 |
| `modal-interact-dark/reopened.png` | pass | 弹窗再次打开且字段为 `Grace`；顶部旧 toast 在遮罩后呈深灰底深绿字，边界与文字完整且不与弹窗重叠。前景标题、输入、× 和底部按钮清晰；遮罩下旧消息不作为前景可读性验收对象。 |
| `modal-interact-dark/cancelled.png` | pass | 取消坐标 `[843,513]` 后遮罩和弹窗消失，顶部旧成功 toast 恢复白底绿字，文字、边框及内边距完整，正文正常；与后续重开共同支持取消关闭路径。 |
| `modal-interact-dark/reopened-again.png` | pass | 取消后再次完整打开，字段仍为 `Grace`；顶部旧 toast 在背景呈深灰底、文字变暗但边界完整，与弹窗分离。弹窗文字和按钮没有明确截断或异常重叠。 |
| `modal-interact-dark/closed.png` | pass | 右上 × 坐标 `[913,387]` 后弹窗与遮罩消失，固定浅色页面恢复；旧 toast 恢复紧凑白框与完整绿字，侧栏、Result、Skeleton 和页脚可辨，无明确异常。 |
| `time-edit/focused.png` | pass | 时间行实际为 `12 : 30 : 00`，第三个秒框蓝边，`00` 有选中底色和光标；小时、分钟、日期及此刻/清除完整，间距正常。 |
| `time-edit/edited.png` | pass | 秒框实际显示单字符 `5` 与末尾光标，保持蓝边，小时 `12`、分钟 `30` 不变；此时尚未补零符合编辑阶段，不将其误记为 `05` 或失败。 |
| `time-edit/submitted.png` | pass | Enter 后秒框收窄，实际显示蓝字 `05`，文本光标消失，完整时间为 `12 : 30 : 05`；此刻/清除随宽度左移而无重叠，支持单字符提交补零。 |
| `time-edit/blurred.png` | pass | 点击外部后秒框蓝边/蓝字消失，普通边框内仍显示 `05`，小时分钟仍为 `12/30`；可见提交值失焦后保持，字段和按钮未截字。 |
| `time-edit-scaled/focused.png` | pass | 1.75x 时间行显示 `12 : 30 : 00`，秒编辑框蓝边、`00` 选中及光标清晰；冒号、日期和右侧按钮均完整，无挤压。 |
| `time-edit-scaled/edited.png` | pass | 放大秒框中仅为 `5`，末尾光标清楚，保持文本编辑宽度与蓝边；小时分钟未变，尚未补零符合该阶段，邻近字段无重叠。 |
| `time-edit-scaled/submitted.png` | pass | Enter 后实际秒值为蓝字 `05`，秒框变窄、文本光标消失；完整时间 `12 : 30 : 05`，右侧按钮仍分离，单字符补零结果可见。 |
| `time-edit-scaled/blurred.png` | pass | 点击外部后蓝色焦点外观消失，普通秒框保留黑色 `05`；`12/30` 不变，日期、标签与操作按钮间距正常，未见明确截字或异常空白。 |
| `time-edit-dark/focused.png` | pass | 深色宿主内固定浅色表单，实际时间 `12 : 30 : 00`；秒框蓝边、`00` 选中及光标可见，字段标签和此刻/清除完整。 |
| `time-edit-dark/edited.png` | pass | 秒文本由 `00` 改成单字符 `5`，末尾光标及蓝边清楚；`12/30` 未变，此图不要求已补零，浅色字段无明确溢出或重叠。 |
| `time-edit-dark/submitted.png` | pass | Enter 后秒框收窄且显示蓝字 `05`，无文本光标，时间为 `12 : 30 : 05`；结合前图单字符 `5`，补零有实际序列证据。 |
| `time-edit-dark/blurred.png` | pass | 点击外部后秒框恢复普通边框和黑字，仍为 `05`，小时分钟保持 `12/30`；页面与宿主层次正常，未见明确截字、重叠或异常空白。 |

## 统计与序列结论

| 场景 | 原始 PNG 已读 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| modal-interact | 8 | 8 | 0 | 0 |
| modal-interact-scaled | 8 | 8 | 0 | 0 |
| modal-interact-dark | 8 | 8 | 0 | 0 |
| time-edit | 4 | 4 | 0 | 0 |
| time-edit-scaled | 4 | 4 | 0 | 0 |
| time-edit-dark | 4 | 4 | 0 | 0 |
| **合计** | **36** | **36** | **0** | **0** |

1. 三组 Modal 均有 `Ada → 聚焦选中 → Grace → 确定后消失并新增成功消息 → 重开仍为 Grace → 取消后消失 → 再开 Grace → × 关闭后消失` 的可见证据。取消和 × 的不同计划坐标与已查看弹窗对应位置一致，两条关闭路径均通过。取消回滚不属于本轮范围，也没有取消前再次修改的序列，不作回滚结论。
2. 三组时间均实际显示 `12:30:00 → 12:30:5（文本编辑态）→ 12:30:05（Enter 后）→ 12:30:05（点击外部、焦点外观消失）`。本轮输入计划为单字符 `5`，不是 `05`，因此提交补零已有图片与序列支持，旧报告的该项证据缺口在本轮消除。
3. `blurred` 发生在 Enter 之后，证明已补零值在外部点击后保持；没有单独覆盖“不按 Enter，直接由单字符编辑态失焦触发补零”的另一条路径，不将其扩称为独立失焦补零验收。
4. 36 张全图未发现可确认的文本截断、非预期控件重叠或异常空白。弹窗遮挡背景属于正常叠层；Skeleton 和下方“欢迎 @Ada”文本区域在滚动视口边缘裁切，不据此判布局失败。成功消息在后续 Modal 图中沿用，不能当作取消/× 的新增提交证据，也没有超时失效的充分证据。
5. 最终批 toast 可见证据为三组 Modal 各 `confirmed/reopened/cancelled/reopened-again/closed`，共 **15 张**：确认后顶部中央消息框包裹完整“Message · 操作成功”，内边距与边框正常，1.75x 下也未见截字、溢出或异常放大；重开后 toast 位于遮罩后，与 Modal 分离。这里只验收本报告这些图片中的 toast 像素，不推及其他消息类型或场景。

## 疑点与验收边界

- **要求范围内待确认疑点：0；明确失败：0。** 编辑态 `5` 不是缺陷，提交及其后失焦态均已显示 `05`。
- 本报告依据最终根目录原始 PNG、对应输入场景、artifact 和 render-report；产物路径、Debug、内置定义与后端信息来自最终批记录，本次未重新核验 EXE 哈希或重跑产物。captures-1 仅保留为历史批次名称，不作为本次结论的图片证据。
- 结论限于上述 Debug 产物的 wgpu/Dx12 CPU 离屏像素与所列交互序列；普通窗口 Glow、系统输入、原生窗口和实际显示器 DPI 未测。整表提交、外部持久化、取消回滚以及独立直接失焦补零路径不在本轮结论内。
