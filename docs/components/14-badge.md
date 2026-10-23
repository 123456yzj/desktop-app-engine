# 14 Badge 状态标签

[组件总览](README.md) / Badge

以浅色背景和语义色文字展示短状态，例如成功、等待或失败。

## 代码演示：不同状态

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "状态标签", "window": {"width": 700, "height": 300}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24},
    "children": [
      {"type": "row", "id": "statuses", "layout": {"gap": 12, "align": "center", "wrap": true}, "children": [
        {"type": "badge", "id": "running", "text": {"kind": "literal", "value": "执行中"}},
        {"type": "badge", "id": "success", "text": {"kind": "literal", "value": "成功"}, "tone": "success"},
        {"type": "badge", "id": "waiting", "text": {"kind": "literal", "value": "等待"}, "tone": "warning"},
        {"type": "badge", "id": "failed", "text": {"kind": "literal", "value": "失败"}, "tone": "danger"},
        {"type": "badge", "id": "skipped", "text": {"kind": "literal", "value": "未执行"}, "tone": "muted"}
      ]}
    ]
  },
  "actions": {}
}
```

预期效果：显示五种颜色的单行标签，文字居中，窗口不足时随 Row 换行。

## 代码演示：选择与关闭

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "可选标签", "window": {"width": 540, "height": 260}},
  "state": {"selected": false, "status": "点击标签选择，点击 × 关闭"},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "badge", "id": "tag", "text": {"kind": "literal", "value": "可选标签"}, "icon": "✓", "checkable": true, "checked": {"kind": "state", "key": "selected"}, "closable": true, "events": {"change": "changed", "close": "closed"}},
    {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}}
  ]},
  "actions": {
    "changed": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "选择状态已更新"}}]},
    "closed": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "标签已关闭"}}]}
  }
}
```

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"badge"` | 状态标签 |
| `text` | 字符串绑定 | 必填 | 标签文字 |
| `tone` | accent / success / warning / danger / muted | accent | 语义色，固定枚举，不是绑定 |
| `closable` | 布尔或绑定 | false | 显示关闭按钮，关闭后保存在控件本地隐藏状态 |
| `checkable` | 布尔或绑定 | false | 点击标签切换 checked |
| `checked` | 布尔或绑定 | true | 选中外观；state 绑定时 change 自动写回 |
| `icon` | 字符串或绑定 | 空 | 标签文字之前的图标文字 |
| `events.change` | 动作 ID 或绑定 | 无 | 载荷为新 checked 布尔值，先写状态再执行动作 |
| `events.close` | 动作 ID 或绑定 | 无 | 关闭时执行，载荷为 null，不改变 checked |

文字和背景的标签语义色由 tone 决定；公共尺寸和 visible 等见 [公共属性](02-common.md)。未绑定 state 的 checked 保存在控件本地。关闭没有取消回调，也没有单独的受控关闭属性。

## 常见问题

- **可以把 tone 绑定到状态吗？** 当前 tone 是静态枚举。动态状态可使用多个不同 tone 的 Badge 配合布尔 visible，或由原生宿主生成视图。
- **有计数气泡、小圆点或关闭按钮吗？** 本组件对应 Tag，closable 显示关闭按钮；计数与圆点使用 [BadgeCount](35-native-display.md)。

相关：[Text](09-text.md) · [Progress](15-progress.md) · [公共显隐](02-common.md)。
