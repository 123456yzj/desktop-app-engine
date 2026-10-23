# 34 扩展数据录入与树

提供 AutoComplete、Cascader、ColorPicker、DatePicker、Mentions、Rate、TimePicker、Transfer、TreeSelect、Upload、Calendar、Tree 的原生桌面输入。

## 代码演示

所有主值绑定 state。Upload 打开本机文件选择器，保留选中文件描述；实际发送文件由应用接管。示例不发起网络上传。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "扩展输入", "window": {"width": 900, "height": 760}},
  "state": {"query": "", "path": [], "color": "#1677ff", "date": "2026-10-22", "mention": "", "rating": 2.5, "time": "12:00:00", "target": [], "treeValue": "", "files": [], "day": "2026-10-22", "selected": [], "checked": [], "expanded": ["rootItem"]},
  "root": {"type": "column", "id": "root", "layout": {"padding": 16, "gap": 12, "overflow": "scroll"}, "children": [
    {"type": "autoComplete", "id": "query", "value": {"kind": "state", "key": "query"}, "options": ["Ada", "Alan", "Grace"], "placeholder": "输入姓名", "allowClear": true},
    {"type": "cascader", "id": "path", "value": {"kind": "state", "key": "path"}, "options": [{"value": "asia", "label": "亚洲", "children": [{"value": "china", "label": "中国"}]}], "showSearch": true},
    {"type": "colorPicker", "id": "color", "value": {"kind": "state", "key": "color"}, "showText": true, "presets": ["#1677ff", "#ff0000"]},
    {"type": "datePicker", "id": "date", "value": {"kind": "state", "key": "date"}, "minDate": "2026-01-01", "maxDate": "2026-12-31", "allowClear": true},
    {"type": "mentions", "id": "mention", "value": {"kind": "state", "key": "mention"}, "options": ["Ada", "Alan"], "prefix": "@", "rows": 3},
    {"type": "rate", "id": "rating", "value": {"kind": "state", "key": "rating"}, "allowHalf": true, "tooltips": ["很差", "较差", "一般", "很好", "优秀"]},
    {"type": "timePicker", "id": "time", "value": {"kind": "state", "key": "time"}, "use12Hours": true, "showNow": true},
    {"type": "transfer", "id": "transfer", "value": {"kind": "state", "key": "target"}, "options": ["read", "write", "admin"], "titles": ["可选权限", "已选权限"], "showSearch": true, "layout": {"height": {"fixed": 180}}},
    {"type": "treeSelect", "id": "treeSelect", "value": {"kind": "state", "key": "treeValue"}, "options": [{"value": "group", "label": "团队", "children": [{"value": "ada", "label": "Ada"}]}], "showSearch": true},
    {"type": "upload", "id": "upload", "value": {"kind": "state", "key": "files"}, "text": "选择文件", "multiple": true, "accept": ".png,.jpg,.txt", "maxCount": 5, "maxSize": 10485760},
    {"type": "calendar", "id": "calendar", "value": {"kind": "state", "key": "day"}, "layout": {"height": {"fixed": 270}}},
    {"type": "tree", "id": "tree", "treeData": [{"key": "rootItem", "title": "团队", "children": [{"key": "ada", "title": "Ada"}, {"key": "alan", "title": "Alan"}]}], "selectedKeys": {"kind": "state", "key": "selected"}, "checkedKeys": {"kind": "state", "key": "checked"}, "expandedKeys": {"kind": "state", "key": "expanded"}, "checkable": true, "layout": {"height": {"fixed": 150}}}
  ]},
  "actions": {}
}
```

## 值与 API

| type | value 类型 | 主要属性与行为 |
| --- | --- | --- |
| autoComplete | string | options、filterOption、allowClear；输入过滤并点击建议 |
| mentions | string | options、prefix、split、rows、allowClear；补全最后一个提及片段 |
| cascader | string[]；multiple 时 string[][] | options 递归 value/label/children、multiple、showSearch、allowClear、changeOnSelect |
| treeSelect | string；multiple/treeCheckable 时 string[] | options、multiple、treeCheckable、showSearch、allowClear |
| colorPicker | 颜色字符串或空字符串 | disabledAlpha、showText、presets、allowClear；颜色输出 `#RRGGBBAA` |
| datePicker | `YYYY-MM-DD` 或空字符串；showTime 时支持 `YYYY-MM-DD HH:mm:ss` | picker=date/week/month/quarter/year、format、minDate/maxDate、showTime、allowClear |
| calendar | `YYYY-MM-DD` 或空字符串 | 月导航、日期选择、minDate/maxDate |
| timePicker | `HH:mm:ss` 或空字符串 | use12Hours、hourStep/minuteStep/secondStep、format、showNow、allowClear |
| rate | number | count 默认 5，1..100；allowHalf、allowClear、character、tooltips |
| transfer | string[]，右侧键列表 | options、titles、showSearch、oneWay；勾选后移动 |
| upload | object[] | accept 后缀、multiple、directory、maxCount、maxSize（字节）、showUploadList；本地选择、移除、预览意图 |
| tree | selectedKeys 字符串数组 | treeData、multiple、selectable、checkable、checkStrictly、checkedKeys、expandedKeys、defaultExpandedKeys、defaultExpandAll |

format 支持常见 `YYYY/MM/DD/HH/mm/ss` 标记或 chrono `%` 格式；存储格式保持上表约定。日期有效性、数值范围、选项与二级键约束在加载及状态更新时校验，非法更新保留原状态。

## 事件

所有主值修改发出 change，先写状态再执行动作。补充事件如下：

| 组件 | 事件载荷 |
| --- | --- |
| AutoComplete/Mentions | search：当前文字；select：选项键；clear：null；submit：当前文字 |
| Cascader/TreeSelect/DatePicker | openChange：boolean；clear：null |
| ColorPicker/TimePicker | clear：null；openChange 已声明但当前没有完整弹层生命周期事件 |
| Rate | hoverChange：悬停的整星编号 |
| Transfer | search：搜索文字；selectChange：当前勾选键数组 |
| Upload | remove/preview：文件对象；reject：拒绝信息对象 |
| Tree | select：选中键数组；check：写回 checkedKeys；expand：写回 expandedKeys |

Upload 文件记录为 `{uid,path,name,size,status:"selected",percent:0}`。选择文件不会自动改变为 uploading/done。maxSize、maxCount 等拒绝信息通过 reject 处理。

## 当前限制

- Upload 尚无 multipart 传输、进度回调、取消请求、拖放或图片墙；文件对话框为同步系统窗口，accept 暂按后缀过滤。
- AutoComplete 的 backfill/defaultActiveFirstOption、TreeSelect 的 treeDefaultExpandAll、Transfer 的 showSelectAll、Tree 的 showLine 尚未完整实现。TreeSelect 的 treeCheckable 采用多选语义，尚非完整父子复选树。
- 日期控件尚无日期范围、时区、禁用日期回调；showToday 控制今天按钮，Calendar 的 fullscreen 与 DatePicker 的 showNow 尚未完整实现。TimePicker 的步长目前控制拖动速度，手工输入未强制吸附。
- 多层选项支持字符串 key/value，尚不支持任意对象值、异步加载和字段名重映射。树选择与展开的视觉、键盘导航仍需继续完善。

[组件总览](README.md) · [表单](32-form.md) · [覆盖与差异](../21-component-coverage.md)
