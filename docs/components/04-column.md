# 04 Column 纵向容器

[组件总览](README.md) / Column

将子组件从上往下排列，适用于页面、表单和详情区。可用子项 `height.fr` 将剩余高度分配给内容区。

## 代码演示：固定头部与填满内容区

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "纵向页面", "window": {"width": 760, "height": 520}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 16, "gap": 12},
    "children": [
      {"type": "text", "id": "heading", "text": {"kind": "literal", "value": "工作区"}, "layout": {"height": {"fixed": 40}}, "style": {"fontSize": 24}},
      {"type": "panel", "id": "body", "title": {"kind": "literal", "value": "剩余空间"}, "layout": {"height": {"fr": 1}, "padding": 16}, "children": [{"type": "text", "id": "description", "text": {"kind": "literal", "value": "改变窗口高度，这块区域随之变化"}}]},
      {"type": "text", "id": "footer", "text": {"kind": "literal", "value": "底部状态栏"}, "layout": {"height": {"fixed": 28}}}
    ]
  },
  "actions": {}
}
```

预期效果：标题和底部状态栏保留固定高度，中间面板占据剩余高度。Column 默认让未固定宽度的孩子横向铺满。

## 代码演示：有限高度与滚动

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "纵向滚动", "window": {"width": 640, "height": 400}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "hint", "text": {"kind": "literal", "value": "在下面的内容区滚动"}},
      {
        "type": "column", "id": "scrollArea", "layout": {"height": {"fixed": 180}, "overflow": "scroll", "gap": 12, "padding": 8},
        "children": [
          {"type": "card", "id": "a", "title": {"kind": "literal", "value": "第一项"}, "layout": {"height": {"fixed": 100}, "padding": 12}},
          {"type": "card", "id": "b", "title": {"kind": "literal", "value": "第二项"}, "layout": {"height": {"fixed": 100}, "padding": 12}},
          {"type": "card", "id": "c", "title": {"kind": "literal", "value": "第三项"}, "layout": {"height": {"fixed": 100}, "padding": 12}}
        ]
      }
    ]
  },
  "actions": {}
}
```

预期效果：内容区高度保持 180，三张卡片超过可视区域，通过滚轮或滚动条查看。将 overflow 改为 clip 可比较裁剪效果。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"column"` | 纵向容器 |
| `children` | 组件数组 | `[]` | 按数组顺序从上到下排列 |
| `layout.gap` | 非负数 | 0 | 子项纵向间距 |
| `layout.align` | 对齐枚举 | `stretch` | 横向对齐 |
| `layout.justify` | 主轴对齐枚举 | `start` | 纵向剩余空间分配 |
| `layout.overflow` | `clip` / `scroll` | `clip` | 内容溢出处理 |
| `state` / `actions` | 对象 | `{}` | 局部状态和动作；独立模块推荐 Module |

其余见 [公共属性](02-common.md)。Column 不接受 title、columns、业务事件；wrap 不能为 true。

## 常见问题

- **内容区为什么没有撑满？** 在 Column 的直接子项上设置 height.fr；如果中间还有容器，各层都要获得有限高度。
- **内容为什么被压缩？** auto/fr 子项默认允许收缩。需要保留尺寸时用固定高度或 shrink: 0，并决定滚动或裁剪方式。
- **如何让底部靠下？** 将中间内容设为 height.fr，或者在确有剩余空间时设置 justify。

相关：[Row](03-row.md) · [Panel](06-panel.md) · [布局组合](20-recipes.md)。
