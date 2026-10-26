"""Render the real packaged gallery, navigating through its search control.

python scripts/test-packaged-gallery.py --exe path/to/gallery.exe --output new-dir
Use --pages button,select,form for a focused run; omit it to visit all 84 pages.
No source definition is passed to the executable.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

from PIL import Image, ImageChops, ImageDraw, ImageFont


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--pages", default="")
    parser.add_argument("--scale", type=float, default=1)
    parser.add_argument("--dark", action="store_true")
    args = parser.parse_args()
    exe = args.exe.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    spec = importlib.util.spec_from_file_location("gallery", Path(__file__).with_name("build-component-gallery.py"))
    gallery = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(gallery)
    entries = [(line.split("|")[0], line.split("|")[1]) for _, group in gallery.GROUPS
               for line in group.strip().splitlines()]
    if args.pages:
        requested = args.pages.split(",")
        assert set(requested) <= {kind for kind, _ in entries}
        entries = [entry for entry in entries if entry[0] in requested]
    env = dict(os.environ, LOCALAPPDATA=str(output / "isolated-cache"))
    rendered = []
    reports = []
    for batch_index in range(0, len(entries), 20):
        batch = entries[batch_index:batch_index + 20]
        steps = [{"name": "initial"}]
        for kind, title in batch:
            steps.extend([
                {"name": kind + "-press", "pointer": [120, 155], "pressed": True},
                {"name": kind + "-open", "pointer": [120, 155], "pressed": False},
                {"name": kind + "-search", "text": title, "select_all": True},
                {"name": kind, "keys": ["Enter"], "wait_ms": 5000 if kind == "plugin" else 0},
            ])
        scenario = output / f"batch-{batch_index // 20}.json"
        images = output / f"batch-{batch_index // 20}"
        scenario.write_text(json.dumps({"output": str(images), "dark": args.dark,
            "pixels_per_point": args.scale, "steps": steps}, ensure_ascii=False), encoding="utf-8")
        result = subprocess.run([str(exe), "--visual-test", str(scenario)], cwd=output, env=env,
                                capture_output=True, timeout=180)
        scenario.with_suffix(".stdout.log").write_bytes(result.stdout)
        scenario.with_suffix(".stderr.log").write_bytes(result.stderr)
        result.check_returncode()
        reports.append(json.loads((images / "render-report.json").read_text(encoding="utf-8")))
        for kind, title in batch:
            image_path = images / (kind + ".png")
            with Image.open(image_path) as image:
                assert image.width >= 1000 and image.height >= 600
                region = image.convert("RGB").crop(tuple(round(v * args.scale) for v in (290, 120, 1000, 280)))
                assert ImageChops.difference(region, Image.new("RGB", region.size, "white")).getbbox(), kind
                if kind == "plugin":
                    chart = image.convert("RGB").crop(tuple(round(v * args.scale) for v in (340, 380, 1360, 620)))
                    blue_pixels = sum(b > 180 and g > 60 and b > r + 80 for r, g, b in chart.getdata())
                    assert blue_pixels > 1000 * args.scale ** 2, "ECharts did not render its blue bars"
            rendered.append({"type": kind, "title": title, "image": str(image_path)})
        print(f"Rendered {len(rendered)}/{len(entries)} pages", flush=True)
    font = ImageFont.truetype(str(Path(os.environ["WINDIR"]) / "Fonts/msyh.ttc"), 16)
    sheets = []
    for start in range(0, len(rendered), 12):
        subset = rendered[start:start + 12]
        sheet = Image.new("RGB", (1440, 3 * 285), "#e8edf3")
        draw = ImageDraw.Draw(sheet)
        for index, entry in enumerate(subset):
            x, y = index % 4 * 360, index // 4 * 285
            draw.text((x + 10, y + 5), entry["title"], fill="#172033", font=font)
            with Image.open(entry["image"]) as image:
                preview = image.crop(tuple(round(v * args.scale) for v in (275, 95, 1410, 890)))
                preview.thumbnail((350, 245))
                sheet.paste(preview, (x + 5, y + 32))
        path = output / f"overview-{start // 12 + 1}.png"
        sheet.save(path)
        sheets.append(str(path))
    (output / "results.json").write_text(json.dumps({"executable": str(exe),
        "exe_sha256": hashlib.sha256(exe.read_bytes()).hexdigest(), "pages_rendered": len(rendered),
        "scale": args.scale, "dark": args.dark, "pages": rendered, "contact_sheets": sheets,
        "reports": reports, "visual_review": "PNG files generated; inspect before accepting"},
        ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
