# 15 Progress 进度条

[组件总览](README.md) / Progress

展示已知完成比例，value 范围为 0..1，可附带字符串说明。

## 代码演示：更新进度与说明

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "任务进度", "window": {"width": 660, "height": 340}},
  "state": {"progress": 0.35, "progressText": "已完成 35%"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "progress", "id": "progress", "value": {"kind": "state", "key": "progress"}, "text": {"kind": "state", "key": "progressText"}},
      {"type": "button", "id": "complete", "text": {"kind": "literal", "value": "模拟完成"}, "events": {"click": "complete"}}
    ]
  },
  "actions": {"complete": {"steps": [{"type": "set", "target": "progress", "value": {"kind": "literal", "value": 1}}, {"type": "set", "target": "progressText", "value": {"kind": "literal", "value": "已完成 100%"}}]}}
}
```

预期效果：初始填充约 35%，点击后进度条填满且说明同步更新。显式 text 独立绑定；条形未提供文字时自动显示百分比。

## 代码演示：圆形与仪表盘

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "进度形态", "window": {"width": 520, "height": 260}},
  "state": {},
  "root": {"type": "row", "id": "page", "layout": {"padding": 24, "gap": 24}, "children": [
    {"type": "progress", "id": "circle", "value": {"kind": "literal", "value": 0}, "percent": 65, "progressType": "circle", "status": "success", "strokeWidth": 6, "layout": {"width": {"fixed": 120}, "height": {"fixed": 120}}},
    {"type": "progress", "id": "dashboard", "value": {"kind": "literal", "value": 0.4}, "progressType": "dashboard", "status": "exception", "showInfo": false, "layout": {"width": {"fixed": 120}, "height": {"fixed": 120}}}
  ]},
  "actions": {}
}
```

圆形显示成功色的 65%，由 percent 覆盖 value；仪表盘显示异常色的 40% 且隐藏数字。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"progress"` | 进度条 |
| `value` | 数字绑定，0..1 | 必填 | 0 为空，1 为完成 |
| `text` | 字符串绑定 | 无 | 进度说明 |
| `percent` | 数字 0..100 或绑定 | 未设置 | 覆盖显示比例；value 仍必填且必须在 0..1 |
| `showInfo` | 布尔或绑定 | true | 是否显示进度文字 |
| `status` | normal / active / success / exception 或绑定 | normal | success/exception 使用成功/危险色；active 当前与 normal 相同，无条纹动画 |
| `progressType` | line / circle / dashboard 或绑定 | line | 条形、圆形或仪表盘 |
| `strokeWidth` | 数字 1..64 或绑定 | 8 | 进度线宽，单位为 point |

自然尺寸 160×32，父 Column 默认可将其横向铺满。圆形和仪表盘建议显式设置宽高；其数字固定显示百分比，不使用 text。默认使用主题 accent 色，轨道使用 border 色。其他公共属性见 [公共配置](02-common.md)。没有业务事件。

## 常见问题

- **value 可以写 35 吗？** 不可以，35% 应为 0.35；超出范围会校验失败。
- **有循环加载或不确定进度模式吗？** 确定比例支持条形、圆形与仪表盘；不确定加载可使用 [Spin](36-native-feedback.md)。
- **进程启动后会自动跟踪进度吗？** 不会，需要通过状态更新提供真实进度；普通 command 结果只报告任务状态和输出。

相关：[Metric](13-metric.md) · [Button](10-button.md) · [HTTP 数据源](../16-http-data-sources.md)。
