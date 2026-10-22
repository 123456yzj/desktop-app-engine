# 18 BarChart 柱状图

[组件总览](README.md) / BarChart

比较多个数值位置上的大小。内置柱图使用数字 X 轴，多个系列以分组方式绘制。需要字符串类目或更丰富图表配置时可使用 [插件组件](19-plugin.md)。

## 代码演示：分组柱图

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "构建耗时比较", "window": {"width": 880, "height": 500}},
  "state": {
    "before": [{"project": 1, "seconds": 48}, {"project": 2, "seconds": 65}, {"project": 3, "seconds": 40}],
    "after": [{"project": 1, "seconds": 32}, {"project": 2, "seconds": 44}, {"project": 3, "seconds": 29}]
  },
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24},
    "children": [
      {
        "type": "barChart", "id": "duration", "title": {"kind": "literal", "value": "项目 1 / 2 / 3 耗时（秒）"},
        "layout": {"height": {"fixed": 340}},
        "series": [
          {"id": "before", "name": "优化前", "data": {"kind": "state", "key": "before"}, "xField": "project", "yField": "seconds"},
          {"id": "after", "name": "优化后", "data": {"kind": "state", "key": "after"}, "xField": "project", "yField": "seconds"}
        ],
        "legend": true, "zoom": false, "pan": false
      }
    ]
  },
  "actions": {}
}
```

预期效果：每个项目数字位置显示两个系列的柱子，可通过图例隐藏系列。自动 Y 范围包含零。

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"barChart"` | 分组柱图 |
| `layout.height` | fixed 轨道 | 必填且 >=120 | 固定图表高度 |
| `series` | 系列数组，最多 32 项 | 必填 | 字段同 [折线图 Series](17-line-chart.md#series-系列) |
| `title` | 字符串绑定 | 无 | 标题 |
| `xAxis` / `yAxis` | linear 轴对象 | 自动范围 | 可设置 min/max |
| `legend` | 布尔 | true | 图例 |
| `zoom` / `pan` | 布尔 | false | 缩放和平移 |

每系列最多 2,000 点。X 必须是有限数字，Y 是有限数字或 null；坐标与手动轴范围绝对值不超过 1e100。其他公共字段见 [公共属性](02-common.md)。

## 常见问题

- **能直接把项目名称放入 X 吗？** 内置柱图的 X 需要数字。分类文字轴可使用 ECharts 插件。
- **支持堆叠、横向条形或单柱颜色吗？** 当前内置 API 没有这些字段，系列色由主题分配。
- **能响应柱子点击吗？** 内置柱图没有业务事件；插件可按 manifest 提供 click。

相关：[LineChart](17-line-chart.md) · [Plugin](19-plugin.md) · [Metric](13-metric.md)。
