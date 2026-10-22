# 23 Divider 分割线

[组件总览](README.md) / Divider

分隔内容段落或同一行的操作，支持水平、纵向、实线、虚线、点线与带标题的分割线。

## 代码演示

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "分割线", "window": {"width": 640, "height": 440}},
  "state": {"section": "项目配置"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "intro", "text": {"kind": "literal", "value": "基本信息"}},
      {"type": "divider", "id": "solid"},
      {"type": "divider", "id": "titled", "text": {"kind": "state", "key": "section"}, "titlePlacement": "start", "variant": "dashed"},
      {"type": "text", "id": "body", "text": {"kind": "literal", "value": "这里填写配置内容。"}},
      {"type": "divider", "id": "dotted", "variant": "dotted", "style": {"borderColor": "accent"}},
      {"type": "space", "id": "links", "children": [
        {"type": "button", "id": "details", "text": {"kind": "literal", "value": "详情"}},
        {"type": "divider", "id": "vertical", "vertical": true},
        {"type": "button", "id": "history", "text": {"kind": "literal", "value": "历史"}}
      ]}
    ]
  },
  "actions": {}
}
```

预期效果：依次显示实线、带起始标题的虚线、强调色点线，底部操作组以纵线分隔。标题绑定状态，更新后重新绘制。

## API

| 属性 | 类型 / 默认值 | 说明 |
| --- | --- | --- |
| `type` | `"divider"`，必填 | 叶组件 |
| `vertical` | 布尔 / false | 纵线不接受 text |
| `text` | 可选字符串绑定 | 水平线标题 |
| `titlePlacement` | start / center / end，默认 center | 标题位置 |
| `variant` | solid / dashed / dotted，默认 solid | 线型 |
| `style.borderColor` | Token / border | 线条颜色 |
| `style.borderWidth` | 非负数 / 0 | 实际线宽至少 1；不额外绘制外框 |
| `style.color` / `fontSize` | Token / 16 | 标题颜色、字号 |

水平无标题自然高度为 16，带标题为字号 + 12；纵线自然尺寸为 16 × 24。可用 layout.width/height、padding 和约束修改尺寸。水平线随父容器宽度拉伸时，线条填满内容区；纵线可设置固定高度。标题不换行，空间不足时裁剪。

## 常见问题

- **与 border 的区别？** Divider 是参与布局的独立节点，border 属于容器外观。
- **能放 children 吗？** 不接受 children；标题使用 text 绑定。
- **为什么 gap 报错？** Divider 是叶组件；间距应写在父容器，或使用 Divider 的尺寸和 padding。
