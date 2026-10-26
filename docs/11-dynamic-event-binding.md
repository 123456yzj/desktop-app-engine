# 状态驱动的按钮动作

events 值接受固定动作 ID、字符串 literal 绑定或 state 绑定。所有支持业务事件的组件共享该机制。

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

固定 events.click 字符串保持兼容。事件提交时从最新状态解析绑定；输入 change 先更新值再选择动作。同批事件按 sequence 排序，前一个事件改变绑定后，后一个事件执行新动作。事件失效、重复、隐藏和禁用检查继续生效。动作开始后仍执行选中动作的固定步骤列表，状态变化只影响后续事件。

加载、状态赋值时校验绑定必须为字符串、非空、路径存在且指向已定义动作。非法状态更新保持旧值与版本。运行时再次解析并校验动作，错误通过 `Runtime::error()` 获取。动作 ID 不是脚本表达式；不新增条件分支或数值运算。

`src/protocol/` 定义 `EventAction`；`src/compiler/` 校验作用域和绑定目标；`src/runtime/events.rs` 在提交时选择动作。
