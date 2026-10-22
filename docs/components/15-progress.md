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

预期效果：初始填充约 35%，点击后进度条填满且说明同步更新。说明文字独立绑定，不会自动由 value 格式化生成。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"progress"` | 进度条 |
| `value` | 数字绑定，0..1 | 必填 | 0 为空，1 为完成 |
| `text` | 字符串绑定 | 无 | 进度说明 |

自然尺寸 160×32，父 Column 默认可将其横向铺满。当前条形使用主题 accent 色，轨道使用 border 色。其他公共属性见 [公共配置](02-common.md)。没有业务事件。

## 常见问题

- **value 可以写 35 吗？** 不可以，35% 应为 0.35；超出范围会校验失败。
- **有循环加载或不确定进度模式吗？** 当前只支持确定比例，没有动画或环形配置。
- **进程启动后会自动跟踪进度吗？** 不会，需要通过状态更新提供真实进度；普通 command 结果只报告任务状态和输出。

相关：[Metric](13-metric.md) · [Button](10-button.md) · [HTTP 数据源](../16-http-data-sources.md)。
