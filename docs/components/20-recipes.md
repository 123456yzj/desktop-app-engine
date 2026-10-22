# 20 常见布局与组合

[组件总览](README.md) / 常见布局与组合

本页把容器与基础组件组合成可运行页面。建议先阅读 [公共布局规则](02-common.md)，再按目标页面选择示例。

## 固定顶部、左右分栏、右侧日志

结构示意：

```text
Column 页面
├─ Row 顶部工具栏（固定高度）
└─ Row 主体（height.fr = 1）
   ├─ Panel 项目区（width.fr = 2）
   └─ Panel 日志区（width.fr = 1，可滚动）
```

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "工作台布局", "window": {"width": 1060, "height": 680}},
  "state": {"project": "demo", "logs": "[INFO] 工作台就绪\n[INFO] 等待操作", "count": 8},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 16, "gap": 16},
    "children": [
      {
        "type": "row", "id": "toolbar", "layout": {"height": {"fixed": 40}, "gap": 12, "align": "center"},
        "children": [
          {"type": "text", "id": "heading", "text": {"kind": "literal", "value": "构建工作台"}, "style": {"fontSize": 22, "fontWeight": "bold"}},
          {"type": "input", "id": "project", "value": {"kind": "state", "key": "project"}, "layout": {"width": {"fr": 1}}, "placeholder": {"kind": "literal", "value": "项目名称"}},
          {"type": "button", "id": "run", "text": {"kind": "literal", "value": "模拟构建"}, "events": {"click": "run"}, "style": {"background": "primary", "color": "onPrimary"}}
        ]
      },
      {
        "type": "row", "id": "main", "layout": {"height": {"fr": 1}, "gap": 16},
        "children": [
          {
            "type": "panel", "id": "projects", "title": {"kind": "literal", "value": "项目概览"}, "layout": {"width": {"fr": 2}, "padding": 16, "gap": 16},
            "children": [
              {"type": "grid", "id": "summary", "columns": [{"fr": 1}, {"fr": 1}], "layout": {"gap": 12}, "children": [
                {"type": "card", "id": "countCard", "layout": {"padding": 12}, "children": [{"type": "metric", "id": "count", "label": {"kind": "literal", "value": "项目数"}, "value": {"kind": "state", "key": "count"}}]},
                {"type": "card", "id": "statusCard", "layout": {"padding": 12, "gap": 8}, "children": [{"type": "text", "id": "statusLabel", "text": {"kind": "literal", "value": "工作区状态"}}, {"type": "badge", "id": "ready", "text": {"kind": "literal", "value": "已就绪"}, "tone": "success"}]}
              ]},
              {"type": "text", "id": "description", "text": {"kind": "literal", "value": "这里可继续放表格或项目列表。右侧日志区独立滚动。"}}
            ]
          },
          {
            "type": "panel", "id": "logPanel", "title": {"kind": "literal", "value": "运行日志"}, "layout": {"width": {"fr": 1}, "padding": 16, "overflow": "scroll"},
            "children": [{"type": "text", "id": "logs", "text": {"kind": "state", "key": "logs"}, "style": {"fontFamily": "monospace", "textOverflowTooltip": false}}]
          }
        ]
      }
    ]
  },
  "actions": {"run": {"steps": [{"type": "concat", "target": "logs", "parts": [{"kind": "literal", "value": "[INFO] 开始构建："}, {"kind": "state", "key": "project"}, {"kind": "literal", "value": "\n[INFO] 示例构建完成"}]}]}}
}
```

预期效果：顶部固定，主体随窗口填满；左右区域宽度约 2:1。输入项目名并点击按钮后，右侧显示模拟日志。本示例只更新状态，没有执行真实构建命令。

布局要点：Column 的直接孩子 main 使用 height.fr，main 的孩子通过 width.fr 分宽。直接放在 Row 下的面板默认纵向 stretch；内部内容仍按纵向排列。

## 标签与输入框对齐的设置表单

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "设置表单", "window": {"width": 760, "height": 460}},
  "state": {"server": "build.example", "path": "/workspace/demo", "message": "尚未保存"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {
        "type": "panel", "id": "settings", "title": {"kind": "literal", "value": "连接设置"}, "layout": {"padding": 16, "gap": 16},
        "children": [
          {"type": "row", "id": "serverRow", "layout": {"gap": 12, "align": "center"}, "children": [
            {"type": "text", "id": "serverLabel", "text": {"kind": "literal", "value": "服务器"}, "layout": {"width": {"fixed": 100}}},
            {"type": "input", "id": "server", "value": {"kind": "state", "key": "server"}, "layout": {"width": {"fr": 1}}}
          ]},
          {"type": "row", "id": "pathRow", "layout": {"gap": 12, "align": "center"}, "children": [
            {"type": "text", "id": "pathLabel", "text": {"kind": "literal", "value": "工作区"}, "layout": {"width": {"fixed": 100}}},
            {"type": "input", "id": "path", "value": {"kind": "state", "key": "path"}, "layout": {"width": {"fr": 1}}}
          ]},
          {"type": "row", "id": "buttons", "layout": {"justify": "end", "gap": 12, "align": "center"}, "children": [
            {"type": "button", "id": "save", "text": {"kind": "literal", "value": "保存"}, "events": {"click": "save"}, "style": {"background": "primary", "color": "onPrimary"}}
          ]}
        ]
      },
      {"type": "text", "id": "message", "text": {"kind": "state", "key": "message"}}
    ]
  },
  "actions": {"save": {"steps": [{"type": "concat", "target": "message", "parts": [{"kind": "literal", "value": "当前设置："}, {"kind": "state", "key": "server"}, {"kind": "literal", "value": " "}, {"kind": "state", "key": "path"}]}]}}
}
```

预期效果：两行标签固定同宽，输入框对齐并填满剩余宽度，保存按钮靠右。点击保存只显示当前值，不写磁盘，也不建立连接。持久化或网络操作需接入对应业务能力。

## 嵌套滚动：外层列表与内层详情

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "嵌套滚动", "window": {"width": 700, "height": 480}},
  "state": {},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 12},
    "children": [
      {"type": "text", "id": "hint", "text": {"kind": "literal", "value": "指针位于详情内时先滚动详情，到边界后继续滚动外层"}},
      {
        "type": "column", "id": "outer", "layout": {"height": {"fixed": 300}, "overflow": "scroll", "gap": 16},
        "children": [
          {"type": "card", "id": "intro", "title": {"kind": "literal", "value": "外层顶部"}, "layout": {"height": {"fixed": 100}, "padding": 12}},
          {"type": "panel", "id": "inner", "title": {"kind": "literal", "value": "内层详情"}, "layout": {"height": {"fixed": 160}, "overflow": "scroll", "padding": 12, "gap": 12}, "children": [
            {"type": "text", "id": "detail1", "layout": {"height": {"fixed": 100}}, "text": {"kind": "literal", "value": "详情第一段"}},
            {"type": "text", "id": "detail2", "layout": {"height": {"fixed": 100}}, "text": {"kind": "literal", "value": "详情第二段"}},
            {"type": "text", "id": "detail3", "layout": {"height": {"fixed": 100}}, "text": {"kind": "literal", "value": "详情第三段"}}
          ]},
          {"type": "card", "id": "footer", "title": {"kind": "literal", "value": "外层底部"}, "layout": {"height": {"fixed": 100}, "padding": 12}}
        ]
      }
    ]
  },
  "actions": {}
}
```

预期效果：外层与内层都有独立滚动范围；内层到边界后的剩余滚轮位移交给外层。固定高度的子项默认不收缩，保证内容确实溢出。

## 布局排查顺序

| 现象 | 优先检查 |
| --- | --- |
| 内容没有撑满 | fr 是否放在直接子项、父级是否提供有限空间 |
| 控件高度异常 | 父 Row 的 align 是否为默认 stretch |
| 设置 gap 后无变化 | 是否设置在拥有 children 的容器上 |
| 内容被挤小 | auto/fr 的 shrink，是否应该固定高度或滚动 |
| 滚动区无限长 | 是否缺少有限 height 或父级剩余高度分配 |
| 宽度很窄时仍保持两列 | Grid 列数固定；自适应换行使用 Row.wrap |
| 标题遮挡或内容空间不足 | 标题、padding、border 和滚动槽均占用内部空间 |

相关：[Row](03-row.md) · [Column](04-column.md) · [Grid](05-grid.md) · [公共属性](02-common.md)。
