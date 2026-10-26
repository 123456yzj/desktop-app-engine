"""Render the utility-class fixture from a packaged EXE; verify actual PNG pixels.

Requires Pillow. Baseline comparison is opt-in and never creates/updates baselines.
Inspect the first run's PNGs before supplying that directory as --baseline.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from PIL import Image, ImageChops


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline", type=Path)
    args = parser.parse_args()
    engine = args.engine.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    executable = output / "utility-classes.exe"
    fixture = Path(__file__).resolve().parents[1] / "examples/utility-classes.json"
    with fixture.open("rb") as source:
        subprocess.run([str(engine), "package", "--output", str(executable)], stdin=source, check=True)

    steps = [
        {"name": "normal"},
        {"name": "hover", "pointer": [80, 220]},
        {"name": "pressed", "pressed": True},
        {"name": "saved", "pressed": False, "pointer": [80, 220]},
        {"name": "disabled-pressed", "pointer": [155, 220], "pressed": True},
        {"name": "disabled-released", "pressed": False},
        {"name": "input-hover", "pointer": [100, 170]},
        {"name": "input-pressed", "pressed": True},
        {"name": "input-focused", "pressed": False},
        {"name": "input-edited", "select_all": True, "text": "Grace"},
        {"name": "input-blurred", "pointer": [700, 350], "pressed": True},
        {"name": "final", "pressed": False},
    ]
    results = []
    # Record the actual files used by install_chinese_font, including fallbacks.
    import os
    font_dir = Path(os.environ.get("WINDIR", r"C:\Windows")) / "Fonts"
    cjk = next((p for n in ("msyh.ttc", "simhei.ttf", "simsun.ttc") if (p := font_dir / n).is_file()), None)
    fonts = [p for p in (cjk, font_dir / "seguisym.ttf", font_dir / "segoeuib.ttf", font_dir / "msyhbd.ttc") if p and p.is_file()]
    (output / "fonts.json").write_text(json.dumps([
        {"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in fonts
    ], indent=2), encoding="utf-8")
    for dark in (False, True):
        for scale in (1, 1.75):
            name = f"{'dark' if dark else 'light'}-{scale}"
            folder = output / name
            scenario = output / f"{name}.json"
            scenario.write_text(json.dumps({"output": str(folder), "dark": dark,
                                           "pixels_per_point": scale, "steps": steps}), encoding="utf-8")
            # Run outside the repository, with no external JSON application argument.
            process = subprocess.run([str(executable), "--visual-test", str(scenario)], cwd=output, capture_output=True)
            (output / f"{name}.stdout.log").write_bytes(process.stdout)
            (output / f"{name}.stderr.log").write_bytes(process.stderr)
            process.check_returncode()
            report = json.loads((folder / "render-report.json").read_text(encoding="utf-8"))
            assert report["definition"] == "embedded", report
            assert Path(report["executable"]).samefile(executable), report
            images = {s["name"]: Image.open(folder / f"{s['name']}.png").convert("RGB") for s in steps}
            assert all(im.size == (int(720 * scale), int(560 * scale)) for im in images.values())

            def pixel(state, x, y):
                return images[state].getpixel((round(x * scale), round(y * scale)))

            def crop(state, rect):
                return images[state].crop(tuple(round(v * scale) for v in rect))

            def equal(a, b):
                return ImageChops.difference(a, b).getbbox() is None

            # Validate pixels in the rendered button interiors, away from glyph edges.
            assert pixel("normal", 56, 220) == (59, 130, 246), (name, "normal", pixel("normal", 56, 220))
            assert pixel("hover", 56, 220) == (37, 99, 235), (name, "hover", pixel("hover", 56, 220))
            assert pixel("pressed", 56, 220) == (29, 78, 216), (name, "pressed", pixel("pressed", 56, 220))
            assert pixel("normal", 135, 220) == (226, 232, 240), (name, "disabled")
            assert pixel("normal", 100, 90) == (248, 250, 252), (name, "card")
            assert pixel("normal", 300, 165) == (255, 255, 255), (name, "input")
            # A rounded corner exposes the card, while explicit radius=0 fills its corner.
            assert pixel("normal", 49, 203) == (248, 250, 252), (name, "rounded corner")
            border = crop("normal", (210, 195, 380, 245))
            blue = [(x, y) for y in range(border.height) for x in range(border.width)
                    if border.getpixel((x, y)) == (59, 130, 246)]
            assert blue, (name, "square border missing")
            left, top = min(x for x, _ in blue), min(y for _, y in blue)
            assert border.getpixel((left + 1, top + 1)) == (59, 130, 246), (name, "square corner")
            assert not equal(crop("normal", (32, 273, 160, 290)), crop("saved", (32, 273, 160, 290))), (name, "save status did not change")
            assert equal(crop("saved", (125, 202, 207, 238)), crop("disabled-pressed", (125, 202, 207, 238))), (name, "disabled style changed")
            assert equal(crop("saved", (32, 273, 160, 290)), crop("disabled-released", (32, 273, 160, 290))), (name, "disabled changed status")
            assert not equal(crop("input-focused", (63, 160, 140, 183)), crop("input-edited", (63, 160, 140, 183))), (name, "editing did not change text pixels")
            assert equal(crop("input-hover", (63, 160, 140, 183)), crop("normal", (63, 160, 140, 183))), (name, "hover changed input text")
            assert equal(crop("hover", (49, 240, 114, 270)), crop("normal", (49, 240, 114, 270))), (name, "unnecessary overflow tooltip")

            if args.baseline:
                baseline = args.baseline.resolve() / name
                for step, actual in images.items():
                    expected = Image.open(baseline / f"{step}.png").convert("RGB")
                    if actual.size != expected.size:
                        raise AssertionError((name, step, "snapshot dimensions differ"))
                    difference = ImageChops.difference(actual, expected)
                    if difference.getbbox() is not None:
                        difference.save(folder / f"{step}.diff.png")
                        raise AssertionError((name, step, "snapshot differs; PNG diff saved"))
            results.append({"case": name, "build": report["build"], "captures": len(images),
                            "pixel_checks": "passed", "snapshots": "passed" if args.baseline else "not requested"})
    (output / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
