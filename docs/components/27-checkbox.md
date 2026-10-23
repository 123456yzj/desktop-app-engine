# 27 Checkbox 多选框

用于同意条款或选择多个选项。单项绑定 boolean；声明 options 时作为多选组，绑定字符串数组。需要统一提交和必填校验时放入 [Form](32-form.md)。

## 代码演示

保存为 JSON 后运行。勾选“同意条款”会先更新 agreed，再用该值启用按钮；最后一项展示禁用的半选外观。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Checkbox 示例", "window": {"width": 600, "height": 360}},
  "state": {"agreed": false, "status": "等待确认"},
  "root": {
    "type": "column", "id": "root", "layout": {"padding": 20, "gap": 12},
    "children": [
      {"type": "checkbox", "id": "agreement", "value": {"kind": "state", "key": "agreed"}, "text": {"kind": "literal", "value": "同意条款"}},
      {"type": "checkbox", "id": "partial", "value": {"kind": "literal", "value": false}, "indeterminate": true, "enabled": {"kind": "literal", "value": false}, "text": {"kind": "literal", "value": "部分选中（只读示意）"}},
      {"type": "button", "id": "confirm", "enabled": {"kind": "state", "key": "agreed"}, "text": {"kind": "literal", "value": "确认"}, "events": {"click": "confirm"}},
      {"type": "text", "id": "result", "text": {"kind": "state", "key": "status"}}
    ]
  },
  "actions": {"confirm": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "已确认"}}]}}
}
```

## 代码演示：选项组

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Checkbox 选项组", "window": {"width": 540, "height": 260}},
  "state": {"selected": ["A"]},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "checkbox", "id": "group", "value": {"kind": "state", "key": "selected"}, "options": [{"label": "选项 A", "value": "A"}, {"label": "选项 B", "value": "B"}, {"label": "选项 C（禁用）", "value": "C", "disabled": true}]},
    {"type": "checkbox", "id": "mirror", "value": {"kind": "state", "key": "selected"}, "options": ["A", "B", "C"], "disabled": true}
  ]},
  "actions": {}
}
```

两组选项读取同一数组，第二组只读展示选中结果。选项 value 必须为唯一非空字符串，选中值不能重复且必须属于 options。

## API

| 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `checkbox` |
| value | boolean 或字符串数组绑定，必填 | 单项为 boolean，options 组为数组；state 只能写顶层键；literal 修改保存在该组件本地 |
| text | 字符串绑定 | 可选说明文字，点击文字也能切换 |
| options | 数组或绑定 | 未设置时为单项；支持字符串数组或 `{label, value, disabled?}` 对象数组 |
| indeterminate | boolean | false；仅影响外观，实际 value 仍为布尔值 |
| events.change | 动作 ID 或绑定 | 载荷为单项 boolean 或组选中字符串数组；先写状态，再执行动作 |

点击或聚焦后按 Space 切换；Tab 在控件间移动。公共 visible/enabled、layout/style 见 [02 公共属性](02-common.md)。

## 常见问题

- 选项组不自动提供全选与半选逻辑；indeterminate 只影响单项外观。动态删除选项时需同时调整选中值。
- 固定 `indeterminate: true` 会一直绘制半选；希望点击后去掉半选时，需要替换描述或使用普通布尔外观。
- `required: true` 的 FormItem 要求单项为 true，选项组至少选择一项。

[组件总览](README.md) · [Radio](28-radio.md) · [Switch](29-switch.md)
