# 30 InputNumber 数字输入框

用于数量、金额、阈值等数值输入，直接读写 JSON number。需要任意字符串或允许空文本时使用 [Input](11-input.md)。

## 代码演示

直接输入数字、点击加减按钮或聚焦后按上下键。价格按两位小数显示并提交；下方指标读取同一数字状态。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "InputNumber 示例", "window": {"width": 600, "height": 360}},
  "state": {"price": 12.5, "submitted": 0},
  "root": {
    "type": "column", "id": "root", "layout": {"padding": 20, "gap": 12},
    "children": [
      {"type": "inputNumber", "id": "price", "value": {"kind": "state", "key": "price"}, "min": 0, "max": 1000, "step": 0.1, "precision": 2, "prefix": "¥", "suffix": "元", "events": {"submit": "submit"}},
      {"type": "inputNumber", "id": "readOnlyPrice", "value": {"kind": "state", "key": "price"}, "min": 0, "max": 1000, "precision": 2, "readOnly": true, "controls": false, "prefix": "只读价格"},
      {"type": "metric", "id": "current", "label": {"kind": "literal", "value": "当前价格"}, "value": {"kind": "state", "key": "price"}, "format": {"type": "number", "precision": 2, "unit": " 元"}},
      {"type": "metric", "id": "submitted", "label": {"kind": "literal", "value": "按 Enter 提交的价格"}, "value": {"kind": "state", "key": "submitted"}, "format": {"type": "number", "precision": 2}}
    ]
  },
  "actions": {"submit": {"steps": [{"type": "set", "target": "submitted", "value": {"kind": "state", "key": "price"}}]}}
}
```

## API

| 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `inputNumber` |
| value | number 绑定，必填 | 范围内的有限数字；state 只支持顶层键 |
| min/max | number | -1e12 / 1e12；要求 min < max，绝对值不超过 1e12 |
| step | number | 1；允许 1e-12..1e12，作用于按钮和上下键 |
| precision | 整数 | 可选 0..12；设置后固定显示小数位，交互时按此精度舍入 |
| readOnly | 布尔或绑定 | false；禁止编辑、按钮和键盘步进 |
| controls | 布尔或绑定 | true；是否显示加减按钮，false 时仍可输入和键盘步进 |
| prefix / suffix | 字符串或绑定 | 空；前后缀文字不写入 value |
| events.change | 动作 ID 或绑定 | 有效数字变化先写状态，再执行动作 |
| events.submit | 动作 ID 或绑定 | 聚焦编辑框后按 Enter；在 Form 内还会触发表单校验 |

## 常见问题

- 编辑过程中允许短暂出现空串、负号等中间文本，但它们不写入 state；失焦或 Enter 时恢复最后有效数字。
- 输入范围外的有限数字会按边界限制；程序赋值越界会被拒绝。NaN、Infinity、null 和数字字符串不能作为 value。
- step 不限制手工输入；例如 step=1 仍可输入 1.5。需要整数交互时同时设置 precision=0。
- 外部状态变化会同步编辑框。程序赋值不自动舍入；希望显示与存储始终一致时，宿主应提供符合 precision 的数字。
- literal 只保留控件本地值；参与统一校验、提交或重置时使用 [Form](32-form.md) 与 state 绑定。
- 当前没有 stringMode、formatter/parser 回调或字符串精度模式，货币符号使用 prefix/suffix，不改变 number 值。

[组件总览](README.md) · [Slider](31-slider.md) · [公共属性](02-common.md)
