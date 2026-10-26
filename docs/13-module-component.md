# module：模块组件与局部响应式变量

协议 0.2 内置 type=module。模块内声明 state 作为局部响应式变量、actions 作为模块动作、children 作为内部界面。模块之间的同名变量和动作各自独立。变量变化驱动绑定属性更新，按钮动作在每次事件提交时按模块上下文选择。

```json
{
  "type": "module",
  "id": "editorModule",
  "state": {"message":"初始内容"},
  "actions": {
    "reset": {"steps":[{"type":"set","target":"message","value":{"kind":"literal","value":"初始内容"}}]}
  },
  "layout": {"gap":12},
  "children": [
    {"type":"input","id":"editorInput","value":{"kind":"state","key":"message"}},
    {"type":"text","id":"editorPreview","text":{"kind":"state","key":"message"}},
    {"type":"button","id":"editorReset","text":{"kind":"literal","value":"重置"},"events":{"click":"reset"}}
  ]
}
```

输入变化会同步更新模块内预览。外部组件无法引用此模块的 message。不同 module 可以重复声明 message/reset，组件 id 仍全树唯一。可以在 module 内放 card，再在 card 内组织 row/grid：module 表达模块边界，布局组件组织外观。

module 具有纵向容器布局、尺寸约束、gap/padding、对齐、滚动与裁剪、公共 style、visible/enabled。默认不添加背景、边框或标题；不接受 title、columns、业务 events；wrap=true 被拒绝。内部用 row 执行横向布局和换行，用 grid 定义网格。空模块合法，局部 state/actions 可省略。

名字就近查找，后代可继承祖先模块和应用级变量。嵌套模块同名变量遮蔽祖先变量。祖先动作保持其声明位置的读写范围。响应式状态、动态动作、命令结果等均沿用 docs/12 的作用域规则。已有容器作用域配置仍兼容。

这是组件实例的作用域封装。现已增加 props/emit/on 通信，详见 [模块通信](14-module-communication.md)。尚未提供 Vue 的模板注册/复用、computed、watch 或 JavaScript 执行环境；隐藏模块保留状态，无动态卸载生命周期。模块组件不依赖前端框架。

类型与结构解析位于 `src/protocol/`，作用域编译位于 `src/compiler/scope.rs`。
