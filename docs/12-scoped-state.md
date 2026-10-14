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

Runtime::state_from 与 set_from 为宿主提供按组件上下文的访问接口。运行时内部 state 仍使用合并的唯一键存储，局部作用域是配置访问隔离，不是任意 Rust 宿主代码的安全边界。纯布局和绘制接收运行时已解析的树；直接使用未编译描述构建纯 Tree 时需先经 scope::compile。

局部状态的类型和写入校验沿用 0.2 规则。状态仅在当前应用实例中存在；隐藏容器不清空局部状态；本次不增加页面路由、动态模块挂载/销毁、状态 watch、条件表达式或异步回调。

## 改动与测试

- src/scope.rs：按声明位置解析名字、拒绝跨模块访问、生成唯一状态/动作键。
- src/v02.rs：容器 state/actions、编译前与运行时校验分离、动态动作按组件作用域解析。
- src/interaction.rs：使用已解析状态、局部事件分派、宿主上下文访问接口。
- src/lib.rs：scope 模块导出。
- docs/description-0.2.schema.json、docs/05-description-format.md、README.md：作用域结构和语义。
- examples/scoped-state.json：两个模块使用同名按钮状态和动作，独立执行下一步/提交。
- tests/scoped_state.rs：模块隔离与增量更新、祖先继承和遮蔽、动作声明上下文、跨模块拒绝、非法写入、输入、真实后台命令结果隔离。
- scripts/schema-verify.py：作用域示例和叶组件非法声明校验。

初次编译暴露 Node 字段和 validate_flat 更新遗漏，修正后 scoped_state 的 6 项测试和严格 Clippy 均通过。不会将失败尝试计为验收通过。

最终补充页面作用域 Dashboard 数据绑定测试后，Clippy 指出测试使用 get().is_none()，已改为 contains_key 后重跑成功。最终结果：

| 实际命令 | 结果 | 退出码 |
| --- | --- | --- |
| cargo fmt -- --check | 通过 | 0 |
| cargo clippy --all-targets --locked --offline -- -D warnings | 通过，无警告 | 0 |
| cargo test --locked --offline | 45 passed，0 failed，0 ignored（作用域新增 7 项） | 0 |
| python scripts/schema-verify.py | Dashboard、命令、步骤按钮、局部作用域示例和非法结构验证通过 | 0 |
| cargo build --release --locked --offline | Finished release，30.34s | 0 |
| scripts/package.ps1 -Description examples/scoped-state.json -Output exports -Name ScopedStateApp | 导出和 --check 成功 | 0 |

日志：evidence/scoped-state-verification.txt，保留其中一次 Clippy 失败与最终成功。验证通过后台进程、事件提交和组件属性解析完成，没有操作鼠标或切换前台窗口。

最新运行时：target-verify/release/desktop-app-engine.exe。独立演示包：exports/ScopedStateApp/ScopedStateApp.exe、app.json、README.txt。此前导出包仍是各轮构建产物。
