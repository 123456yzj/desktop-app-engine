# 24 Layout 页面布局

[组件总览](README.md) / Layout

组合页头、侧栏、内容区和页脚，适合桌面工作台。Layout 会根据直属子节点是否包含 Sider 自动选择横向或纵向排列。

## 代码演示：页头、侧栏、内容与页脚

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "页面布局", "window": {"width": 960, "height": 640}},
  "state": {},
  "root": {
    "type": "layout", "id": "page", "style": {"background": "surface"},
    "children": [
      {"type": "header", "id": "header", "layout": {"padding": 16}, "style": {"background": "primary"}, "children": [
        {"type": "text", "id": "title", "text": {"kind": "literal", "value": "项目工作台"}, "style": {"color": "onPrimary", "fontSize": 24}}
      ]},
      {"type": "layout", "id": "body", "children": [
        {"type": "sider", "id": "nav", "layout": {"padding": 16, "gap": 16}, "style": {"background": "surfaceRaised"}, "children": [
          {"type": "text", "id": "navTitle", "text": {"kind": "literal", "value": "项目导航"}},
          {"type": "button", "id": "overview", "text": {"kind": "literal", "value": "概览"}},
          {"type": "button", "id": "settings", "text": {"kind": "literal", "value": "设置"}}
        ]},
        {"type": "content", "id": "content", "layout": {"padding": 24, "gap": 16, "overflow": "scroll"}, "children": [
          {"type": "text", "id": "heading", "text": {"kind": "literal", "value": "内容区"}, "style": {"fontSize": 28}},
          {"type": "card", "id": "summary", "title": {"kind": "literal", "value": "说明"}, "layout": {"padding": 16}, "children": [
            {"type": "text", "id": "description", "text": {"kind": "literal", "value": "侧栏默认宽 200，内容占据剩余宽度；页头和页脚保留固定高度。"}}
          ]}
        ]}
      ]},
      {"type": "footer", "id": "footer", "layout": {"padding": 12}, "children": [
        {"type": "text", "id": "copyright", "text": {"kind": "literal", "value": "项目工作台 · 本地运行"}, "style": {"textAlign": "center", "color": "textMuted"}}
      ]}
    ]
  },
  "actions": {}
}
```

预期效果：页头 64、页脚 48 point，中间区域分配剩余高度；侧栏 200 point，右侧内容自适应。窗口变化时区域重新计算。背景色在示例中显式声明。

## API

| type | 默认尺寸 | 内部排列与用途 |
| --- | --- | --- |
| `layout` | width.fr=1、height.fr=1 | 有直属 sider 时横向，否则纵向；可嵌套 |
| `header` | width.fr=1、height.fixed=64 | 纵向容器，页头 |
| `sider` | width.fixed=200、height.fr=1 | 纵向容器，侧栏 |
| `content` | width.fr=1、height.fr=1 | 纵向容器，内容占剩余空间 |
| `footer` | width.fr=1、height.fixed=48 | 纵向容器，页脚 |

五种节点均支持 children、公共 layout/style、visible/enabled、局部 state/actions；不接受业务事件。显式声明 width/height 会替代对应默认值。默认无额外背景或内边距。不提供 title 字段，标题放在子 Text 中。

## 常见问题

- **页头为何和侧栏并排？** Layout 在直属子节点含 Sider 时横向排列。将 Sider 与 Content 放进内层 Layout，让外层处理 Header 和 Footer。
- **如何放右侧栏？** 将 Sider 放在 Content 后方，也可左右各放一个。
- **如何修改侧栏宽度？** 指定 sider.layout.width.fixed；需要用户拖动时用 [Splitter](25-splitter.md)。
- **侧栏支持折叠与断点吗？** 当前提供区域与尺寸分配；可绑定 visible 隐藏侧栏，未提供独立折叠按钮或断点配置。
