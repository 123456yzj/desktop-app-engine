# 21 Flex 弹性布局

[组件总览](README.md) / Flex

用于横向或纵向的块级排列、主轴分布、交叉轴对齐、弹性分栏和换行。方向固定时，也可继续使用 [Row](03-row.md) 或 [Column](04-column.md)。主要参考 [Antdv Next Flex](https://www.antdv-next.cn/components/flex-cn) 的布局场景，配置使用本引擎的逻辑 point 和轨道对象。

## 代码演示：水平布局、对齐和弹性内容

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Flex 工具栏", "window": {"width": 720, "height": 240}},
  "state": {},
  "root": {
    "type": "flex", "id": "toolbar",
    "layout": {"padding": 24, "gap": 12, "align": "center"},
    "children": [
      {"type": "text", "id": "name", "text": {"kind": "literal", "value": "项目工作台"}, "layout": {"width": {"fr": 1}}},
      {"type": "button", "id": "refresh", "text": {"kind": "literal", "value": "刷新"}},
      {"type": "button", "id": "create", "text": {"kind": "literal", "value": "新建项目"}, "style": {"background": "primary", "color": "onPrimary"}}
    ]
  },
  "actions": {}
}
```

预期效果：标题占据按钮之外的剩余宽度，按钮沿交叉轴居中，间距 12。将标题的 `width` 改为 `{"auto":true}`，再给容器设置 `justify: "spaceEvenly"`，可看到三项等距分布。

## 代码演示：纵向布局与主轴对齐

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "纵向 Flex", "window": {"width": 480, "height": 400}},
  "state": {},
  "root": {
    "type": "flex", "id": "page", "vertical": true,
    "layout": {"padding": 24, "gap": 12, "justify": "spaceBetween", "align": "start"},
    "children": [
      {"type": "text", "id": "title", "text": {"kind": "literal", "value": "确认当前配置"}, "style": {"fontSize": 24}},
      {"type": "text", "id": "description", "text": {"kind": "literal", "value": "顶部标题、中间说明、底部操作按主轴分散排列。"}},
      {"type": "button", "id": "confirm", "text": {"kind": "literal", "value": "确认"}}
    ]
  },
  "actions": {}
}
```

预期效果：三个元素从上到下排列，两端贴近内边距，剩余高度分配给项间距。可依次替换为 `start`、`center`、`end`、`spaceAround`、`spaceEvenly`，比较六种主轴对齐。

## 代码演示：自动换行与独立行列间距

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Flex 换行", "window": {"width": 520, "height": 360}},
  "state": {},
  "root": {
    "type": "flex", "id": "tags",
    "layout": {"padding": 24, "wrap": true, "columnGap": 12, "rowGap": 24, "align": "center"},
    "children": [
      {"type": "button", "id": "a", "text": {"kind": "literal", "value": "项目 A"}, "layout": {"width": {"fixed": 140}}},
      {"type": "button", "id": "b", "text": {"kind": "literal", "value": "项目 B"}, "layout": {"width": {"fixed": 140}}},
      {"type": "button", "id": "c", "text": {"kind": "literal", "value": "项目 C"}, "layout": {"width": {"fixed": 140}}},
      {"type": "button", "id": "d", "text": {"kind": "literal", "value": "项目 D"}, "layout": {"width": {"fixed": 140}}}
    ]
  },
  "actions": {}
}
```

预期效果：同一行间距为 12，行间距为 24；缩窄窗口后每行能容纳的项目减少。Flex 可嵌套，内层方向与对齐各自配置。

## API

| 属性 | 类型 / 默认值 | 说明 |
| --- | --- | --- |
| `type` | `"flex"`，必填 | 弹性容器 |
| `vertical` | 布尔 / false | true 时纵向排列 |
| `children` | 组件数组 / [] | 顺序排列，可嵌套容器 |
| `layout.gap` | 非负数 / 0 | 通用间距 |
| `layout.rowGap` / `columnGap` | 非负数 / 回退到 gap | 分别控制纵向、横向间距 |
| `layout.justify` | start / center / end / spaceBetween / spaceAround / spaceEvenly | 主轴分布，默认 start |
| `layout.align` | stretch / start / center / end | 交叉轴对齐，默认 stretch |
| `layout.wrap` | 布尔 / false | 横向布局自动换行 |
| 子项 `width.fr` / `height.fr` | 正数轨道 | 在对应主轴按权重分配空间 |
| `state` / `actions` | 对象 / {} | 容器局部作用域 |

尺寸、样式与滚动见 [公共属性](02-common.md)。不接受业务事件。

## 常见问题

- **分散对齐没有变化？** 子项的 fr 可能已经用完剩余空间；将子项改为内容尺寸再比较。
- **是否支持 CSS 字符串？** gap 使用数值；`flex` 简写对应本引擎的轨道和 shrink；不接受 CSS 单位、DOM component、反向换行或 baseline。
- **纵向能否换列？** 目前仅横向支持 wrap。多列内容可选择 Grid 或 Masonry。
- **Space 与 Flex 如何选择？** [Space](22-space.md) 默认提供 8 point 间距和 center 对齐，适合小型操作组；Flex 默认 gap 为 0、align 为 stretch，适合页面布局。
