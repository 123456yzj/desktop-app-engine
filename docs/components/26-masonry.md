# 26 Masonry 瀑布流

[组件总览](README.md) / Masonry

将不同高度的卡片依次放入当前最短的列，适用于摘要、素材和说明卡片。与 [Grid](05-grid.md) 的统一行高不同，每列独立向下排布。

## 代码演示：响应列数、动态内容和独立间距

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "瀑布流", "window": {"width": 900, "height": 620}},
  "state": {"note": "编辑这里的文字，观察卡片高度与后续排布变化。"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "input", "id": "note", "value": {"kind": "state", "key": "note"}},
      {"type": "masonry", "id": "cards", "columnCount": 3, "minColumnWidth": 220,
        "layout": {"height": {"fr": 1}, "columnGap": 16, "rowGap": 20, "overflow": "scroll"},
        "children": [
          {"type": "card", "id": "a", "title": {"kind": "literal", "value": "动态说明"}, "layout": {"padding": 16}, "children": [
            {"type": "text", "id": "noteText", "text": {"kind": "state", "key": "note"}}
          ]},
          {"type": "card", "id": "b", "title": {"kind": "literal", "value": "较高卡片"}, "layout": {"padding": 16, "height": {"fixed": 180}}, "children": [
            {"type": "text", "id": "bText", "text": {"kind": "literal", "value": "各列无需等待相邻列结束同一行。"}}
          ]},
          {"type": "card", "id": "c", "title": {"kind": "literal", "value": "短卡片"}, "layout": {"padding": 16, "height": {"fixed": 100}}},
          {"type": "card", "id": "d", "title": {"kind": "literal", "value": "后续内容"}, "layout": {"padding": 16, "height": {"fixed": 140}}},
          {"type": "card", "id": "e", "title": {"kind": "literal", "value": "项目概况"}, "layout": {"padding": 16, "height": {"fixed": 120}}},
          {"type": "card", "id": "f", "title": {"kind": "literal", "value": "更多信息"}, "layout": {"padding": 16, "height": {"fixed": 160}}}
        ]
      }
    ]
  },
  "actions": {}
}
```

预期效果：宽窗口最多三列；缩窄后变成两列、一列，横向间距 16、纵向间距 20。动态说明换行增高时，后面的卡片重新选择最短列；超出可用高度时滚动。

## API

| 属性 | 类型 / 默认值 | 说明 |
| --- | --- | --- |
| `type` | `"masonry"`，必填 | 瀑布流容器 |
| `children` | 数组 / [] | 按声明顺序选择当前最短列；同高度优先左列 |
| `columnCount` | 整数 / 3 | 固定列数或响应布局的最大列数，1..64 |
| `minColumnWidth` | 可选正数 | 根据可用宽度自动减少列数，至少一列 |
| `layout.gap` | 非负数 / 0 | 行列间距 |
| `layout.rowGap` / `columnGap` | 可选非负数 | 分别覆盖 gap |
| `layout.align` | stretch / start / center / end | 列内水平对齐，默认 stretch |
| `layout.overflow` | clip / scroll | 默认 clip，配合有限高度使用 scroll |

响应列数为 `floor((可用宽度 + 列间距) / (最小列宽 + 列间距))`，限定在 1..columnCount；再均分剩余宽度。窗口比 minColumnWidth 更窄时单列适应窗口。组件真实高度来自文本与子树测量，固定高度和 min/max 同样有效；不把卡片拉成相同高度。

## 常见问题

- **columns 数组能用吗？** Masonry 使用 columnCount 数字；columns 轨道数组属于 Grid。
- **布局会自动更新吗？** 每次布局根据当前内容与窗口重新测量，无需 fresh 开关。隐藏节点不占列位置。
- **如何设置独立 itemRender 或指定列？** 子节点直接就是卡片内容；当前不提供指定列、列跨度或 itemRender 函数。
- **是否虚拟化？** 当前测量全部可见子节点，绘制按裁剪区处理；大量固定行高数据可使用 Table。
