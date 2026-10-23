# 09 Text 文本

[组件总览](README.md) / Text

显示标题、说明或状态字符串。支持绑定、换行、字体、颜色和文字对齐；日志适合使用等宽字体。

## 代码演示：标题、预览和日志

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "文本排版", "window": {"width": 720, "height": 420}},
  "state": {"message": "修改输入内容，下面文字随之更新"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "heading", "text": {"kind": "literal", "value": "项目概览"}, "style": {"fontSize": 26, "fontWeight": "bold", "color": "textPrimary"}},
      {"type": "input", "id": "input", "value": {"kind": "state", "key": "message"}},
      {"type": "text", "id": "preview", "text": {"kind": "state", "key": "message"}, "style": {"color": "textMuted"}},
      {"type": "text", "id": "log", "text": {"kind": "literal", "value": "[INFO] preparing\n[INFO] complete"}, "style": {"fontFamily": "monospace", "textOverflowTooltip": false}}
    ]
  },
  "actions": {}
}
```

预期效果：大号粗体标题、动态预览和两行等宽日志。文字颜色不会从父容器继承。

## 代码演示：编辑、复制与文字样式

编辑区和预览读取同一字符串；编辑时先写回 message，再执行 change 动作。复制按钮将文字写入剪贴板，并把 copy 载荷交给动作。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "文本编辑与复制", "window": {"width": 680, "height": 420}},
  "state": {"message": "可编辑并复制", "copied": "尚未复制"},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "text", "id": "editor", "text": {"kind": "state", "key": "message"}, "editable": true, "copyable": true, "events": {"copy": "copied"}},
    {"type": "text", "id": "preview", "text": {"kind": "state", "key": "message"}, "strong": true, "underline": true, "ellipsis": true, "layout": {"width": {"fixed": 220}}},
    {"type": "text", "id": "code", "text": {"kind": "literal", "value": "cargo build"}, "code": true},
    {"type": "text", "id": "old", "text": {"kind": "literal", "value": "已过期的说明"}, "italic": true, "delete": true},
    {"type": "text", "id": "copied", "text": {"kind": "state", "key": "copied"}}
  ]},
  "actions": {"copied": {"eventPayload": "可编辑并复制", "steps": [{"type": "set", "target": "copied", "value": {"kind": "event"}}]}}
}
```

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"text"` | 文本组件 |
| `text` | 字符串绑定 | 必填 | literal/state/模块 prop |
| `copyable` | 布尔或绑定 | false | 显示复制按钮，将当前文字写入剪贴板 |
| `editable` | 布尔或绑定 | false | 使用多行编辑器；需要业务动作读取结果时将 text 绑定到顶层 state |
| `ellipsis` | 布尔或绑定 | false | 在可用宽度内截断文字 |
| `code` | 布尔或绑定 | false | 代码文字外观 |
| `italic` / `underline` / `delete` / `strong` | 布尔或绑定 | false | 斜体、下划线、删除线、粗体；用于非编辑视图 |
| `events.change` | 动作 ID 或绑定 | 无 | 载荷为新字符串；先写 text 的 state 绑定，再执行动作 |
| `events.copy` | 动作 ID 或绑定 | 无 | 载荷为复制的字符串；动作使用 event 时须声明 eventPayload 样本 |
| `style.fontSize` | 非负数 | 16 | 字号 |
| `style.fontFamily` | proportional / monospace | proportional | 字体族 |
| `style.fontWeight` | normal / bold | normal | 等宽字体优先于字重 |
| `style.textWrap` | 布尔 | `true` | 是否按可用宽度换行 |
| `style.textAlign` | start / center / end | start | center 同时垂直居中 |
| `style.textOverflowTooltip` | 布尔 | `true` | 截断时悬停显示完整内容 |

editable 的 literal 文本修改保存在控件本地；state 绑定会写回状态。文字装饰、编辑和复制使用原生控件，不支持自定义渲染回调或任意图标子树。其余见 [公共属性](02-common.md)。没有子组件。

## 常见问题

- **可以绑定数字或模板字符串吗？** text 需要字符串，不解释 `{{value}}`。数字展示用 Metric，字符串拼接通过 concat 动作。
- **如何单行截断？** 设置 textWrap:false，并提供有限宽度。默认截断悬停提示可关闭。
- **如何显示长日志？** 将 Text 放入有限高度、overflow:scroll 的 Column/Panel；需要查看完整任务输出也可用 [运行日志](../19-runtime-logs.md)。

相关：[Metric](13-metric.md) · [Input](11-input.md) · [公共样式](02-common.md)。
