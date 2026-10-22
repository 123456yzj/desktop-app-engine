"""Check complete JSON demos and local links in docs/components using the engine CLI."""

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs" / "components"
JSON_BLOCK = re.compile(r"^```json\s*\n(.*?)^```\s*$", re.MULTILINE | re.DOTALL)
LINK = re.compile(r"(?<!!)\[[^\]\n]+\]\(([^)\n]+)\)")


def anchors(text):
    """GitHub-style slugs for the plain Markdown headings used by these docs."""
    result = set()
    counts = {}
    # Fenced examples may contain lines that resemble Markdown headings.
    text = re.sub(r"^```[^\n]*\n.*?^```\s*$", "", text, flags=re.MULTILINE | re.DOTALL)
    for title in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", text, re.MULTILINE):
        slug = re.sub(r"[^\w\-\s]", "", title.lower()).replace(" ", "-")
        occurrence = counts.get(slug, 0)
        counts[slug] = occurrence + 1
        result.add(f"{slug}-{occurrence}" if occurrence else slug)
    return result


def check_links(path, text):
    errors = []
    for match in LINK.finditer(text):
        target = match.group(1).strip().removeprefix("<").removesuffix(">")
        parsed = urlsplit(target)
        if parsed.scheme or parsed.netloc:
            continue
        destination = path.parent / unquote(parsed.path) if parsed.path else path
        label = f"{path.relative_to(ROOT)}:{text.count(chr(10), 0, match.start()) + 1}"
        if not destination.exists():
            errors.append(f"{label}: missing link {target}")
        elif parsed.fragment and destination.suffix == ".md":
            if unquote(parsed.fragment) not in anchors(destination.read_text(encoding="utf-8")):
                errors.append(f"{label}: missing anchor {target}")
    return errors


def prepare_plugins(document, temporary):
    """Copy real manifest and entry only; --check never executes the worker."""
    for name in document.get("plugins", []):
        if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9_-]+", name):
            raise ValueError("invalid plugin name")
        source = ROOT / "plugins" / name
        manifest_path = source / "plugin.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        entry = (source / manifest["entry"]).resolve()
        relative_entry = entry.relative_to(source.resolve())
        destination = temporary / "plugins" / name
        destination.mkdir(parents=True, exist_ok=True)
        shutil.copy2(manifest_path, destination / "plugin.json")
        target_entry = destination / relative_entry
        target_entry.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(entry, target_entry)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, default=ROOT / "target/debug/desktop-app-engine.exe")
    parser.add_argument("--temp-dir", type=Path, help="Existing parent for temporary example files")
    parser.add_argument("--schema", action="store_true", help="Also validate JSON Schema (requires Python jsonschema)")
    options = parser.parse_args()
    engine = options.engine.resolve()
    if not engine.is_file():
        parser.error(f"engine not found: {engine}; build desktop-app-engine first")
    if options.temp_dir is not None and not options.temp_dir.is_dir():
        parser.error(f"temporary parent does not exist: {options.temp_dir}")
    validator = None
    if options.schema:
        try:
            from jsonschema import Draft202012Validator
        except ImportError:
            parser.error("--schema requires the Python jsonschema package")
        schema = json.loads((ROOT / "docs/description-0.2.schema.json").read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema)

    failures = []
    checked = 0
    pages = sorted(DOCS.glob("*.md"))
    if not pages:
        parser.error(f"no component documentation found under {DOCS}")
    with tempfile.TemporaryDirectory(prefix="dae-component-docs-", dir=options.temp_dir) as directory:
        temporary = Path(directory)
        for page in pages:
            text = page.read_text(encoding="utf-8")
            failures.extend(check_links(page, text))
            examples = list(JSON_BLOCK.finditer(text))
            if page.name != "README.md" and not examples:
                failures.append(f"{page.name}: missing a complete JSON example")
            for number, match in enumerate(examples, 1):
                label = f"{page.name} example {number}"
                try:
                    document = json.loads(match.group(1))
                    required = {"schemaVersion", "app", "state", "root", "actions"}
                    if not isinstance(document, dict) or not required.issubset(document):
                        raise ValueError("expected a complete application, not a JSON fragment")
                    if validator is not None:
                        error = next(validator.iter_errors(document), None)
                        if error is not None:
                            raise ValueError(f"JSON Schema {error.json_path}: {error.validator} validation failed")
                    prepare_plugins(document, temporary)
                    fixture = temporary / f"{page.stem}-{number}.json"
                    fixture.write_text(json.dumps(document, ensure_ascii=False), encoding="utf-8")
                    result = subprocess.run(
                        [str(engine), "--app", str(fixture), "--check"],
                        capture_output=True, encoding="utf-8", errors="replace", timeout=30,
                        cwd=ROOT,
                    )
                    if result.returncode:
                        raise ValueError(f"engine exited {result.returncode}: {result.stderr.strip()} {result.stdout.strip()}")
                    checked += 1
                    print(f"PASS {label}")
                except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
                    failures.append(f"{label}: {error}")

    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)
    print(f"Checked {checked} examples across {len(pages)} pages; {len(failures)} failures.")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
