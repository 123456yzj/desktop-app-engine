"""Exercise Select through a packaged EXE and verify actual PNG pixels (requires Pillow).

Run with --engine target/debug/desktop-app-engine.exe --output <new directory>.
Images require visual inspection; pixel assertions do not replace that inspection.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from PIL import Image, ImageChops, ImageStat


def fixture():
    def text(key, label, height=20, binding=False):
        return {"type": "text", "id": key, "class": f"h-[{height}px] text-sm",
                "text": {"kind": "state", "key": label} if binding else {"kind": "literal", "value": label}}

    def select(key, **props):
        return {"type": "select", "id": key,
                "class": "h-10 w-[520px] text-sm bg-white text-slate-800 border border-slate-300 rounded-lg hover:border-blue-500 disabled:bg-slate-100 disabled:text-slate-400",
                "value": {"kind": "state", "key": key}, "placeholder": {"kind": "literal", "value": "请选择语言"},
                "options": {"kind": "literal", "value": [
                    {"value": "rust", "label": "Rust"},
                    {"value": "disabled", "label": "不可选择", "disabled": True},
                    {"value": "python", "label": "Python"},
                    {"value": "typescript", "label": "TypeScript"}]}, **props}

    return {"schemaVersion": "0.2", "app": {"name": "Select interaction acceptance",
            "window": {"width": 900, "height": 700}},
            "state": {"single": "rust", "simple": "rust", "multiple": ["rust"],
                      "tags": [], "disabled": "rust", "loading": "rust"}, "actions": {},
            "root": {"type": "column", "id": "root", "class": "p-6 gap-3", "children": [
                text("title", "Select / 交互验收", 28),
                text("singleLabel", "单选 / 搜索、清空与键盘"),
                select("single", showSearch=True, allowClear=True),
                text("singleValue", "single", 24, True),
                text("simpleLabel", "基础单选 / 无扩展配置"), select("simple"),
                text("simpleValue", "simple", 24, True),
                text("multipleLabel", "多选 / 最多两项"),
                select("multiple", mode="multiple", maxCount=2, allowClear=True),
                text("multipleHint", "选择后保持展开，达到上限禁用未选项", 24),
                text("tagsLabel", "标签 / 输入新值后 Enter"),
                select("tags", mode="tags", allowClear=True),
                text("tagsHint", "单选、多选、标签共用边框及关闭逻辑", 24),
                select("disabled", enabled={"kind": "literal", "value": False}, allowClear=True),
                select("loading", loading=True, showSearch=True)]}}


def steps():
    result = [{"name": "normal"}, {"name": "hover", "pointer": [200, 115]}]

    def click(name, x, y):
        result.extend([{"name": name + "-press", "pointer": [x, y], "pressed": True},
                       {"name": name, "pressed": False}])

    click("open", 200, 115)
    click("disabled-option", 200, 194)
    result.append({"name": "filtered", "text": "py"})
    result.append({"name": "chosen", "keys": ["Enter"]})
    click("reopen", 200, 115)
    result.append({"name": "empty", "text": "missing"})
    result.append({"name": "escape", "keys": ["Escape"]})
    result.append({"name": "escape-reopen", "keys": ["ArrowDown"]})
    result.append({"name": "tab", "keys": ["Tab"]})
    click("clear", 536, 115)
    click("outside", 800, 115)
    click("simple-open", 200, 235)
    result.append({"name": "simple-arrow", "keys": ["ArrowDown"]})
    result.append({"name": "simple-chosen", "keys": ["Enter"]})
    click("single-again", 200, 115)
    click("outside-close", 800, 115)
    click("multiple-open", 250, 355)
    result.append({"name": "multiple-added", "keys": ["ArrowDown", "Enter"]})
    click("max-count", 200, 502)
    result.append({"name": "multiple-escape", "keys": ["Escape"]})
    click("tag-open", 200, 475)
    result.append({"name": "tag-input", "text": "NewLanguage"})
    result.append({"name": "tag-added", "keys": ["Enter"]})
    result.append({"name": "tag-escape", "keys": ["Escape"]})
    click("disabled-control", 200, 563)
    click("loading-open", 200, 615)
    result.append({"name": "loading-enter", "keys": ["Enter"]})
    result.append({"name": "loading-escape", "keys": ["Escape"]})
    return result


def verify(folder, scale):
    images = {step["name"]: Image.open(folder / (step["name"] + ".png")).convert("RGB") for step in steps()}
    assert all(im.size == (int(900 * scale), int(700 * scale)) for im in images.values())

    def crop(name, rect):
        return images[name].crop(tuple(round((v + 8) * scale) for v in rect))

    def equal(a, b, rect):
        return ImageChops.difference(crop(a, rect), crop(b, rect)).getbbox() is None

    # Check continuous top/bottom borders, not just the rounded corners.
    for name in ("normal", "hover", "open"):
        for y in (97, 134):
            line = crop(name, (45, y, 520, y + 1))
            assert sum(1 for x in range(line.width) if (lambda c: c[2] > c[0] + 10)(line.getpixel((x, 0)))) > line.width * 0.8, (name, y, "border missing")
    assert not equal("normal", "hover", (40, 96, 520, 99)), "hover border did not change"
    assert not equal("normal", "open", (30, 142, 540, 280)), "popup missing"
    difference = ImageChops.difference(crop("open", (505, 145, 530, 168)), crop("disabled-option", (505, 145, 530, 168)))
    assert max(ImageStat.Stat(difference).mean) < 3, "disabled option changed selection"
    assert not equal("open", "filtered", (30, 142, 540, 280)), "search did not filter"
    assert not equal("normal", "chosen", (24, 148, 200, 172)), "Enter did not commit"
    assert equal("normal", "chosen", (300, 180, 500, 208)), "single popup did not close"
    assert not equal("reopen", "empty", (30, 142, 540, 280)), "empty message missing"
    assert equal("normal", "escape", (300, 180, 500, 208)), "Escape did not close"
    assert not equal("escape", "escape-reopen", (30, 142, 540, 280)), "keyboard reopen failed"
    assert equal("normal", "tab", (300, 180, 500, 208)), "Tab did not close"
    assert not equal("chosen", "clear", (24, 148, 200, 172)), "clear did not write state"
    assert equal("normal", "clear", (300, 180, 500, 208)), "clear opened popup"
    assert not equal("normal", "simple-chosen", (24, 268, 200, 292)), "basic selector keyboard did not skip disabled option"
    assert equal("normal", "outside-close", (300, 180, 500, 208)), "outside click did not close"
    assert not equal("multiple-open", "multiple-added", (35, 342, 250, 371)), "multiple selection missing"
    assert equal("multiple-added", "max-count", (35, 342, 250, 371)), "maxCount allowed an extra value"
    assert not equal("tag-input", "tag-added", (35, 461, 300, 491)), "tag creation failed"
    assert equal("normal", "disabled-control", (24, 544, 544, 584)), "disabled control changed"
    assert equal("normal", "loading-escape", (35, 610, 200, 630)), "loading allowed selection"
    sheet = Image.new("RGB", (900, 700 * 3), "white")
    for index, name in enumerate(("hover", "open", "multiple-added")):
        sheet.paste(images[name].resize((900, 700)), (0, index * 700))
    sheet.save(folder / "overview.png")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cases", default="light-1,light-1.75,dark-1")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, LOCALAPPDATA=str(output / "isolated-cache"))
    source = json.dumps(fixture(), ensure_ascii=False).encode("utf-8")
    (output / "fixture.json").write_bytes(source)
    exe = output / "select-acceptance.exe"
    package = subprocess.run([str(args.engine.resolve()), "package", "--output", str(exe)],
                             input=source, capture_output=True, cwd=output, env=env)
    (output / "package.stdout.log").write_bytes(package.stdout)
    (output / "package.stderr.log").write_bytes(package.stderr)
    package.check_returncode()
    results = []
    for case in args.cases.split(","):
        theme, scale_text = case.split("-")
        scale = float(scale_text)
        folder = output / case
        scenario = output / (case + ".json")
        scenario.write_text(json.dumps({"output": str(folder), "dark": theme == "dark",
                                       "pixels_per_point": scale, "steps": steps()}), encoding="utf-8")
        render = subprocess.run([str(exe), "--visual-test", str(scenario)], cwd=output, env=env, capture_output=True)
        (output / (case + ".stdout.log")).write_bytes(render.stdout)
        (output / (case + ".stderr.log")).write_bytes(render.stderr)
        render.check_returncode()
        report = json.loads((folder / "render-report.json").read_text())
        assert report["definition"] == "embedded" and Path(report["executable"]).samefile(exe)
        verify(folder, scale)
        results.append({"case": case, "captures": len(steps()), "build": report["build"], "pixels": "passed"})
    (output / "results.json").write_text(json.dumps({"source_sha256": hashlib.sha256(source).hexdigest(),
                                                    "results": results}, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
