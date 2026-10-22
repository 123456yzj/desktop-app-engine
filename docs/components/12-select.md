# 12 Select 下拉选择

[组件总览](README.md) / Select

从选项中选择一个字符串值。支持标签和值分离、禁用选项，以及从状态或命令输出加载选项。

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

## API

| 属性 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `type` | 字符串 | 必填 `"select"` | 单选下拉 |
| `value` | 字符串绑定 | 必填 | state 可写，path 为空；literal 为本地值 |
| `options` | 绑定 | 必填 | array 格式或 lines 字符串 |
| `optionsFormat` | array / lines | array | 选项解析方式 |
| `placeholder` | 字符串绑定 | 空 | 当前值为空且没有匹配项时显示 |
| `events.change` | 动作 ID 或绑定 | 无 | 写入新值后执行 |
| `events.open` | 动作 ID 或绑定 | 无 | 每次从关闭到展开时执行一次 |

array 格式支持字符串数组 `["dev","test"]`，或对象数组；对象字段为 `label`、`value`、可选 `disabled:false`。value 唯一，最多 10,000 项。state 对象数组的每项应声明同一组字段，避免类型结构不一致。

## 动态选项

将 options 绑定字符串状态并设置 optionsFormat 为 lines，可按换行拆出选项；去除每行首尾空白和空行。也可用 `{"kind":"state","key":"branchJob","path":"/stdout"}` 绑定命令输出，具体命令结果结构见 [命令文档](../10-command-binding.md)。

open 动作可请求数据，或执行已定义命令；仅刷新选项而保持展开不会重复触发 open。空选项仍可展开。加载后不会自动选第一项，也不会清空原值；旧值不在列表时保留显示。

## 常见问题

- **支持搜索和多选吗？** 当前提供单选，没有搜索、远程搜索或多选字段。
- **为什么只能看到 10 项？** 列表同时显示最多 10 项，更多内容通过滚动查看。
- **如何禁用整个下拉？** 使用公共 enabled 布尔绑定；禁用单项使用 disabled。
- **change 中如何读取新值？** 从绑定 state 读取，普通控件事件不直接提供 event 绑定。

相关：[Select 协议专题](../18-select.md) · [HTTP](../16-http-data-sources.md) · [公共属性](02-common.md)。
