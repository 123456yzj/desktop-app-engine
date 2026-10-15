# Select 下拉选择

协议 0.2 新增 `type: "select"`。`value` 为字符串绑定，绑定顶层 state 时选择会先写入该状态，再触发可选的 `change` 动作。组件支持通用的 visible、enabled、layout、style 和 placeholder。模块 prop 为只读，不能用作 value。

```json
{
  "type": "select",
  "id": "branch",
  "value": {"kind": "state", "key": "branch"},
  "options": {"kind": "state", "key": "branches"},
  "placeholder": {"kind": "literal", "value": "请选择分支"}
}
```

默认 `optionsFormat: "array"`，options 可以绑定字符串数组，例如 `["main", "dev"]`；也可以是对象数组，例如 `[{"label":"开发分支", "value":"dev", "disabled":false}]`。value 必须唯一，最多 10000 项，对象不接受未声明字段。

state 数组沿用引擎的固定元素类型约束，绑定对象数组时各项应使用相同字段，例如统一声明 disabled；literal 选项不受 state 元素类型约束。

`optionsFormat: "lines"` 时 options 绑定字符串，按换行拆分，去除每行首尾空白并忽略空行，每行同时用作 label 和 value。适合绑定 command 的 stdout：

```json
"options": {"kind":"state", "key":"branchJob", "path":"/stdout"},
"optionsFormat": "lines"
```

命令应只向 stdout 输出选项，进度或诊断放到 stderr，且不得输出重复项。下拉列表最多同时显示 10 项，更多选项可滚动查看；高度根据字号、内边距和行间距计算，并受屏幕可用空间限制。选项为空时仍可展开，以便处理 `open` 事件加载选项；禁用由 `enabled` 控制。每次从关闭变为展开时触发一次 `open`，载荷为 null；刷新选项期间保持展开不会重复触发。加载新选项不会自动选择第一项，也不会改变已有 value。已选值不在新列表中时保留显示，用户仍可选择新选项。选项禁用时不能选择，非法 change 载荷会被 Runtime 拒绝。

实现沿用 [底层组件设计](08-component-engine-design.md) 的 arena、布局和事件提交链路；下拉弹层由 egui ComboBox 提供焦点、键盘与浮层交互。[多仓库构建工具](20-turbo-build.md) 通过宿主接收 open 事件，在独立业务仓库刷新分支，选择后通过按钮执行 git checkout。
