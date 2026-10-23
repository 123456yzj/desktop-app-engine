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
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {
        "type": "card", "id": "summary", "title": {"kind": "literal", "value": "构建任务"}, "extra": "附加说明", "bordered": true, "hoverable": true,
        "layout": {"padding": 16, "gap": 12}, "style": {"borderWidth": 1, "borderColor": "border", "radius": 10},
        "children": [
          {"type": "metric", "id": "count", "label": {"kind": "literal", "value": "任务数量"}, "value": {"kind": "state", "key": "count"}, "format": {"type": "number", "precision": 0, "unit": " 个"}},
          {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}},
          {"type": "button", "id": "confirm", "text": {"kind": "literal", "value": "标记已确认"}, "events": {"click": "confirm"}}
        ]
      },
      {"type": "card", "id": "loadingCard", "title": {"kind": "literal", "value": "加载中的卡片"}, "loading": true, "layout": {"height": {"fixed": 80}}, "children": [
        {"type": "text", "id": "pending", "text": {"kind": "literal", "value": "加载结束后显示这段内容"}}
      ]}
    ]
  },
  "actions": {"confirm": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "已确认"}}]}}
}
```

预期效果：第一张卡片显示标题、附加文字、任务数、状态文字和按钮，悬停时突出边框；点击按钮后状态文字变为“已确认”。第二张卡片显示加载指示并隐藏正文。卡片本身没有点击事件。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"card"` | 卡片容器 |
| `title` | 字符串绑定 | 无 | 非空时显示顶部标题 |
| `loading` | 布尔或绑定 | false | 显示加载指示并隐藏子内容，不自动发起数据请求 |
| `bordered` | 布尔或绑定 | true | 绘制卡片边框 |
| `hoverable` | 布尔或绑定 | false | 悬停时突出边框 |
| `extra` | 字符串或绑定 | 空 | 标题区右侧的附加文字，不接受组件子树 |
| `children` | 组件数组 | `[]` | 默认纵向排列 |
| `state` / `actions` | 对象 | `{}` | 局部状态和动作 |
| `layout` | 容器布局 | 公共默认 | padding/gap 默认均为 0 |
| `style.background` | 颜色 | `surfaceRaised` | 默认背景 |
| `style.radius` | 非负数 | 绘制至少 6 | 卡片圆角 |

其他尺寸、样式和显隐见 [公共属性](02-common.md)。不接受 columns、wrap:true 或业务 events。

## 常见问题

- **有 header/footer 插槽吗？** 当前通过 children 中组合 Row、Text、Button 表达；title 是字符串绑定，extra 是字符串或绑定。
- **多张卡片如何对齐？** 放入 Grid，使用同样的 padding 和 gap，并根据需要为卡片设置高度。
- **卡片能封装复用逻辑吗？** 需要私有状态与父子通信时，在外层使用 Module；当前没有模板注册机制。

相关：[Grid](05-grid.md) · [Metric](13-metric.md) · [Module](08-module.md)。
