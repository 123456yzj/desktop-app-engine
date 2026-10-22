# 19 Plugin 插件组件

[组件总览](README.md) / Plugin

引用已安装插件注册的组件，属性、事件与方法由 manifest 声明。当前宿主通过外部 Node 进程运行插件，接收 SVG 并绘制到原生窗口。

## 使用前准备

本页使用仓库 `plugins/echarts/` 的 `echarts.chart`。将应用描述保存在仓库根目录，准备 ECharts 生产依赖与 Node 运行环境；插件资源位于描述文件旁的 plugins 目录。完整安装步骤和正式打包资源要求见 [插件机制](../15-plugins.md#安装和运行)。

`--check` 只校验描述及 manifest；实际执行插件需使用 `--check-plugins`，并在窗口内检查交互。正式打包需要完整内置插件与 Node 资源。

## 代码演示：分类柱图与点击反馈

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "插件分类图表", "window": {"width": 880, "height": 520}},
  "plugins": ["echarts"],
  "state": {"selectedName": "尚未点击"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "selected", "text": {"kind": "state", "key": "selectedName"}},
      {
        "type": "plugin", "id": "chart", "component": "echarts.chart", "layout": {"height": {"fixed": 340}},
        "props": {"option": {"kind": "literal", "value": {"animation": false, "xAxis": {"type": "category", "data": ["项目 A", "项目 B", "项目 C"]}, "yAxis": {"type": "value"}, "series": [{"type": "bar", "data": [32, 44, 29]}]}}},
        "events": {"click": "chartClicked"}
      }
    ]
  },
  "actions": {"chartClicked": {"eventPayload": {"seriesIndex": 0, "dataIndex": 0, "name": "", "value": 0}, "steps": [{"type": "set", "target": "selectedName", "value": {"kind": "event", "path": "/name"}}]}}
}
```

预期效果：在资源已安装的环境显示带项目名称的柱图；点击柱子后，顶部文字显示该项目名称。本页用 literal 保存图表 option，避免将复杂插件 JSON 与引擎状态数组类型约束混在一起。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| 顶层 `plugins` | 包名数组 | `[]` | 应用启用的插件包 |
| `type` | 字符串 | 必填 `"plugin"` | 插件叶组件 |
| `component` | 字符串 | 必填 | 已注册的包名.组件名 |
| `props` | 属性名到绑定的对象 | `{}` | 经 manifest 校验；支持 literal/state/模块 prop |
| `events` | 插件事件到动作的映射 | `{}` | 事件与载荷以 manifest 为准 |

自然高度 240，建议明确尺寸；支持公共 style、visible、enabled，没有 children 或容器布局字段。props 不建立新的模块作用域。

## 事件与方法

插件事件处理动作必须声明与 manifest 匹配的 eventPayload 样本，可通过 event/path 读取载荷。当前 ECharts 适配器 click 提供 seriesIndex、dataIndex、name、数字 value。

方法通过动作步骤 pluginCall 调用：target 填组件 id，method 填 manifest 方法名，args 为绑定，可选 resultTarget 保存结果。调用异步，组件必须已经挂载；后续动作步骤不等待返回。完整方法示例见 [插件事件和方法](../15-plugins.md#事件和方法)。

## 常见问题

- **可以直接安装 Vue 组件包吗？** 需要本引擎 manifest 和适配 worker，不能直接使用 Vue install API。
- **支持浏览器 DOM、HTML tooltip 和 JS formatter 吗？** 当前是 SVG 宿主，仅支持 JSON 可表达的插件配置；ECharts 适配器目前只转发 click。
- **隐藏后会继续运行吗？** 隐藏或移出可见裁剪区域后卸载实例，重新显示会挂载新实例；旧实例响应不会写入新实例。
- **插件是否有独立权限沙箱？** 当前与应用具有相同本机权限，只安装可信插件。

相关：[内置折线图](17-line-chart.md) · [内置柱图](18-bar-chart.md) · [插件完整契约](../15-plugins.md)。
