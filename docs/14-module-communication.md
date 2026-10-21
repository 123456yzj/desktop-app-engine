# 模块通信：props、emit 与父级处理动作

模块保持私有 state/actions，通过显式接口交换数据：父级状态 → props → 子级界面；子级动作 → emit → 父级 on 处理动作 → 父级状态 → 其他模块 props。

## props

module 的 props 是名字到绑定的对象。声明位置按父作用域解析，不读取同一模块自己的局部状态。支持父级 state、literal，以及从父模块 prop 继续传递。

```json
"props": {"message":{"kind":"state","key":"sharedMessage"}}
```

内部组件和动作使用 `{"kind":"prop","key":"message","path":""}` 读取。path 遵循 RFC 6901，可读取对象字段；链式传递组合路径。父级更新后所有引用该 prop 的组件依赖同一个实际状态键版本，在下一帧更新；没有重复拷贝或额外派生状态。

Props 只读：set/concat/command 的 target 不可为 prop 名，input.value 不可直接绑定 prop。输入应写模块局部 state，然后 emit。prop 与当前 module 的 state 不可同名；内层的局部 state 可以遮蔽继承的 prop 名。外部组件无法读取子模块 props 和私有状态。既有祖先共享 state 访问方式保留兼容，新模块通信建议使用 props/emit；不是对任意宿主代码的安全沙箱。

## emits 与 on

module 的 emits 是事件名到载荷类型样本的对象。样本只声明类型，不是要发送的数据。沿用状态类型规则：string、number、boolean、null、array、object；对象字段结构一致。

```json
"emits": {"messageChanged":""},
"on": {"messageChanged":"receiveMessage"}
```

on 的动作在父作用域查找。可为固定动作 ID 或父级 state/literal 动作绑定。事件名必须出现在 emits 中。没有 on 的已声明事件允许发出，结果为不调用处理动作。on 不自动向祖先冒泡；父模块需要继续转发时，在处理动作里再次 emit 自己声明的事件。

模块内部动作新增 emit 步骤：

```json
{"type":"emit","event":"messageChanged","payload":{"kind":"state","key":"draft"}}
```

emit 按动作声明所在的最近 module 选择事件源，payload 支持 literal/state/prop，处理动作中也可用 event 转发。不能在应用顶层动作里 emit，因为没有模块事件源。

父级处理动作在自己的声明作用域读写状态，通过 event 读取载荷：

```json
"receiveMessage": {
  "steps":[{"type":"set","target":"sharedMessage","value":{"kind":"event"}}]
}
```

对象载荷可以用 `{"kind":"event","path":"/message"}` 读取字段。event 只在事件处理动作的步骤绑定中有效；不用于 props 声明、UI 属性或控件 events 动作 ID。依赖 event 的动作不能直接绑定按钮/input/table 事件，必须经 emit 调用。set/concat/emit 的值和 command 参数均支持 event。

## 类型、时序与动态处理动作

加载时以 emits 样本校验处理动作的 event 路径和赋值类型，同时校验实际 emit 载荷类型。Runtime 更新状态后再次校验监听动作与所有 props 路径，非法更新保留旧状态。初次绑定到动作的载荷契约会保留，动态 on 切换后不会让旧动作失去类型上下文。

尚未被任何 on 使用、但将来可能动态选择的 event 动作，可声明可选 eventPayload 类型样本（根值非 null）：

```json
"receiveOther": {
  "eventPayload":{"message":""},
  "steps":[{"type":"set","target":"sharedMessage","value":{"kind":"event","path":"/message"}}]
}
```

活动监听器的样本必须匹配显式 eventPayload。未声明 eventPayload 的动作从现有监听器推导；使用 event 但没有任何类型上下文的动作加载时报错。显式 eventPayload 动作只能通过模块事件调用。

输入事件先写局部 state，再执行动作；emit 在该步骤同步调用父级处理动作，然后继续原动作。每次动作独立捕获开始时快照，原动作后续步骤仍读取原快照。处理动作失败会停止后续步骤，之前成功写入保留。调用深度最多 32 层，超过会返回诊断而不是无限递归。处理动作启动 command 时后台执行，emit 不等待进程结束；本次没有新增进程完成回调、watch 或 computed。

## 实现与验证

`src/compiler/scope.rs` 解析 props、父级监听动作和 emit 所在模块；事件样例及名称映射保存在 `src/compiler/metadata.rs`。运行时动作读取这些元数据执行同步通信。
