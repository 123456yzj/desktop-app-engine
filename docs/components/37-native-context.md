# 37 配置、装饰与桌面工具

ConfigProvider/App 为子树提供配置，Watermark/BorderBeam 装饰内容，Affix 按父裁剪区域约束位置。Ant Design 的 Util 属于工具接口，本引擎通过 Rust 检查函数及既有布局、状态、主题接口提供桌面能力。

## 代码演示

主题、字号和禁用设置向下传递；嵌套配置可覆盖主题和尺寸。加载动画与装饰在子内容之后绘制。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "配置与装饰", "window": {"width": 760, "height": 560}},
  "state": {},
  "root": {"type": "configProvider", "id": "config", "theme": "dark", "componentSize": "large", "locale": "zh-CN", "direction": "ltr", "layout": {"padding": 20, "gap": 12, "overflow": "scroll"}, "children": [
    {"type": "app", "id": "appContext", "children": [
      {"type": "affix", "id": "affix", "offsetTop": 8, "children": [{"type": "button", "id": "fixedAction", "text": {"kind": "literal", "value": "固定操作"}}]},
      {"type": "watermark", "id": "watermark", "content": "内部资料", "rotate": -22, "gap": 100, "layout": {"height": {"fixed": 180}}, "children": [{"type": "text", "id": "document", "text": {"kind": "literal", "value": "带水印的内容"}}]},
      {"type": "borderBeam", "id": "beam", "duration": 6, "borderWidth": 2, "layout": {"height": {"fixed": 100}, "padding": 12}, "children": [{"type": "text", "id": "featured", "text": {"kind": "literal", "value": "边框动画区域"}}]}
    ]}
  ]},
  "actions": {}
}
```

## API

| type/工具 | 属性与行为 |
| --- | --- |
| configProvider/app | children/state/actions；componentSize=small/medium/large、componentDisabled、theme=light/dark、direction=ltr/rtl、locale=zh-CN/en-US |
| watermark | children、content 字符串、gap=120 point（16..1000）、rotate=-22 度 |
| borderBeam | children、duration=6 秒（0.1..3600）、borderWidth=2（1..32） |
| affix | children、offsetTop/offsetBottom，0..10000 point |
| `v02::util::component_api(Kind)` | 返回扩展属性的类型/枚举/范围、原生事件、事件写回属性、主属性、是否容器/输入控件 |

Rust 嵌入宿主可调用 `desktop_app_engine::v02::util::component_api(desktop_app_engine::v02::Kind::Pagination)` 检查真实契约。结果描述扩展属性；旧组件的原有字段和公共布局样式见 [Schema](../description-0.2.schema.json)。插件事件仍由 manifest 定义。

## 当前限制

- ConfigProvider 的 locale 当前只覆盖部分控件文案，RTL 为基础镜像布局；不等价于完整国际化包。App 当前提供配置作用域，没有独立消息队列或 useApp hook。
- Affix 当前依据父 clip 调整位置，尚非完整滚动容器监听、吸附边界与 change 回调。
- Watermark 是视觉装饰，不提供浏览器 DOM 变更恢复机制；BorderBeam 只绘制原生矩形周边动画。
- Util 没有独立 JSON 节点。浏览器事件合成、DOM 测量、CSS 样式工具不适用于本机渲染；`component_api` 是桌面协议检查工具，未声明与官网每个工具函数逐一兼容。

[组件总览](README.md) · [公共属性](02-common.md)
