# 16 Table 表格

[组件总览](README.md) / Table

展示对象数组，内置固定表头、固定行高、可见行绘制、排序、单行选择、列宽拖动和双轴滚动。

## 代码演示：数据列与格式化

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "服务器表格", "window": {"width": 820, "height": 500}},
  "state": {
    "notice": "点击行或列标题试试",
    "servers": [{"id": "a", "name": "节点 A", "load": 0.42, "online": true}, {"id": "b", "name": "节点 B", "load": 0.18, "online": false}, {"id": "c", "name": "节点 C", "load": 0.76, "online": true}]
  },
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "text", "id": "notice", "text": {"kind": "state", "key": "notice"}},
      {
        "type": "table", "id": "servers", "rows": {"kind": "state", "key": "servers"}, "rowKey": "id", "rowHeight": 36,
        "layout": {"height": {"fr": 1}},
        "columns": [
          {"id": "name", "field": "name", "title": "节点", "width": {"fr": 1}},
          {"id": "load", "field": "load", "title": "负载", "width": {"fixed": 160}, "format": {"type": "percent", "precision": 1}},
          {"id": "online", "field": "online", "title": "在线", "width": {"auto": true}}
        ],
        "events": {"rowSelect": "selected", "sortChange": "sorted"}
      }
    ]
  },
  "actions": {
    "selected": {"steps": [{"type": "set", "target": "notice", "value": {"kind": "literal", "value": "已改变选中行"}}]},
    "sorted": {"steps": [{"type": "set", "target": "notice", "value": {"kind": "literal", "value": "已改变排序"}}]}
  }
}
```

预期效果：负载显示为百分比，在线显示 true/false；点击列标题切换排序，点击行高亮选择并更新提示；拖动表头列边缘改变宽度。行获得焦点后可用上下方向键移动选择。

## 代码演示：受控分页

每页两行，翻页先写回 page，再执行 pageChange 动作；下方指标读取当前页。排序作用于全体数据，再截取当前页。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "表格分页", "window": {"width": 640, "height": 400}},
  "state": {"page": 1},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "table", "id": "users", "rows": {"kind": "literal", "value": [{"id": "a", "name": "Ada"}, {"id": "b", "name": "Alan"}, {"id": "c", "name": "Grace"}]}, "rowKey": "id", "columns": [{"id": "name", "field": "name", "title": "姓名", "width": {"fr": 1}}], "pageSize": 2, "current": {"kind": "state", "key": "page"}, "bordered": true, "emptyText": "暂无用户", "layout": {"height": {"fixed": 200}}},
    {"type": "metric", "id": "current", "label": {"kind": "literal", "value": "当前页"}, "value": {"kind": "state", "key": "page"}}
  ]},
  "actions": {}
}
```

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"table"` | 数据表格 |
| `rows` | 对象数组绑定 | 必填 | 每项结构一致 |
| `rowKey` | 字符串 | 必填 | 行对象中的唯一字符串字段 |
| `columns` | 列对象数组，1..64 项 | 必填 | 见下表 |
| `rowHeight` | 16..512 | 32 | 固定行高 |
| `pageSize` | 整数 1..10000 或绑定 | 未设置 | 声明后启用分页；未声明时保留全量虚拟滚动 |
| `current` | 整数 1..1000000000 或绑定 | 1 | 当前页，从 1 开始；state 绑定时翻页写回 |
| `loading` | 布尔或绑定 | false | 显示加载指示，暂不显示表格内容 |
| `emptyText` | 字符串或绑定 | 暂无数据 | 数据为空时显示 |
| `bordered` | 布尔或绑定 | false | 绘制表格外边框 |
| `events.rowSelect` | 动作 ID 或绑定 | 无 | 选中行变化时触发 |
| `events.sortChange` | 动作 ID 或绑定 | 无 | 排序变化时触发 |
| `events.pageChange` | 动作 ID 或绑定 | 无 | 载荷为新页码数字；先写回 current，再执行动作 |

表格自然高度 280，建议设置 fixed 高度或在 Column 中使用 height.fr。表格自带双轴滚动，不能给叶节点配置 layout.overflow。

### Column 数据列

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `id` | 非空字符串 | 必填 | 表内唯一列 ID |
| `field` | 非空字符串 | 必填 | 行对象直接字段名，不是 JSON Pointer |
| `title` | 字符串 | 必填 | 表头标题，直接字符串 |
| `width` | fixed/fr/auto 轨道 | 必填 | 最小显示宽度 48；auto 最多抽样前 200 行 |
| `format` | 格式对象 | text/precision:0/unit:"" | number/percent 支持 0..12 位小数与后缀 |

同列非空值必须为同类数字、字符串或布尔，布尔只能用 text 格式。缺字段会报错；合法的可空单元格显示 `—`，稳定排序时 null 始终排末尾。state 中可空字段需要通过初始数组中的 null 和同字段非 null 值推导类型。

## 事件说明

rowSelect payload 为行键字符串，sortChange 为 field/descending 对象。JSON 动作可用 event Binding 读取载荷，使用时必须声明 eventPayload 样本。选择与排序保存在表格本地状态；分页可绑定 current，通过 pageChange 先写状态再调用动作。

选中行键跨页保留。current 超出最后一页时显示最后一页，状态本身不自动改写；未绑定 state 的页码保存在表格本地。pageSize 仅做本地数据分页，不自动请求远程页面。

再次点击同一行不重复触发 rowSelect。用户拖动后的列宽优先于自动计算。空数组显示“暂无数据”。

## 常见问题

- **支持操作列、自定义单元格、分页吗？** pageSize 启用排序后分页，current 可绑定状态；尚无筛选、展开行、单元格组件、合并单元格或可变行高。
- **能用 layout.columns 吗？** 不可以；数据列是组件顶层 columns，与 Grid 轨道不同。
- **排序会改写 rows 状态吗？** 不会，排序顺序是表格本地状态。

相关：[Grid](05-grid.md) · [Metric 格式](13-metric.md) · [公共属性](02-common.md)。
