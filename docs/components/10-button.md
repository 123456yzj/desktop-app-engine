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
          {"type": "button", "id": "confirm", "text": {"kind": "literal", "value": "确认"}, "enabled": {"kind": "state", "key": "canConfirm"}, "variant": "solid", "icon": "✓", "events": {"click": "confirm"}, "style": {"background": "primary", "color": "onPrimary", "radius": 6, "paddingX": 20, "paddingY": 8, "hover": {"background": "accent"}, "disabled": {"color": "textMuted"}}},
          {"type": "button", "id": "reset", "text": {"kind": "literal", "value": "重置"}, "shape": "round", "events": {"click": "reset"}},
          {"type": "button", "id": "loading", "text": {"kind": "literal", "value": "处理中"}, "loading": true}
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

预期效果：带图标的实心确认按钮更新文字并禁用；点击圆角重置按钮恢复。“处理中”按钮因 loading=true 禁止点击。主按钮使用 primary/onPrimary 适配主题。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"button"` | 按钮 |
| `text` | 字符串绑定 | 必填 | 按钮文字 |
| `events.click` | 动作 ID 或动作绑定 | 无 | 点击触发 |
| `htmlType` | `button` / `submit` / `reset` | `button` | submit/reset 须位于 Form 内，处理校验提交或恢复初始字段值 |
| `enabled` | 布尔绑定 | true | 是否允许操作 |
| `loading` | 布尔或绑定 | false | 加载状态禁用按钮，不自动跟踪动作或请求 |
| `icon` | 字符串或绑定 | 无 | 按钮图标文字，不接受组件子树 |
| `iconPlacement` | start / end 或绑定 | start | 图标位于文字之前或之后 |
| `shape` | default / circle / round 或绑定 | default | 默认、圆形或大圆角外形 |
| `variant` | outlined / solid / dashed / filled / text / link 或绑定 | outlined | 按钮外观；dashed 当前使用普通边框 |
| `danger` | 布尔或绑定 | false | 使用危险语义色 |
| `block` | 布尔或绑定 | false | 测量时占满可用宽度，最终尺寸仍受父布局限制 |
| `href` | 字符串或绑定 | 无 | 点击时交给系统打开地址，可同时触发 click |
| `style.paddingX` / `paddingY` | 非负数 | 10 / 6 | 文字内边距 |
| `style.hover` / `pressed` / `disabled` | 状态样式 | 主题反馈 | 可指定 background/color/borderColor |

字体、对齐、尺寸及 disabled/size 见 [公共属性](02-common.md)。没有 children 或任意组件图标插槽；loading 目前通过禁用表达等待状态。

## 事件与常见问题

- click 指向当前或祖先作用域已定义的动作；普通按钮未配置 events 时只展示样式。Form 内可用 htmlType=submit/reset 触发表单处理；同时声明 click 时先执行 click，详见 [Form](32-form.md)。
- click 载荷为 null，不提供鼠标坐标；业务数据从 state 读取。一个动作可有多个步骤，但步骤读取同一次开始快照。
- 执行脚本见 [命令绑定](../10-command-binding.md)，请求接口见 [HTTP](../16-http-data-sources.md)。
- 工具栏按钮被拉高时，检查父 Row 的 align，通常设置为 center。

相关：[Row](03-row.md) · [Input](11-input.md) · [动态动作绑定](../11-dynamic-event-binding.md)。
