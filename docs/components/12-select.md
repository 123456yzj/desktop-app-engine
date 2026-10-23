# 12 Select 下拉选择

[组件总览](README.md) / Select

从选项中单选、多选或输入标签。支持标签和值分离、禁用选项、搜索和清除，以及从状态或命令输出加载选项。

## 代码演示：标签、值与联动

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "环境选择", "window": {"width": 680, "height": 360}},
  "state": {
    "environment": "dev", "message": "尚未修改选择",
    "environments": [{"label": "开发环境", "value": "dev", "disabled": false}, {"label": "测试环境", "value": "test", "disabled": false}, {"label": "生产环境（暂不可选）", "value": "prod", "disabled": true}]
  },
  "root": {
    "type": "column", "id": "page", "layout": {"padding": 24, "gap": 16},
    "children": [
      {"type": "select", "id": "environment", "value": {"kind": "state", "key": "environment"}, "options": {"kind": "state", "key": "environments"}, "placeholder": {"kind": "literal", "value": "请选择环境"}, "events": {"change": "changed"}},
      {"type": "text", "id": "value", "text": {"kind": "state", "key": "environment"}},
      {"type": "text", "id": "message", "text": {"kind": "state", "key": "message"}}
    ]
  },
  "actions": {"changed": {"steps": [{"type": "concat", "target": "message", "parts": [{"kind": "literal", "value": "已选择："}, {"kind": "state", "key": "environment"}]}]}}
}
```

预期效果：下拉显示中文标签，预览显示 dev/test。生产环境不可选，切换后提示同步更新。

## 代码演示：搜索、多选与标签

第一个下拉最多选择两项，第二个允许输入 options 以外的新标签。多选值为不重复的非空字符串数组，清除后为 `[]`。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "多选与标签", "window": {"width": 640, "height": 360}},
  "state": {"permissions": [], "tags": []},
  "root": {"type": "column", "id": "page", "layout": {"padding": 24, "gap": 16}, "children": [
    {"type": "select", "id": "permissions", "value": {"kind": "state", "key": "permissions"}, "options": {"kind": "literal", "value": ["read", "write", "admin"]}, "mode": "multiple", "showSearch": true, "allowClear": true, "maxCount": 2, "notFoundContent": "没有匹配权限"},
    {"type": "select", "id": "tags", "value": {"kind": "state", "key": "tags"}, "options": {"kind": "literal", "value": ["Rust", "Desktop"]}, "mode": "tags", "showSearch": true, "allowClear": true}
  ]},
  "actions": {}
}
```

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"select"` | 选择控件 |
| `value` | 字符串或字符串数组绑定 | 必填 | single 为字符串，multiple/tags 为数组；state 可写且 path 为空，literal 为本地值 |
| `options` | 绑定 | 必填 | array 格式或 lines 字符串 |
| `optionsFormat` | array / lines | array | 选项解析方式 |
| `placeholder` | 字符串绑定 | 空 | 当前值为空且没有匹配项时显示 |
| `mode` | single / multiple / tags 或绑定 | single | 单选、多选或可添加标签 |
| `showSearch` | 布尔或绑定 | false | 按选项标签过滤；tags 模式需开启以输入新标签 |
| `allowClear` | 布尔或绑定 | false | 清除单选为 `""`，清除多选为 `[]` |
| `loading` | 布尔或绑定 | false | 弹出列表显示加载指示 |
| `maxCount` | 整数 1..10000 或绑定 | 10000 | multiple/tags 允许选择的最大数量 |
| `notFoundContent` | 字符串或绑定 | 暂无选项 | 搜索无结果时显示 |
| `events.change` | 动作 ID 或绑定 | 无 | 写入新值后执行 |
| `events.open` | 动作 ID 或绑定 | 无 | 每次从关闭到展开时执行一次 |
| `events.search` | 动作 ID 或绑定 | 无 | 搜索文字变化，载荷为字符串 |
| `events.clear` | 动作 ID 或绑定 | 无 | 清除按钮触发，载荷为 null；先 change，再 clear |

array 格式支持字符串数组 `["dev","test"]`，或对象数组；对象字段为 `label`、`value`、可选 `disabled:false`。value 唯一，最多 10,000 项。state 对象数组的每项应声明同一组字段，避免类型结构不一致。

## 动态选项

将 options 绑定字符串状态并设置 optionsFormat 为 lines，可按换行拆出选项；去除每行首尾空白和空行。也可用 `{"kind":"state","key":"branchJob","path":"/stdout"}` 绑定命令输出，具体命令结果结构见 [命令文档](../10-command-binding.md)。

open 动作可请求数据，或执行已定义命令；仅刷新选项而保持展开不会重复触发 open。空选项仍可展开，加载后不会自动选第一项。single/multiple 的非空选中值必须属于 options；动态移除选项时需同时调整选中值，可由宿主使用 set_many 原子更新。tags 允许 options 之外的非空值。

## 常见问题

- **支持远程搜索吗？** search 动作可请求远程数据，再由应用更新 options；控件本身不自动发请求或防抖。
- **列表如何显示更多选项？** 下拉列表提供滚动，单选原有视图与带搜索/多选的视图显示高度不同。
- **如何禁用整个下拉？** 使用公共 enabled 布尔绑定；禁用单项使用 disabled。
- **change 中如何读取新值？** 从已更新的 state 读取，或使用 event Binding；后者须声明与字符串/数组匹配的 eventPayload 样本。

相关：[Select 协议专题](../18-select.md) · [HTTP](../16-http-data-sources.md) · [公共属性](02-common.md)。
