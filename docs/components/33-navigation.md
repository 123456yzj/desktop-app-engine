# 33 导航、分页与内容切换

本页覆盖 Anchor、Breadcrumb、Dropdown、Menu、Pagination、Steps、Tabs、Segmented、Collapse、Carousel。组件使用原生桌面控件；页面路由由事件动作实现。

## 代码演示

菜单选择、分页、分段选择、标签页和折叠状态写回 state。Tabs/Collapse 的 `items.key` 对应直属子组件的 `id`。选中后先写状态，再运行 change 或后续点击动作。下拉菜单点击后显示收到的键值。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "导航与切换", "window": {"width": 900, "height": 740}},
  "state": {"menu": ["home"], "page": 1, "pageSize": 10, "tab": "overview", "segment": "day", "panels": ["details"], "step": 0, "slide": 0, "last": "尚未点击"},
  "root": {"type": "column", "id": "root", "layout": {"padding": 16, "gap": 12, "overflow": "scroll"}, "children": [
    {"type": "breadcrumb", "id": "breadcrumb", "items": [{"key": "home", "label": "首页"}, {"key": "settings", "label": "设置"}], "separator": ">"},
    {"type": "anchor", "id": "anchor", "direction": "horizontal", "items": [{"key": "intro", "label": "介绍", "href": "#overview"}]},
    {"type": "menu", "id": "menu", "mode": "horizontal", "selectedKeys": {"kind": "state", "key": "menu"}, "items": [{"key": "home", "label": "首页"}, {"key": "settings", "label": "设置"}]},
    {"type": "dropdown", "id": "dropdown", "text": "操作菜单", "items": [{"key": "settings", "label": "打开设置"}], "events": {"click": "capture"}},
    {"type": "text", "id": "last", "text": {"kind": "state", "key": "last"}},
    {"type": "steps", "id": "steps", "current": {"kind": "state", "key": "step"}, "items": [{"key": "start", "label": "填写"}, {"key": "review", "label": "确认"}, {"key": "finish", "label": "完成"}]},
    {"type": "tabs", "id": "tabs", "activeKey": {"kind": "state", "key": "tab"}, "items": [{"key": "overview", "label": "概览"}, {"key": "history", "label": "历史"}], "children": [
      {"type": "text", "id": "overview", "text": {"kind": "literal", "value": "概览内容"}},
      {"type": "text", "id": "history", "text": {"kind": "literal", "value": "历史内容"}}
    ]},
    {"type": "segmented", "id": "segment", "value": {"kind": "state", "key": "segment"}, "options": ["day", "week", "month"]},
    {"type": "collapse", "id": "collapse", "activeKey": {"kind": "state", "key": "panels"}, "items": [{"key": "details", "label": "详情"}], "children": [{"type": "text", "id": "details", "text": {"kind": "literal", "value": "展开后的内容"}}]},
    {"type": "carousel", "id": "carousel", "current": {"kind": "state", "key": "slide"}, "layout": {"height": {"fixed": 100}}, "children": [
      {"type": "text", "id": "slideOne", "text": {"kind": "literal", "value": "第一张"}},
      {"type": "text", "id": "slideTwo", "text": {"kind": "literal", "value": "第二张"}}
    ]},
    {"type": "pagination", "id": "pagination", "current": {"kind": "state", "key": "page"}, "pageSize": {"kind": "state", "key": "pageSize"}, "total": 123, "showSizeChanger": true, "showQuickJumper": true, "showTotal": true}
  ]},
  "actions": {"capture": {"eventPayload": "settings", "steps": [{"type": "set", "target": "last", "value": {"kind": "event"}}]}}
}
```

## API 与事件

新组件的 text/title/value/options 和扩展属性可直接写字面量，也可写 Binding。`disabled`、`size: small|medium|large` 为公共扩展属性；旧组件原有字段继续使用 Binding。可写状态只接受顶层 key，`path` 必须为空。`value` 与其主属性别名不能同时声明。

| type | 主值与常用属性 | 事件与载荷 |
| --- | --- | --- |
| anchor | items、direction；条目 key/label/href | click：`{key,href}` |
| breadcrumb | items、separator，默认 `/` | click：`{key,href}` |
| dropdown | items、text、open、trigger=click/hover/contextMenu、placement | click：键字符串；openChange：boolean，写回 open |
| menu | selectedKeys 字符串数组；items、mode、multiple、selectable、openKeys、inlineCollapsed、inlineIndent、theme | change：选中键数组；click：键字符串；select/deselect：`{key,selectedKeys}`；openChange：展开键数组，写回 openKeys |
| pagination | current 从 1 开始；pageSize 默认 10；total、pageSizeOptions、hideOnSinglePage、showLessItems、showQuickJumper、showSizeChanger、showTotal、simple | change：页码；showSizeChange：每页条数，写回 pageSize |
| steps | current 从 0 开始；items、initial、direction | change：步骤索引 |
| tabs | activeKey 字符串；items、defaultActiveKey、tabPlacement、tabBarGutter、tabType、hideAdd、hideTabBar | change/tabClick：键字符串；edit：`{action:"add"或"remove",key}` |
| segmented | value 字符串；options；vertical | change：字符串 |
| collapse | activeKey 字符串数组；items、accordion | change：展开键数组 |
| carousel | current 从 0 开始；children 或 items、autoplay、autoplaySpeed（毫秒）、dots、arrows、infinite | change：幻灯片索引 |

Menu 的 defaultSelectedKeys/defaultOpenKeys、Pagination 的 defaultCurrent/defaultPageSize、Tabs 的 defaultActiveKey、Collapse 的 defaultActiveKey 用于本地初始状态。字面量主值可交互并保存在控件本地；state Binding 每帧读取外部状态。

`hideTabBar: true` 隐藏 Tabs 的标题栏并释放其布局空间，内容仍按 `activeKey` 切换。适用于左侧 Menu 或搜索选择器控制的组件文档页面；这是本引擎提供的扩展属性。

items 最多 10000 项、嵌套最多 32 层，key/value 在整棵列表内唯一。Menu 单选最多一个键；accordion 最多一个展开键。选中键与树展开键必须属于当前数据，更新数据时需同时移除失效键。

## 桌面差异与当前限制

- Anchor/Breadcrumb 发出点击意图，尚未自动滚动到目标或切换页面。offsetTop/targetOffset 已有契约，尚未实现自动锚定。
- Menu 的子菜单以内嵌展开呈现；Dropdown 的左右对齐尚未完整区分。
- Steps 的 status/percent/progressDot、Tabs 的 centered/line/card 外观与 tabScroll、Pagination 的 align、Segmented 的 block、Collapse 的 bordered/ghost 尚未完整实现。Tabs 的增删事件由应用更新 items/状态，不自动新增组件节点。
- Collapse 当前把所有标题排在内容之前，展开内容按子节点顺序排列；Carousel 尚无滑动动画、触摸手势和响应断点对象。

[组件总览](README.md) · [覆盖与差异](../21-component-coverage.md)
