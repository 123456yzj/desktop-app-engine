# 修复后反馈组件交互 PNG 逐图视觉验收（最终批）

## 范围、证据与方法

- 最终批图根：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final`。下表路径均相对此根目录，18 行结论全部来自本次重新打开的新图。
- 按六组 `render-report.json` 的 captures 清单，使用 `read` 分别打开全部 **18 张原始 PNG**（Popconfirm 12 张、Notification 6 张），逐张检查并比较同组前后状态；未以 sheet 代替原图。
- 参考 `scripts/test-packaged-all-components.py::interaction_cases()`、`docs/all-components-review-feedback-interactions.md` 与 `docs/testing-policy.md`。综合结论同时包含可见交互变化及视觉位置要求；关闭成功不抵消 opened 的相交缺陷。
- 六组报告均记录：实际 EXE 为 `C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe`，`definition=embedded`，**Debug**，`egui_kittest 0.31.1 / wgpu (offscreen; not Glow)`，适配器名称为 Microsoft Basic Render Driver。
- 普通组为 light、1 倍，dark 组为 dark、1 倍，原图均为 1360×900；scaled 为 light、1.75 倍，原图为 2380×1575。逻辑画布均为 1360×900。下文涉及 rect 的数值均为报告中的逻辑点；scaled 原图像素应乘 1.75，不能直接使用 Read 缩小预览坐标当原图像素。
- 本次为已有产物截图的人工复核，未重新构建或启动 EXE；产物来源与后端信息依据随图报告，未独立校验 EXE 哈希。结论限于这些离屏截图的实际可见结果；未验收 Release、Glow、原生窗口、系统输入或真实显示器 DPI。

## 逐图结果

| 相对图根路径 | 状态 | 观察 |
| --- | --- | --- |
| `popconfirm-interact/opened.png` | pass | 触发按钮下方约 x773–1013、y370–438 出现完整白色确认气泡，问题“确定执行演示操作吗？”及“取消”“确定”均可读；没有空白内容、重复气泡或出界裁剪。页脚仍为欢迎语，无本轮成功 Message。 |
| `popconfirm-interact/confirmed.png` | pass | 对比 opened，气泡、问题及两个按钮消失；顶部出现绿色“Message · 操作成功”，页脚变为“Popconfirm 操作已确认。”，确认实际产生了可见反馈；原弹层区域无残影。 |
| `popconfirm-interact/reopened.png` | pass | 对比 confirmed，同一触发按钮下方再次出现完整问题和取消/确定按钮；旧成功 Message 与确认页脚保留，未遮挡重新打开的气泡，无重复弹层。 |
| `popconfirm-interact/cancelled.png` | pass | 对比 reopened，气泡及两按钮确实收起；旧成功 Message 与确认页脚保持，没有新增可见确认反馈。取消步骤的可见结果成立，未见残留空白气泡。 |
| `popconfirm-interact-scaled/opened.png` | pass | 1.75 倍原图中确认气泡贴近触发按钮下方，问题及取消/确定按钮完整，文字与边界清楚，没有异常重叠或出界；页脚仍为欢迎语，尚无成功 Message。 |
| `popconfirm-interact-scaled/confirmed.png` | pass | 对比同组 opened，气泡消失，顶部绿色成功 Message 与页脚“Popconfirm 操作已确认。”同时可见；放大后仍无残留弹层或文字截断，确认变化明确。 |
| `popconfirm-interact-scaled/reopened.png` | pass | 对比 confirmed，确认问题和两个操作按钮重新显示；已有成功 Message、旧页脚与气泡互不遮挡，气泡内容完整，无重复或空白。 |
| `popconfirm-interact-scaled/cancelled.png` | pass | 对比 reopened，确认气泡及按钮完全消失，原位置恢复页面背景；旧成功 Message 和确认页脚不变，没有新增可见反馈，取消后的收起结果明确。 |
| `popconfirm-interact-dark/opened.png` | pass | 深色宿主外框内固定浅色页面正常，触发按钮下方显示完整确认问题及取消/确定按钮；没有异常裁剪或重复气泡，页脚为欢迎语，无成功 Message。 |
| `popconfirm-interact-dark/confirmed.png` | pass | 对比 opened，确认气泡消失，顶部出现绿色成功 Message，页脚更新为“Popconfirm 操作已确认。”；确认反馈清楚，无空白弹层残留。 |
| `popconfirm-interact-dark/reopened.png` | pass | 对比 confirmed，原位置重新显示问题及两按钮，旧成功 Message 和确认页脚仍在；内容均可读，未见异常重叠或出界。 |
| `popconfirm-interact-dark/cancelled.png` | pass | 对比 reopened，问题、气泡与两按钮消失；旧成功 Message 与确认页脚保留，未出现额外可见反馈，取消收起的视觉结果成立。 |
| `notification-close/opened.png` | pass | 通知标题、关闭 × 与正文完整可读；卡片约 x1078–1344、y683–763，指南按钮顶部 y771，卡片实体与按钮留出 8 点间距。原图可见完整蓝色圆弧、问号与“指南”，底部阴影在间隙渐隐，未见压住按钮上缘或遮挡按钮内容。 |
| `notification-close/closed.png` | pass | 对比 opened，通知卡片、标题、正文及 × 全部消失，原位置重新露出 Skeleton 和页面背景；指南圆按钮顶部完整恢复，没有残留空白通知框。确实存在关闭后的可见变化，并非依 closed 文件名判定。 |
| `notification-close-scaled/opened.png` | pass | 通知完整显示标题、× 与正文；报告逻辑 rect 为 [1077.969,682.719,1344,763]，与指南顶部 y771 相隔 8 点（原图 14 像素）。逐图可见卡片与圆按钮分离，蓝色顶部圆弧、问号和标签完整，阴影在间隙渐隐，没有可见遮挡。 |
| `notification-close-scaled/closed.png` | pass | 对比同组 opened，整张通知及其阴影、标题、正文、× 均消失；Skeleton 与页面背景重新可见，指南圆按钮顶部完整，没有通知残影或空白卡片，关闭结果通过。 |
| `notification-close-dark/opened.png` | pass | 深色宿主内浅色通知的标题、× 和正文完整；通知 y683–763、指南从 y771 开始，实体间距 8 点。按钮上缘圆弧完整、问号与标签清楚，阴影在间隙渐隐，未见通知底边或阴影压住按钮。 |
| `notification-close-dark/closed.png` | pass | 对比 opened，通知卡片及全部文字、× 与阴影消失，原位置恢复 Skeleton/页面背景；指南圆按钮完整恢复，关闭后的可见结果明确，未见残留通知框。 |

## 统计

| 分组 | 实际逐图查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| popconfirm-interact | 4 | 4 | 0 | 0 |
| popconfirm-interact-scaled | 4 | 4 | 0 | 0 |
| popconfirm-interact-dark | 4 | 4 | 0 | 0 |
| notification-close | 2 | 2 | 0 | 0 |
| notification-close-scaled | 2 | 2 | 0 | 0 |
| notification-close-dark | 2 | 2 | 0 | 0 |
| **合计** | **18** | **18** | **0** | **0** |

## 具体缺陷与交互结论

### N1：Notification 与指南按钮的位置冲突已在最终批消除

- 历史：`all-components-repaired-captures-1` 批曾有三个 Notification opened 因与指南按钮顶部相交而失败（该批总计 15 pass / 3 fail）；本报告现以最终批重新逐图复核结果为准。
- 最终批证据：三个 `notification-close{,-scaled,-dark}/opened.png`，均重新使用 `read` 打开原图检查。
- 普通/dark 的通知 rect 为 [1078.094,683,1344,763]，scaled 为 [1077.969,682.719,1344,763]；三组指南按钮顶部均为 y771，通知实体与按钮均无交集，垂直间距 **8 逻辑点**，scaled 对应原图 **14 像素**。
- 阴影单独目视检查：通知底部灰色阴影在卡片下方间隙渐隐，指南顶部蓝色圆弧保持完整，未见被切平、盖灰或文字受遮挡；不把 rect 不相交本身当作阴影通过的依据。此为视觉无可见遮挡结论，不声称已做逐像素零差异检测。
- 最终批本责任范围内未发现尚存的具体视觉缺陷。

### Notification 关闭：三组均实际消失

- 参照 `interaction_cases()` 的关闭步骤，对 `notificationDemo` 的实际 rect 使用 `target_position=[0.58,0.36]`；按最终批 rect 换算，普通/dark 约为 (1232.319,711.8)，scaled 逻辑约为 (1232.267,711.62)，与图中标题行关闭 × 位置对应。
- 不因目标存在或文件名为 closed 就假定点击成功。逐图对比确认：三组 closed 中卡片、标题、正文、关闭 × 及阴影都消失，底层 Skeleton 与页面背景恢复，指南按钮继续完整可见。
- 因此最终批关闭后的视觉结果为 **3/3 通过**，未见因卡片位置变化而关闭失效。

### Popconfirm：三组状态链均有实际可见变化

- 脚本依次打开、在逻辑 (865,416) 点击确认、重开、在 (807,416) 点击取消。三组图片均显示：有问题和两按钮 → 气泡消失并出现成功 Message/确认页脚 → 气泡重现 → 气泡再次消失。
- cancelled 中的成功 Message 与“操作已确认”是前一步已有反馈，reopened 中已经存在，不能把保留的旧反馈判成取消再次确认；本批未看到新增确认反馈。
- 静态图能验证确认反馈、重开和取消步骤后的收起，不能独立证明每次关闭事件来源或排除不可见的重复事件分发。此限制不改变本批可见状态链的通过结论。
- 18 张图未见空白弹层、异常重复文字；滚动视口底部 Skeleton 截断及 dark 宿主内固定浅色页面不列为缺陷。
