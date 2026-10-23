# 28 Radio 单选框

用于选项较少、希望直接展示所有选项的单选场景。当前 `radio` 表示一个单选组，选中值为字符串；选项较多时使用 [Select](12-select.md)。

## 代码演示

两个单选组共用 mode，横向按钮与纵向单选视图会同步。禁用选项不能点击，方向键导航会跳过它。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Radio 示例", "window": {"width": 600, "height": 420}},
  "state": {"mode": "auto", "options": [{"label": "自动", "value": "auto", "disabled": false}, {"label": "手动", "value": "manual", "disabled": false}, {"label": "维护中", "value": "maintenance", "disabled": true}]},
  "root": {
    "type": "column", "id": "root", "layout": {"padding": 20, "gap": 16},
    "children": [
      {"type": "radio", "id": "horizontal", "optionType": "button", "value": {"kind": "state", "key": "mode"}, "options": {"kind": "state", "key": "options"}},
      {"type": "radio", "id": "vertical", "vertical": true, "value": {"kind": "state", "key": "mode"}, "options": {"kind": "state", "key": "options"}},
      {"type": "text", "id": "value", "text": {"kind": "state", "key": "mode"}}
    ]
  },
  "actions": {}
}
```

## API

| 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `radio` |
| value | 字符串绑定，必填 | 空字符串表示未选择；非空值必须在 options 中 |
| options | 数组绑定，必填 | 字符串数组，或 `{label, value, disabled?}` 对象数组；最多 256 项，value 唯一 |
| vertical | boolean | false；true 时纵向排列 |
| optionType | default / button 或绑定 | default；单选圆点或按钮外观，不改变字符串值和事件契约 |
| events.change | 动作 ID 或绑定 | 先写选中字符串，再执行动作 |

点击或聚焦后按 Space 选择。方向右/下选择下一个可用项，左/上选择上一个；首尾循环，跳过禁用项。Tab 可移出当前选项。value 绑定 state 只支持空 path；literal 在本地保存选择。

## 常见问题

- 更新 options 时，当前非空 value 仍须存在；使用 `set_many` 可在宿主中原子更新选项和值。
- 配置和程序赋值可以保留一个已被禁用的选中项，用户操作不能新选中它。
- 横向组选项不会自动换行，窄区域建议使用 vertical。公共样式和布局见 [02 公共属性](02-common.md)。

[组件总览](README.md) · [Checkbox](27-checkbox.md) · [Form](32-form.md)
