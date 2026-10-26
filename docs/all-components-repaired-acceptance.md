# All Components：四类缺陷修复与 183 张图片回归验收

验收日期：2026-10-26。**最终批 183/183 张原始 PNG 已逐张查看，183 pass / 0 fail / 0 uncertain；本批覆盖场景通过。**

本报告承接 [旧 180 张图片验收](all-components-image-acceptance.md)，保留旧批 162 pass / 15 fail / 3 uncertain 的历史结论。通过范围是本批实际产物离屏图片及其输入序列，不等于 84 种组件全部状态验收。

## 修复与实际证据

| 项目 | 根因及修改 | 最终图片证据（相对最终图片根目录） |
| --- | --- | --- |
| F1 通知遮挡指南 | 固定底部偏移不考虑 FloatButton；按当前可见浮动按钮的实际内容、动态 offset 测量预留区域，底部通知向上避让并保留 8 点间隔。首轮修复仍低估标题行高度，进一步让测量和绘制共用相同最小行高。 | `notification{,-scaled,-dark}/{opened,settled}.png`、`notification-close{,-scaled,-dark}/{opened,closed}.png`；主体不相交，圆钮顶部及文字完整，关闭后通知与阴影消失。记录底边 y763 / 圆钮顶部 y771，scaled 为 14 原图像素间隔。 |
| F2 日期低对比 | DatePicker Area 未继承组件作用域的浅色 style；弹层显式继承当前 style。 | `date-dark/opened.png`、`date-select-dark/opened.png`；星期及当月日期为清晰深色，选择回显 `2026-10-23`，清除恢复占位。 |
| F3 图表刻度截字 | 浮点计算尾数未经格式化且标签可换行；刻度按 12 有效位格式化、去尾零和负零，限制科学记数，并按相邻刻度间距单行布局。 | `chart{,-scaled,-dark}/page-scroll.png`；横轴为完整短数字，无长尾、换行残迹或内部裁剪。 |
| F4 Calendar 今天裁剪 | 示例强制 300 点高度，且自然高度估算未覆盖实际网格与今天行；改为按字体、模式、周行、间距及 showToday 测量，示例使用自然高度。 | `form/scroll-4.png`；今天文字、下边框和圆角完整。 |
| 单字符时间补零 | 旧计划直接输入 `05`，没有证明补零；交互计划改为只输入 `5`，保留提交与失焦截图。 | `time-edit{,-scaled,-dark}/{edited,submitted,blurred}.png`；编辑中为 `5`，Enter 后为 `05`，随后失焦保持 `05`。不额外声称未按 Enter 直接失焦路径已测。 |

相关实现：`src/components/chart.rs`、`src/components/egui/charts.rs`、`src/components/egui/widgets/{float_button,date_picker}.rs`、`src/components/egui/widgets/shared/toast.rs`、`src/components/layout/measure.rs`。渲染报告中的浮动目标记录真实 Area bounds，以便通知移位后仍使用目标相对位置关闭。

## 最终产物与方法

- 实际 EXE：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-repaired-4\all-components.exe`。
- SHA-256：`7a962d8e3d9128a2d788cf219b229933ae6f12bd3479c31b594fc7f4fd650ad5`。
- 图片根目录：`C:\Users\yzj123\AppData\Local\Temp\opencode\all-components-repaired-final`。
- 通过打包流程自身 `--check-plugins` 校验，再启动上述实际 EXE 的 `--visual-test`，从其内置定义生成全部图片；各报告均确认 `definition: embedded`，并核对 executable 与产物清单一致。
- 构建类型 Debug；`egui_kittest 0.31.1 / wgpu`，Dx12 / CPU / Microsoft Basic Render Driver，driver `10.0.26100.9711`。离屏执行未占用用户桌面的鼠标、键盘或前台窗口。
- 逻辑尺寸 1360×900。light 1x、light 1.75x、dark 宿主 1x；PNG 原尺寸分别为 1360×900 和 2380×1575。Read 大图预览缩至 2000×1324，分报告的预览坐标不代表原图坐标。
- 示例 ConfigProvider 固定浅色，dark 仅表示深色宿主，不构成组件全面深色主题验收。
- 六个子代理独占分组逐张用 Read 打开原始 PNG；主代理复看关键缺陷及时间编辑序列。联系表仅为导航，没有代替原图。没有执行自动像素快照差异判定。

## 逐图记录

| 报告 | 查看 | pass | fail | uncertain |
| --- | ---: | ---: | ---: | ---: |
| [六分区页面](all-components-repaired-review-pages.md) | 30 | 30 | 0 | 0 |
| [Tag、Transfer、Chart](all-components-repaired-review-charts-transfer.md) | 36 | 36 | 0 | 0 |
| [弹层与录入初态](all-components-repaired-review-overlays-inputs.md) | 30 | 30 | 0 | 0 |
| [级联、联想及日期交互](all-components-repaired-review-selection-interactions.md) | 33 | 33 | 0 | 0 |
| [Modal 及时间编辑](all-components-repaired-review-modal-time.md) | 36 | 36 | 0 | 0 |
| [Popconfirm 及通知交互](all-components-repaired-review-feedback-interactions.md) | 18 | 18 | 0 | 0 |
| **总计** | **183** | **183** | **0** | **0** |

相较旧 180 张，新增三组时间失焦图，共多 3 张。校核依据各目录 `render-report.json` 的 captures：183 张均有且只有一条逐图记录，遗漏 0、重复 0；全部原图尺寸、图片 SHA-256、EXE SHA-256 均已记录。

本地入口（最终图片根目录）：

- `index.html`：场景、原图、渲染报告和实际 EXE 链接。
- `per-image-review.html`：183 条人工逐图结论及图片链接。
- `per-image-review.json`：逐图结论、观察、尺寸、哈希、对应报告与产物信息。
- `interaction-plan.json`：本轮交互输入，时间发送文本为 `5`。
- 清单复核脚本：`C:\Users\yzj123\AppData\Local\Temp\opencode\audit-repaired-reviews.py`。仅校核已有证据，不自动判定视觉通过。

## 构建与执行记录

- `cargo fmt --all`：完成。
- `cargo test --all-targets --all-features --locked`：48 个库测试通过，0 失败；其余两个二进制目标无单元测试。
- `cargo build --bin desktop-app-engine --features visual-tests --locked`：通过。
- `git diff --check`：通过。
- 实际打包及最终回归命令：

```powershell
python "scripts/test-packaged-all-components.py" --engine "target/debug/desktop-app-engine.exe" --output "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4" --plan "C:/Users/yzj123/AppData/Local/Temp/opencode/notification-recheck-plan.json"
python "scripts/test-packaged-all-components.py" --exe "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-4/all-components.exe" --output "C:/Users/yzj123/AppData/Local/Temp/opencode/all-components-repaired-final" --with-interactions
```

过程中的失败与恢复：

1. `all-components-repaired-1`、`-2` 的打包自校验曾因插件 5 秒响应超时失败；旧 EXE 也曾复现。只读检查进程归属并对插件做独立 RPC 探测，未杀用户 Node 进程，未修改超时或跳过校验。后续旧产物检查及 `-3`、`-4` 打包均成功；确切超时根因未确认，不能声称该运行环境问题已修复。
2. `-3` 的首次全量采集被外层 120 秒执行限制中断，随后使用该 EXE 在独立目录 `all-components-repaired-captures-1` 完成 183 张。首轮逐图为 174 pass / 9 fail，通知仍相交约 4 点；该批不作为最终通过证据。
3. 修正标题行测量后重新构建、打包 `-4`，先检查六张通知图，再使用相同新 EXE 在全新目录生成最终 183 张，重新逐张验收。旧图片均保留。

## 未测范围

原生 Glow 窗口、实际系统输入与显示器 DPI、Release；通知其他角、多个 toast 的互相堆叠与极小视口；分组/单点柱图；纵向 Descriptions；Transfer 搜索过滤及禁用状态；Modal 取消回滚；时间不按 Enter 直接失焦；其余 84 种组件的未覆盖状态组合。

编译及逻辑测试结果不作为视觉通过依据。本批最终未发现明确视觉失败项；历史失败和环境超时边界如上保留。
