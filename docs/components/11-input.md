# 11 Input 输入框

[组件总览](README.md) / Input

编辑单行字符串。value 绑定可写状态时，输入自动写回状态，其他引用同一状态的组件随之刷新。

## 代码演示：实时预览与 Enter 提交

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "输入联动", "window": {"width": 680, "height": 380}},
  "state": {"query": "", "submitted": "尚未提交"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "input", "id": "query", "value": {"kind": "state", "key": "query"}, "placeholder": {"kind": "literal", "value": "输入关键词后按 Enter"}, "events": {"submit": "submit"}},
      {"type": "text", "id": "live", "text": {"kind": "state", "key": "query"}},
      {"type": "text", "id": "submitted", "text": {"kind": "state", "key": "submitted"}}
    ]
  },
  "actions": {"submit": {"steps": [{"type": "concat", "target": "submitted", "parts": [{"kind": "literal", "value": "已提交："}, {"kind": "state", "key": "query"}]}]}}
}
```

预期效果：输入时第一段预览立即变化；按 Enter 后第二段文字显示本次提交值。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"input"` | 单行输入 |
| `value` | 字符串绑定 | 必填 | 可写 state 或 literal 本地编辑值 |
| `placeholder` | 字符串绑定 | 空 | 输入提示 |
| `events.change` | 动作 ID 或动作绑定 | 无 | 新值先写入状态，再触发动作 |
| `events.submit` | 动作 ID 或动作绑定 | 无 | Enter 提交 |

自然测量尺寸为 220×34，最终尺寸由父布局和 layout 决定。颜色、字体、边框、圆角与状态样式见 [公共属性](02-common.md)。

## 值与事件规则

- 可写 state 的 path 必须为空；编辑对象内字段时先使用独立字符串状态组织数据。
- 模块内可绑定局部字符串状态；只读 prop 不可作为 value。
- literal 仍可编辑，但只保存在组件本地缓冲，不写入业务状态。需要在动作中读取输入时使用 state。
- change 动作从 state 读取已经写入的新值；不通过 event 绑定读取输入载荷。

## 常见问题

- **可以输入数字、密码、多行吗？** 当前是单行字符串输入，没有数字解析、密码模式或多行字段。
- **输入长度有限制吗？** 控件最多接收 16,384 个 Unicode 字符；字符串状态还有协议大小限制。
- **需要搜索防抖吗？** 当前没有 debounce 配置；可使用 submit 或按钮明确提交。

相关：[Text](09-text.md) · [Button](10-button.md) · [Module](08-module.md)。
