# 页面与模块的响应式状态作用域

0.2 增加容器局部 state/actions，复用现有绑定和事件机制，无新增依赖。页面和模块由容器树表达，不引入业务专用管理器。

## 配置与范围

```json
{
  "type": "card",
  "id": "moduleA",
  "state": {"button":{"text":"下一步","action":"next","enabled":true}},
  "actions": {
    "next": {"steps":[{"type":"set","target":"button","value":{"kind":"literal","value":{"text":"已完成","action":"next","enabled":false}}}]}
  },
  "children": [{
    "type":"button",
    "id":"buttonA",
    "text":{"kind":"state","key":"button","path":"/text"},
    "enabled":{"kind":"state","key":"button","path":"/enabled"},
    "events":{"click":{"kind":"state","key":"button","path":"/action"}}
  }]
}
```

只有 moduleA 和其后代可读取这份 button 状态；另一模块也能声明 button/next，互不影响。页面可在外层 column 声明页面数据，模块后代可以继承；全局数据继续放顶层 state。查找顺序为当前容器、祖先容器、应用；同名就近遮蔽，没有跨模块引用语法。局部状态不会自动向父级发布。

动作在其声明作用域中解析，避免子模块调用祖先动作时写入子模块的同名状态。局部动作可显式更新可见的祖先共享状态，因此需要只影响模块的数据必须在模块内声明。动态动作状态保留用户配置的动作名，在具体组件作用域中解析。

模块所有绑定均在启动时解析：输入、可见性、禁用、文字、指标、表格、图表、动态事件、set/concat、命令参数与结果。内部键以组件 ID 和状态名生成，组件 ID 仍全树唯一；@scope/ 前缀保留且拒绝配置使用。Reactive 缓存版本按解析后的键计算，修改 A 的局部状态不会刷新 B 的同名状态依赖。

Runtime::state_from 与 set_from 为宿主提供按组件上下文的访问接口。运行时内部 state 使用合并的唯一键存储，只读访问器提供状态，更新统一经过校验。纯布局和绘制接收运行时已解析的树，宿主通过 `compiler::compile_at` 或 `Runtime::new` 编译原始描述。

局部状态的类型和写入校验沿用 0.2 规则。状态仅在当前应用实例中存在；隐藏容器不清空局部状态；本次不增加页面路由、动态模块挂载/销毁、状态 watch、条件表达式或异步回调。

作用域编译位于 `src/compiler/scope.rs`，名称映射保存在编译元数据中。
