# 35 图片、列表与展示组件

本页覆盖 FloatButton、Icon、Avatar、BadgeCount、Descriptions、Empty、Image、List、Listy、QRCode、Timeline。

## 代码演示

展示列表、描述项、状态徽标、二维码与时间轴。Image 未配置 src 时展示占位；使用 `file:///` 绝对路径或 HTTP(S) 地址可加载图片。FloatButton 固定在窗口右下方。

```json
{
  "schemaVersion": "0.2",
  "app": {"name": "原生展示", "window": {"width": 860, "height": 720}},
  "state": {},
  "root": {"type": "column", "id": "root", "layout": {"padding": 16, "gap": 12, "overflow": "scroll"}, "children": [
    {"type": "icon", "id": "icon", "name": "CheckOutlined", "rotate": 15, "layout": {"width": {"fixed": 32}, "height": {"fixed": 32}}},
    {"type": "avatar", "id": "avatar", "text": "Ada", "shape": "circle", "layout": {"width": {"fixed": 48}, "height": {"fixed": 48}}},
    {"type": "badgeCount", "id": "count", "count": 120, "overflowCount": 99, "text": "未读消息"},
    {"type": "descriptions", "id": "descriptions", "column": 2, "bordered": true, "items": [{"label": "姓名", "content": "Ada"}, {"label": "角色", "content": "开发者"}]},
    {"type": "empty", "id": "empty", "description": "尚无任务"},
    {"type": "image", "id": "image", "alt": "配置 src 后显示图片", "preview": true, "layout": {"width": {"fixed": 220}, "height": {"fixed": 120}}},
    {"type": "list", "id": "list", "items": [{"key": "a", "label": "任务 A", "description": "等待处理"}, {"key": "b", "label": "任务 B", "description": "已完成"}], "bordered": true, "layout": {"height": {"fixed": 120}}},
    {"type": "listy", "id": "listy", "items": [{"key": "first", "label": "固定高度虚拟行"}], "itemHeight": 48, "layout": {"height": {"fixed": 80}}},
    {"type": "qrCode", "id": "qr", "value": "https://example.com", "errorLevel": "M", "layout": {"width": {"fixed": 160}, "height": {"fixed": 160}}},
    {"type": "timeline", "id": "timeline", "items": [{"label": "创建", "description": "09:00"}, {"label": "完成", "description": "09:30"}], "pending": "等待下一项"},
    {"type": "floatButton", "id": "float", "icon": "+", "description": "新建", "tooltip": "新建任务", "offsetX": 24, "offsetY": 24}
  ]},
  "actions": {}
}
```

## API 与事件

| type | 属性 | 事件 |
| --- | --- | --- |
| floatButton | text/icon、description、tooltip、shape=circle/square、offsetX/offsetY | click：null |
| icon | name、spin、rotate（角度） | click：null |
| avatar | src、alt、text/icon、shape=circle/square | load/error：来源字符串；目前没有对外开放 preview 属性 |
| badgeCount | count、overflowCount=99、showZero、dot、status | 无；旧 `badge` 对应 Tag |
| descriptions | items 的 label/content、column=3、bordered、colon、direction | 无 |
| empty | description 或 text | 无 |
| image | src、alt、fallback、preview=true、fit=contain/cover/fill | load/error：来源字符串，状态变化时发出；preview：来源字符串 |
| list/listy | items 的 label/description、bordered、split、itemHeight=48、loading | 无；固定行高虚拟滚动 |
| qrCode | value 字符串必填；errorLevel=L/M/Q/H、bordered、status=active/expired/loading/scanned | 无 |
| timeline | items 的 label/description、reverse、pending、mode=left/right/alternate | 无 |

图片加载器支持本地文件、HTTP(S)、PNG/JPEG/GIF/WebP 和 SVG；具体解码能力由已启用的加载器决定。预览使用原生窗口，图片错误可配置 fallback 或 alt。

## 当前限制

- Icon 采用内置文字图形映射，尚未打包完整 Ant Design 图标集；未知名称回退信息符号。
- Image 的 cover 目前与 contain 使用相同的宽高比策略；尚无分组预览、裁切、旋转、下载按钮或浏览器 ref。
- Descriptions 的 bordered 为行底纹表现，尚无合并单元格；Timeline 为基础节点与连线，尚无自定义 dot/render 回调。
- List/Listy 采用相同的固定高度虚拟列表；itemHeight 应覆盖标题和说明，尚无可变行高、loadMore 或任意子树渲染。
- FloatButton 尚无按钮组或自动 BackTop；BadgeCount 为独立徽标，尚未支持任意子树上的角标偏移。QRCode 由本机编码生成，容量错误直接显示诊断。

[组件总览](README.md) · [状态标签 Tag](14-badge.md)
