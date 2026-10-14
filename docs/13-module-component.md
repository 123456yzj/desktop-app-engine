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

示例 examples/modules.json 用 module 封装页面和两个独立模块，卡片位于模块内部。同一个按钮“下一步 → 提交 → 已提交”，模块 A 的操作不影响模块 B。

改动：src/v02.rs（Module 类型和严格字段校验）、src/scope.rs（局部作用域声明）、docs/description-0.2.schema.json、examples/modules.json、tests/module.rs、scripts/schema-verify.py、README.md、docs/05-description-format.md。现有布局和绘制已支持纵向容器路径，module 复用该路径。

## 验证与产物

本地 GNU 离线工具链，以下实际命令均退出 0：

| 命令 | 结果 |
| --- | --- |
| cargo fmt -- --check | 通过 |
| cargo clippy --all-targets --locked --offline -- -D warnings | 通过，无警告 |
| cargo test --locked --offline | 49 passed，0 failed，0 ignored（module 新增 4 项） |
| python scripts/schema-verify.py | 模块及已有示例、非法 module 字段/事件验证通过 |
| cargo build --release --locked --offline | Finished release，21.93s |
| scripts/package.ps1 -Description examples/modules.json -Output exports -Name ModuleApp | 导出与独立包 --check 通过 |

模块测试覆盖真实布局矩形、动态按钮属性更新、同名状态/动作独立、嵌套继承与遮蔽、空模块、父级禁用传播、非法专属字段/事件及跨作用域引用。测试通过后台事件和布局接口，不操作鼠标或切换窗口。日志：evidence/module-verification.txt。

最新运行时：target-verify/release/desktop-app-engine.exe。示例包：exports/ModuleApp/ModuleApp.exe、app.json、README.txt；先前导出包不自动替换。
