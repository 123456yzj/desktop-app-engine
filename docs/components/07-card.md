# 07 Card 卡片

[组件总览](README.md) / Card

将一个主题的指标、说明与操作聚合为卡片。适合放入 Grid 构成概览区。当前与 Panel 共用纵向布局、标题和默认外观规则。

## 代码演示：指标与操作组合

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "指标卡片", "window": {"width": 660, "height": 420}},
  "state": {"count": 12, "status": "待处理"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24},
    "children": [
      {
        "type": "card", "id": "summary", "title": {"kind": "literal", "value": "构建任务"},
        "layout": {"padding": 16, "gap": 12}, "style": {"borderWidth": 1, "borderColor": "border", "radius": 10},
        "children": [
          {"type": "metric", "id": "count", "label": {"kind": "literal", "value": "任务数量"}, "value": {"kind": "state", "key": "count"}, "format": {"type": "number", "precision": 0, "unit": " 个"}},
          {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}},
          {"type": "button", "id": "confirm", "text": {"kind": "literal", "value": "标记已确认"}, "events": {"click": "confirm"}}
        ]
      }
    ]
  },
  "actions": {"confirm": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "已确认"}}]}}
}
```

预期效果：卡片依次显示标题、任务数、状态文字和按钮。点击后状态文字变为“已确认”。卡片本身没有点击事件，操作通过内部按钮触发。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"card"` | 卡片容器 |
| `title` | 字符串绑定 | 无 | 非空时显示顶部标题 |
| `children` | 组件数组 | `[]` | 默认纵向排列 |
| `state` / `actions` | 对象 | `{}` | 局部状态和动作 |
| `layout` | 容器布局 | 公共默认 | padding/gap 默认均为 0 |
| `style.background` | 颜色 | `surfaceRaised` | 默认背景 |
| `style.radius` | 非负数 | 绘制至少 6 | 卡片圆角 |

其他尺寸、样式和显隐见 [公共属性](02-common.md)。不接受 columns、wrap:true 或业务 events。

## 常见问题

- **有 header/footer 插槽吗？** 当前通过 children 中组合 Row、Text、Button 表达；title 只接受字符串绑定。
- **多张卡片如何对齐？** 放入 Grid，使用同样的 padding 和 gap，并根据需要为卡片设置高度。
- **卡片能封装复用逻辑吗？** 需要私有状态与父子通信时，在外层使用 Module；当前没有模板注册机制。

相关：[Grid](05-grid.md) · [Metric](13-metric.md) · [Module](08-module.md)。
