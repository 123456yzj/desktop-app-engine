# 05 Grid 网格

[组件总览](README.md) / Grid

按固定列轨道从左到右、从上到下放置孩子。适用于指标卡片、多列信息和固定侧栏加弹性列。列数由 columns 明确指定。

## 代码演示：两列卡片

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "卡片网格", "window": {"width": 800, "height": 480}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {
        "type": "grid", "id": "cards", "columns": [{"fr": 1}, {"fr": 1}], "layout": {"gap": 16},
        "children": [
          {"type": "card", "id": "a", "title": {"kind": "literal", "value": "项目 A"}, "layout": {"padding": 16}, "children": [{"type": "text", "id": "aText", "text": {"kind": "literal", "value": "第一行左侧"}}]},
          {"type": "card", "id": "b", "title": {"kind": "literal", "value": "项目 B"}, "layout": {"padding": 16}, "children": [{"type": "text", "id": "bText", "text": {"kind": "literal", "value": "第一行右侧"}}]},
          {"type": "card", "id": "c", "title": {"kind": "literal", "value": "项目 C"}, "layout": {"padding": 16}, "children": [{"type": "text", "id": "cText", "text": {"kind": "literal", "value": "第二行左侧"}}]},
          {"type": "card", "id": "d", "title": {"kind": "literal", "value": "项目 D"}, "layout": {"padding": 16}, "children": [{"type": "text", "id": "dText", "text": {"kind": "literal", "value": "第二行右侧"}}]}
        ]
      }
    ]
  },
  "actions": {}
}
```

预期效果：两列等宽卡片，共两行，行列间距均为 16。默认每行高度由该行内容所需最大高度决定。

## 常用变体

| 需求 | 修改 |
| --- | --- |
| 三等分列 | columns 设置为 `[{"fr":1},{"fr":1},{"fr":1}]` |
| 左列 220，其余分给右列 | columns 设置为 `[{"fixed":220},{"fr":1}]` |
| 两列比例 1:2 | columns 设置为 `[{"fr":1},{"fr":2}]` |
| 某一行分配剩余高度 | 给该行子项设置 `layout.height: {"fr":1}`，并确保 Grid 有有限高度 |

同一行多个 height.fr 取最大权重作为该行权重。单行 Grid 自身声明 fixed/fr 高度时，该行也可填满所分配区域。需要整页布局可参考 [布局组合](20-recipes.md)。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"grid"` | 网格容器 |
| `columns` | 轨道数组 | 必填，1..64 项 | 每项恰好指定 fixed 或 fr |
| `children` | 组件数组 | `[]` | 行优先排列 |
| `layout.gap` | 非负数 | 0 | 行列共用间距 |
| `layout.align` | 对齐枚举 | `stretch` | 单元格内对齐 |
| `layout.overflow` | `clip` / `scroll` | `clip` | 区域溢出处理 |
| `state` / `actions` | 对象 | `{}` | 局部状态与动作 |

其余见 [公共属性](02-common.md)。Grid 的 columns 是轨道数组，与 Table 的数据列对象不同。

## 常见问题

- **会随窗口自动减少列数吗？** 当前列数固定；需要按宽度换行可考虑 Row 的 wrap。
- **支持跨行跨列吗？** 当前没有 span、自动列或瀑布流。
- **为什么 auto、justify 或 wrap:true 报错？** Grid 列只接受 fixed/fr，拒绝 layout.justify，wrap 不能为 true。

相关：[Row](03-row.md) · [Card](07-card.md) · [公共布局](02-common.md)。
