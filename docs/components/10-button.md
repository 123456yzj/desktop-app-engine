# 10 Button 按钮

[组件总览](README.md) / Button

通过 click 事件触发动作。适用于更新状态、发起请求或执行外部命令。按钮本身的文字与启用状态可绑定。

## 代码演示：主按钮与禁用按钮

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "按钮操作", "window": {"width": 680, "height": 360}},
  "state": {"message": "等待操作", "canConfirm": true},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {
        "type": "row", "id": "actions", "layout": {"gap": 12, "align": "center"},
        "children": [
          {"type": "button", "id": "confirm", "text": {"kind": "literal", "value": "确认"}, "enabled": {"kind": "state", "key": "canConfirm"}, "events": {"click": "confirm"}, "style": {"background": "primary", "color": "onPrimary", "radius": 6, "paddingX": 20, "paddingY": 8, "hover": {"background": "accent"}, "disabled": {"color": "textMuted"}}},
          {"type": "button", "id": "reset", "text": {"kind": "literal", "value": "重置"}, "events": {"click": "reset"}}
        ]
      },
      {"type": "text", "id": "message", "text": {"kind": "state", "key": "message"}}
    ]
  },
  "actions": {
    "confirm": {"steps": [{"type": "set", "target": "message", "value": {"kind": "literal", "value": "已确认"}}, {"type": "set", "target": "canConfirm", "value": {"kind": "literal", "value": false}}]},
    "reset": {"steps": [{"type": "set", "target": "message", "value": {"kind": "literal", "value": "等待操作"}}, {"type": "set", "target": "canConfirm", "value": {"kind": "literal", "value": true}}]}
  }
}
```

预期效果：确认后文字更新，确认按钮禁用；点击重置恢复。主按钮使用 primary/onPrimary 适配主题。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"button"` | 按钮 |
| `text` | 字符串绑定 | 必填 | 按钮文字 |
| `events.click` | 动作 ID 或动作绑定 | 无 | 点击触发 |
| `enabled` | 布尔绑定 | true | 是否允许操作 |
| `style.paddingX` / `paddingY` | 非负数 | 10 / 6 | 文字内边距 |
| `style.hover` / `pressed` / `disabled` | 状态样式 | 主题反馈 | 可指定 background/color/borderColor |

字体、对齐、尺寸等见 [公共属性](02-common.md)。没有 children、图标插槽或内置 loading 字段；可用绑定文字和 enabled 表达任务状态。

## 事件与常见问题

- click 指向当前或祖先作用域已定义的动作；没有 events 的按钮只展示样式，不执行业务操作。
- 普通 click 动作不使用 event 绑定，直接读取 state。一个动作可有多个步骤，但步骤读取同一次开始快照。
- 执行脚本见 [命令绑定](../10-command-binding.md)，请求接口见 [HTTP](../16-http-data-sources.md)。
- 工具栏按钮被拉高时，检查父 Row 的 align，通常设置为 center。

相关：[Row](03-row.md) · [Input](11-input.md) · [动态动作绑定](../11-dynamic-event-binding.md)。
