# 13 Metric 指标

[组件总览](README.md) / Metric

以小标签和放大数值展示统计量。适用于任务数、资源占用、金额或吞吐量，常与 Card、Grid 组合。

## 代码演示：数字和百分比

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "指标展示", "window": {"width": 760, "height": 340}},
  "state": {"tasks": 128, "usage": 0.426},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24},
    "children": [
      {"type": "grid", "id": "metrics", "columns": [{"fr": 1}, {"fr": 1}], "layout": {"gap": 16}, "children": [
        {"type": "card", "id": "tasksCard", "layout": {"padding": 16}, "children": [{"type": "metric", "id": "tasks", "label": {"kind": "literal", "value": "任务数量"}, "value": {"kind": "state", "key": "tasks"}, "format": {"type": "number", "precision": 0, "unit": " 个"}}]},
        {"type": "card", "id": "usageCard", "layout": {"padding": 16}, "children": [{"type": "metric", "id": "usage", "label": {"kind": "literal", "value": "CPU 使用率"}, "value": {"kind": "state", "key": "usage"}, "format": {"type": "percent", "precision": 1}}]}
      ]},
      {"type": "metric", "id": "balance", "label": {"kind": "literal", "value": "账户余额"}, "value": {"kind": "literal", "value": 128}, "prefix": "¥", "suffix": " 元", "format": {"type": "number", "precision": 2}},
      {"type": "metric", "id": "pending", "label": {"kind": "literal", "value": "待加载指标"}, "value": {"kind": "literal", "value": 0}, "loading": true}
    ]
  },
  "actions": {}
}
```

预期效果：任务数量显示 `128 个`，CPU 使用率显示 `42.6%`，余额显示 `¥128.00 元`，最后一个指标显示加载指示。percent 接收比例数值，显示时乘 100。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"metric"` | 数值指标 |
| `label` | 字符串绑定 | 必填 | 指标说明 |
| `value` | 数字绑定 | 必填 | 只接受有限数字 |
| `prefix` / `suffix` | 字符串或绑定 | 空 | 在格式化结果前后追加文字，不改变 value |
| `loading` | 布尔或绑定 | false | 显示加载指示；value 仍须提供合法数字 |
| `format.type` | text / number / percent | text | 显示格式 |
| `format.precision` | 整数 0..12 | 0 | 小数位；text 格式只能为 0 |
| `format.unit` | 字符串 | 空 | 格式化后追加的单位 |

基础字号默认 16，数值约为基础字号的 1.875 倍。其他尺寸、颜色见 [公共属性](02-common.md)。没有业务事件。

## 常见问题

- **为什么字符串 "42" 不合法？** 数值不做隐式转换，请在状态中提供数字。
- **能显示空值吗？** Metric 的 value 必须是数字；需要空值提示时可用 Text 与 visible 组织界面。
- **可以格式化日期或货币吗？** 当前没有日期或本地化货币格式；固定货币符号可使用 prefix，单位可使用 suffix 或 format.unit，避免重复添加。

相关：[Card](07-card.md) · [Grid](05-grid.md) · [Table 格式](16-table.md)。
