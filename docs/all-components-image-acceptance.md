# All Components：180 张原始 PNG 逐张验收

> 历史批次报告：后续已修复四类缺陷，并重新打包、生成与逐张验收 183 张图片。最新结论为 183 pass / 0 fail / 0 uncertain，见 [修复后回归验收](all-components-repaired-acceptance.md)。下文保留旧产物的失败记录。

验收日期：2026-10-26。结论：**180/180 张已逐张查看，162 张 pass，15 张 fail，3 张 uncertain；整体未通过验收。**

## 对象与方法

- 使用六个并行子代理分工，每张原始 PNG 均通过 Read 打开并目视检查；联系表仅为导航，不代替原图。主代理复看四类问题的代表性原图，并校核所有逐图记录。
- 用户建议使用 `gpt-6-luna`；当前子代理调用接口不提供模型选择参数，本轮没有指定或确认该模型，不据此标注模型身份。
- 复验对象是先前实际打包 EXE 已生成的图片，本轮没有重新构建或重新执行 GUI 测试。
- 产物：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-fixed-final\all-components.exe`。
- 已记录 SHA-256：`4e042cdd3c0bc493e0f25135dcbb21c0c131a652bb99fad3c6dfb014ff1dde92`。
- 场景报告为 Debug、`definition: embedded`、`egui_kittest 0.31.1 / wgpu` 离屏，Dx12 / Microsoft Basic Render Driver CPU；普通窗口为 Glow。
- 逻辑尺寸 1360×900；原始 PNG 分别为 1360×900（1x）与 2380×1575（1.75x）。Read 会将大图缩小预览为 2000×1324，分报告中 scaled 坐标已注明是预览坐标。
- 示例固定浅色，dark 场景是深色宿主；仍需检查浅色弹层的实际文字对比度，不能以“非组件深色测试”为理由忽略可见缺陷。

证据根目录：

| 别名 | 路径 |
| --- | --- |
| final | `C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-fixed-final` |
| interactions | `C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-fixed-interactions` |

## 逐图报告与统计

各报告包含每张图片单独一行的路径、结论和实际可见依据。

| 独占分工 / 报告 | 查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| [六个分区页面](all-components-review-pages.md) | 30 | 29 | 1 | 0 |
| [Tag、Transfer、Chart](all-components-review-charts-transfer.md) | 36 | 33 | 3 | 0 |
| [弹层及录入初态](all-components-review-overlays-inputs.md) | 30 | 23 | 7 | 0 |
| [级联、联想及日期交互](all-components-review-selection-interactions.md) | 33 | 32 | 1 | 0 |
| [Modal 及时间编辑交互](all-components-review-modal-time.md) | 33 | 30 | 0 | 3 |
| [Popconfirm 及通知交互](all-components-review-feedback-interactions.md) | 18 | 15 | 3 | 0 |
| **合计** | **180** | **162** | **15** | **3** |

清单校核以两个目录的 `*/render-report.json` 中的 captures 为准，逐一匹配六份报告的图片记录，检查重复、遗漏、多余记录、原图尺寸和文件 SHA-256。结果：**180 张均有且只有一条记录，遗漏 0，重复 0**。不将目录顶层 `*-sheet.png` 联系表重复计入原图总量。

## 确认的问题

以下是四类可见缺陷，共影响 15 张图片；不能因其他交互达成而判这些图片通过。

| 编号 | 问题与影响 | 失败图 |
| --- | --- | --- |
| F1 | 通知卡片遮住右下“指南”圆形按钮大部分主体及问号；通知自身内容完整，但浮动控件发生碰撞。 | final：`notification{,-scaled,-dark}/{opened,settled}.png`（6）；interactions：`notification-close{,-scaled,-dark}/opened.png`（3） |
| F2 | 深色宿主下日历白色面板上的星期和当月日期为极浅色文字，明显难辨，年月/今天又是深底，出现局部混色。 | final：`date-dark/opened.png`；interactions：`date-select-dark/opened.png`（2） |
| F3 | 缩放后的折线图横轴刻度出现换行残迹及截字，位于图表内部，不是页面视口正常裁剪。 | final：`chart{,-scaled,-dark}/page-scroll.png`（3） |
| F4 | 独立 Calendar 的“今天”按钮下部及下边框被组件内部裁剪，约 x269–321、y553–578，远离页面视口下沿。 | final：`form/scroll-4.png`（1） |

主代理复看的代表原图为 final 下 `notification/settled.png`、`date-dark/opened.png`、`chart/page-scroll.png`、`form/scroll-4.png`，四类问题均保留。这里确认像素问题，未将推测的源码原因写成已定位根因。

## 证据不足：3 张

- interactions：`time-edit{,-scaled,-dark}/edited.png`。
- 可以确认秒数从 `00` 编辑为 `05`，且后续提交保留 `05`；输入计划直接发送文本 `"05"`，因此**没有测试输入单字符 `5` 后自动补零**。三张保留 uncertain，不能据此宣称自动补零过程已验收。

## 主代理复核后的结论修订

- 子代理初审统计为 150 pass / 15 fail / 15 uncertain。核对实际配置及输入计划后，12 项 uncertain 改为 pass；没有改变 15 张 fail。
- Tag 初始 `tagChecked: true`；normal 蓝底到文件名 checked 对应的淡底，是取消选中，不能按文件名要求结果为 true。三张切换结果图通过。
- Transfer 选项 read/write/admin 都是普通字符串，并未配置 disabled。三张 normal 图不应因缺少未配置的禁用行为证据而判 uncertain；禁用分支仍列为未测。
- Modal 的输入计划记录了取消按钮及 × 的不同坐标点击，结合按钮位置及弹窗消失图，可确认对应退出结果，六张改为 pass。取消前没有新一轮编辑，不额外声称取消回滚语义已测。

## 本地检查入口

- 原有总览：final 目录 `index.html`。
- 本轮逐图总览：final 目录 `per-image-review.html`，包含 180 条结论和原图链接，可按状态筛选。
- 本轮机器可读校核结果：final 目录 `per-image-review.json`，包括每图结论、观察、图片路径、尺寸、哈希及对应子报告路径。
- 可复核命令：`python "C:/Users/yzj123/AppData/Local/Temp/opencode/audit-all-components-reviews.py"`。该脚本只校核已有记录与图片，不会自动判定视觉通过，也不启动应用。

## 验收边界与后续

- 180 张全部查看不等于 84 种组件的所有状态全部覆盖；PNG 外的行为仍不得据此推断。
- 本轮未修复上述四类问题。后续修复须重新构建、打包、生成受影响的实际 EXE 图片，再逐张复验；不能覆盖旧图后沿用本报告结论。
- 原生 Glow 窗口、真实系统输入与 DPI、Release、通知其他三角、分组/单点柱图、纵向 Descriptions、Transfer 搜索过滤及禁用分支、取消回滚、单数字自动补零仍未验收。
- 本报告为逐图人工视觉查看与状态序列对照，没有执行自动像素快照差异判定。
