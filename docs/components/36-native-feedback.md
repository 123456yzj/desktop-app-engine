# 36 弹窗、提示与反馈

本页覆盖 Modal、Drawer、Popover、Tooltip、Popconfirm、Message、Notification、Tour、Alert、Result、Skeleton、Spin。

## 代码演示

点击按钮打开模态框、抽屉、通知或导览；关闭操作通过 openChange 写回 state。Modal 的确定只产生 ok，应用动作决定保存与关闭。通知两秒后自动关闭。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "原生反馈", "window": {"width": 840, "height": 720}},
  "state": {"modal": false, "drawer": false, "message": false, "notification": false, "tour": false, "step": 0, "name": "Ada"},
  "root": {"type": "column", "id": "root", "layout": {"padding": 16, "gap": 12, "overflow": "scroll"}, "children": [
    {"type": "space", "id": "buttons", "children": [
      {"type": "button", "id": "showModal", "text": {"kind": "literal", "value": "打开对话框"}, "events": {"click": "showModal"}},
      {"type": "button", "id": "showDrawer", "text": {"kind": "literal", "value": "打开抽屉"}, "events": {"click": "showDrawer"}},
      {"type": "button", "id": "showMessages", "text": {"kind": "literal", "value": "显示通知"}, "events": {"click": "showMessages"}},
      {"type": "button", "id": "showTour", "text": {"kind": "literal", "value": "开始导览"}, "events": {"click": "showTour"}}
    ]},
    {"type": "modal", "id": "modal", "open": {"kind": "state", "key": "modal"}, "title": "编辑资料", "events": {"ok": "save"}, "children": [{"type": "input", "id": "name", "value": {"kind": "state", "key": "name"}}]},
    {"type": "drawer", "id": "drawer", "open": {"kind": "state", "key": "drawer"}, "title": "帮助", "placement": "right", "width": 360, "children": [{"type": "text", "id": "help", "text": {"kind": "literal", "value": "抽屉中的内容"}}]},
    {"type": "message", "id": "message", "open": {"kind": "state", "key": "message"}, "text": "保存成功", "status": "success", "duration": 2},
    {"type": "notification", "id": "notification", "open": {"kind": "state", "key": "notification"}, "title": "通知", "description": "资料已经更新", "duration": 2},
    {"type": "popover", "id": "popover", "text": "点击查看详情", "title": "说明", "description": "原生气泡内容"},
    {"type": "tooltip", "id": "tooltip", "text": "悬停查看提示", "description": "操作帮助"},
    {"type": "popconfirm", "id": "confirm", "text": "确认操作", "description": "确定继续吗？", "okText": "继续", "cancelText": "取消"},
    {"type": "tour", "id": "tour", "open": {"kind": "state", "key": "tour"}, "current": {"kind": "state", "key": "step"}, "steps": [{"key": "intro", "label": "欢迎", "description": "查看功能介绍"}, {"key": "finish", "label": "完成", "description": "现在开始使用"}]},
    {"type": "alert", "id": "alert", "text": "数据已更新", "description": "可以继续编辑", "status": "success", "closable": true, "showIcon": true},
    {"type": "result", "id": "result", "title": "操作完成", "subTitle": "所有任务已处理", "status": "success"},
    {"type": "skeleton", "id": "skeleton", "rows": 2, "avatar": true, "active": true},
    {"type": "spin", "id": "spin", "spinning": false, "tip": "加载中", "children": [{"type": "text", "id": "loaded", "text": {"kind": "literal", "value": "已加载内容"}}]}
  ]},
  "actions": {
    "showModal": {"steps": [{"type": "set", "target": "modal", "value": {"kind": "literal", "value": true}}]},
    "showDrawer": {"steps": [{"type": "set", "target": "drawer", "value": {"kind": "literal", "value": true}}]},
    "showMessages": {"steps": [{"type": "set", "target": "message", "value": {"kind": "literal", "value": true}}, {"type": "set", "target": "notification", "value": {"kind": "literal", "value": true}}]},
    "showTour": {"steps": [{"type": "set", "target": "step", "value": {"kind": "literal", "value": 0}}, {"type": "set", "target": "tour", "value": {"kind": "literal", "value": true}}]},
    "save": {"steps": [{"type": "set", "target": "modal", "value": {"kind": "literal", "value": false}}, {"type": "set", "target": "message", "value": {"kind": "literal", "value": true}}]}
  }
}
```

## API 与事件

| type | 常用属性 | 事件 |
| --- | --- | --- |
| modal/drawer | open 默认 false；title/text/children、width/height、closable、maskClosable、keyboard、footer、okText/cancelText、confirmLoading；drawer placement | openChange：boolean，先写 open；ok/cancel/close：null |
| popover/tooltip | open、text/title/description、trigger=click/hover/contextMenu | openChange：boolean |
| popconfirm | 上述气泡属性；okText/cancelText、showCancel | openChange：boolean；confirm/cancel：null |
| message/notification | open 默认 true；text/title/description、status、duration 秒，0 为常驻、closable、placement | openChange：boolean；close：null |
| tour | open 默认 false；steps 的 label/description、current 从 0 开始、nextText/prevText/finishText | change：索引；openChange：boolean；close/finish：null |
| alert | text/description、status、closable、showIcon、banner | close：null，随后本地隐藏 |
| result | title/text/subTitle、status=success/error/info/warning/403/404/500 | 无 |
| skeleton | rows、avatar、round、active | 无 |
| spin | children、spinning=true、tip、delay 毫秒、percent | 无；spinning 时子控件禁用 |

Modal/Drawer 阻挡背景输入；Escape 关闭最上层模态框，有内部弹层时优先处理弹层。取消、遮罩和关闭按钮会按 openChange、cancel、close 顺序提交。Modal 默认 maskClosable=false，Drawer 默认 true。footer 默认为 Modal 显示、Drawer 隐藏。

## 当前限制

- 弹窗为声明式组件，由 state/actions 控制；没有 React hooks、context holder、静态 Promise API、DOM portal/ref。可把公共提示放在 App/ConfigProvider 子树中。
- Message/Notification 尚无队列堆叠、去重 key、maxCount 与更新已有通知 API；同位置多个提示可能重叠。重新显示时，应让受控 open 先变 false 再变 true。
- Tour 为步骤窗口，尚无目标组件高亮、挖空遮罩、跟随定位。气泡当前以文字内容为主，任意子树作为触发器的布局和点击传递尚需完善。
- Modal/Drawer 尚无打开关闭动画、destroyOnHidden、异步确认完成协议；业务动作应在保存成功时更新 open。

[组件总览](README.md) · [覆盖与差异](../21-component-coverage.md)
