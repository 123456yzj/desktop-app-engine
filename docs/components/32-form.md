# 32 Form 表单

用于一组输入字段的标签、业务校验、统一提交和重置。Form 负责处理事件，FormItem 管理一个绑定 state 的输入控件。字段初始不满足必填规则时，应用仍能正常加载。

## 代码演示

### 校验、提交与重置

直接提交会显示字段错误并更新状态说明。姓名至少两个字符、年龄至少 18、同意条款后再次提交，结果显示最新姓名。输入框按 Enter 也会提交表单；重置恢复初始字段值并清除错误。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Form 示例", "window": {"width": 640, "height": 580}},
  "state": {"name": "", "age": 16, "agreed": false, "status": "尚未提交"},
  "root": {
    "type": "column", "id": "root", "layout": {"padding": 20, "gap": 12, "overflow": "scroll"},
    "children": [
      {"type": "form", "id": "profile", "title": {"kind": "literal", "value": "用户资料"}, "events": {"submit": "save", "invalid": "invalid", "reset": "reset"}, "children": [
        {"type": "formItem", "id": "nameItem", "label": {"kind": "literal", "value": "姓名"}, "rules": [{"required": true, "message": "请输入姓名"}, {"minLength": 2, "maxLength": 20}], "children": [
          {"type": "input", "id": "name", "value": {"kind": "state", "key": "name"}, "placeholder": {"kind": "literal", "value": "至少两个字符"}}
        ]},
        {"type": "formItem", "id": "ageItem", "label": {"kind": "literal", "value": "年龄"}, "rules": [{"min": 18, "max": 100, "message": "年龄须在 18 到 100 之间"}], "children": [
          {"type": "inputNumber", "id": "age", "value": {"kind": "state", "key": "age"}, "min": 0, "max": 120, "step": 1, "precision": 0}
        ]},
        {"type": "formItem", "id": "agreementItem", "rules": [{"required": true, "message": "请先同意条款"}], "children": [
          {"type": "checkbox", "id": "agreement", "value": {"kind": "state", "key": "agreed"}, "text": {"kind": "literal", "value": "同意条款"}}
        ]},
        {"type": "space", "id": "buttons", "children": [
          {"type": "button", "id": "saveButton", "htmlType": "submit", "text": {"kind": "literal", "value": "提交"}},
          {"type": "button", "id": "resetButton", "htmlType": "reset", "text": {"kind": "literal", "value": "重置"}}
        ]}
      ]},
      {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}}
    ]
  },
  "actions": {
    "save": {"steps": [{"type": "concat", "target": "status", "parts": [{"kind": "literal", "value": "已提交："}, {"kind": "state", "key": "name"}]}]},
    "invalid": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "请修正表单错误"}}]},
    "reset": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "已恢复初始值"}}]}
  }
}
```

### 正则校验与数组必填

账号要求 3～12 位小写字母、数字或下划线；权限组至少选择一项。pattern 使用 Rust regex，锚点 `^` 和 `$` 明确要求整个字符串匹配。两个字段合法后提交，状态显示“校验通过”。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "正则与数组校验", "window": {"width": 640, "height": 440}},
  "state": {"account": "", "permissions": [], "status": "尚未提交"},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "form", "id": "access", "events": {"submit": "saved"}, "children": [
      {"type": "formItem", "id": "accountItem", "label": {"kind": "literal", "value": "账号"}, "rules": [{"required": true}, {"pattern": "^[a-z0-9_]{3,12}$", "message": "请输入 3～12 位小写字母、数字或下划线"}], "children": [
        {"type": "input", "id": "account", "value": {"kind": "state", "key": "account"}}
      ]},
      {"type": "formItem", "id": "permissionsItem", "label": {"kind": "literal", "value": "权限"}, "rules": [{"required": true, "message": "至少选择一项权限"}], "children": [
        {"type": "checkbox", "id": "permissions", "value": {"kind": "state", "key": "permissions"}, "options": ["read", "write", "admin"]}
      ]},
      {"type": "button", "id": "submit", "htmlType": "submit", "text": {"kind": "literal", "value": "提交"}}
    ]},
    {"type": "text", "id": "status", "text": {"kind": "state", "key": "status"}}
  ]},
  "actions": {"saved": {"steps": [{"type": "set", "target": "status", "value": {"kind": "literal", "value": "校验通过"}}]}}
}
```

### 表单局部状态

表单可声明 state/actions，遵循现有词法作用域。页面的同名状态不影响表单。重置只恢复 FormItem 管理的 count，result 等其他状态由 reset 动作自行处理。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "局部表单", "window": {"width": 600, "height": 360}},
  "state": {"count": 99},
  "root": {
    "type": "form", "id": "settings", "layout": {"padding": 20},
    "state": {"count": 3, "result": 0},
    "actions": {"save": {"steps": [{"type": "set", "target": "result", "value": {"kind": "state", "key": "count"}}]}},
    "events": {"submit": "save"},
    "children": [
      {"type": "formItem", "id": "countItem", "label": {"kind": "literal", "value": "并发数"}, "rules": [{"min": 1, "max": 8}], "children": [
        {"type": "inputNumber", "id": "count", "value": {"kind": "state", "key": "count"}, "min": 1, "max": 8, "precision": 0}
      ]},
      {"type": "button", "id": "save", "htmlType": "submit", "text": {"kind": "literal", "value": "保存设置"}},
      {"type": "button", "id": "reset", "htmlType": "reset", "text": {"kind": "literal", "value": "重置并发数"}},
      {"type": "metric", "id": "result", "label": {"kind": "literal", "value": "已保存并发数"}, "value": {"kind": "state", "key": "result"}}
    ]
  },
  "actions": {}
}
```

## API

| Form 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `form`，纵向排列，默认 gap=12 |
| children | 组件数组 | []；可嵌套布局容器，不能嵌套另一个 Form |
| title | 字符串绑定 | 可选标题 |
| state/actions | 对象 | 局部状态和动作；参见 [Module](08-module.md) |
| events.submit | 动作 ID 或绑定 | 校验通过后执行，通过 state 读取当前字段值 |
| events.invalid | 动作 ID 或绑定 | 校验失败后执行，同时显示行内错误 |
| events.reset | 动作 ID 或绑定 | 字段恢复初始值并清除校验状态后执行 |

| FormItem 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `formItem`，必须位于 Form 内 |
| children | 数组，必填 | 恰好一个输入控件；包括基础输入及 [扩展输入](34-native-inputs.md) 中的 AutoComplete/Cascader/ColorPicker/DatePicker/Mentions/Rate/TimePicker/Transfer/TreeSelect/Upload/Calendar，以及 Segmented |
| label | 字符串绑定 | 可选顶部标签；required 时显示 `*` |
| rules | 规则数组 | []，最多 32 条；按顺序显示第一个失败信息 |

| 规则字段 | 类型 | 说明 |
| --- | --- | --- |
| required | boolean | true 时拒绝空白字符串、空数组和 false；数字 0 有效 |
| minLength/maxLength | 非负整数 | 仅字符串字段，按 Unicode 字符数计算，上限 16384 |
| min/max | number | 仅数字字段，有限且绝对值 <=1e12；与控件输入范围分别配置 |
| pattern | 字符串 | Rust regex 格式，最多 4096 字节，仅字符串字段；规则编译失败会拒绝加载 |
| message | 字符串 | 可选自定义错误文字，缺省自动生成；不能是空白字符串 |

## 交互与边界

- FormItem 的控件必须绑定可写顶层 state，不能用 literal、prop 或非空 path。同一 Form 的 FormItem 不得重复绑定同一状态。
- submit/reset 按钮设置 `htmlType`，默认 `button`；必须位于 Form 内。若同时声明 events.click，会先执行按钮动作，再处理表单。
- Input/InputNumber 按 Enter 时，先处理自己的 change/submit，再提交最近的 Form。Form 的 submit 动作只在校验通过后执行一次。
- 仅可见且启用的字段参与校验，祖先的 visible/enabled 同样生效。首次提交后，状态变化会刷新错误，直到重置。
- 重置原子恢复所有受管理字段的文档初始值，包括隐藏和禁用字段；普通展示状态不自动重置。若初始值不满足当前动态选项等协议约束，重置失败并保留原值。
- 规则非空的 FormItem 始终保留一行错误区域，避免提交后布局跳动；长错误信息截断并提供悬停提示。
- 当前没有嵌套字段路径、动态字段列表、异步校验、横向标签布局或自动滚动聚焦。FormItem 不接受自己的业务 events。

[组件总览](README.md) · [Input](11-input.md) · [公共属性](02-common.md)
