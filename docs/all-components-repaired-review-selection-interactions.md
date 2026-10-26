# 修复后选择类交互原始 PNG 最终批逐图视觉验收

## 范围、证据与判定

- 验收日期：2026-10-26。最终批图片根：`C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final/`；下表路径均相对此根。
- **本报告的最终批结论取代此前 `all-components-repaired-captures-1` 批次结论。** 已按最终批 9 份 `render-report.json` 的 `captures`，使用 `read` 分别打开全部 **33 张原始 PNG**（Cascader 12、Autocomplete 12、Date 9），未以 sheet、文件名或字节相同代替视觉验收。下表逐行观察与统计均经最终批画面复核；旧报告 `docs/all-components-review-selection-interactions.md` 仅作问题复查参考。
- 已参考 `docs/testing-policy.md` 和 `scripts/test-packaged-all-components.py` 的 `interaction_cases()`：Cascader 打开→展开亚洲→选择中国→清除；Autocomplete 打开→输入 Gr→选择 Grace→清除；Date 打开→点击日期网格→清除。步骤名称仅用于定位，通过与否按实际可见内容及前后变化判定。
- 已逐份核对最终批 9 份渲染报告，均记录：**Debug**，`definition: embedded`，实际 EXE 为 `C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe`；`egui_kittest 0.31.1 / wgpu (offscreen; not Glow)`，Dx12 / CPU / Microsoft Basic Render Driver，驱动 `10.0.26100.9711`；字体记录为 `egui defaults plus install_chinese_font system fonts`。这些是既有证据的元数据，本次未重新运行 EXE 或独立校验其哈希。
- 普通、dark 为 1360×900、1 倍；scaled 为 2380×1575、1.75 倍，逻辑尺寸均为 1360×900。Read 对 scaled 的预览缩小为 2000×1324；下文位置描述以控件/区域为主。
- 每张均查看页头、侧栏、导航、主体可见表单、滚动区边缘、浮动指南、页脚和日志栏；检查文字可读性、裁切、互压、对齐、弹层边界及状态变化。弹层遮挡后方表单属于正常浮层行为；滚动视口外内容不作通过声明。dark 宿主中的固定浅色页面不作为缺陷。
- `pass`：目标状态可见，未见明确视觉异常；`fail`：存在明确问题；`uncertain`：画面不足以确认目标状态。结论仅覆盖本批离屏 PNG 展示的状态与变化，不覆盖原生 Glow 窗口、系统输入、实际显示器 DPI、未截图的其他交互路径。

## 最终批每图一行

| 原始 PNG 相对路径 | 状态 | 观察与前后对照 |
| --- | --- | --- |
| `cascader-select/opened.png` | pass | 地区空框蓝边，下方白色弹层有空搜索框及右指三角“亚洲”；尚无子项，文字、边框完整。浮层正常遮住时间行，其余可见页面未见截字或互压。 |
| `cascader-select/expanded.png` | pass | 相较 opened，“亚洲”变蓝且三角向下，弹层增高并出现缩进的“中国”“日本”和层级线，真实展开成立；子项完整，未越出画面。 |
| `cascader-select/selected.png` | pass | 地区由空变为 `asia / china`，右侧新增“清除”，弹层关闭，时间 12:30:00 行重新完整露出；值与按钮不重叠，全页可见文字正常。 |
| `cascader-select/cleared.png` | pass | 相较 selected，地区路径与地区“清除”均消失，恢复全宽蓝边空框和箭头；无弹层残留，日期仍为 2026-10-22，时间未变。 |
| `cascader-select-scaled/opened.png` | pass | 放大地区空框下实际显示搜索框和折叠“亚洲”；弹层与触发框宽度协调，阴影、边框、文字完整，时间行被正常遮挡，其余页面清楚。 |
| `cascader-select-scaled/expanded.png` | pass | 放大弹层由单个父项变为蓝色展开“亚洲”及“中国”“日本”两子项，缩进和层级线清楚；弹层底边仍在欢迎输入区文字上方，无选项截断。 |
| `cascader-select-scaled/selected.png` | pass | 放大地区框完整回显 `asia / china` 并新增清除按钮；浮层消失，日期及时间行完整可见，左右列和按钮间距正常。 |
| `cascader-select-scaled/cleared.png` | pass | 已选路径及地区清除按钮消失，地区空框恢复全宽蓝边；无浮层，日期 2026-10-22、时间 12:30:00 保持，页面未见新增错位。 |
| `cascader-select-dark/opened.png` | pass | 深色宿主内地区弹层为白底深字，“亚洲”和折叠箭头可读，搜索框及弹层边框完整；没有浅字丢失，后方时间行正常被遮挡。 |
| `cascader-select-dark/expanded.png` | pass | 相较 opened，实际新增“中国”“日本”，父项“亚洲”蓝色且三角向下；白底深色子项、层级线均清楚，弹层未裁切。 |
| `cascader-select-dark/selected.png` | pass | 地区显示 `asia / china` 与独立清除按钮，弹层关闭、时间行恢复；浅色表单文字清楚，未见主题导致的低对比度或重叠。 |
| `cascader-select-dark/cleared.png` | pass | 地区路径及对应清除按钮消失，蓝边空框保留箭头且无弹层；日期、时间原值仍可见，其余页面未见新增裁切错位。 |
| `autocomplete-select/opened.png` | pass | 姓名为空，显示“输入姓名”和光标，蓝边框下实际有 Ada、Alan、Grace 三项；三行及弹层边框完整，正常遮住后方日期行。 |
| `autocomplete-select/typed.png` | pass | 姓名实际变为 `Gr` 并出现清除按钮；候选从三项缩为唯一 Grace，弹层高度随之缩小，过滤变化成立，文字未裁切。 |
| `autocomplete-select/selected.png` | pass | 姓名从 Gr 补全为完整 `Grace`，候选层关闭，清除按钮保留；日期标签及日期框恢复可见，输入值与按钮没有互压。 |
| `autocomplete-select/cleared.png` | pass | Grace 及姓名清除按钮消失，恢复全宽蓝边框与“输入姓名”占位；候选层无残留，日期仍为 2026-10-22、时间不变。 |
| `autocomplete-select-scaled/opened.png` | pass | 放大姓名框为空且有占位、光标，实际显示 Ada、Alan、Grace 三行；候选字形和边框完整，弹层轻微右移、外扩但未侵入地区列。 |
| `autocomplete-select-scaled/typed.png` | pass | 放大姓名框显示 Gr 与光标，右侧清除按钮完整；列表只剩 Grace 且明显变矮，相较 opened 确有过滤，无文字越界。 |
| `autocomplete-select-scaled/selected.png` | pass | Gr 实际补全为 Grace，候选层关闭，姓名清除按钮保留；日期行露出，左右表单列及清除按钮无交叠，可见页面未见裁字。 |
| `autocomplete-select-scaled/cleared.png` | pass | 姓名 Grace 及清除按钮消失，恢复全宽蓝边“输入姓名”；无候选层，日期和时间保留原值，页面未见新增错位。 |
| `autocomplete-select-dark/opened.png` | pass | 深色宿主内姓名空框有占位及光标，下方白底深字候选层实际显示 Ada、Alan、Grace；三项完整，无主题低对比度问题。 |
| `autocomplete-select-dark/typed.png` | pass | 姓名为 Gr，清除按钮出现，候选只剩 Grace；与 opened 三行相比过滤和弹层缩小均明确，文字与边框清楚。 |
| `autocomplete-select-dark/selected.png` | pass | 姓名由 Gr 变为 Grace，候选层消失，日期行完整恢复，清除按钮保留；浅色输入框内文字清楚，无互压裁切。 |
| `autocomplete-select-dark/cleared.png` | pass | Grace 及姓名清除按钮消失，显示全宽蓝边“输入姓名”空框；候选未残留，日期 2026-10-22、时间 12:30:00 保持。 |
| `date-select/opened.png` | pass | 左下实际展开 2026 年 10 月日历，星期与六行日期深字清楚，22 浅蓝高亮，23 在其右侧；年月及今天按钮完整，底边贴图底但未见内容截断。 |
| `date-select/selected.png` | pass | 日历关闭，日期框明确显示 `2026-10-23`；对照 opened 高亮的 22，实际日期已变更，清除按钮保留，时间 12:30:00 不变，回显完整。 |
| `date-select/cleared.png` | pass | 2026-10-23 与日期清除按钮均消失，日期框恢复全宽蓝边“选择日期”；日历无残留，姓名、地区和时间未受影响，清除变化成立。 |
| `date-select-scaled/opened.png` | pass | 放大日历实际显示 2026/10月、七列星期和六行日期，22 浅蓝高亮，23 清楚；今天按钮完整，弹层贴图底并遮住触发区和部分页脚，无明确内容截断。 |
| `date-select-scaled/selected.png` | pass | 日历关闭，放大日期框完整显示 `2026-10-23`，与 opened 的 22 高亮不同，选择变更成立；清除按钮完整，时间不变，列间距清楚。 |
| `date-select-scaled/cleared.png` | pass | 已选日期及其清除按钮消失，恢复全宽蓝边“选择日期”；没有日历残留，时间值和其他表单保持，放大文字与控件无互压。 |
| `date-select-dark/opened.png` | pass | 深色宿主内白色日历的星期行及当月日期均为清楚深字（约 x276–563、y633–814），跨月日期为灰字；2026、10月、今天均可读，22 浅蓝高亮、23 清楚。旧报告的近白字问题未复现；今天按钮完整，底边贴图底。 |
| `date-select-dark/selected.png` | pass | 日历消失，浅色日期框明确回显 `2026-10-23` 与清除按钮；对照 opened 的 22 高亮，日期实际改为 23，文字清楚且未截断，时间保持原值。 |
| `date-select-dark/cleared.png` | pass | 2026-10-23 及日期清除按钮消失，改为全宽蓝边“选择日期”；无日历残留，姓名/地区/时间未改变，清空结果清楚。 |

## 最终批统计

| 组件 | 原始 PNG 已查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| Cascader（普通 / scaled / dark 各 4） | 12 | 12 | 0 | 0 |
| Autocomplete（普通 / scaled / dark 各 4） | 12 | 12 | 0 | 0 |
| Date（普通 / scaled / dark 各 3） | 9 | 9 | 0 | 0 |
| **合计** | **33** | **33** | **0** | **0** |

| 场景 | 已查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| 普通 1 倍 | 11 | 11 | 0 | 0 |
| scaled 1.75 倍 | 11 | 11 | 0 | 0 |
| dark 1 倍 | 11 | 11 | 0 | 0 |

## 缺陷与边界观察

- **最终批未发现明确缺陷，33 pass、0 fail、0 uncertain。** 三组 Cascader 均有打开、真实展开、路径回显、关闭和清空证据；三组 Autocomplete 均有三项→Gr/单项 Grace→Grace/关闭→占位/清空证据。
- **Date 三组均确实变更并清空**：opened 的 2026 年 10 月 22 日高亮 → selected 显示 `2026-10-23` → cleared 显示“选择日期”且清除按钮消失。没有仅凭日历收起认定选择成功。
- **旧缺陷复查**：旧报告唯一失败项为 `date-select-dark/opened.png` 的星期和当月日期近白字低对比度。本批同路径画面已为白底深字，星期、日期、年月和今天按钮均可辨，22 高亮清楚，故本次判 pass；未由图片推断源码根因。
- 非失败观察：Cascader 展开项为中文“亚洲 / 中国”，选择回显为 `asia / china`，路径选择与清空均成立；未据截图自行推定必须回显中文。
- 非失败观察：Autocomplete 弹层相对输入框略向右偏且右边缘略外扩，scaled 更明显；未截断候选或侵入地区列。
- 非失败观察：三张 Date opened 的日历均上移遮挡触发区，延伸覆盖部分页脚/日志高度，底边贴近截图边界；今天按钮及最后一行日期可见完整，未见确定的内容裁切。
- 本次为既有打包产物离屏图片的人工复核，未重新构建或启动测试；未检查再次打开、其他日期、月份切换、键盘交互及真实原生窗口表现。

本次仅更新本报告为最终批验收结论。
