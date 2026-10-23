# 11 Input 输入框

[组件总览](README.md) / Input

编辑单行、多行或密码字符串。value 绑定可写状态时，输入自动写回状态，其他引用同一状态的组件随之刷新。

## 代码演示：实时预览与 Enter 提交

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "输入联动", "window": {"width": 680, "height": 380}},
  "state": {"query": "", "submitted": "尚未提交"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "input", "id": "query", "value": {"kind": "state", "key": "query"}, "placeholder": {"kind": "literal", "value": "输入关键词后按 Enter"}, "allowClear": true, "showCount": true, "maxLength": 40, "prefix": "搜索", "status": "warning", "events": {"submit": "submit"}},
      {"type": "text", "id": "live", "text": {"kind": "state", "key": "query"}},
      {"type": "text", "id": "submitted", "text": {"kind": "state", "key": "submitted"}}
    ]
  },
  "actions": {"submit": {"steps": [{"type": "concat", "target": "submitted", "parts": [{"kind": "literal", "value": "已提交："}, {"kind": "state", "key": "query"}]}]}}
}
```

预期效果：输入框带搜索前缀、警告边框、字符计数和清除按钮。输入时预览立即变化；按 Enter 后第二段文字显示本次提交值。

## 代码演示：多行、密码与只读

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "输入模式", "window": {"width": 640, "height": 420}},
  "state": {"notes": "第一行\n第二行", "password": "", "reference": "只读内容"},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "input", "id": "notes", "value": {"kind": "state", "key": "notes"}, "multiline": true, "rows": 3, "maxLength": 200, "showCount": true},
    {"type": "input", "id": "password", "value": {"kind": "state", "key": "password"}, "password": true, "placeholder": {"kind": "literal", "value": "输入密码"}},
    {"type": "input", "id": "reference", "value": {"kind": "state", "key": "reference"}, "readOnly": true, "suffix": "只读"}
  ]},
  "actions": {}
}
```

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"input"` | 字符串输入 |
| `value` | 字符串绑定 | 必填 | 可写 state 或 literal 本地编辑值 |
| `placeholder` | 字符串绑定 | 空 | 输入提示 |
| `allowClear` | 布尔或绑定 | false | 显示清除按钮；只读时禁止清除 |
| `password` | 布尔或绑定 | false | 掩码显示内容，value 仍为原始字符串 |
| `multiline` | 布尔或绑定 | false | 多行编辑 |
| `rows` | 整数 1..100 或绑定 | 3 | 多行模式的自然高度行数 |
| `maxLength` | 整数 1..1048576 或绑定 | 16384 | 输入与程序赋值的 Unicode 字符数上限；仍受协议状态大小约束 |
| `readOnly` | 布尔或绑定 | false | 禁止编辑与清除 |
| `prefix` / `suffix` | 字符串或绑定 | 空 | 前后缀文字，不写入 value |
| `showCount` | 布尔或绑定 | false | 显示当前字符数和 maxLength |
| `status` | default / error / warning 或绑定 | default | 默认、错误或警告边框 |
| `events.change` | 动作 ID 或动作绑定 | 无 | 新值先写入状态，再触发动作 |
| `events.submit` | 动作 ID 或动作绑定 | 无 | Enter 提交 |
| `events.focus` / `events.blur` | 动作 ID 或绑定 | 无 | 获得或失去编辑焦点，载荷为 null |
| `events.clear` | 动作 ID 或绑定 | 无 | 清除按钮触发，载荷为 null；先提交 change，再提交 clear |

自然测量尺寸为 220×34，最终尺寸由父布局和 layout 决定。颜色、字体、边框、圆角与状态样式见 [公共属性](02-common.md)。

## 值与事件规则

- 可写 state 的 path 必须为空；编辑对象内字段时先使用独立字符串状态组织数据。
- 模块内可绑定局部字符串状态；只读 prop 不可作为 value。
- literal 仍可编辑，但只保存在组件本地缓冲，不写入业务状态。需要在动作中读取输入时使用 state。
- change 载荷为新字符串，动作可从 state 读取，也可用 event Binding；使用 event 时须声明 eventPayload 字符串样本。submit 载荷为 null。

## 常见问题

- **可以输入数字、密码、多行吗？** value 仍为字符串；password 开启密码模式，multiline/rows 配置多行。数字输入使用 [InputNumber](30-input-number.md)。
- **输入长度有限制吗？** 默认上限为 16,384 个 Unicode 字符，可通过 maxLength 调整；字符串状态还有协议大小限制。
- **需要搜索防抖吗？** 当前没有 debounce 配置；可使用 submit 或按钮明确提交。

相关：[Text](09-text.md) · [Button](10-button.md) · [Module](08-module.md)。
