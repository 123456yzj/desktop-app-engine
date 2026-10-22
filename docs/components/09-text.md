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

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"text"` | 文本组件 |
| `text` | 字符串绑定 | 必填 | literal/state/模块 prop |
| `style.fontSize` | 非负数 | 16 | 字号 |
| `style.fontFamily` | proportional / monospace | proportional | 字体族 |
| `style.fontWeight` | normal / bold | normal | 等宽字体优先于字重 |
| `style.textWrap` | 布尔 | `true` | 是否按可用宽度换行 |
| `style.textAlign` | start / center / end | start | center 同时垂直居中 |
| `style.textOverflowTooltip` | 布尔 | `true` | 截断时悬停显示完整内容 |

其余见 [公共属性](02-common.md)。没有业务事件或子组件。

## 常见问题

- **可以绑定数字或模板字符串吗？** text 需要字符串，不解释 `{{value}}`。数字展示用 Metric，字符串拼接通过 concat 动作。
- **如何单行截断？** 设置 textWrap:false，并提供有限宽度。默认截断悬停提示可关闭。
- **如何显示长日志？** 将 Text 放入有限高度、overflow:scroll 的 Column/Panel；需要查看完整任务输出也可用 [运行日志](../19-runtime-logs.md)。

相关：[Metric](13-metric.md) · [Input](11-input.md) · [公共样式](02-common.md)。
