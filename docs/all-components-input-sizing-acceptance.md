# 姓名联想与 Mentions 尺寸修复验收

## 结论

本轮实际打包产物的 27 张定向 PNG 已逐张查看：27 通过、0 失败、0 不确定。检查了尺寸与留白协调、文字位置、弹层对齐，以及真实渲染后的输入、选择、清除、失焦和三行文本状态。

此前 [183 图回归](all-components-repaired-acceptance.md) 对应 repaired-4 产物；本轮修改后的产物只完成下述定向验收，未重新运行全部 183 图。

## 修复内容

- `src/components/egui/widgets/shared/suggestions.rs`：单行文字垂直居中，采用组件字体和内边距；编辑区域使用明确分配的矩形，多行编辑不再仅使用分配高度的 60%；弹层以下方左角为锚点，扣除 Frame 边距后与实际编辑框等宽。存在清除按钮时，弹层对齐编辑框，不包含右侧清除按钮。
- `src/components/layout/measure.rs`：Mentions 自然高度根据 rows、文本行高、内边距和边框计算，替代固定 100 点高度。
- `examples/all-components.json`：姓名联想保留与地区框一致的 40 逻辑点高度；Mentions 保留三行，设置 `max-w-[840px] text-sm`，左右对齐上方表单。

## 实际测试对象与证据

- EXE：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-input-sizing-2\all-components.exe`
- SHA-256（artifact.json）：`1f85405512adb67a25eb739c6a8410f1eb2f0e480e42dc7704dd136abeeb963d`
- 构建：Debug，启用 `visual-tests`。产物自身的离屏入口运行实际 EXE，三个 render-report 均记录 `definition: embedded`。
- 证据根目录：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-input-sizing-2`；入口 `index.html`，另有 `artifact.json`、`fonts.json`、逐场景配置和进程日志。
- 渲染：egui_kittest 0.31.1 / wgpu，Dx12，Microsoft Basic Render Driver，Cpu，驱动 `10.0.26100.9711`。未创建桌面测试窗口或发送系统输入。
- 逻辑尺寸 1360×900；`suggestions` 浅色 1×（1360×900 PNG）；`suggestions-scaled` 浅色 1.75×（2380×1575 PNG）；`suggestions-dark` 深色宿主 1×（1360×900 PNG）。示例 ConfigProvider 固定浅色，深色宿主不代表组件自身深色主题全面验收。
- 已滚动到完整可见位置的 Mentions 目标矩形为 840×62 逻辑点；姓名联想与地区框均为 40 逻辑点高。几何记录仅作补充，结论来自下列实际 PNG 的视觉检查。
- 首次 `all-components-input-sizing-1` 的 21 张截图保留为修正弹层定位前的证据，不计入本轮通过数。

## 逐图检查

下表每个场景单元格对应一张原始 PNG，路径为“证据根目录 / 场景目录 / 文件名”。共 9×3=27 张，全部逐张打开检查，未以联系表或文件名代替检查。

| PNG 文件名 | suggestions | suggestions-scaled | suggestions-dark | 实际可见结果与检查点 |
| --- | --- | --- | --- | --- |
| opened.png | 通过 | 通过 | 通过 | 姓名框与地区框等高；占位文字居中；Ada、Alan、Grace 三个选项可见，弹层紧贴框底且左右对齐。 |
| typed.png | 通过 | 通过 | 通过 | 输入 Gr，筛出 Grace；文字居中；清除按钮完整；弹层与扣除按钮后的编辑区域等宽。 |
| selected.png | 通过 | 通过 | 通过 | 实际显示 Grace，弹层关闭；输入文字与清除按钮无裁剪。 |
| cleared.png | 通过 | 通过 | 通过 | Grace 清空，恢复“输入姓名”占位符；清除按钮消失，编辑框恢复整列宽度。 |
| mentions-normal.png | 通过 | 通过 | 通过 | “欢迎 @Ada”完整显示，框宽对齐上方 840 点表单；上下留白适合三行输入，边框完整。 |
| mentions-typed.png | 通过 | 通过 | 通过 | 实际输入“欢迎 @Al”；Alan 筛选项可见；弹层与多行框等宽、紧贴框底，无水平偏移。 |
| mentions-selected.png | 通过 | 通过 | 通过 | 实际显示“欢迎 @Alan”，弹层关闭，表明已选中候选项。 |
| mentions-blurred.png | 通过 | 通过 | 通过 | 外部点击后“欢迎 @Alan”保持，弹层关闭，普通边框完整。 |
| multiline.png | 通过 | 通过 | 通过 | “第一行”“第二行”“第三行”分别占一行，第三行及下边距完整，无裁剪或额外大块空白。 |

姓名联想场景中页面底部的 Mentions 位于滚动视口边缘；其完整边框和三行内容使用后续滚动后的 mentions-* / multiline 截图验收，不将视口裁切误判为输入框自身裁剪。

## 检查命令与复现材料

已执行并通过：

```powershell
cargo build --bin desktop-app-engine --features visual-tests --locked
cargo test --lib --all-features --locked
cargo fmt --all -- --check
git diff --check
```

库测试结果：48 通过、0 失败；仅作为逻辑检查，不替代视觉结论。

定向交互计划保留在 `C:\Users\yzj123\AppData\Local\Temp\opencode\suggestions-fix-plan.json`，生成器为同目录 `create-suggestions-plan.py`；计划复用 `scripts/test-packaged-all-components.py` 的菜单、目标点击和离屏坐标点击步骤。每个场景实际执行的配置同时保留在证据根目录的 `suggestions.json`、`suggestions-scaled.json`、`suggestions-dark.json`，各截图尺寸、目标矩形、字体与后端记录见对应子目录 `render-report.json`。

## 未测范围

本轮未做 Release、原生 Glow 窗口、系统输入法、真实显示器 DPI 或隔离原生窗口验收。离屏缩放测试不能证明这些链路通过；旧 repaired-4 的全量回归结论也不扩展到本轮新产物。
