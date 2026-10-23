# 31 Slider 滑动输入条

用于音量、比例、阈值等范围内的近似选择。需要精确输入时可与 [InputNumber](30-input-number.md) 绑定同一状态。

## 代码演示

横向滑条、纵向滑条和数字框共用 volume。拖动后按 5 的步长吸附，数值框允许精确输入；指标显示最新值。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "Slider 示例", "window": {"width": 600, "height": 440}},
  "state": {"volume": 35},
  "root": {
    "type": "column", "id": "root", "layout": {"padding": 20, "gap": 12},
    "children": [
      {"type": "slider", "id": "horizontal", "value": {"kind": "state", "key": "volume"}, "min": 0, "max": 100, "step": 5},
      {"type": "row", "id": "controls", "layout": {"gap": 16, "align": "start"}, "children": [
        {"type": "slider", "id": "vertical", "vertical": true, "value": {"kind": "state", "key": "volume"}, "min": 0, "max": 100, "step": 5, "layout": {"width": {"fixed": 40}, "height": {"fixed": 180}}},
        {"type": "inputNumber", "id": "number", "value": {"kind": "state", "key": "volume"}, "min": 0, "max": 100, "step": 5, "precision": 0}
      ]},
      {"type": "metric", "id": "value", "label": {"kind": "literal", "value": "音量"}, "value": {"kind": "state", "key": "volume"}, "format": {"type": "number", "unit": "%"}}
    ]
  },
  "actions": {}
}
```

## 代码演示：范围、刻度与反向

两个范围滑块共用升序数组；第二个反向显示，数值顺序仍保持从小到大。拖动靠近的滑块可调整对应端点，键盘方向键调整当前端点。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "范围滑块", "window": {"width": 620, "height": 280}},
  "state": {"range": [20, 80]},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 24}, "children": [
    {"type": "slider", "id": "range", "value": {"kind": "state", "key": "range"}, "range": true, "min": 0, "max": 100, "step": 5, "marks": [{"value": 0, "label": "低"}, {"value": 50, "label": "中"}, {"value": 100, "label": "高"}]},
    {"type": "slider", "id": "reversed", "value": {"kind": "state", "key": "range"}, "range": true, "reverse": true, "min": 0, "max": 100, "step": 5}
  ]},
  "actions": {}
}
```

## API

| 字段 | 类型 | 默认/说明 |
| --- | --- | --- |
| type | 字符串 | `slider` |
| value | number 或二元数字数组绑定，必填 | range=false 为数字，range=true 为 `[低值, 高值]`；state 绑定顶层键，literal 为本地状态 |
| min/max | number | 0 / 100；要求 -1e12 <= min < max <= 1e12 |
| step | number | 1；允许 1e-12..1e12，以 min 为起点吸附 |
| vertical | boolean | false；true 时从下向上递增 |
| range | 布尔或绑定 | false；开启双滑块，两个值须在 min..max 内且低值 <= 高值 |
| reverse | 布尔或绑定 | false；反转显示方向，不反转数组的升序约束 |
| marks | 数组或绑定 | 无；项为 `{value: 数字, label: 字符串}`，用于刻度文字，不改变 step 吸附 |
| events.change | 动作 ID 或绑定 | 载荷为数字或二元数组；先写状态，再执行动作 |

点击或拖动轨道调整数值；点击后保持焦点，右/上键增加 step，左/下键减少，reverse 时方向相反。单滑块还支持 Home/End 到达 min/max 和悬停显示当前数值。范围模式通过鼠标选定活动端点，再用方向键调整，端点不能交叉。默认横向自然尺寸 220×34 point，纵向 40×180；公共尺寸可覆盖。

## 常见问题

- 当跨度不能被 step 整除时，max 仍可通过轨道端点或 End 到达。
- 程序设置的范围内数字可以不在步长网格上，用户下次交互时才吸附。
- 当前没有拖动结束事件或整段范围拖动；刻度不提供独立点击或格式化回调。
- value 必须处于范围内，越界配置和状态更新会被拒绝。

[组件总览](README.md) · [InputNumber](30-input-number.md) · [Form](32-form.md)
