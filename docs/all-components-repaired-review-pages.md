# 修复后六分区逐张视觉验收（最终批次）

## 证据与方法

- 图片根：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final`。本报告最终结论**取代 captures-1 批次**；按 layout/form/navigation/display/data/feedback 六份 `render-report.json` 清单，重新使用 read **逐张打开全部 30 张原始 PNG**（1360 × 900），未用 sheet 替代，也未凭字节相同跳过图片。
- 历史：captures-1 初轮曾逐张查看 30 图，结果 pass 30、fail 0、uncertain 0，今天按钮裁剪问题当轮已未见。
- 参考 `docs/all-components-review-pages.md` 旧问题及 `docs/testing-policy.md`；结论来自实际像素观察，不以生成成功判 pass。旧报告仅供问题对照，本次未对旧 PNG 做逐像素差异比较。
- 六份最终报告记录的对象：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe`，**Debug、embedded 内置定义**；`egui_kittest 0.31.1 / wgpu` 离屏渲染，**Dx12、CPU、Microsoft Basic Render Driver**。
- 本组均为 `theme: light`，逻辑尺寸 1360 × 900，`pixels_per_point: 1.0`。dark 参数仅代表宿主主题，不代表固定浅色组件已完成深色验收；本组未覆盖 dark。此缩放记录不等于系统 DPI 验证，离屏结果不证明 Glow、原生窗口或系统输入通过。
- 本次审核已有截图，未重新构建、启动 EXE 或执行交互。pass 仅表示该图可见内容未发现明确视觉缺陷；正常滚动边界裁剪、示例图片占位、Skeleton、Empty 和等待弧段不作失败。

## 逐张结果

以下全部为最终批次重新逐图观察的结果，路径相对上述最终图片根；坐标为原图近似像素位置。

| 相对路径 | 结论 | 具体观察 |
| --- | --- | --- |
| `layout/top.png` | pass | 三列卡片的 84、72% 进度条及主/禁用按钮完整，Splitter 的 40% 与说明分离；下方 Masonry 在主视口底边正常裁剪，无叠字。 |
| `layout/scroll-1.png` | pass | 淡色水印未影响正文；自然高度卡片两行说明完整，Module 草稿框、发送按钮及 Affix 边框连续；顶部 40% 为主视口上沿裁剪。 |
| `layout/scroll-2.png` | pass | BorderBeam 蓝线沿右边缘，未压住文字；三列卡片互不覆盖，Module 输入与发送按钮上下分离，底部 Affix 完整。 |
| `layout/scroll-3.png` | pass | BorderBeam 蓝线延伸到右下角，正文仍清晰；水印卡片、自然高度卡片、Module 和 Affix 均无异常截字或重叠。 |
| `layout/scroll-4.png` | pass | 蓝色边缘片段位于 BorderBeam 右下边界；草稿内容、父级说明、发送按钮及 Affix 均完整，页脚与指南按钮无覆盖正文。 |
| `form/top.png` | pass | Ada、年龄 28 与加减按钮对齐；条款勾选、保存/重置按钮清楚；Radio 选中/禁用状态和 Switch 显示正常。 |
| `form/scroll-1.png` | pass | Slider 轨道及滑块完整；Rust、姓名联想、地区、日期 2026-10-22、时间 12:30:00 与清除/此刻按钮无挤压，色块及 #1677ff 可读；底部输入区属视口裁剪。 |
| `form/scroll-2.png` | pass | 欢迎 @Ada 输入区、三颗半金色星级清楚；穿梭框搜索栏、write/admin 与 read 分栏及双向箭头完整；顶部日期/时间控件仅在主视口上沿裁剪。 |
| `form/scroll-3.png` | pass | 上沿可见 admin 与穿梭箭头，下拉框、Upload 选择本地文件按钮和本地记录说明完整；日历年月、星期及 22 日高亮清晰，后续日期在主视口底边之外，不据此判断今天按钮。 |
| `form/scroll-4.png` | pass | 重点复核：今天按钮约 x269–321、y547–577，文字、左右边线、下边框及底部圆角均完整，旧报告下部齐平切断现象未见；六周日期完整，下方研发团队、Ada/Alan 分行且不重叠。 |
| `navigation/top.png` | pass | Anchor、Dropdown、三步骤、日/周/月及 Collapse 展开说明可读，控件间距正常；轮播下部在主视口底边裁剪。 |
| `navigation/scroll-1.png` | pass | 轮播箭头/指示点与蓝色卡片分离，两行文案完整；分页 1、2、3、…、13、10/页、跳至 1、共123条及下方说明均无叠字。 |
| `navigation/scroll-2.png` | pass | 第一步与日选中底色清楚，折叠说明未覆盖轮播；分页前一页为浅色禁用态，页码及跳转输入边框完整。 |
| `navigation/scroll-3.png` | pass | Dropdown 横向边框连续，轮播卡片文案和左右箭头清楚；分页输入与总条数间距足够，底部说明完整。 |
| `navigation/scroll-4.png` | pass | 滚到底后轮播和分页均完整落在视口内，Anchor/Pagination 说明无截字，页脚及指南文字未遮挡内容。 |
| `display/top.png` | pass | Ada 头像、99+ 徽标、可选 Tag 及描述表格分隔清楚；明确标注的图片占位和二维码完整；List 在主视口下沿裁剪合理。 |
| `display/scroll-1.png` | pass | 任务 B/C、虚拟行 3/4 与时间线三组文字/时间分离；二维码残部在主视口上沿、任务 A 标题在列表自身上沿裁剪，未发现新的非边界截字。 |
| `display/scroll-2.png` | pass | 09:00/09:10/09:20 与节点标签无重叠，等待弧段开口向下且位于文字左侧；Empty 图标及暂无更多任务居中完整，留白符合空状态。 |
| `display/scroll-3.png` | pass | 任务 B/C、虚拟行及时间线清晰；等待弧段方向变化但未覆盖文字，空状态与时间线保持距离，无异常残留。 |
| `display/scroll-4.png` | pass | 等待弧段仍与标签分离；Empty 图标和说明完整，列表仅在主视口上边界裁剪，页面底部未挤压内容。 |
| `data/top.png` | pass | Alpha/Beta/Gamma、42.0%/18.0%/76.0% 与 true/false 列清楚，1/2 分页和折线图标题分离；图表下部为正常主视口裁剪。 |
| `data/scroll-1.png` | pass | 折线横轴及请求量图例可读；六根原生蓝柱、刻度与任务量图例分离；插件标题完整，插件主体尚在视口外，不作加载失败。 |
| `data/scroll-2.png` | pass | ECharts Plugin 标题、四根蓝柱、0–25 刻度及 Layout/Input/Display/Feedback 标签真实可见且完整，无错误提示；上方原生图表为边界裁剪。 |
| `data/scroll-3.png` | pass | 插件四柱高度分别约 20/18/24/22，Display 最高；横轴标签、基线、网格与标题清楚，未见加载失败或文字覆盖。 |
| `data/scroll-4.png` | pass | 四根插件柱及四类标签完整，图表底部留白足够；上方原生图例清楚，侧栏和顶部数据选中状态一致，无新异常。 |
| `feedback/top.png` | pass | 两排弹层入口按钮文字完整，绿色 Alert 两行说明及关闭按钮清楚；Result 勾号和说明居中；Skeleton 在主视口底边正常裁剪。 |
| `feedback/scroll-1.png` | pass | Result 及准备就绪说明完整，Skeleton 圆形和三条灰条清楚；Spin 开关关闭，内容说明及操作按钮边框完整；顶部 Alert 残部属视口裁剪。 |
| `feedback/scroll-2.png` | pass | Skeleton 两长一短灰条未覆盖标题；关闭态 Switch 与标签分离，Spin 说明和内容中的操作按钮无截字。 |
| `feedback/scroll-3.png` | pass | Result、Skeleton、开关及 Spin 卡片分区清晰；操作按钮四边完整，无加载遮罩误盖内容或错误提示。 |
| `feedback/scroll-4.png` | pass | Spin 卡片完整位于主视口内，说明和操作按钮清晰；指南按钮位于内容区外下方，未覆盖卡片；顶部 Alert 为正常滚动裁剪。 |

## 汇总与问题

| 目录 | 实际查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| layout | 5 | 5 | 0 | 0 |
| form | 5 | 5 | 0 | 0 |
| navigation | 5 | 5 | 0 | 0 |
| display | 5 | 5 | 0 | 0 |
| data | 5 | 5 | 0 | 0 |
| feedback | 5 | 5 | 0 | 0 |
| **合计** | **30** | **30** | **0** | **0** |

- 最终批次旧问题复核：`form/scroll-4.png` 的今天按钮显示完整，截图范围内可判修复通过。
- 最终批次其余页面未发现新的明确视觉回归。多张滚到底后的画面相近，仍分别打开检查。
- 这 30 图未展示打开的 toast，不能据此单独判定 toast 标题测量修复通过；本次结论覆盖最终 EXE 生成的六分区页面截图。
- 未测范围：其余目录、弹层展开/悬停/输入提交、今天按钮点击效果、动画全过程、深色组件、其他缩放、Release、Glow、原生窗口与系统 DPI；这些结论不由本次截图推定。
