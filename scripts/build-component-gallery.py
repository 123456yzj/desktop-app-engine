"""Build the browsable gallery from the working examples and curated variants.

Run from any directory: python scripts/build-component-gallery.py
The six-section all-components.json remains the source of working demo fixtures.
"""
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "examples/all-components.json"
OUTPUT = ROOT / "examples/component-gallery.json"

# One entry per protocol type, including the engine's layout and extension types.
GROUPS = [
    ("通用", """
button|Button 按钮|触发操作或提交数据，区分主要操作、辅助操作与危险操作。
floatButton|FloatButton 悬浮按钮|将常用操作固定在窗口角落，方便随时访问。
icon|Icon 图标|用内置符号表达操作含义和状态。
text|Typography 排版|展示标题、正文及可编辑文字，建立清晰的信息层级。
"""),
    ("布局", """
divider|Divider 分割线|分隔内容区域，帮助用户辨识信息结构。
flex|Flex 弹性布局|沿水平或垂直方向排列内容，并按空间调整排列。
grid|Grid 栅格|使用固定宽度或比例列组织内容。
layout|Layout 布局|组合顶部、侧栏、内容和底部区域，构建应用骨架。
header|Header 顶部区域|承载品牌、标题和全局操作。
sider|Sider 侧边区域|承载目录、导航或辅助信息。
content|Content 内容区域|展示页面主要信息与操作。
footer|Footer 底部区域|承载状态、说明和底部操作。
masonry|Masonry 瀑布流|保留卡片自然高度，把内容放入当前最短的一列。
space|Space 间距|为一组控件提供一致的间距与分隔。
splitter|Splitter 面板分割|拖动分割条调整两个面板的空间比例。
row|Row 横向排列|将子组件从左到右排列，适用于工具栏和操作组。
column|Column 纵向排列|将子组件从上到下排列，适用于内容和表单区域。
"""),
    ("导航", """
anchor|Anchor 锚点|列出页面章节入口，点击后显示所选章节信息。
breadcrumb|Breadcrumb 面包屑|展示内容所在层级，帮助理解当前位置。
dropdown|Dropdown 下拉菜单|把不常用操作收纳到下拉菜单中。
menu|Menu 导航菜单|通过分组、子菜单和选中态组织导航入口。
pagination|Pagination 分页|切换数据页码，也可调整每页条数。
steps|Steps 步骤条|展示任务流程，并切换当前步骤。
tabs|Tabs 标签页|在相关内容之间切换，每个标签对应独立内容。
"""),
    ("数据录入", """
autoComplete|AutoComplete 自动完成|输入文字后从建议列表中补全内容。
cascader|Cascader 级联选择|从层级数据中逐级选择目标路径。
checkbox|Checkbox 多选框|切换单个布尔值，或从选项组中选择多个值。
colorPicker|ColorPicker 颜色选择器|选取颜色，或从预设色板中快速选择。
datePicker|DatePicker 日期选择器|选择日期，并限制可选日期范围。
form|Form 表单|把字段校验、提交和重置组合成完整的数据录入流程。
formItem|FormItem 表单项|为输入字段提供标签、必填标识与校验反馈。
input|Input 输入框|录入文本，支持清除、密码和多行等形式。
inputNumber|InputNumber 数字输入框|在范围内录入数字，并控制步长和精度。
mentions|Mentions 提及|通过前缀触发候选项，补全人员或其他引用。
radio|Radio 单选框|从一组互斥选项中选择一个值。
rate|Rate 评分|通过星级表达评分，支持半星与只读展示。
select|Select 选择器|从选项中选择内容，支持搜索、多选和自定义标签。
slider|Slider 滑动输入条|在连续范围内选择数值，或选择一个区间。
switch|Switch 开关|立即切换功能的开启与关闭状态。
timePicker|TimePicker 时间选择器|录入时、分、秒，支持不同的显示格式。
transfer|Transfer 穿梭框|把候选项移动到已选列表中。
treeSelect|TreeSelect 树选择|从树形组织中选择一个或多个节点。
upload|Upload 文件选择|选择本地文件并管理记录，适用于后续文件处理流程。
"""),
    ("数据展示", """
avatar|Avatar 头像|用文字或图标表示用户与对象。
badgeCount|Badge 徽标数|展示未读数量、上限计数或状态圆点。
badge|Tag 标签|标记分类与状态，也可切换选中或关闭标签。
calendar|Calendar 日历|浏览月份并在日历中选择日期。
card|Card 卡片|把相关内容放在带标题和边框的容器内。
panel|Panel 面板|组织有标题的内容块，适用于配置和数据区域。
carousel|Carousel 走马灯|在有限区域内切换多张内容。
collapse|Collapse 折叠面板|折叠次要信息，按需展开详细内容。
descriptions|Descriptions 描述列表|以标签和值的形式展示结构化详情。
empty|Empty 空状态|在没有数据时说明当前情况。
image|Image 图片|展示图片或来源缺失时的占位信息。
list|List 列表|用一致的行结构展示条目和说明。
listy|Listy 虚拟列表|在固定高度区域内滚动浏览大量条目。
popover|Popover 气泡卡片|点击触发器，在临时气泡中查看补充信息。
qrCode|QRCode 二维码|在本机生成可扫描的文本或链接二维码。
segmented|Segmented 分段控制器|在一组相关视图或选项间快速切换。
metric|Statistic 统计数值|突出显示关键指标，并设置数字格式与前后缀。
table|Table 表格|展示结构化数据，支持排序、选行与分页。
timeline|Timeline 时间轴|沿时间顺序展示事件与进展。
tooltip|Tooltip 文字提示|在悬停时解释图标或补充简短说明。
tour|Tour 漫游导览|使用分步说明窗口引导用户了解操作流程。
tree|Tree 树形控件|展示层级关系，并提供展开、选择与勾选操作。
"""),
    ("反馈", """
alert|Alert 警告提示|在内容区域内说明状态、提醒事项或错误。
drawer|Drawer 抽屉|从窗口边缘打开辅助内容，保留当前页面上下文。
message|Message 全局提示|用短暂消息反馈操作结果。
modal|Modal 对话框|在独立对话框中确认操作或编辑内容。
notification|Notification 通知提醒|展示需要更多说明的通知信息。
popconfirm|Popconfirm 气泡确认框|在执行操作前就近询问是否继续。
progress|Progress 进度|使用进度条、圆环或仪表盘展示完成比例。
result|Result 结果|集中展示任务结果，并说明下一步。
skeleton|Skeleton 骨架屏|在加载内容前预留大致的结构。
spin|Spin 加载中|用加载指示覆盖内容，并暂时禁用区域内操作。
"""),
    ("配置与扩展", """
watermark|Watermark 水印|在内容区域叠加重复的标识文字。
affix|Affix 固钉|在父区域的可见范围内约束操作的位置。
app|App 应用配置|为子内容提供统一的配置作用域。
borderBeam|BorderBeam 边框流光|为重点内容添加沿边框移动的视觉装饰。
configProvider|ConfigProvider 全局配置|统一设置子组件尺寸、主题、方向和禁用状态。
module|Module 模块|封装局部状态，并通过属性和事件与父级通信。
lineChart|LineChart 折线图|展示连续数据的变化趋势，支持图例、缩放与平移。
barChart|BarChart 柱状图|对比不同数据项的数量与分布。
plugin|Plugin 插件|通过 ECharts 扩展展示图表，并接收图表点击事件。
"""),
]


def literal(value):
    return {"kind": "literal", "value": value}


def binding(key):
    return {"kind": "state", "key": key}


def node(kind, id, children=None, **props):
    result = {"type": kind, "id": id, **props}
    if children is not None:
        result["children"] = children
    return result


def text(id, value, css="text-sm text-slate-600"):
    if any(token.startswith("p-") for token in css.split()):
        typography = " ".join(token for token in css.split() if token.startswith(("text-", "font-")))
        return node("column", id, [node("text", id + "Label", text=literal(value), **{"class": typography})], **{"class": css})
    return node("text", id, text=literal(value), **{"class": css})


def button(id, label, action="clicked", primary=False):
    return node("button", id, text=literal(label), events={"click": action}, **{
        "class": "h-10 px-4 py-2 text-sm rounded-lg " + (
            "bg-blue-500 text-white border-0 hover:bg-blue-600 active:bg-blue-700" if primary else
            "bg-white text-slate-700 border border-slate-300 hover:border-blue-500 hover:text-blue-600")})


def walk(value):
    if isinstance(value, dict):
        yield value
        for child in value.get("children", []):
            yield from walk(child)


def main():
    base = json.loads(SOURCE.read_text(encoding="utf-8"))
    state = copy.deepcopy(base["state"])
    actions = copy.deepcopy(base["actions"])
    fixtures = {n["id"]: n for n in walk(base["root"])}
    by_type = {}
    for item in walk(base["root"]):
        by_type.setdefault(item["type"], item)
    catalogue = [(group, *line.split("|")) for group, entries in GROUPS
                 for line in entries.strip().splitlines()]
    assert len(catalogue) == 84 and len({entry[1] for entry in catalogue}) == 84
    assert {entry[1] for entry in catalogue} == set(by_type)

    def clone(source, prefix):
        """Give demo trees unique IDs; preserve matching Tabs/Collapse keys."""
        ids = {n["id"]: prefix + "_" + n["id"] for n in walk(source)}

        def rewrite(value):
            if isinstance(value, dict):
                if value.get("kind") == "state":
                    result = copy.deepcopy(value)
                    key = result["key"]
                    current = state.get(key)
                    if isinstance(current, list) and any(isinstance(v, str) and v in ids for v in current):
                        state[prefix + "_" + key] = [ids.get(v, v) for v in current]
                        result["key"] = prefix + "_" + key
                    return result
                return {key: (ids.get(item, item) if key in ("id", "key") and isinstance(item, str)
                              else copy.deepcopy(item) if key in ("events", "on", "actions") else rewrite(item))
                        for key, item in value.items()}
            if isinstance(value, list):
                return [rewrite(item) for item in value]
            return value

        result = rewrite(copy.deepcopy(source))
        result.pop("visible", None)
        return result

    def boxes(prefix, count=3):
        return [text(prefix + str(i), "区域 " + str(i + 1),
                     "p-4 h-16 bg-blue-50 text-blue-700 border border-blue-100 rounded-lg")
                for i in range(count)]

    def variant(kind, prefix, changes=None, fixture=None):
        result = clone(fixtures[fixture] if fixture else by_type[kind], prefix)
        result.update(changes or {})
        return result

    def demo_value(prefix, value):
        state[prefix] = value
        return binding(prefix)

    demos = {}

    def add(kind, title, description, children):
        demos.setdefault(kind, []).append((title, description, children))

    def pair(kind, title, description, changes, second_title, second_description, second):
        add(kind, title, description, [variant(kind, kind + "_a", changes)])
        add(kind, second_title, second_description, [variant(kind, kind + "_b", second)])

    add("button", "按钮类型", "主要操作、默认操作与文字操作。点击后可在底部查看操作反馈。", [
        node("space", "buttonTypes", [button("primary", "主要按钮", primary=True),
             button("default", "默认按钮"), node("button", "linkButton", text=literal("文字按钮"),
              variant="link", events={"click": "clicked"}, **{"class": "h-10 px-3 text-blue-600 hover:text-blue-500 active:text-blue-700"})],
             **{"class": "gap-3 flex-wrap"})])
    add("button", "禁用与加载", "禁用按钮无法触发操作；加载按钮用于等待处理结果。", [
        node("space", "buttonStates", [variant("button", "disabled", {"enabled": literal(False)}, "disabledButton"),
             variant("button", "loading", {"loading": True, "text": literal("处理中")}, "basicButton")],
             **{"class": "gap-3"})])
    add("button", "危险操作", "使用危险色提醒删除等需要谨慎执行的操作。", [
        node("button", "dangerButton", text=literal("删除记录"), danger=True,
             events={"click": "clicked"}, **{"class": "w-32 h-10 px-4 py-2 bg-white border border-red-300 rounded-lg"})])

    for kind in ["row", "column", "flex", "space", "grid", "masonry"]:
        first = node(kind, kind + "Basic", boxes(kind + "Box"), **{"class": "gap-3"})
        second = node(kind, kind + "Variant", boxes(kind + "Alt", 4), **{"class": "gap-5"})
        titles = ("基础排列", "较大间距")
        if kind == "grid":
            first["columns"] = [{"fr": 1}, {"fr": 1}, {"fr": 1}]
            second["columns"] = [{"fixed": 180}, {"fr": 1}]
            titles = ("等比例三列", "固定列与弹性列")
        elif kind == "masonry":
            first["columnCount"] = 3
            second["columnCount"] = 2
            first["children"][1]["class"] = "p-4 h-28 bg-slate-100 text-slate-700 rounded-lg"
            second["children"][2]["class"] = "p-4 h-32 bg-blue-50 text-blue-700 rounded-lg"
            titles = ("三列瀑布流", "双列混合高度")
        elif kind == "flex":
            first["vertical"] = False
            second["vertical"] = True
            titles = ("水平排列", "垂直排列")
        elif kind == "space":
            second["separator"] = "/"
            titles = ("统一间距", "带分隔符")
        add(kind, titles[0], "按当前方向和列规则排列内容。", [first])
        add(kind, titles[1], "调整排列参数，观察相同内容的不同组织方式。", [second])

    for kind in ["layout", "header", "sider", "content", "footer"]:
        for index in range(2):
            shell = node("layout", f"{kind}Shell{index}", [
                node("header", f"{kind}Header{index}", [text(f"{kind}HeadText{index}", "Header · 应用标题")],
                     **{"class": "h-12 p-3 bg-blue-50 text-blue-700 rounded"}),
                node("layout", f"{kind}Body{index}", [
                    node("sider", f"{kind}Side{index}", [text(f"{kind}SideText{index}", "Sider\n组件目录")],
                         **{"class": ("w-[150px]" if index == 0 else "w-[210px]") + " p-3 bg-slate-100 rounded"}),
                    node("content", f"{kind}Content{index}", [text(f"{kind}MainText{index}", "Content · 主要内容")],
                         **{"class": "p-4 bg-white border border-slate-200 rounded"}, layout={"width": {"fr": 1}})
                ], **{"class": "gap-3"}, layout={"height": {"fr": 1}}),
                node("footer", f"{kind}Footer{index}", [text(f"{kind}FootText{index}", "Footer · 状态与说明")],
                     **{"class": "h-10 p-2 bg-slate-100 rounded"})
            ], **{"class": "h-[250px] gap-3"})
            add(kind, "标准应用布局" if index == 0 else "加宽侧栏", "在完整布局中观察 " + kind + " 区域与其他区域的关系。", [shell])

    pair("divider", "水平分割", "分隔上下两段内容。", {}, "带标题的分割线", "为内容分组提供名称。", {"text": literal("更多内容")})
    pair("splitter", "拖动分栏", "拖动中间分割条；比例会保留。", {"class": "h-[180px]"},
         "不同初始比例", "左右面板从 60% / 40% 开始，可继续拖动。", {"value": demo_value("splitSecond", 0.6), "class": "h-[180px]"})
    for child in walk(demos["splitter"][1][2][0]):
        if child["type"] == "metric":
            child["value"] = binding("splitSecond")
    pair("text", "标题与正文", "利用字号和字重建立阅读层级。", {"text": literal("让信息更清晰"), "class": "text-2xl font-bold text-slate-900"},
         "可编辑文字", "点击编辑，直接修改示例文字。", {"text": demo_value("editableText", "点击编辑这段文字"), "editable": True, "class": "text-base text-slate-700"})
    pair("icon", "状态图标", "勾选图标表达完成状态。", {"name": "CheckOutlined", "class": "w-10 h-10 text-green-600"},
         "旋转图标", "通过角度调整图标方向。", {"name": "SettingOutlined", "rotate": 45, "class": "w-10 h-10 text-blue-600"})

    # Inputs have independent values when the variant changes their value shape.
    pair("input", "可清除输入", "输入姓名，悬停后使用清除操作。", {"value": demo_value("inputBasic", "Ada"), "class": "h-10 px-3 py-2 bg-white border border-slate-300 rounded-lg hover:border-blue-500", "allowClear": True},
         "密码输入", "密码以掩码显示。", {"value": demo_value("inputPassword", "desktop123"), "password": True, "allowClear": False, "class": "h-10 px-3 py-2 bg-white border border-slate-300 rounded-lg"})
    add("input", "多行与字数", "填写简短说明，最多 120 个字符。", [variant("input", "inputMultiline", {
        "value": demo_value("inputMultiline", "这是一段多行说明。"), "multiline": True,
        "rows": literal(3), "maxLength": 120, "showCount": True, "class": "h-32 px-3 py-2 bg-white border border-slate-300 rounded-lg"})])
    pair("inputNumber", "整数与范围", "使用步进按钮或输入数字，范围为 0–120。", {}, "金额与精度", "以 0.5 为步长，保留两位小数。", {
        "value": demo_value("amount", 128.5), "min": 0, "max": 10000, "step": 0.5, "precision": 2, "prefix": "¥ ", "class": "h-10 bg-white border border-slate-300 rounded-lg"})
    pair("checkbox", "单个复选框", "表示是否同意或是否启用。", {}, "多选组", "多个选项可以同时选中。", {
        "text": literal("选择权限"), "value": demo_value("checkboxGroup", ["read"]), "options": literal(["read", "write", "admin"])})
    pair("radio", "基础单选", "一次只能选择一个选项。", {}, "按钮式单选", "同一组选项使用按钮外观。", {"optionType": "button", "value": demo_value("radioSecond", "server")})
    pair("switch", "基础开关", "点击切换功能状态。", {"text": literal("接收通知")}, "文字与加载", "显示开关含义，加载时暂停操作。", {
        "value": demo_value("switchSecond", True), "checkedChildren": "开", "unCheckedChildren": "关", "loading": True, "text": literal("正在保存")})
    pair("slider", "单值滑块", "拖动或使用方向键，步长为 5。", {}, "范围滑块", "分别拖动两个滑块选择区间。", {"range": True, "value": demo_value("sliderRange", [20, 70])})
    pair("select", "搜索与单选", "点击展开；输入关键字搜索，也可使用上下键和 Enter 选择。", {"value": demo_value("selectBasic", "rust")},
         "多选与数量上限", "可以选择两项，选择后保持弹层打开。", {"mode": "multiple", "value": demo_value("selectMultiple", ["rust"]), "maxCount": 2})
    add("select", "自定义标签", "输入新内容并按 Enter 创建标签。", [variant("select", "selectTags", {
        "mode": "tags", "value": demo_value("selectTags", []), "showSearch": True, "placeholder": literal("输入或选择标签")})])
    add("select", "禁用状态", "禁用选择器保留已选内容，无法展开和修改。", [variant("select", "selectDisabled", {"enabled": literal(False)})])
    pair("autoComplete", "姓名补全", "输入 A 等文字过滤候选姓名。", {}, "不同建议内容", "在建议中选择语言，也可以自由输入。", {
        "value": demo_value("completeLanguage", ""), "options": literal(["Rust", "Python", "TypeScript"]), "placeholder": literal("输入语言")})
    pair("cascader", "逐级选择", "从亚洲下选择国家。", {}, "多路径选择", "选择多条层级路径。", {"multiple": True, "value": demo_value("cascaderMultiple", [])})
    pair("treeSelect", "单个节点", "从团队树中选择一个成员。", {}, "多选节点", "允许选择多个树节点。", {"multiple": True, "value": demo_value("treeMultiple", [])})
    pair("colorPicker", "预设颜色", "点击色块选择颜色，也可以使用预设色板。", {}, "绿色起点", "使用另一种初始颜色并隐藏颜色文字。", {"value": demo_value("colorSecond", "#52c41a"), "showText": False})
    pair("datePicker", "日期范围", "仅允许选择 2026 年内的日期。", {}, "日期格式", "显示年月日，状态仍保存标准日期。", {"value": demo_value("dateSecond", "2026-06-18"), "format": "YYYY/MM/DD"})
    pair("timePicker", "24 小时时间", "编辑时、分、秒，或清除当前值。", {}, "12 小时显示", "使用 12 小时制查看与录入时间。", {"value": demo_value("timeSecond", "18:30:00"), "use12Hours": True})
    pair("mentions", "人员提及", "输入 @ 后选择成员。", {}, "主题提及", "使用 # 引用主题。", {"prefix": "#", "value": demo_value("mentionsSecond", "讨论 #"), "options": literal(["布局", "表单", "图表"])})
    pair("rate", "半星评分", "点击不同位置选择半星或整星。", {}, "只读评分", "展示评分，禁用修改。", {"value": literal(4), "enabled": literal(False)})
    pair("transfer", "双向穿梭", "勾选条目，再点击箭头移动。", {"class": "h-[240px]"}, "单向选择", "只保留向已选列表添加的方向。", {"oneWay": True, "value": demo_value("transferSecond", []), "class": "h-[240px]"})
    pair("upload", "选择多个文件", "选择本地文件后管理记录；最多五项，每项不超过 10 MB。", {}, "单个 JSON 文件", "仅选择一个 JSON 文件，供本地处理使用。", {
        "value": demo_value("uploadSecond", []), "multiple": False, "maxCount": 1, "accept": ".json", "text": literal("选择 JSON 文件")})
    for kind in ["form", "formItem"]:
        add(kind, "资料校验与提交", "姓名必填且为 2–30 个字符，年龄须在 18–100 岁之间。提交结果显示在底部。", [variant("form", kind + "Full", fixture="profileForm")])
        compact = clone(fixtures["profileForm"], kind + "Compact")
        compact["children"] = [text(kind + "CompactIntro", "必填字段"), clone(fixtures["nameItem"], kind + "Required"),
                               node("space", kind + "CompactButtons", [button(kind + "Validate", "校验姓名", primary=True)], **{"class": "gap-3"})]
        compact["children"][-1]["children"][0]["htmlType"] = "submit"
        compact["children"][-1]["children"][0].pop("events")
        add(kind, "必填字段", "清空姓名后提交，查看字段下方的错误提示。", [compact])

    pair("anchor", "水平目录", "点击后，底部显示章节标识；当前不自动滚动。", {}, "垂直目录", "适合侧边章节索引。", {"direction": "vertical", "class": "h-24"})
    pair("breadcrumb", "基础层级", "用斜线分隔当前层级。", {}, "自定义分隔符", "使用 > 表达层级关系。", {"separator": ">"})
    pair("dropdown", "点击菜单", "点击触发器后选择操作，结果显示在底部。", {}, "悬停触发", "把指针移动到触发器上展开。", {"trigger": "hover", "text": literal("悬停展开操作")})
    menu_items = [{"key": "overview", "label": "概览"}, {"key": "settings", "label": "设置"}, {"key": "locked", "label": "禁用入口", "disabled": True}]
    for index, mode in enumerate(["horizontal", "inline"]):
        add("menu", "水平菜单" if index == 0 else "分组菜单", "点击入口切换选中项；禁用入口不会响应。", [node("menu", "demoMenu" + str(index),
            items=menu_items if index == 0 else [{"type": "group", "key": "workspace", "label": "工作区", "children": menu_items}],
            mode=mode, selectedKeys=demo_value("menu" + str(index), ["overview"]), **{"class": "h-12" if index == 0 else "h-40"})])
    pair("pagination", "完整分页", "切换页码、跳页或修改每页条数。", {}, "简洁分页", "用紧凑形式展示当前页。", {"current": demo_value("simplePage", 1), "simple": True, "showSizeChanger": False, "showQuickJumper": False})
    pair("steps", "水平流程", "点击步骤切换当前位置。", {}, "垂直流程", "适合侧边的任务进展。", {"direction": "vertical", "current": demo_value("verticalStep", 1), "class": "h-32"})
    for index in range(2):
        prefix = "exampleTabs" + str(index)
        children = [text(prefix + "Overview", "这里是概览内容。", "h-20 p-4 bg-blue-50 text-blue-700 rounded-lg"),
                    text(prefix + "History", "这里是历史记录。", "h-20 p-4 bg-slate-50 text-slate-600 rounded-lg")]
        add("tabs", "顶部标签" if index == 0 else "侧边标签", "切换标签，观察相应内容。", [node("tabs", prefix, children,
            items=[{"key": children[0]["id"], "label": "概览"}, {"key": children[1]["id"], "label": "历史"}],
            activeKey=demo_value(prefix + "Key", children[0]["id"]), tabPlacement="top" if index == 0 else "start", **{"class": "h-40"})])

    pair("avatar", "圆形头像", "使用名字的文字头像。", {}, "方形头像", "方形头像适合项目或团队标识。", {"shape": "square", "text": literal("Dev")})
    pair("badgeCount", "计数上限", "超过 99 的数量显示为 99+。", {"count": 120, "overflowCount": 99, "status": "error"}, "状态圆点", "使用圆点表示新消息。", {"dot": True, "count": 1, "status": "processing", "text": literal("新消息")})
    pair("badge", "分类标签", "通过颜色表达不同分类。", {"text": literal("进行中"), "checkable": False, "closable": False, "class": "w-[100px] h-8 px-3 py-1 bg-blue-50 text-blue-600 rounded"},
         "可选与可关闭", "点击切换选中态，或关闭标签。", {"text": literal("筛选条件"), "checkable": True, "closable": True, "checked": demo_value("galleryTagChecked", True), "class": "w-[120px] h-8 px-3 py-1 rounded"})
    pair("calendar", "浏览与选择", "切换月份，点击日期。", {}, "限定日期", "仅在限定时间内选择日期。", {"value": demo_value("calendarSecond", "2026-06-18"), "minDate": "2026-06-01", "maxDate": "2026-06-30"})
    for kind in ["card", "panel"]:
        for index in range(2):
            add(kind, "标准容器" if index == 0 else "加载占位", "展示有标题的内容。" if index == 0 else "数据准备期间隐藏内容，显示加载占位。", [node(kind, kind + str(index),
                [text(kind + "Inner" + str(index), "项目概览\n本周已有 24 项任务完成。")], title=literal("项目摘要"), loading=bool(index),
                **{"class": "p-4 h-36 bg-white border border-slate-200 rounded-lg"})])
    pair("carousel", "箭头与指示点", "点击箭头或指示点切换幻灯片。", {}, "只显示指示点", "从底部指示点切换内容。", {"current": demo_value("carouselSecond", 0), "arrows": False})
    pair("collapse", "可折叠内容", "点击标题展开或收起说明。", {}, "默认收起", "点击标题查看按需展开的内容。", {"activeKey": demo_value("collapseSecond", [])})
    pair("descriptions", "双列详情", "用双列排列标签和值。", {}, "单列详情", "纵向阅读结构化信息。", {"column": 1, "bordered": False, "class": "h-48"})
    pair("empty", "暂无任务", "没有数据时提供清晰的说明。", {"description": "暂无任务"}, "搜索无结果", "说明为什么当前列表为空。", {"description": "未找到匹配结果，请尝试其他关键词。"})
    pair("image", "缺少来源", "未设置来源时显示说明占位。", {"class": "w-[300px] h-[180px]"}, "错误回退", "无效本地来源展示回退说明。", {"src": "file:///component-gallery-missing-image.png", "alt": "图片暂不可用", "class": "w-[300px] h-[180px]"})
    pair("list", "条目列表", "每行包含标题和说明。", {}, "加载列表", "尚未取得数据时显示加载状态。", {"loading": True})
    pair("listy", "可滚动列表", "把指针放在列表区域内滚动。", {"items": [{"key": "item" + str(i), "label": "任务 " + str(i + 1), "description": "固定行高的虚拟条目"} for i in range(100)], "class": "h-48"},
         "紧凑行高", "使用较小行高显示更多项目。", {"itemHeight": 32, "class": "h-48", "items": [{"key": "compact" + str(i), "label": "条目 " + str(i + 1)} for i in range(30)]})
    pair("qrCode", "文本二维码", "本机生成二维码，无需网络请求。", {}, "过期状态", "在二维码上展示已过期提示。", {"status": "expired"})
    pair("segmented", "水平切换", "在日、周、月视图间切换。", {}, "垂直切换", "选项按纵向排列。", {"vertical": True, "value": demo_value("segmentVertical", "week"), "class": "h-32"})
    pair("metric", "关键指标", "突出展示组件总数。", {}, "百分比格式", "格式化数值并标记增长方向。", {"label": literal("本月增长"), "value": literal(0.128), "format": {"type": "percent", "precision": 1}, "prefix": "↑ "})
    pair("table", "排序与分页", "点击列标题排序，点击行查看底部反馈。", {}, "空数据", "当没有数据时展示空状态说明。", {"rows": literal([]), "emptyText": "暂无节点数据"})
    pair("timeline", "时间顺序", "从创建到完成展示过程。", {}, "倒序记录", "最新进展展示在前面。", {"reverse": True, "pending": "等待下一次更新"})
    pair("tree", "勾选节点", "展开团队，选择或勾选成员。", {}, "仅选择", "保留展开和选择，隐藏复选框。", {"checkable": False, "selectedKeys": demo_value("treeSecondSelected", [])})
    pair("popover", "点击气泡", "点击查看补充说明，点击外部关闭。", {}, "悬停气泡", "悬停时显示补充说明。", {"trigger": "hover", "text": literal("悬停查看信息")})
    pair("tooltip", "操作提示", "悬停在触发器上查看提示。", {}, "另一条说明", "用简短文字解释当前操作。", {"text": literal("悬停查看保存说明"), "description": "保存当前修改并继续编辑。"})
    pair("popconfirm", "确认操作", "选择确定执行，或取消关闭。", {}, "删除确认", "在执行危险操作前再次确认。", {"text": literal("删除记录"), "description": "确定删除这条演示记录吗？", "okText": "删除"})
    pair("alert", "成功反馈", "可以关闭的成功提示。", {}, "警告信息", "提示需要留意的事项。", {"status": "warning", "text": literal("还有未保存的修改"), "description": "离开前请先保存当前内容。"})
    pair("result", "成功结果", "完成任务后的集中反馈。", {}, "没有找到页面", "页面不存在时给出明确反馈。", {"status": "404", "title": literal("没有找到内容"), "subTitle": "请返回目录选择其他组件。"})
    pair("skeleton", "头像与正文", "预留头像和三行正文空间。", {}, "简洁正文", "只展示两行圆角占位。", {"rows": 2, "avatar": False, "round": True})
    pair("progress", "线性进度", "用水平进度条展示完成比例。", {}, "圆环进度", "适用于紧凑的完成率展示。", {"progressType": "circle", "percent": 72, "class": "w-32 h-32"})
    add("progress", "仪表盘", "使用开口圆环强调当前完成比例。", [variant("progress", "progressDashboard", {"progressType": "dashboard", "percent": 45, "class": "w-32 h-32"})])
    add("spin", "切换加载", "打开开关后覆盖加载指示，并禁用区域内按钮。", [clone(fixtures["spinSwitch"], "spinToggle"), variant("spin", "spinToggleContent")])
    add("spin", "已完成内容", "加载结束时恢复内容与交互。", [variant("spin", "spinLoaded", {"spinning": False})])

    # Overlay nodes stay in their own page, so only the active page can open them.
    for kind in ["modal", "drawer", "message", "notification", "tour"]:
        for index in range(2):
            prefix = kind + "Example" + str(index)
            open_key = prefix + "Open"
            state[open_key] = False
            actions[prefix + "Show"] = {"steps": [{"type": "set", "target": open_key, "value": literal(True)}]}
            changes = {"open": binding(open_key)}
            if kind == "modal":
                actions[prefix + "Save"] = {"steps": [{"type": "set", "target": open_key, "value": literal(False)},
                    {"type": "set", "target": "status", "value": literal("资料已保存。")}]}
                changes.update({"events": {"ok": prefix + "Save"}, "width": 520 if index == 0 else 680})
                title, desc = ("编辑资料", "点击打开对话框，修改姓名后确定保存。") if index == 0 else ("宽对话框", "同一表单采用更宽的内容区域。")
            elif kind == "drawer":
                changes.update({"placement": "right" if index == 0 else "left", "width": 420})
                title, desc = ("右侧抽屉", "从右边查看辅助说明。") if index == 0 else ("左侧抽屉", "从左边打开同样的说明内容。")
            elif kind in ["message", "notification"]:
                changes.update({"status": "success" if index == 0 else "warning", "duration": 3,
                                "placement": "top" if kind == "message" else "topRight"})
                changes["text" if kind == "message" else "description"] = literal("保存成功" if index == 0 else "请先检查未完成的内容")
                title, desc = ("成功通知", "点击后显示成功消息，三秒后自动关闭。") if index == 0 else ("警告通知", "点击后显示需要关注的事项。")
            else:
                changes["current"] = demo_value(prefix + "Step", index)
                title, desc = ("从头开始", "通过上一步、下一步浏览完整导览。") if index == 0 else ("从第二步开始", "从指定步骤进入导览。")
            overlay = variant(kind, prefix, changes)
            if kind == "drawer":
                overlay["children"] = [text(prefix + "Guide", "左侧逐个选择组件，右侧查看说明与用法。\n使用快速跳转可搜索中英文名称。\n每张示例卡片都可单独体验。\n按 Escape 或关闭按钮退出。")]
            if kind == "tour":
                overlay["steps"][1]["description"] = "左侧组件目录按分类排列，可搜索并逐个查看。"
            add(kind, title, desc, [button(prefix + "Trigger", "打开" + title, prefix + "Show", True), overlay])

    for index in range(2):
        floating = variant("floatButton", "floatExample" + str(index), {"shape": "circle" if index == 0 else "square", "offsetX": 28 + index * 140, "offsetY": 76})
        add("floatButton", "圆形按钮" if index == 0 else "方形按钮", "对应按钮固定显示在窗口右下方，点击可查看说明。", [text("floatNote" + str(index), "请查看窗口右下角的悬浮操作。"), floating])
    pair("watermark", "文字水印", "在内容上叠加重复标记。", {}, "自定义水印", "调整文字、间距和旋转角度。", {"content": "内部资料", "rotate": -10, "gap": 120})
    pair("borderBeam", "基础流光", "光束沿内容边框移动。", {}, "较慢的流光", "用更长周期和较粗的边框突出内容。", {"duration": 10, "borderWidth": 3})
    pair("affix", "顶部固定", "操作位置受父级可见范围约束。", {}, "底部固定", "将操作约束到父级底部。", {"offsetTop": 0, "offsetBottom": 8})
    for kind in ["app", "configProvider"]:
        for index in range(2):
            add(kind, "统一配置" if index == 0 else "统一禁用", "对子组件应用统一尺寸。" if index == 0 else "对子组件统一禁用，按钮和输入均无法操作。", [node(kind, kind + "Scope" + str(index),
                [node("space", kind + "Controls" + str(index), [button(kind + "Button" + str(index), "作用域按钮"),
                 variant("switch", kind + "Switch" + str(index), {"text": literal("功能开关")})], **{"class": "gap-3"})],
                componentSize="large" if index == 0 else "medium", componentDisabled=bool(index), **{"class": "p-4 gap-3"})])
    pair("module", "模块通信", "编辑局部草稿并发送给父级，底部显示反馈。", {}, "不同初始草稿", "模块拥有独立草稿，发送时更新父级数据。", {"state": {"draft": "另一份模块草稿"}})
    for kind in ["lineChart", "barChart"]:
        pair(kind, "基础图表", "悬停查看数据；折线图还可缩放与平移。", {}, "简洁图表", "隐藏图例，保留数据与坐标。", {"legend": False, "title": literal("简洁趋势"), "class": "h-[280px]"})
    plugin_line = copy.deepcopy(by_type["plugin"]["props"])
    plugin_line["option"]["value"]["series"][0]["type"] = "line"
    plugin_line["option"]["value"]["title"]["text"] = "ECharts Line"
    pair("plugin", "ECharts 柱状图", "点击柱子，在底部查看所选类别。", {}, "ECharts 折线图", "相同数据使用折线形式展示。", {"props": plugin_line})

    assert set(demos) == {entry[1] for entry in catalogue}, set(entry[1] for entry in catalogue) - set(demos)
    pages = []
    full_width = {"layout", "header", "sider", "content", "footer", "row", "column", "grid", "masonry", "flex", "space", "form", "formItem", "splitter", "pagination", "table", "module", "lineChart", "barChart", "plugin"}
    for group, kind, title, description in catalogue:
        cards = []
        for index, (heading, detail, children) in enumerate(demos[kind]):
            prefix = kind + "Usage" + str(index)
            cards.append(node("column", prefix, [
                node("column", prefix + "Preview", children, **{"class": "p-5 gap-3 min-h-[110px]"}),
                 node("divider", prefix + "Divider", **{"class": "border-slate-200"}),
                node("column", prefix + "Explanation", [text(prefix + "Title", heading, "text-base font-bold text-slate-800"),
                     text(prefix + "Detail", detail)], **{"class": "p-4 gap-2"})
            ], **{"class": "bg-white border border-slate-200 rounded-lg"}))
        note = {
            "upload": "文件选择仅记录本地文件，不会自动上传。",
            "image": "本页展示缺少来源与错误占位。配置有效图片来源后可加载和预览。",
            "anchor": "当前点击提供章节信息反馈，尚未自动滚动到目标。",
            "tour": "导览以步骤窗口呈现，目前不高亮页面目标。",
            "affix": "当前按父裁剪区域约束位置，尚未实现完整的滚动吸附。",
            "app": "当前提供子树配置作用域。",
        }.get(kind, "可直接操作示例；修改会在切换组件后保留。")
        pages.append(node("column", "doc_" + kind, [
            text(kind + "Category", group + " / 组件文档", "text-xs text-slate-400"),
            text(kind + "Heading", title, "text-3xl font-bold text-slate-900"),
            text(kind + "Description", description, "text-base text-slate-600"),
            text(kind + "ExamplesHeading", "用法示例", "text-xl font-bold text-slate-900"),
            node("grid", kind + "Examples", cards, columns=[{"fr": 1}] if kind in full_width else [{"fr": 1}, {"fr": 1}], **{"class": "gap-5"}),
            node("column", kind + "UsageNote", [text(kind + "NoteTitle", "使用说明", "text-base font-bold text-slate-800"),
                text(kind + "Note", note)], **{"class": "p-5 gap-2 bg-slate-50 rounded-lg"}),
            text(kind + "FooterNote", "Desktop App Engine  ·  组件实验室", "text-xs text-slate-400")
        ], **{"class": "p-8 gap-5 overflow-auto bg-white"}, layout={"height": {"fr": 1}}))

    items = [{"key": "doc_" + kind, "label": title} for _, kind, title, _ in catalogue]
    state.update({"activePage": "doc_button", "selectedMenu": ["doc_button"], "menuSelectionJson": '["doc_button"]',
                  "status": "就绪 · 从左侧选择组件，或使用快速跳转搜索。"})
    actions["galleryNavigate"] = {"eventPayload": "doc_button", "steps": [
        {"type": "set", "target": "activePage", "value": {"kind": "event"}}]}
    # Actions read one snapshot for all steps. Use the already-written Select value
    # as its action ID, then set the corresponding menu array directly.
    for _, kind, _, _ in catalogue:
        actions["doc_" + kind] = {"steps": [
            {"type": "set", "target": "selectedMenu", "value": literal(["doc_" + kind])}]}
    menu_groups = [{"type": "group", "key": "group" + str(index), "label": group,
                    "children": [{"key": "doc_" + kind, "label": title} for g, kind, title, _ in catalogue if g == group]}
                   for index, (group, _) in enumerate(GROUPS)]
    menu_height = 84 * 40 + len(GROUPS) * 30
    help_drawer = clone(fixtures["drawerDemo"], "galleryHelp")
    help_drawer["children"] = [text("galleryHelpText", "左侧按分类逐个列出 84 种组件。\n快速跳转可搜索中英文组件名。\n右侧包含组件说明和多种用法。\n左右区域分别滚动。\n底部显示操作反馈；Escape 关闭弹窗。")]
    root = node("configProvider", "galleryConfig", [node("app", "galleryApp", [
        node("column", "galleryShell", [
            node("row", "galleryHeader", [node("icon", "galleryLogo", name="AppstoreOutlined", **{"class": "w-8 h-8 text-blue-600"}),
                text("galleryBrand", "组件实验室", "text-xl font-bold text-slate-900"),
                text("galleryVersion", "DESKTOP / 0.2", "text-xs text-slate-400"),
                text("galleryHeaderHint", "84 种组件  ·  按组件探索与体验", "text-sm text-slate-500")], **{"class": "h-20 p-5 gap-4 items-center bg-white border border-slate-200"}),
            node("row", "galleryBody", [
                node("column", "gallerySidebar", [text("galleryDirectoryTitle", "组件目录", "text-sm font-bold text-slate-900"),
                    node("select", "galleryJump", value=binding("activePage"), options=literal([{ "value": item["key"], "label": item["label"]} for item in items]),
                         showSearch=True, events={"change": binding("activePage")}, **{"class": "h-10 text-sm bg-white border border-slate-300 rounded-lg hover:border-blue-500"}),
                    text("gallerySearchHint", "点击上方搜索中英文组件名", "text-xs text-slate-400"),
                    node("column", "galleryMenuScroll", [node("menu", "galleryMenu", items=menu_groups, mode="inline", selectedKeys=binding("selectedMenu"),
                         events={"click": "galleryNavigate"}, **{"class": f"h-[{menu_height}px] text-sm text-slate-600"}, layout={"shrink": 0})],
                         **{"class": "overflow-auto"}, layout={"height": {"fr": 1}})
                ], **{"class": "w-[260px] p-4 gap-3 bg-white border border-slate-200"}, layout={"height": {"fr": 1}, "shrink": 0}),
                node("tabs", "galleryPages", pages, items=items, activeKey=binding("activePage"), hideTabBar=True,
                     layout={"width": {"fr": 1}, "height": {"fr": 1}})
            ], **{"class": "gap-0"}, layout={"height": {"fr": 1}}),
            node("row", "galleryStatus", [node("text", "galleryStatusText", text=binding("status"), ellipsis=True, **{"class": "text-xs text-slate-500"})],
                 **{"class": "h-9 p-2 bg-white border border-slate-200"})
        ], **{"class": "gap-0 bg-white text-slate-800"}, layout={"height": {"fr": 1}}), help_drawer,
        clone(fixtures["messageDemo"], "galleryMessage")
    ], layout={"height": {"fr": 1}})], theme="light", locale="zh-CN", componentSize="medium", layout={"height": {"fr": 1}})
    result = {"schemaVersion": "0.2", "app": {"name": "组件实验室 · 84 种组件", "window": {"width": 1440, "height": 960}},
              "plugins": base["plugins"], "state": state, "root": root, "actions": actions}
    ids = [n["id"] for n in walk(root)]
    assert len(ids) == len(set(ids)), "Duplicate node IDs"
    OUTPUT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Generated {OUTPUT}: {len(pages)} pages, {sum(map(len, demos.values()))} examples, {len(ids)} nodes")


if __name__ == "__main__":
    main()
