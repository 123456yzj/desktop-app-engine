# 状态驱动的按钮动作

0.2 的 events 值现在接受固定动作 ID、字符串 literal 绑定或 state 绑定。所有支持业务事件的组件共享该机制，0.1 保持原协议。

```json
{
  "type": "button",
  "id": "advance",
  "text": {"kind":"state","key":"wizard","path":"/label"},
  "enabled": {"kind":"state","key":"wizard","path":"/enabled"},
  "events": {"click":{"kind":"state","key":"wizard","path":"/action"}}
}
```

wizard 可以初始化为 `{"label":"下一步","action":"toSecond","enabled":true,"page":"第一步"}`。动作通过一次 set 更新整个 wizard 对象，让文字、点击动作、禁用状态和页面说明同步变化。使用多个独立状态键仍受已有逐步骤提交语义约束；需要同步变化时应组合到同一状态对象。

完整示例 examples/step-button.json：同一按钮连续点击，依次进入第二步、第三步、执行本地提交演示；最后按钮显示“已提交”并禁用。另一个固定绑定的按钮重置流程。演示不执行外部命令或真实数据提交。

固定 events.click 字符串保持兼容。事件提交时从最新状态解析绑定；输入 change 先更新值再选择动作。同批事件按 sequence 排序，前一个事件改变绑定后，后一个事件执行新动作。事件失效、重复、隐藏和禁用检查继续生效。动作开始后仍执行选中动作的固定步骤列表，状态变化只影响后续事件。

加载、状态赋值时校验绑定必须为字符串、非空、路径存在且指向已定义动作。非法状态更新保持旧值与版本。运行时再次解析并校验动作，错误通过 runtime.error 显示。动作 ID 不是脚本表达式；不新增条件分支、数值运算或自动异步完成回调。

改动：src/v02.rs（EventAction 类型与语义校验）、src/interaction.rs（提交时动态解析）、docs/description-0.2.schema.json、docs/05-description-format.md、examples/step-button.json、tests/dynamic_events.rs、scripts/schema-verify.py、README.md。

## 验证与产物

本地 GNU 离线工具链，实际完成以下检查，均退出 0：

| 命令 | 结果 |
| --- | --- |
| cargo fmt -- --check | 通过 |
| cargo clippy --all-targets --locked --offline -- -D warnings | 通过，无警告 |
| cargo test --locked --offline | 38 passed，0 failed，0 ignored（新增 4 项） |
| python scripts/schema-verify.py | Dashboard、命令、步骤按钮示例和非法结构用例通过 |
| cargo build --release --locked --offline | Finished release，19.81s |
| scripts/package.ps1 -Description examples/step-button.json -Output exports -Name StepButtonApp | 导出及独立包 --check 通过 |

测试覆盖同一按钮按状态切换动作与文字、禁用后不执行、固定重置动作、literal 事件绑定、非法状态更新保持原值与版本、输入更新后解析动作，以及同批事件按顺序切换动作。测试调用事件提交与实际组件属性解析路径，不操作桌面鼠标或窗口。构建与测试日志：evidence/dynamic-events-verification.txt。

最新运行时：target-verify/release/desktop-app-engine.exe。可直接运行的步骤演示包：exports/StepButtonApp/StepButtonApp.exe，同目录包含 app.json 和 README.txt。此前导出包为历史构建，不自动替换。
