# All Components 修复方案与验收记录

对象：`examples/all-components.json`。本轮先记录方案，再修改、打包和逐项图片复验。

## 修复清单

| # | 问题 | 实施方案 | 验收要求 | 状态 |
| --- | --- | --- | --- | --- |
| 1 | 可关闭 Tag 文字截断 | 自然测量预留关闭按钮宽度，与绘制区域一致 | 1x、1.75x 完整显示文字和关闭按钮，点击关闭有效 | 已修复；示例图片及关闭交互已查看 |
| 2 | 柱图首尾半柱裁切 | 自动横轴范围加入柱组半间距，保留显式范围语义 | 首尾柱完整，分组柱及单点数据合理 | 已修复；示例首尾柱已查看，分组/单点未测 |
| 3 | Descriptions bordered 没有边框 | 按行列分配空间，绘制外框及单元格分隔，支持纵向布局 | 标签内容对齐，边框完整，示例高度适配 | 已修复；水平两列已查看，纵向未测 |
| 4 | 图表抢占页面滚轮 | 图表缩放要求 Alt，普通滚轮交给页面滚动 | 指针在图中仍能滚动页面；Alt 缩放保留 | 已修复；页面滚动和 Alt 缩放图片已查看 |
| 5 | Transfer 三等分比例失衡 | 中间使用紧凑操作区，两侧等宽边框面板和可用搜索区 | 搜索、选中、双向移动均可见且可用 | 已修复；面板/搜索区显示、选中及双向移动图片已查看 |
| 6 | 录入控件尺寸与对齐不统一 | 统一分配宽高和输入布局，补齐示例标签；时间补零 | 输入、选择、联想、级联、日期、时间对齐，弹层和编辑可用 | 已修复；布局、输入、选择、清除及时间编辑图片已查看 |
| 7 | Modal 空白过大且按钮主次不明 | 内容自适应，底部按钮右对齐，确认按钮突出 | 对话框紧凑，确认/取消/关闭交互正常 | 已修复；编辑、确认、重开、取消及 × 关闭图片已查看 |
| 8 | Notification 贴右下边界 | 根据锚点方向计算安全偏移，底部预留状态区间距 | 四角通知均在视口内留足间距，关闭可用 | 已修复；右下显示及关闭已查看，其余三角未测 |
| 9 | Cascader/Popconfirm 弹层空白 | 级联弹层限制合理宽高，空标题不占行且避免重复文案 | 弹层紧凑，展开、选择、确认和取消可用 | 已修复；级联展开/选择/清除和气泡确认/取消图片已查看 |

## 基线与验证方法

- 修复前 Debug 打包产物：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-current-audit\all-components.exe`。
- 修复前图片与详细定位：同目录 `review.md`、各场景 PNG 和联系表。
- 先执行格式、编译和适用逻辑测试；这些检查不代表视觉通过。
- 使用新构建的实际打包 EXE 的 `--visual-test` 入口，确认报告为内置定义，并记录构建类型、产物哈希、主题、缩放及后端。对修改涉及的普通/弹层/交互状态实际渲染 PNG，逐项查看并与基线比较。
- 覆盖浅色宿主 1x、浅色宿主 1.75x 和深色宿主 1x 关键状态；示例 `configProvider` 固定浅色，深色宿主场景不构成组件自身深色主题验收。禁止用绘制参数或内部状态断言替代图片验收。
- 离屏后端为 wgpu；普通窗口为 Glow。离屏图片不能证明真实窗口、系统输入、显示器 DPI 已通过。

## 执行记录

- 九项实施完成，实际打包 Debug EXE 的示例图片与关键交互已复验；各项范围见上表。
- Context7 文档查询受月度配额限制，API 用法已结合本地 egui 0.31.1 依赖源码核对。
- 首轮复验发现日期清除按钮过窄、通知覆盖底部状态区，修正后重新构建并打包为下述最终产物；`all-components-fixed-audit-4` 等中间产物不作为最终入口。

### 最终产物与本地检查入口

- 汇总页：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-fixed-final\index.html`，含前后对比、场景联系表、原始 PNG 和渲染报告链接。
- 实际产物：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-fixed-final\all-components.exe`。
- SHA-256：`4e042cdd3c0bc493e0f25135dcbb21c0c131a652bb99fad3c6dfb014ff1dde92`，见同目录 `artifact.json`。
- 补充交互证据：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-fixed-interactions`，复用上述最终 EXE，场景定义为该目录 `plan.json`。
- 两个目录共生成 180 张场景 PNG；该数量表示生成量，不表示每一张均已逐一验收。已查看的关键状态及未测分支见上表。
- 报告确认 `definition: "embedded"`，`executable` 为最终产物；逻辑尺寸 1360×900，缩放 1x / 1.75x。
- 构建类型 Debug；后端 `egui_kittest 0.31.1 / wgpu`，Dx12，CPU `Microsoft Basic Render Driver`，驱动 `10.0.26100.9711`。字体来自本机 Windows 字体，检查页目录 `fonts.json` 记录生成检查页时的字体哈希。

### 代表性图片

以下路径相对于最终产物目录；补充交互目录由“交互/”标注：

| 检查内容 | 证据 |
| --- | --- |
| Tag 和水平 Descriptions | `display/top.png`、`tag-sheet.png`、`tag-scaled-sheet.png` |
| 柱图边界、普通滚动和 Alt 缩放 | `chart-sheet.png`、`chart-scaled-sheet.png` |
| Transfer 选中和双向移动 | `transfer-sheet.png` |
| 录入布局与弹层 | `inputs/scrolled.png`、`cascader/opened.png`、`date/opened.png` |
| 级联、联想、日期选择与清除 | 交互/`cascader-select-sheet.png`、`autocomplete-select-sheet.png`、`date-select-sheet.png`，及对应 `-scaled-sheet.png` |
| 时间补零和编辑 | 交互/`time-edit-sheet.png`、`time-edit-scaled-sheet.png` |
| Modal 布局与操作 | `modal/settled.png`；交互/`modal-interact-sheet.png` |
| 右下通知与关闭 | `notification/settled.png`、`notification-scaled-sheet.png`；交互/`notification-close-sheet.png` |
| Popconfirm 文案与操作 | `popconfirm/settled.png`；交互/`popconfirm-interact-sheet.png` |

### 执行命令与结果

```powershell
cargo test --all-targets --all-features --locked
cargo build --bin desktop-app-engine --features visual-tests --locked
python "scripts/test-packaged-all-components.py" --engine "target/debug/desktop-app-engine.exe" --output "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-final"
cargo check --all-targets --all-features --locked
cargo fmt --all -- --check
git diff --check
```

- 以上检查成功；库单元测试 41 项通过，包含柱间距和可关闭 Badge 自然宽度的逻辑/纯布局测试。编译和逻辑测试不作为视觉正确的依据。
- 打包脚本复制 engine、ECharts 插件及 PATH 上 Node 到独立 producer，随后用生成的 EXE 执行离屏场景，校验内置定义、EXE 身份与 PNG 尺寸。输出目录需不存在，复现时请更换新目录。
- 补充交互使用同一实际产物，通过 `--exe`、`--plan` 入口复现（输出目录同样需新建）：

```powershell
python "scripts/test-packaged-all-components.py" --exe "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-final/all-components.exe" --plan "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-interactions/plan.json" --output "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-interactions-rerun"
python "scripts/test-packaged-all-components.py" --index-only --output "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-final" --related "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-fixed-interactions" --baseline "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-current-audit"
```

### 未测范围

- Release 构建、隔离原生 Glow 窗口、操作系统输入、真实显示器 DPI。
- Notification 另外三个角、分组/单点柱图、纵向 Descriptions，以及全部组件的状态与配置组合。
- Transfer 搜索过滤的完整交互分支；当前查看了搜索区显示和选中/双向移动。
- 组件自身深色主题、系统文件对话框、有效图片预览。
- 图片为实际渲染后人工查看与前后对照，未运行自动像素快照差异判定。
