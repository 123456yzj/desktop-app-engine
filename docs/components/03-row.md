# 03 Row 横向容器

[组件总览](README.md) / Row

将子组件从左向右排列，适用于工具栏、标签组和左右分栏。需要固定列轨道时使用 [Grid](05-grid.md)，纵向排列使用 [Column](04-column.md)。

## 代码演示：间距、对齐与换行

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "横向工具栏", "window": {"width": 620, "height": 360}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "hint", "text": {"kind": "literal", "value": "缩窄窗口，观察按钮自动换行"}},
      {
        "type": "row", "id": "toolbar", "layout": {"gap": 12, "align": "center", "wrap": true},
        "children": [
          {"type": "button", "id": "scan", "text": {"kind": "literal", "value": "扫描项目"}, "layout": {"width": {"fixed": 160}}},
          {"type": "button", "id": "build", "text": {"kind": "literal", "value": "构建项目"}, "layout": {"width": {"fixed": 160}}},
          {"type": "button", "id": "logs", "text": {"kind": "literal", "value": "查看日志"}, "layout": {"width": {"fixed": 160}}}
        ]
      }
    ]
  },
  "actions": {}
}
```

预期效果：三个等宽按钮横向排列，间距 12；窗口变窄时按钮依次移到下一行。这里按钮仅展示布局，没有绑定业务动作。

## 代码演示：固定侧栏与弹性内容

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "左右分栏", "window": {"width": 900, "height": 520}},
  "state": {},
  "root": {
    "type": "row", "id": "page", "layout": {"padding": 16, "gap": 16},
    "children": [
      {"type": "panel", "id": "sidebar", "title": {"kind": "literal", "value": "侧栏"}, "layout": {"width": {"fixed": 220}, "padding": 16}, "children": [{"type": "text", "id": "nav", "text": {"kind": "literal", "value": "项目列表"}}]},
      {"type": "panel", "id": "content", "title": {"kind": "literal", "value": "内容区"}, "layout": {"width": {"fr": 1}, "padding": 16}, "children": [{"type": "text", "id": "body", "text": {"kind": "literal", "value": "此区域占据侧栏之外的剩余宽度"}}]}
    ]
  },
  "actions": {}
}
```

预期效果：左侧宽度为 220，右侧随窗口变化。根 Row 使用窗口可用高度，两个面板按默认 stretch 填满交叉轴。

两列比例分栏时，将左侧 `width` 改为 `{"fr":1}`，右侧改为 `{"fr":2}`；容器扣除内边距和 gap 后按 1:2 分配宽度。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"row"` | 横向容器 |
| `children` | 组件数组 | `[]` | 按数组顺序排列 |
| `layout.gap` | 非负数 | 0 | 相邻项和换行间距 |
| `layout.rowGap` / `columnGap` | 非负数 | 回退到 gap | 独立行间距与项间距 |
| `layout.align` | 对齐枚举 | `stretch` | 纵向对齐，工具栏常用 center |
| `layout.justify` | 主轴对齐枚举 | `start` | start/center/end/spaceBetween/spaceAround/spaceEvenly |
| `layout.wrap` | 布尔 | `false` | 按子项基准宽度换行 |
| `layout.overflow` | `clip` / `scroll` | `clip` | 区域溢出处理 |
| `state` / `actions` | 对象 | `{}` | 兼容容器局部作用域；独立模块推荐 Module |

尺寸、约束、样式、visible、enabled 见 [公共属性](02-common.md)。Row 不支持 `title`、`columns` 或业务事件。

## 常见问题

- **按钮为什么被拉高？** 默认 align 为 stretch，改成 center 或 start；也可给子项固定高度。
- **spaceBetween 没有效果？** 检查子项是否通过 fr 占完了剩余宽度。
- **窄窗口下内容被裁剪？** 默认不换行；启用 wrap，或设计横向滚动区域。最小尺寸超过容器时会保留最小尺寸并按 overflow 处理。

相关：[Column](04-column.md) · [Grid](05-grid.md) · [布局组合](20-recipes.md)。
