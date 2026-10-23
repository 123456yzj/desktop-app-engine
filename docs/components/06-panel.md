# 06 Panel 面板

[组件总览](README.md) / Panel

为一组内容提供背景与可选标题，内部默认纵向排列。适用于设置区、列表区和滚动详情。当前 Panel 与 [Card](07-card.md) 共用布局和默认外观规则，可按内容语义选择。

## 代码演示：标题固定、正文滚动

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "滚动面板", "window": {"width": 680, "height": 420}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 12},
    "children": [
      {
        "type": "panel", "id": "details", "title": {"kind": "literal", "value": "操作说明"}, "extra": "共三步", "bordered": true, "hoverable": true,
        "layout": {"height": {"fixed": 240}, "padding": 16, "gap": 12, "overflow": "scroll"},
        "children": [
          {"type": "text", "id": "step1", "layout": {"height": {"fixed": 90}}, "text": {"kind": "literal", "value": "第一步：选择目标项目。"}},
          {"type": "text", "id": "step2", "layout": {"height": {"fixed": 90}}, "text": {"kind": "literal", "value": "第二步：检查配置。"}},
          {"type": "text", "id": "step3", "layout": {"height": {"fixed": 90}}, "text": {"kind": "literal", "value": "第三步：开始执行并查看日志。"}}
        ]
      }
    ]
  },
  "actions": {}
}
```

预期效果：标题保留在面板顶部，正文在剩余区域滚动。默认具有 raised surface 背景与至少 6 point 圆角；padding 和 gap 需要显式设置。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"panel"` | 面板容器 |
| `title` | 字符串绑定 | 无 | 非空标题占据独立标题区 |
| `loading` | 布尔或绑定 | false | 显示加载指示并隐藏子内容，状态由应用控制 |
| `bordered` | 布尔或绑定 | true | 绘制面板边框 |
| `hoverable` | 布尔或绑定 | false | 悬停时突出边框，不产生点击事件 |
| `extra` | 字符串或绑定 | 空 | 标题区右侧的附加文字，不接受组件子树 |
| `children` | 组件数组 | `[]` | 默认纵向排列 |
| `state` / `actions` | 对象 | `{}` | 局部状态和动作 |
| `layout` | 容器布局 | 公共默认 | 支持 gap、对齐、滚动，wrap 不能为 true |
| `style.background` | 颜色 | `surfaceRaised` | 面板默认背景 |
| `style.radius` | 非负数 | 绘制至少 6 | 面板圆角 |

其余见 [公共属性](02-common.md)。没有业务事件、折叠或可拖拽分割 API。

## 常见问题

- **如何把正文横向排布？** Panel 的 children 中再放一个 Row 或 Grid。
- **为什么设置标题后可用正文变少？** 标题会从内容高度中扣除，设置固定高度时要为标题、padding 和滚动槽留出空间。
- **如何动态修改标题或显示加载？** title 使用 state 字符串绑定；loading 可绑定布尔状态，加载结束后设为 false。示例和 [Card](07-card.md) 使用同一套加载规则。

相关：[Card](07-card.md) · [Column](04-column.md) · [布局组合](20-recipes.md)。
