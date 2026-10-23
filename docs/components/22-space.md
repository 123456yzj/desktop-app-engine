# 22 Space 间距

[组件总览](README.md) / Space

统一排列按钮、状态标签和小型控件组。默认横向、间距 8、交叉轴居中；使用 [Flex](21-flex.md) 可从无间距、拉伸对齐的默认布局开始。

## 代码演示：操作组、分隔符与换行

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "操作间距", "window": {"width": 560, "height": 260}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24},
    "children": [{
      "type": "space", "id": "actions", "separator": "|", "layout": {"wrap": true, "rowGap": 16},
      "children": [
        {"type": "button", "id": "edit", "text": {"kind": "literal", "value": "编辑"}},
        {"type": "button", "id": "copy", "text": {"kind": "literal", "value": "复制"}},
        {"type": "badge", "id": "status", "text": {"kind": "literal", "value": "已同步"}, "tone": "success"}
      ]
    }]
  },
  "actions": {}
}
```

预期效果：操作按钮和状态居中对齐，横向间距 8，换行后的行间距 16；可见相邻子节点之间绘制 `|`。需要单独布局和样式的分隔线时，也可以显式插入 Divider 子节点。

## 代码演示：纵向间距

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "纵向间距", "window": {"width": 480, "height": 320}},
  "state": {"name": ""},
  "root": {
    "type": "space", "id": "form", "vertical": true,
    "layout": {"padding": 24, "gap": 16, "align": "stretch"},
    "children": [
      {"type": "text", "id": "label", "text": {"kind": "literal", "value": "项目名称"}},
      {"type": "input", "id": "name", "value": {"kind": "state", "key": "name"}},
      {"type": "button", "id": "save", "text": {"kind": "literal", "value": "保存"}}
    ]
  },
  "actions": {}
}
```

预期效果：三项纵向排列，间距 16，交叉轴拉伸。输入框编辑共享状态。

## API

| 属性 | 类型 / 默认值 | 说明 |
| --- | --- | --- |
| `type` | `"space"`，必填 | 间距容器 |
| `vertical` | 布尔 / false | 切换纵向 |
| `children` | 数组 / [] | 顺序排列 |
| `separator` | 字符串或绑定 / 空 | 在可见相邻子节点之间绘制分隔文字，不接受组件子树 |
| `layout.gap` | 非负数 / 8 | 设为 0 可关闭默认间距 |
| `layout.rowGap` / `columnGap` | 非负数 / 回退到 gap | 独立行列间距 |
| `layout.align` | stretch / start / center / end，默认 center | 交叉轴对齐 |
| `layout.wrap` | 布尔 / false | 仅横向可为 true |
| `layout.justify` | 六种主轴对齐 / start | 同 Flex |

支持公共尺寸、样式、滚动和局部 state/actions，不接受业务事件。间距使用数值，不接受 small/medium/large；常用 8、16、24。原生组件不添加 HTML 包裹元素。

## 常见问题

- **纵向控件没有铺满宽度？** 默认 align 是 center；表单可改成 stretch。
- **separator 会自动出现在每项之间吗？** 会在可见相邻子节点之间绘制；隐藏项不产生分隔符。它不作为独立布局节点占位，内容较密时需留出足够 gap；需控制分隔线布局时使用 Divider。
- **需要页面分栏？** 使用 Flex、Grid、Layout 或 Splitter，Space 主要用于控件组。
