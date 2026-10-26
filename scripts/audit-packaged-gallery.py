"""Generate reproducible, per-component PNG evidence from an embedded gallery EXE.

Run pages first, inspect its images, then states. --launch runs a worker with its
output redirected to worker.log; progress.json records completed/failed items.
Layout rectangles only locate input. Acceptance requires reviewing actual PNGs.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
INTERACTIVE = {
    'button', 'floatButton', 'autoComplete', 'cascader', 'checkbox', 'colorPicker',
    'datePicker', 'input', 'inputNumber', 'mentions', 'radio', 'rate', 'select',
    'slider', 'switch', 'timePicker', 'transfer', 'treeSelect', 'upload',
    'anchor', 'breadcrumb', 'dropdown', 'menu', 'pagination', 'steps', 'tabs',
    'carousel', 'collapse', 'popover', 'segmented', 'table', 'tooltip', 'tree',
    'alert', 'popconfirm', 'splitter', 'lineChart', 'barChart', 'plugin', 'badge',
    'calendar', 'listy',
}


def walk(node):
    yield node
    for child in node.get('children', []):
        yield from walk(child)


def write_json(path, value):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding='utf-8')
    temporary.replace(path)


def navigate(kind, title):
    return [
        {'name': 'jump-press', 'pointer': [120, 155], 'pressed': True, 'capture': False},
        {'name': 'jump-open', 'pressed': False, 'capture': False},
        {'name': 'jump-search', 'text': title, 'select_all': True, 'capture': False},
        {'name': 'jump-select', 'keys': ['Enter'], 'capture': False},
        {'name': 'blur-press', 'pointer': [280, 85], 'pressed': True, 'capture': False},
        {'name': 'blur', 'pressed': False, 'capture': False},
    ]


def scroll(name, delta, capture=False):
    return {'name': name, 'pointer': [1398, 500], 'wheel': [0, delta],
            'wait_ms': 250, 'capture': capture}


def render(exe, folder, steps, scale):
    if folder.exists():
        # Preserve incomplete evidence before retrying an interrupted scenario.
        folder.rename(folder.with_name(folder.name + '-incomplete-' + str(time.time_ns())))
    folder.mkdir(parents=True, exist_ok=False)
    scenario = folder / 'scenario.json'
    write_json(scenario, {'output': str(folder), 'dark': False,
                         'pixels_per_point': scale, 'steps': steps})
    cache = folder / 'isolated-cache'
    try:
        result = subprocess.run([str(exe), '--visual-test', str(scenario)], cwd=folder,
            env=dict(os.environ, LOCALAPPDATA=str(cache)),
            capture_output=True, timeout=240)
    finally:
        shutil.rmtree(cache, ignore_errors=True)
    (folder / 'stdout.log').write_bytes(result.stdout)
    (folder / 'stderr.log').write_bytes(result.stderr)
    result.check_returncode()
    report = json.loads((folder / 'render-report.json').read_text(encoding='utf-8'))
    assert report['definition'] == 'embedded'
    for capture in report['captures']:
        with Image.open(folder / (capture['name'] + '.png')) as image:
            image.verify()
    return report


def sheet(folder, report, scale, filename='review.png'):
    captures = report['captures']
    width, height = 740, 505
    result = Image.new('RGB', (width * 2, height * ((len(captures) + 1) // 2)), '#e8edf3')
    draw = ImageDraw.Draw(result)
    font = ImageFont.truetype(str(Path(os.environ['WINDIR']) / 'Fonts/msyh.ttc'), 15)
    for index, capture in enumerate(captures):
        x, y = index % 2 * width, index // 2 * height
        draw.text((x + 10, y + 6), capture['name'], font=font, fill='#172033')
        with Image.open(folder / (capture['name'] + '.png')) as image:
            crop = image.crop(tuple(round(v * scale) for v in (275, 85, 1425, 915)))
            crop.thumbnail((730, 475))
            result.paste(crop, (x + 5, y + 28))
    result.save(folder / filename)


def target_position(kind):
    return {
        'autoComplete': [0.25, 0.10], 'mentions': [0.25, 0.12],
        'cascader': [0.30, 0.12], 'treeSelect': [0.30, 0.12],
        'datePicker': [0.12, 0.10], 'colorPicker': [0.03, 0.08],
        'checkbox': [0.03, 0.5], 'radio': [0.05, 0.5],
        'timePicker': [0.06, 0.10], 'transfer': [0.05, 0.18],
        'anchor': [0.08, 0.15], 'breadcrumb': [0.08, 0.2],
        'menu': [0.10, 0.20], 'pagination': [0.12, 0.15],
        'steps': [0.15, 0.12], 'tabs': [0.08, 0.12],
        'carousel': [0.06, 0.10], 'collapse': [0.12, 0.08],
        'segmented': [0.12, 0.12], 'table': [0.15, 0.08],
        'tree': [0.02, 0.08], 'alert': [0.98, 0.15],
        'upload': [0.2, 0.12], 'badge': [0.3, 0.5],
        'calendar': [0.18, 0.40], 'listy': [0.5, 0.5],
    }.get(kind, [0.5, 0.5])


def state_steps(control, placement, index, capture=True):
    kind, node_id = control['type'], control['id']
    prefix = f'{index:02d}-{kind}'
    steps = [scroll(prefix + '-reset', 10000)]
    if placement != 'top':
        steps.append(scroll(prefix + '-middle', -400))
    if placement == 'end':
        steps.append(scroll(prefix + '-end', -10000))
    pointer = {'target': node_id, 'target_position': target_position(kind)}
    if kind == 'floatButton':
        pointer = {'pointer': [1440 - control.get('offsetX', 28) - 24,
                               960 - control.get('offsetY', 76) - 24]}
    steps.append({'name': prefix + '-hover', **pointer, 'wait_ms': 900 if kind == 'tooltip' else 0})
    # Do not launch a desktop file dialog during offscreen acceptance.
    if kind != 'upload':
        steps.append({'name': prefix + '-pressed', **pointer, 'pressed': True})
        steps.append({'name': prefix + '-changed', 'pressed': False,
                      'wait_ms': 1000 if kind == 'plugin' else 250})
        if kind in {'input', 'autoComplete', 'mentions', 'text', 'inputNumber', 'timePicker'}:
            edited = {'mentions': '@A', 'inputNumber': '42', 'timePicker': '9'}.get(kind, 'A')
            steps.append({'name': prefix + '-edited', 'text': edited,
                          'select_all': kind in {'inputNumber', 'timePicker'}})
        if kind in {'slider', 'splitter'}:
            steps.insert(-1, {'name': prefix + '-drag', 'target': node_id,
             'target_position': [0.70, 0.5]})
        if kind == 'button':
            steps.extend([
                {'name': prefix + '-keyboard-focus', 'pointer': [280, 85], 'keys': ['Tab']},
                {'name': prefix + '-keyboard-activate', 'keys': ['Enter']},
            ])
        if kind == 'badge' and control.get('closable'):
            steps.extend([
                {'name': prefix + '-close-press', 'target': node_id,
                 'target_position': [0.94, 0.5], 'pressed': True, 'capture': False},
                {'name': prefix + '-closed', 'pressed': False},
            ])
        if kind == 'listy':
            steps.append({'name': prefix + '-scrolled', **pointer, 'wheel': [0, -400]})
    steps.extend([
        {'name': prefix + '-escape', 'keys': ['Escape'], 'capture': False},
        {'name': prefix + '-outside-press', 'pointer': [280, 85], 'pressed': True, 'capture': False},
        {'name': prefix + '-outside', 'pressed': False, 'capture': False},
    ])
    return steps


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--phase', choices=['pages', 'states', 'all'], default='pages')
    parser.add_argument('--pages', default='')
    parser.add_argument('--scale', type=float, default=1)
    parser.add_argument('--launch', action='store_true')
    args = parser.parse_args()
    exe, output = args.exe.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if args.launch:
        command = [sys.executable, str(Path(__file__).resolve()), '--exe', str(exe),
                   '--output', str(output), '--phase', args.phase, '--scale', str(args.scale)]
        if args.pages:
            command.extend(['--pages', args.pages])
        with (output / f'{args.phase}-worker.log').open('ab') as log:
            worker = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=log,
                creationflags=subprocess.CREATE_NO_WINDOW)
        progress_files = 'pages-progress.json, states-progress.json' if args.phase == 'all' else args.phase + '-progress.json'
        print(f'Worker PID {worker.pid}; progress in {output}: {progress_files}')
        return
    if args.phase == 'all':
        for phase in ['pages', 'states']:
            command = [sys.executable, str(Path(__file__).resolve()), '--exe', str(exe),
                       '--output', str(output), '--phase', phase, '--scale', str(args.scale)]
            if args.pages:
                command.extend(['--pages', args.pages])
            subprocess.run(command, cwd=ROOT, check=True)
        return
    source = (ROOT / 'examples/component-gallery.json').read_bytes()
    manifest = {'exe_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
                'definition_sha256': hashlib.sha256(source).hexdigest(), 'scale': args.scale}
    package_verification = exe.parent / 'verification.json'
    if package_verification.exists():
        packaged = json.loads(package_verification.read_text(encoding='utf-8'))
        if packaged['source_sha256'] != manifest['definition_sha256']:
            raise RuntimeError('Current gallery definition differs from packaged definition')
    manifest_path = output / 'audit-manifest.json'
    if manifest_path.exists():
        if json.loads(manifest_path.read_text(encoding='utf-8')) != manifest:
            raise RuntimeError('Output contains evidence for a different EXE, definition or scale')
    else:
        write_json(manifest_path, manifest)
    definition = json.loads(source)
    nodes = {n['id']: n for n in walk(definition['root'])}
    pages = [n for n in nodes.values() if n['id'].startswith('doc_')]
    if args.pages:
        pages = [p for p in pages if p['id'][4:] in args.pages.split(',')]
    progress = {'exe': str(exe), 'exe_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
                'phase': args.phase, 'completed': [], 'failed': [], 'total': len(pages),
                'visual_acceptance': 'pending individual image review'}
    progress_path = output / (args.phase + '-progress.json')
    for page in pages:
        kind = page['id'][4:]
        title = nodes[kind + 'Heading']['text']['value']
        try:
            folder = output / 'pages' / kind
            if args.phase == 'pages':
                if (folder / 'render-report.json').exists():
                    report = json.loads((folder / 'render-report.json').read_text(encoding='utf-8'))
                else:
                    steps = navigate(kind, title) + [
                        {'name': 'top', 'pointer': [280, 85], 'wait_ms': 5000 if kind == 'plugin' else 0},
                        scroll('middle', -400, True), scroll('end', -10000, True)]
                    if kind == 'plugin':
                        steps[-1]['wait_ms'] = 5000
                    report = render(exe, folder, steps, args.scale)
                sheet(folder, report, args.scale)
                progress['completed'].append({'type': kind, 'examples': len(nodes[kind + 'Examples']['children']),
                                               'captures': len(report['captures']), 'review': str(folder / 'review.png')})
            else:
                baseline = json.loads((folder / 'render-report.json').read_text(encoding='utf-8'))
                total_captures = 0
                for usage in nodes[kind + 'Examples']['children']:
                    controls = [n for n in walk(usage['children'][0]) if n['type'] in INTERACTIVE
                                or n['type'] == 'text' and n.get('editable')]
                    steps = navigate(kind, title)
                    for index, control in enumerate(controls):
                        candidates = [(c['name'], c.get('targets', {}).get(control['id'])) for c in baseline['captures']]
                        candidates = [(name, r) for name, r in candidates if r is not None]
                        if control['type'] == 'floatButton':
                            placement = 'top'
                        elif not candidates:
                            # Closed overlay children and inactive carousel/tab panels
                            # are reviewed with their parent open-state screenshots.
                            continue
                        else:
                            placement, _ = max(candidates, key=lambda c: (c[1][2] - c[1][0]) * (c[1][3] - c[1][1]))
                        if control['id'].startswith(('formRequired_', 'formItemRequired_')) or control['id'] in {'formValidate', 'formItemValidate'}:
                            placement = 'end'
                        steps.extend(state_steps(control, placement, index))
                    if len(steps) == 6:
                        continue
                    state_folder = output / 'states' / usage['id']
                    if (state_folder / 'render-report.json').exists():
                        report = json.loads((state_folder / 'render-report.json').read_text(encoding='utf-8'))
                    else:
                        report = render(exe, state_folder, steps, args.scale)
                    sheet(state_folder, report, args.scale)
                    total_captures += len(report['captures'])
                progress['completed'].append({'type': kind, 'captures': total_captures})
            print(f'{args.phase}: {kind} completed', flush=True)
        except Exception as error:
            progress['failed'].append({'type': kind, 'error': str(error)})
            print(f'{kind}: {error}', flush=True)
        write_json(progress_path, progress)
    progress['finished'] = True
    write_json(progress_path, progress)


if __name__ == '__main__':
    main()
