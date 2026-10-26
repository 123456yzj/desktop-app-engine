# 六个分区页面 PNG 逐张视觉验收

## 范围与方法

- 证据根目录：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-final`。
- 独占范围：`layout`、`form`、`navigation`、`display`、`data`、`feedback`；各目录 `top.png`、`scroll-1.png` 至 `scroll-4.png`，共 **30 张**。
- 实际逐张使用 read 打开并查看原始 PNG **30 张**（1360 × 900）；未使用联系表或已有报告代替观察。检查包括整图文字、重叠、留白、边框、控件比例、滚动裁剪和错误提示。
- 本次只审核现有图片，未运行 GUI、构建或交互测试。图片对应的 EXE、构建类型、渲染后端及生成参数未核实；结论仅覆盖这些截图可见像素，不证明交互、动画全过程或原生窗口表现。
- 图像占位、示例固定浅色、Skeleton 和等待状态不作为加载失败。页面/内部列表的正常滚动边界裁剪不作为缺陷。此组浅色截图不构成组件深色主题验收。

## 逐张结果

以下路径均相对证据根目录。坐标为原图近似像素位置。

| 路径 | 结论 | 实际观察 / 问题位置 |
| --- | --- | --- |
| `layout/top.png` | pass | 三列卡片中的总数 84、72% 进度条、主按钮/禁用按钮清晰；Splitter 的 40% 预览与分隔条无文字重叠。底部 Masonry 卡片在主滚动视口下沿正常截断。 |
| `layout/scroll-1.png` | pass | Watermark 卡片有淡色斜向水印；右侧自然高度卡片文案换行完整；Module 输入框及“发送给父级”、Affix 操作边框清楚。顶部 40% 仅露下部属于视口边界裁剪。 |
| `layout/scroll-2.png` | pass | Module 草稿框与发送按钮上下分离，Affix 文案完整；BorderBeam 右侧出现蓝色边缘片段，未覆盖文字。 |
| `layout/scroll-3.png` | pass | Masonry 三列高度不同但互不覆盖；BorderBeam 蓝线位于右下边缘；下方 Module 卡片及 Affix 按钮完整可见。 |
| `layout/scroll-4.png` | pass | BorderBeam 底边蓝线片段与标题分离；自然高度卡片两行说明、Module 草稿内容和父级按钮均可读，底部无异常残留。 |
| `form/top.png` | pass | 姓名 Ada、年龄 28 和加减按钮对齐；已勾选条款、保存/重置按钮完整；下方 Radio、Switch 比例正常，无截字。 |
| `form/scroll-1.png` | pass | Slider 滑块和轨道、Rust 下拉框完整；日期 2026-10-22、时间 12:30:00、清除按钮及 #1677ff 色块分隔清楚。底部欢迎输入区延伸出视口属正常裁剪。 |
| `form/scroll-2.png` | pass | 欢迎 @Ada 多行框、星级评分与穿梭框可见；write/admin 和已选 read 分列，双向箭头未覆盖列表。顶部日期控件在主视口上沿正常裁剪。 |
| `form/scroll-3.png` | pass | Upload 大型浅色区域内按钮“选择本地文件”清晰，下面有“文件仅记录在本地”说明；日历年月选择和 22 日高亮可见，末行在主视口下沿裁剪合理。 |
| `form/scroll-4.png` | fail | 日历六周日期及下方研发团队树可见；但日历“今天”按钮约 x269–321、y553–578 的下部被齐平切断，左右边线在同一水平处终止且下边框缺失。此位置远离主视口下沿 y758，属于组件内部非主滚动边界裁剪。树的 Ada/Alan 未重叠。 |
| `navigation/top.png` | pass | Anchor、Dropdown、三个步骤、日/周/月选项均可读；Collapse 标题和展开说明分离；Carousel 卡片下部位于主视口边界，裁剪合理。 |
| `navigation/scroll-1.png` | pass | Carousel 蓝色卡片两行文案完整；分页 1、2、3、…、13、10/页、跳至 1 和总计 123 条同排且无叠字；底部说明完整。 |
| `navigation/scroll-2.png` | pass | 步骤 1 选中底色清楚；日选项选中；折叠说明、Carousel 和分页区均无覆盖，分页前一页按钮呈禁用浅色。 |
| `navigation/scroll-3.png` | pass | Dropdown 横向按钮边框连续；轮播左右箭头与指示点间距正常；页码输入框和总条数未互相挤压。 |
| `navigation/scroll-4.png` | pass | 底部 Anchor/Pagination 解释文字完整，Carousel 标题与说明无截字；主区域已滚动到底，外层页脚和指南按钮清晰。 |
| `display/top.png` | pass | Ada 头像、99+ 徽标、Tag 与描述表格分隔明确；图片显示明确写出的占位说明，旁边二维码完整；底部 List 进入主视口下沿的裁剪合理。 |
| `display/scroll-1.png` | pass | 任务 B/C 状态、虚拟行 3/4、时间线 09:00/09:10/09:20 均可读；顶部二维码在主视口上沿裁剪，List 首项在列表自身上边界部分裁剪，按滚动边界现象记录，不误判为全局截字。 |
| `display/scroll-2.png` | pass | 时间线节点文字和时间分离，等待下一步旁有蓝色弧形加载标记；底部 Empty“暂无更多任务”居中，留白与空状态相符。 |
| `display/scroll-3.png` | pass | 任务 B/C、虚拟行及时间线均可读；等待标记蓝色弧段与文字不重叠；空状态图标及说明完整。 |
| `display/scroll-4.png` | pass | 等待下一步的蓝色弧段位置变化，但未覆盖标签；Empty 居中且与时间线保持距离；页面上沿仅保留滚动后的任务列表部分，裁剪合理。 |
| `data/top.png` | pass | Alpha/Beta/Gamma 三行节点、42.0%/18.0%/76.0% 和在线布尔值列清楚；1/2 分页与折线图标题分离，折线图下部在主视口边缘正常裁剪。 |
| `data/scroll-1.png` | pass | 折线图横轴刻度与“请求量”图例可读；六根蓝柱、0–100 纵轴和“任务量”图例分离；插件标题位于下方，插件画面尚在视口外，不据此认定加载失败。 |
| `data/scroll-2.png` | pass | ECharts Plugin 标题和四根柱子真实可见；Layout/Input/Display/Feedback 横轴标签完整，0–25 纵轴刻度清楚，无插件错误提示。上方原生柱图在主视口上沿裁剪合理。 |
| `data/scroll-3.png` | pass | 插件四柱高低不同，Display 最高；横轴文字与图形无重叠，标题、网格及基线清晰，无加载失败画面。 |
| `data/scroll-4.png` | pass | ECharts 四类标签和四根蓝柱完整，底部留白有限且未挤压图表；外层数据标签选中状态与左侧目录一致。 |
| `feedback/top.png` | pass | Modal/Drawer/Message/Notification/Tour 及 Popover/Tooltip/Popconfirm 两排按钮文字完整；绿色 Alert 有成功说明和关闭按钮，Result 勾号居中；Skeleton 下部进入视口边界。 |
| `feedback/scroll-1.png` | pass | Result 勾号及准备就绪说明完整；Skeleton 圆形和三条灰条清晰；Spin 开关关闭，内容卡片中的操作按钮无覆盖，顶部 Alert 残部为正常滚动裁剪。 |
| `feedback/scroll-2.png` | pass | 三条 Skeleton 灰条长短有别；关闭态 Switch 与“切换 Spin 加载状态”标签间距正常，Spin 卡片说明及操作框完整。 |
| `feedback/scroll-3.png` | pass | Result、Skeleton、Switch、Spin 内容区纵向分区清楚；灰条未遮住标题，操作框边线完整，无错误消息。 |
| `feedback/scroll-4.png` | pass | 主滚动到底后 Spin 卡片完整落在视口内；按钮“内容中的操作”清楚，右下指南按钮与卡片无重叠，顶部 Alert 残部属视口边界裁剪。 |

## 问题聚类

| 聚类 | 范围 / 图片 | 可见证据 |
| --- | --- | --- |
| 日历底部操作按钮内部裁剪 | `form/scroll-4.png`，1 张 | “今天”按钮下部和下边框缺失，约 x269–321、y553–578；主视口仍继续显示下方树结构，不能归为主视口底部的正常滚动裁剪。仅记录像素问题，不推断代码原因。 |

其余可见内容未发现明确的文字重叠、异常比例、非边界截字或错误/插件失败提示。BorderBeam 的局部蓝线、图片占位、Empty 空状态、Skeleton 与等待弧形均按示例或当前状态处理。滚动到底后多张截图内容相近，每张仍分别打开检查。

## 统计与限制

| 范围 | 实际查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| layout | 5 | 5 | 0 | 0 |
| form | 5 | 4 | 1 | 0 |
| navigation | 5 | 5 | 0 | 0 |
| display | 5 | 5 | 0 | 0 |
| data | 5 | 5 | 0 | 0 |
| feedback | 5 | 5 | 0 | 0 |
| **合计** | **30** | **29** | **1** | **0** |

未覆盖这六目录以外的 150 张图片；未验证弹出层打开、悬停、输入提交、动画时序、深色组件、缩放变化或实际 EXE 的运行交互。pass 仅表示该张截图可见内容未发现需判失败的问题。
