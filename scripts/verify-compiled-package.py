"""Exercise the real package compiler with different inputs and no runtime JSON."""
import json
import os
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
WORK = ROOT / 'tmp-verify' / f'compiled package 验证 {time.time_ns()}'
WORK.mkdir(parents=True)
SOURCE = WORK / 'input.json'
SCRIPT = ROOT / 'scripts' / 'package.ps1'


def run(args, expected=0):
    result = subprocess.run(args, cwd=ROOT, capture_output=True, encoding='utf-8',
                            errors='replace', timeout=300)
    if result.returncode != expected:
        raise AssertionError(f'{args}\nexit={result.returncode}\n{result.stdout}\n{result.stderr}')
    return result.stdout + result.stderr


packages = []
for example, name, title in [('hello.json', 'CompiledA', '编译应用 A'),
                             ('hello.json', 'CompiledB', '编译应用 B'),
                             ('dashboard.json', 'CompiledDashboard', '编译看板 C')]:
    description = json.loads((ROOT / 'examples' / example).read_text(encoding='utf-8'))
    description['app']['name'] = title
    SOURCE.write_text(json.dumps(description, ensure_ascii=False), encoding='utf-8')
    output = WORK / name
    print(f'Building {name}: {title}', flush=True)
    run(['pwsh', '-NoProfile', '-File', str(SCRIPT), '-Description', str(SOURCE),
         '-Output', str(output), '-Name', name, '-Runtime', 'desktop-app-engine'])
    package = output / name
    exe = package / f'{name}.exe'
    assert not list(package.rglob('*.json')), 'application package must not ship JSON descriptions'
    SOURCE.unlink()
    assert title in run([str(exe), '--check']), 'packaged EXE must contain this input, not a fixed example'
    decoy = package / 'app.json'
    decoy.write_text('{invalid JSON', encoding='utf-8')
    assert title in run([str(exe), '--check']), 'neighbor JSON must not override a packaged definition'
    decoy.unlink()
    assert '--app' in run([str(exe), '--app', str(SOURCE), '--check'], expected=2)
    packages.append((exe, title))
    print(f'PASS {name}: input removed; no external JSON; override rejected', flush=True)

for exe, title in packages:
    assert title in run([str(exe), '--check']), 'later builds must not change previously packaged apps'
print(f'PASS all compiled-package checks: {WORK}', flush=True)
