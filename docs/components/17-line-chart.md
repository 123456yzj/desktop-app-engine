# 17 LineChart 折线图

[组件总览](README.md) / LineChart

展示数值 X/Y 趋势，支持多个系列、图例显隐、悬停、滚轮缩放和拖拽平移。X 为数字坐标。

## 代码演示：趋势、断点与缩放

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "负载趋势", "window": {"width": 880, "height": 500}},
  "state": {"trend": [{"time": 0, "load": 0.2}, {"time": 1, "load": 0.5}, {"time": 2, "load": null}, {"time": 3, "load": 0.35}, {"time": 4, "load": 0.7}]},
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24},
    "children": [
      {
        "type": "lineChart", "id": "trend", "title": {"kind": "literal", "value": "CPU 负载趋势"},
        "layout": {"height": {"fixed": 340}},
        "series": [{"id": "cpu", "name": "CPU", "data": {"kind": "state", "key": "trend"}, "xField": "time", "yField": "load", "monotonicX": true}],
        "xAxis": {"scale": "linear"}, "yAxis": {"min": 0, "max": 1}, "legend": true, "zoom": true, "pan": true
      }
    ]
  },
  "actions": {}
}
```

预期效果：time=2 的空值形成折线断点。悬停查看数据，滚轮缩放，拖拽平移，双击或图表聚焦后按 Home 恢复自动范围；点击图例隐藏或显示系列。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"lineChart"` | 折线图 |
| `layout.height` | fixed 轨道 | 必填且 >=120 | 当前必须明确固定高度 |
| `series` | 系列数组，最多 32 项 | 必填 | 可为空数组 |
| `title` | 字符串绑定 | 无 | 图表标题 |
| `xAxis` / `yAxis` | 坐标轴对象 | 自动范围 | scale 仅 linear；可选 min/max |
| `legend` | 布尔 | true | 图例显隐 |
| `zoom` | 布尔 | false | 是否消费图内滚轮进行缩放 |
| `pan` | 布尔 | false | 是否允许拖拽平移 |

### Series 系列

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `id` / `name` | 非空字符串 | 必填 | id 图内唯一，name 为图例名 |
| `data` | 对象数组绑定 | 必填 | 系列数据 |
| `xField` / `yField` | 非空字符串 | 必填 | 对象直接字段名 |
| `monotonicX` | 布尔 | false | 声明有效点 X 非递减，支持按屏幕宽度采样 |

X 必须为有限数字，Y 为有限数字或 null；坐标与手动轴范围绝对值不超过 1e100。非单调系列最多 20,000 点；monotonicX:true 必须与实际顺序一致。min/max 同时指定时必须 min<max。

## 常见问题

- **可以把 height 改成 fr 吗？** 当前图表校验要求 fixed>=120；可由外部容器分配布局，但图表自身仍需固定高度。
- **支持日期轴或字符串分类吗？** 当前只有线性数字轴；日期可预先转为数字，但没有日期格式化配置。
- **点击点能触发动作吗？** 当前没有业务 events；图例、悬停、缩放属于本地交互。
- **zoom:false 时滚轮去哪？** 交给外层可滚动容器。

相关：[BarChart](18-bar-chart.md) · [Plugin](19-plugin.md) · [公共属性](02-common.md)。
