# 25 Splitter 面板分割

[组件总览](README.md) / Splitter

将区域分成两个可调整面板，适用于文件导航与编辑器、编辑与预览。支持横向、纵向、嵌套、比例绑定和尺寸约束。

## 代码演示：拖动、比例绑定与重置

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "可调整面板", "window": {"width": 900, "height": 560}},
  "state": {"ratio": 0.3, "status": "拖动中间分割条"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 16, "gap": 12},
    "children": [
      {"type": "space", "id": "toolbar", "children": [
        {"type": "button", "id": "reset", "text": {"kind": "literal", "value": "恢复 30%"}, "events": {"click": "reset"}},
        {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}},
        {"type": "metric", "id": "ratio", "label": {"kind": "literal", "value": "左侧比例"}, "value": {"kind": "state", "key": "ratio"}, "format": {"type": "percent", "precision": 0}}
      ]},
      {"type": "splitter", "id": "split", "value": {"kind": "state", "key": "ratio"}, "layout": {"height": {"fr": 1}}, "events": {"change": "resized"}, "children": [
        {"type": "panel", "id": "files", "title": {"kind": "literal", "value": "文件"}, "layout": {"minWidth": 160, "maxWidth": 500, "padding": 16}, "children": [
          {"type": "text", "id": "fileList", "text": {"kind": "literal", "value": "src\ndocs\napp.json"}}
        ]},
        {"type": "panel", "id": "preview", "title": {"kind": "literal", "value": "预览"}, "layout": {"minWidth": 200, "padding": 16}, "children": [
          {"type": "text", "id": "previewText", "text": {"kind": "literal", "value": "拖动分割条调整比例。点击分割条聚焦后，用左右方向键调整；Shift 配合方向键微调。"}}
        ]}
      ]}
    ]
  },
  "actions": {
    "reset": {"steps": [{"type": "set", "target": "ratio", "value": {"kind": "literal", "value": 0.3}}]},
    "resized": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "比例已更新"}}]}
  }
}
```

预期效果：初始左侧占扣除分割条后宽度的 30%；拖动更新 ratio，再触发 change 动作，指标同步变化。面板 min/max 限制实际尺寸，双击分割条恢复约 50%，仍遵守约束。

## 代码演示：纵向与嵌套

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "嵌套面板", "window": {"width": 800, "height": 540}},
  "state": {},
  "root": {
    "type": "splitter", "id": "outer", "layout": {"padding": 16},
    "children": [
      {"type": "panel", "id": "left", "title": {"kind": "literal", "value": "导航"}, "layout": {"padding": 16, "minWidth": 120}},
      {"type": "splitter", "id": "right", "vertical": true, "value": {"kind": "literal", "value": 0.65}, "children": [
        {"type": "panel", "id": "editor", "title": {"kind": "literal", "value": "编辑器"}, "layout": {"padding": 16, "minHeight": 100}},
        {"type": "panel", "id": "console", "title": {"kind": "literal", "value": "控制台"}, "layout": {"padding": 16, "minHeight": 80}}
      ]}
    ]
  },
  "actions": {}
}
```

预期效果：外层左右分割，右侧上下分割；上下布局聚焦分割条后使用上下方向键。未绑定状态的比例也会保存在当前视图中，普通重绘不重置。

## API

| 属性 | 类型 / 默认值 | 说明 |
| --- | --- | --- |
| `type` | `"splitter"`，必填 | 双面板容器 |
| `children` | 恰好两个组件，必填 | 超过两区可嵌套 |
| `vertical` | 布尔 / false | true 为上下分割 |
| `value` | 数字绑定 / 0.5 | 第一个面板占可分配长度的比例，范围 [0,1] |
| `barSize` | 数值 / 8 | 分割条占位宽或高，范围 4..32 |
| `resizable` | 布尔 / true | false 时保留分割条但禁止调整 |
| 子项 minWidth/maxWidth | 非负数 | 左右分割的尺寸限制 |
| 子项 minHeight/maxHeight | 非负数 | 上下分割的尺寸限制 |
| `events.change` | 动作 ID / 动作绑定 | 比例先写入 state，再调用动作 |

value 支持 literal、本地缺省或可写 state；state 仅接受空 path，prop 只读不允许。change 载荷为新的数字比例，先写绑定状态再执行动作；动作可读取该状态，或声明数字 eventPayload 样本后使用 event Binding。禁止 gap/rowGap/columnGap/wrap/align/justify，面板填满交叉轴，分割条间距由 barSize 控制。

## 常见问题

- **输入比例与实际尺寸不同？** 最小/最大尺寸优先，拖动时写回受约束后的比例。窗口缩放不会自行修改状态；恢复足够空间后重新采用该状态比例。
- **面板宽高由谁决定？** 主轴由比例和 min/max 控制，子项 width/height 轨道不参与主轴分配；交叉轴填满。
- **最小尺寸之和超过窗口？** 保留最小尺寸，由容器 overflow 裁剪或滚动。
- **隐藏一个面板？** 另一个面板填满，分割条不绘制，重新显示后采用保留比例。
- **能否折叠和自动销毁面板？** 当前不提供折叠按钮和销毁选项；可用 visible 绑定控制显示。
