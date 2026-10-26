"""Package all-components and capture real EXE pixels without desktop input.

Images require visual review; generating them does not constitute acceptance.
"""
import argparse
import hashlib
import html
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]


def write_index(output, related=None, baseline=None):
    """Create a local navigation page; never automatically approve images."""
    artifact = json.loads((output / "artifact.json").read_text(encoding="utf-8"))
    executable = Path(artifact["exe"])
    folders = [output] + ([related.resolve()] if related else [])
    reports = []
    for folder in folders:
        for report_file in sorted(folder.glob("*/render-report.json")):
            report = json.loads(report_file.read_text(encoding="utf-8"))
            reports.append((report_file, report))
    fonts = Path(os.environ.get("WINDIR", r"C:\Windows")) / "Fonts"
    names = [next((n for n in ["msyh.ttc", "simhei.ttf", "simsun.ttc"] if (fonts / n).exists()), "")]
    names += ["seguisym.ttf", "segoeuib.ttf", "msyhbd.ttc"]
    (output / "fonts.json").write_text(json.dumps({n: hashlib.sha256((fonts / n).read_bytes()).hexdigest()
        for n in names if n and (fonts / n).exists()}, indent=2), encoding="utf-8")
    sections = []
    if baseline:
        for title, before, after in [
            ("Tag / Descriptions", "display/top.png", "display/top.png"),
            ("BarChart", "data-scrollbar/middle.png", "chart/page-scroll.png"),
            ("Form", "form/scroll-1.png", "inputs/scrolled.png"),
            ("Transfer", "form/scroll-2.png", "transfer/normal.png"),
            ("Modal", "overlay-modal/opened.png", "modal/settled.png"),
            ("Notification", "notification-settled/settled.png", "notification/settled.png"),
            ("Cascader", "input-cascaderDemo/opened.png", "cascader/opened.png"),
            ("Popconfirm", "overlay-popconfirm/opened.png", "popconfirm/settled.png"),
        ]:
            paths = [baseline.resolve() / before, output / after]
            if all(p.exists() for p in paths):
                figures = "".join(f'<figure><figcaption>{label}</figcaption><a href="{p.as_uri()}"><img src="{p.as_uri()}"></a></figure>'
                    for label, p in zip(["Before", "After"], paths))
                sections.append(f'<h3>{title}</h3><div class="pair">{figures}</div>')
    for report_file, report in reports:
        folder = report_file.parent
        sheet = folder.parent / (folder.name + "-sheet.png")
        links = " ".join(f'<a href="{(folder / (c["name"] + ".png")).as_uri()}">{html.escape(c["name"])}</a>'
            for c in report["captures"])
        sections.append(f'<details><summary>{html.escape(folder.name)} — {len(report["captures"])} PNG; '
            f'{report["build"]}; {report["pixels_per_point"]}x; {report["theme"]}</summary>'
            f'<p>{links} <a href="{report_file.as_uri()}">render-report.json</a></p>'
            f'<a href="{sheet.as_uri()}"><img src="{sheet.as_uri()}"></a></details>')
    count = sum(len(r["captures"]) for _, r in reports)
    page = f'''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>All Components 修复检查</title>
<style>body{{font:16px system-ui;margin:32px;background:#f6f8fb;color:#172336}}img{{max-width:100%}}.pair{{display:flex;gap:16px}}figure{{margin:0;width:50%}}summary{{padding:12px;cursor:pointer}}details{{background:white;margin:12px 0;padding:8px}}a{{color:#2460ca}}</style>
<h1>All Components 修复检查</h1><p>{len(reports)} 个场景 / {count} 张实际 EXE PNG。
Debug；内置定义；wgpu / Dx12 CPU 离屏。普通窗口为 Glow，未做原生窗口/DPI 验收。</p>
<p>示例 configProvider 固定浅色；dark 场景仅验证深色宿主。图片生成不自动表示验收通过，逐项结论见项目 docs/all-components-fix-plan.md。</p>
<p><a href="{executable.as_uri()}">打开 EXE</a> · <a href="{(output / 'artifact.json').as_uri()}">产物哈希</a> · <a href="{(output / 'fonts.json').as_uri()}">字体哈希</a></p>
{''.join(sections)}</html>'''
    (output / "index.html").write_text(page, encoding="utf-8")
    print(f"Review entry: {output / 'index.html'} ({count} captures)")


def enter(index):
    return [
        {"name": "initial", "capture": False},
        {"name": "menu-press", "pointer": [110, 180 + 42 * index], "pressed": True, "capture": False},
        {"name": "page", "pressed": False, "capture": False},
    ]


def click(target, name="opened", position=None):
    press = {"name": name + "-press", "target": target, "pressed": True, "capture": False}
    if position:
        press["target_position"] = position
    return [press, {"name": name, "pressed": False, "wait_ms": 200}]


def point(x, y, name):
    return [{"name": name + "-press", "pointer": [x, y], "pressed": True, "capture": False},
        {"name": name, "pressed": False, "wait_ms": 300}]


def interaction_cases():
    """Real input sequences; screenshots, not case names, determine acceptance."""
    cases = []
    for suffix, scale, dark in [("", 1, False), ("-scaled", 1.75, False), ("-dark", 1, True)]:
        form = enter(1) + [{"name": "scroll", "pointer": [1110, 630], "wheel": [0, -440],
            "capture": False, "wait_ms": 300}]
        definitions = [
            ("cascader-select", form + click("cascaderDemo") + point(716, 607, "expanded")
                + point(750, 642, "selected") + click("cascaderDemo", "cleared", [0.93, 0.5])),
            ("autocomplete-select", form + click("autoCompleteDemo")
                + [{"name": "typed", "text": "Gr", "wait_ms": 200}]
                + point(300, 575, "selected") + click("autoCompleteDemo", "cleared", [0.93, 0.5])),
            ("date-select", form + click("datePickerDemo", position=[0.4, 0.5])
                + point(470, 774, "selected") + click("datePickerDemo", "cleared", [0.93, 0.5])),
            ("modal-interact", enter(5) + click("showModal") + click("modalName", "focused")
                + [{"name": "edited", "select_all": True, "text": "Grace"}]
                + point(902, 513, "confirmed") + click("showModal", "reopened")
                + point(843, 513, "cancelled") + click("showModal", "reopened-again")
                + point(913, 387, "closed")),
            ("popconfirm-interact", enter(5) + click("popconfirmDemo")
                + point(865, 416, "confirmed") + click("popconfirmDemo", "reopened")
                + point(807, 416, "cancelled")),
            ("notification-close", enter(5) + click("showNotification")
                + click("notificationDemo", "closed", [0.58, 0.36])),
            ("time-edit", form + point(839, 620, "focused")
                + [{"name": "edited", "select_all": True, "text": "5"},
                   {"name": "submitted", "keys": ["Enter"]}]
                + point(1150, 470, "blurred")),
        ]
        for name, steps in definitions:
            cases.append({"name": name + suffix, "scale": scale, "dark": dark, "steps": steps})
    return cases


def render(exe, output, name, steps, scale=1, dark=False):
    scenario = output / (name + ".json")
    images = output / name
    scenario.write_text(json.dumps({"output": str(images), "dark": dark,
        "pixels_per_point": scale, "steps": steps}), encoding="utf-8")
    env = dict(os.environ, LOCALAPPDATA=str(output / "isolated-cache"))
    result = subprocess.run([str(exe), "--visual-test", str(scenario)], cwd=output,
        env=env, capture_output=True, timeout=180)
    (output / (name + ".stdout.log")).write_bytes(result.stdout)
    (output / (name + ".stderr.log")).write_bytes(result.stderr)
    if result.returncode:
        print(result.stderr.decode("utf-8", errors="replace"))
    result.check_returncode()
    report = json.loads((images / "render-report.json").read_text(encoding="utf-8"))
    assert report["definition"] == "embedded"
    assert Path(report["executable"]).samefile(exe)
    sheet = Image.new("RGB", (1360, 475 * ((len(report["captures"]) + 1) // 2)), "#e8edf3")
    draw = ImageDraw.Draw(sheet)
    for index, capture in enumerate(report["captures"]):
        with Image.open(images / (capture["name"] + ".png")) as image:
            assert image.getbbox() and image.size == (round(1360 * scale), round(900 * scale))
            x, y = index % 2 * 680, index // 2 * 475
            draw.text((x + 8, y + 5), f"{name}/{capture['name']}", fill="black")
            sheet.paste(image.resize((680, 450)), (x, y + 25))
    sheet.save(output / (name + "-sheet.png"))
    print(f"{name}: {len(report['captures'])} PNG; embedded; {report['adapter']}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path)
    parser.add_argument("--exe", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--plan", type=Path, help="JSON list of custom cases; uses existing EXE")
    parser.add_argument("--with-interactions", action="store_true", help="Also capture selection, dialog and single-digit time input sequences")
    parser.add_argument("--index-only", action="store_true", help="Index existing captures without running tests")
    parser.add_argument("--related", type=Path, help="Additional capture directory to index")
    parser.add_argument("--baseline", type=Path, help="Before-fix audit directory for side-by-side images")
    args = parser.parse_args()
    output = args.output.resolve()
    if args.index_only:
        if not output.is_dir():
            parser.error("--index-only requires an existing output directory")
        write_index(output, args.related, args.baseline)
        return
    output.mkdir(parents=True, exist_ok=False)
    exe = args.exe.resolve() if args.exe else output / "all-components.exe"
    if not args.exe:
        if not args.engine:
            parser.error("--engine or --exe is required")
        producer = output / "producer"
        runtime = producer / "plugins/runtime"
        runtime.mkdir(parents=True)
        engine = producer / "desktop-app-engine.exe"
        shutil.copy2(args.engine.resolve(), engine)
        shutil.copytree(ROOT / "plugins/echarts", producer / "plugins/echarts")
        node = shutil.which("node")
        if not node:
            raise RuntimeError("Node runtime not found on PATH")
        shutil.copy2(node, runtime / "node.exe")
        with (ROOT / "examples/all-components.json").open("rb") as source:
            subprocess.run([str(engine), "package", "--output", str(exe)],
                cwd=ROOT, stdin=source, check=True, timeout=300)
    (output / "artifact.json").write_text(json.dumps({"exe": str(exe),
        "sha256": hashlib.sha256(exe.read_bytes()).hexdigest(),
        "definition": "examples/all-components.json", "build": "see render-report.json"}, indent=2), encoding="utf-8")
    if args.plan:
        for case in json.loads(args.plan.read_text(encoding="utf-8")):
            render(exe, output, case["name"], case["steps"], case.get("scale", 1), case.get("dark", False))
        write_index(output, args.related, args.baseline)
        return
    for index, name in enumerate(["layout", "form", "navigation", "display", "data", "feedback"]):
        steps = enter(index) + [{"name": "top", "wait_ms": 1000}]
        steps += [{"name": f"scroll-{n}", "pointer": [1110, 630], "wheel": [0, -440],
            "wait_ms": 1000 if name == "data" else 150} for n in range(1, 5)]
        render(exe, output, name, steps)
    for suffix, scale, dark in [("", 1, False), ("-scaled", 1.75, False), ("-dark", 1, True)]:
        render(exe, output, "tag" + suffix, enter(3) + [{"name": "normal"}]
            + click("checkableTag", "checked", [0.35, 0.5])
            + click("checkableTag", "closed", [0.94, 0.5]), scale, dark)
        for name, target in [("modal", "showModal"), ("notification", "showNotification"), ("popconfirm", "popconfirmDemo")]:
            render(exe, output, name + suffix, enter(5) + click(target)
                + [{"name": "settled", "wait_ms": 1000}], scale, dark)
        render(exe, output, "inputs" + suffix, enter(1) + [
            {"name": "scrolled", "pointer": [1110, 630], "wheel": [0, -440], "wait_ms": 200}], scale, dark)
        for name, target, position in [("cascader", "cascaderDemo", [0.5, 0.5]),
                ("date", "datePickerDemo", [0.4, 0.5]), ("autocomplete", "autoCompleteDemo", [0.3, 0.5])]:
            render(exe, output, name + suffix, enter(1) + [
                {"name": "scrolled", "pointer": [1110, 630], "wheel": [0, -440], "capture": False, "wait_ms": 200}]
                + click(target, position=position), scale, dark)
        render(exe, output, "transfer" + suffix, enter(1) + [
            {"name": "scroll-1", "pointer": [1110, 630], "wheel": [0, -440], "capture": False, "wait_ms": 200},
            {"name": "normal", "wheel": [0, -440], "wait_ms": 200}]
            + click("transferDemo", "selected", [0.025, 0.46])
            + click("transferDemo", "moved-right", [0.5, 0.51])
            + click("transferDemo", "selected-right", [0.55, 0.64])
            + click("transferDemo", "moved-left", [0.5, 0.70]), scale, dark)
        render(exe, output, "chart" + suffix, enter(4) + [
            {"name": "before"},
            {"name": "zoom", "target": "lineChartDemo", "target_position": [0.5, 0.5], "alt": True, "wheel": [0, 160], "wait_ms": 200},
            {"name": "page-scroll", "pointer": [1110, 630], "wheel": [0, -440], "wait_ms": 1000},
            {"name": "lower", "pointer": [1110, 630], "wheel": [0, -440], "wait_ms": 3000},
        ], scale, dark)
    if args.with_interactions:
        cases = interaction_cases()
        (output / "interaction-plan.json").write_text(json.dumps(cases, ensure_ascii=False, indent=2), encoding="utf-8")
        for case in cases:
            render(exe, output, case["name"], case["steps"], case["scale"], case["dark"])
    write_index(output, args.related, args.baseline)
    print("Captures generated. Visual review still required.")


if __name__ == "__main__":
    main()
