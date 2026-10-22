# 08 Module 模块

[组件总览](README.md) / Module

用局部 state/actions 封装一组控件，通过 props 接收父级数据，通过 emits/on 向父级发送事件。Module 默认纵向排列，没有默认卡片背景或标题；外观由内部 Card、Panel、Row、Grid 组合。

## 代码演示：局部编辑、父级接收

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "模块通信", "window": {"width": 720, "height": 440}},
  "state": {"sharedMessage": "父级初始内容"},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "parentPreview", "text": {"kind": "state", "key": "sharedMessage"}},
      {
        "type": "module", "id": "editor", "layout": {"gap": 12},
        "state": {"draft": "模块内的草稿"},
        "props": {"message": {"kind": "state", "key": "sharedMessage"}},
        "emits": {"changed": ""}, "on": {"changed": "receive"},
        "actions": {"publish": {"steps": [{"type": "emit", "event": "changed", "payload": {"kind": "state", "key": "draft"}}]}},
        "children": [
          {"type": "text", "id": "received", "text": {"kind": "prop", "key": "message"}},
          {"type": "input", "id": "draftInput", "value": {"kind": "state", "key": "draft"}},
          {"type": "button", "id": "publish", "text": {"kind": "literal", "value": "发送给父级"}, "events": {"click": "publish"}}
        ]
      }
    ]
  },
  "actions": {"receive": {"steps": [{"type": "set", "target": "sharedMessage", "value": {"kind": "event"}}]}}
}
```

预期效果：编辑输入框只改变模块草稿；点击按钮后，父级文字与模块收到的 prop 同时显示新内容。`emits.changed` 的空字符串声明载荷类型，实际发送的是 draft 的值。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"module"` | 模块边界 |
| `children` | 组件数组 | `[]` | 模块内部界面 |
| `state` | 状态对象 | `{}` | 私有响应式数据 |
| `actions` | 动作对象 | `{}` | 在声明作用域执行 |
| `props` | 名称到绑定的对象 | `{}` | 绑定在父作用域解析，模块内部只读 |
| `emits` | 名称到 JSON 类型样本的对象 | `{}` | 声明事件载荷契约 |
| `on` | 事件名到父级动作的映射 | `{}` | 监听已声明的 emits 事件 |
| `layout` | 容器布局 | 纵向、公用默认 | 支持 gap/padding/对齐/滚动；wrap 不能为 true |

公共 style、visible、enabled 见 [公共属性](02-common.md)。Module 不接受 title、columns 或控件式 events。

## 作用域规则

- 状态和动作从当前作用域向祖先查找，同名就近遮蔽。不同模块可以有同名 draft、publish，但组件 id 仍要求全树唯一。
- 父级和兄弟模块无法直接引用模块私有状态；需要共享时使用 props/emit/on。
- props 与本模块 state 不可同名；props 不能作为输入框 value 或状态写入目标。先编辑局部 state，再 emit。
- 动作始终按声明位置解析状态，调用位置不会改变其作用域。

## 常见问题

- **隐藏模块会重置草稿吗？** 不会，隐藏保留状态；当前没有动态卸载生命周期。
- **可以注册模板反复实例化吗？** 当前需要在 JSON 中分别声明实例，没有 Vue 式模板注册、computed 或 watch。
- **为什么 event 绑定不能放在按钮动作中？** 普通控件事件不能直接读取原始载荷。由内部动作 emit，再在父级处理动作中用 event。
- **事件会自动冒泡吗？** 不会。父模块需在处理动作中显式再次 emit。

相关：[模块专题](../13-module-component.md) · [模块通信契约](../14-module-communication.md) · [作用域](../12-scoped-state.md)。
