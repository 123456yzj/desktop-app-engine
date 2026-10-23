# 29 Switch 开关

用于立即启用或关闭某项设置。需要等用户提交后再统一处理的布尔选择，也可以使用 [Checkbox](27-checkbox.md)。

## 代码演示

切换第一个开关后，文字在“启用高级设置”和“高级设置已关闭”之间切换，高级区域随 state 显示或隐藏。第二个开关演示 literal 的本地状态，第三个展示加载时禁用。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Switch 示例", "window": {"width": 600, "height": 360}},
  "state": {"advanced": false},
  "root": {
    "type": "column", "id": "root", "layout": {"padding": 20, "gap": 12},
    "children": [
      {"type": "switch", "id": "advancedSwitch", "value": {"kind": "state", "key": "advanced"}, "checkedChildren": "启用高级设置", "unCheckedChildren": "高级设置已关闭"},
      {"type": "switch", "id": "localSwitch", "value": {"kind": "literal", "value": true}, "text": {"kind": "literal", "value": "仅保存在本控件"}},
      {"type": "switch", "id": "loadingSwitch", "value": {"kind": "literal", "value": true}, "text": {"kind": "literal", "value": "正在保存"}, "loading": true},
      {"type": "card", "id": "advancedPanel", "visible": {"kind": "state", "key": "advanced"}, "title": {"kind": "literal", "value": "高级设置"}, "layout": {"padding": 12}, "children": [
        {"type": "text", "id": "hint", "text": {"kind": "literal", "value": "此区域由开关绑定的布尔状态控制。"}}
      ]}
    ]
  },
  "actions": {}
}
```

## API

| 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `switch` |
| value | boolean 绑定，必填 | state 绑定顶层键；literal 为本地状态 |
| text | 字符串绑定 | 可选，显示在开关右侧 |
| loading | 布尔或绑定 | false；加载时禁用切换，由应用控制等待状态 |
| checkedChildren / unCheckedChildren | 字符串或绑定 | 未设置时使用 text；分别显示开启/关闭状态的右侧文字，不接受组件子树 |
| events.change | 动作 ID 或绑定 | 先写布尔状态，再执行动作 |

点击或聚焦后按 Space 切换，Tab 移动焦点。禁用状态不接受切换；父容器的 enabled 同样生效。其他公共属性见 [02 公共属性](02-common.md)。

## 常见问题

- loading 目前通过禁用表达等待状态，不自动跟踪保存请求；请求完成后由应用恢复 false。
- `value: "true"` 是字符串，会被拒绝。请使用 JSON 布尔值 true/false。
- 当前表单 required 规则要求布尔值为 true；允许关闭的普通设置无需 required。

[组件总览](README.md) · [Checkbox](27-checkbox.md) · [Form](32-form.md)
