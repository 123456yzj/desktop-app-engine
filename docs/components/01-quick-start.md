# 01 快速开始

[组件总览](README.md) / 快速开始

本页用输入框、预览和按钮完成第一个界面。需要 Windows 开发环境与已构建的制作端引擎；环境准备见 [项目 README](../../README.md)。以下命令均在仓库根目录执行。

## 第一个完整应用

将下面内容以 UTF-8 保存为仓库根目录的 `component-demo.json`：

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "组件入门", "window": {"width": 680, "height": 400}},
  "state": {"name": "桌面应用", "message": "点击按钮生成问候"},
  "root": {
    "type": "column", "id": "page",
    "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "heading", "text": {"kind": "literal", "value": "第一个组件界面"}, "style": {"fontSize": 24, "fontWeight": "bold"}},
      {"type": "input", "id": "nameInput", "value": {"kind": "state", "key": "name"}, "placeholder": {"kind": "literal", "value": "输入名字"}},
      {"type": "text", "id": "livePreview", "text": {"kind": "state", "key": "name"}},
      {"type": "button", "id": "greetButton", "text": {"kind": "literal", "value": "生成问候"}, "events": {"click": "greet"}, "style": {"background": "primary", "color": "onPrimary"}},
      {"type": "text", "id": "result", "text": {"kind": "state", "key": "message"}}
    ]
  },
  "actions": {
    "greet": {"steps": [{"type": "concat", "target": "message", "parts": [{"kind": "literal", "value": "你好，"}, {"kind": "state", "key": "name"}]}]}
  }
}
```

先校验，再打开窗口：

```powershell
cargo build --bin desktop-app-engine --locked --offline
.\target\debug\desktop-app-engine.exe --app .\component-demo.json --check
.\target\debug\desktop-app-engine.exe --app .\component-demo.json
```

预期效果：输入框下面的预览随输入更新；点击“生成问候”，最下方显示 `你好，` 加上当前名字。开发时可追加 `--theme light` 或 `--theme dark` 检查主题。

## 读懂描述

| 部分 | 用途 |
| --- | --- |
| `schemaVersion` | 本教程固定为 `"0.2"` |
| `app` | 应用名与初始窗口大小，宽高范围 160..8192 |
| `state` | 应用初始数据，组件用绑定读取 |
| `root` | 页面根组件，子组件放入 `children` |
| `actions` | 事件引用的动作，按 `steps` 顺序执行 |

组件 `id` 必须全树唯一，即使分属不同模块也不能重复。固定文字使用 `{"kind":"literal","value":"文字"}`，动态文字使用 `{"kind":"state","key":"状态名"}`。属性绑定、状态类型与事件规则见 [公共属性](02-common.md)。

## 如何使用后续示例

每个 `json` 代码块都包含完整应用。复制整个代码块替换 `component-demo.json`，重复上面的校验与运行命令即可。布局示例通常保持 `state`、`actions` 为空对象；交互示例已经声明必要状态和动作。

插件组件需要描述文件旁的 `plugins/` 资源，按 [插件教程](19-plugin.md) 准备后再运行。

## 修改后检查什么

1. 执行 `--check`，确认字段、类型、状态和动作引用合法。
2. 打开应用，观察示例页描述的效果，并调整窗口大小检查布局。
3. 操作输入、按钮、下拉或表格，验证联动及禁用行为。

`--check` 不打开窗口、不发送 HTTP 请求，也不执行业务命令；它不能代替实际交互检查。插件的实际执行验证使用 `--check-plugins`。

## 批量检查本教程

仓库维护者可使用 Python 3.9 或以上版本运行：

```powershell
python .\scripts\check-component-docs.py
```

脚本将各页完整 JSON 示例提取到临时目录，用 `target/debug/desktop-app-engine.exe` 执行 `--check`，同时检查本教程的本地链接和锚点。插件示例只校验仓库 manifest，不执行 JavaScript。引擎路径不同可用 `--engine 路径`；未构建时先执行本页的构建命令。

## 下一步

从 [公共属性与布局](02-common.md) 学习尺寸与绑定，再阅读 [Row](03-row.md)、[Column](04-column.md)、[Grid](05-grid.md) 和 [常见布局](20-recipes.md)。应用交付见 [打包说明](../06-packaging-decision.md)。
