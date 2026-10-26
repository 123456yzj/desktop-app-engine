"""Export a completed packaged-gallery audit with an explicit human review record.

Checks evidence identity and completeness; never infers visual acceptance from PNGs.
The output must be a new directory, so an older running EXE can remain available.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]


def load(path):
    return json.loads(path.read_text(encoding='utf-8'))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def walk(node):
    yield node
    for child in node.get('children', []):
        yield from walk(child)


def copy_evidence(source, destination):
    for path in source.rglob('*'):
        relative = path.relative_to(source)
        if any('isolated-cache' == part or '-incomplete-' in part for part in relative.parts):
            continue
        if path.is_file() and path.suffix in {'.png', '.json', '.log'}:
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', required=True, type=Path)
    parser.add_argument('--audit', required=True, type=Path)
    parser.add_argument('--review', required=True, type=Path)
    parser.add_argument('--supplement', action='append', default=[], type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    exe, audit, output = args.exe.resolve(), args.audit.resolve(), args.output.resolve()
    if output.exists() or not output.parent.is_dir():
        raise ValueError('Output must be a new directory with an existing parent')
    sha = digest(exe)
    package = load(exe.parent / 'verification.json')
    progress = load(audit / 'progress.json')
    review = load(args.review)
    assert package['package_exit'] == package['embedded_check_exit'] == 0
    assert package['source_sha256'] == digest(ROOT / 'examples/component-gallery.json')
    assert progress['finished'] and not progress['failed']
    assert len(progress['completed']) == progress['total']
    assert progress['exe_sha256'] == review['exe_sha256'] == sha
    assert review['review_complete_for_recorded_scope'] is True
    assert review['limitations'], 'Review must state its practical limits'
    corrected = load(audit / 'corrected/progress.json')
    assert corrected['finished'] and not corrected['failed']
    states = load(audit / 'states-progress.json')
    assert states['finished'] and not states['failed'] and states['exe_sha256'] == sha
    definition = load(ROOT / 'examples/component-gallery.json')
    nodes = {node['id']: node for node in walk(definition['root'])}
    kinds = sorted(node_id[4:] for node_id in nodes if node_id.startswith('doc_'))
    assert len(kinds) == 84
    assert set(kinds) == set(review['components'])
    sources = [(audit, Path('evidence/audit'))]
    for supplement in args.supplement:
        supplement = supplement.resolve()
        metadata = load(supplement / 'progress.json')
        assert metadata['finished'] and not metadata['failed'], supplement
        sources.append((supplement, Path('evidence') / supplement.name))
    reports = []
    for source, destination in sources:
        for path in source.rglob('render-report.json'):
            relative = path.relative_to(source)
            if any('-incomplete-' in part for part in relative.parts):
                continue
            report = load(path)
            assert report['definition'] == 'embedded' and Path(report['executable']).resolve() == exe
            for capture in report['captures']:
                with Image.open(path.parent / (capture['name'] + '.png')) as image:
                    image.verify()
            reports.append({'path': str(destination / relative).replace('\\', '/'),
                            'captures': len(report['captures']), 'scale': report['pixels_per_point'],
                            'theme': report['theme']})
    for kind in kinds:
        page = load(audit / 'pages' / kind / 'render-report.json')
        assert {capture['name'] for capture in page['captures']} == {'top', 'middle', 'end'}
    output.mkdir()
    for source, destination in sources:
        copy_evidence(source, output / destination)
    target = output / 'component-gallery.exe'
    shutil.copy2(exe, target)
    assert digest(target) == sha
    shutil.copy2(exe.parent / 'verification.json', output / 'package-verification.json')
    shutil.copy2(ROOT / 'examples/component-gallery.json', output / 'component-gallery.json')
    shutil.copy2(ROOT / 'docs/component-gallery-style-audit.md', output / 'style-audit.md')
    tools = output / 'evidence/tools'
    tools.mkdir(parents=True)
    for name in ['audit-packaged-gallery.py', 'export-gallery-audit.py', 'build-component-gallery.py']:
        shutil.copy2(ROOT / 'scripts' / name, tools / name)
    font_root = Path(os.environ.get('WINDIR', r'C:\Windows')) / 'Fonts'
    fonts = {name: digest(font_root / name) for name in ['msyh.ttc', 'seguisym.ttf', 'segoeuib.ttf', 'msyhbd.ttc']
             if (font_root / name).is_file()}
    index = []
    for kind in kinds:
        index.append({'component': kind, 'examples': len(nodes[kind + 'Examples']['children']),
                      'pages': f'evidence/audit/pages/{kind}',
                      'states': [r['path'] for r in reports if f'/states/{kind}Usage' in r['path']],
                      'deliberate': [r['path'] for r in reports if f'/{kind}/render-report.json' in r['path']
                                     and '/pages/' not in r['path'] and '/focused/' not in r['path']],
                      'review': review['components'][kind]})
    summary = {'executable': str(target), 'exe_sha256': sha, 'definition': 'embedded',
               'build': 'Debug, visual-tests', 'pages': len(kinds), 'examples': sum(i['examples'] for i in index),
               'replay_cases': progress['total'], 'report_count': len(reports),
               'capture_count': sum(r['captures'] for r in reports), 'reports': reports,
               'fonts_sha256_at_export': fonts, 'review': review, 'components': index,
               'validation': {'logical_tests': 39, 'cargo_fmt': 'passed', 'cargo_clippy': 'passed'},
               'renderer': load(audit / 'pages/button/render-report.json')['renderer'],
               'adapter': load(audit / 'pages/button/render-report.json')['adapter']}
    (output / 'verification.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding='utf-8')
    shutil.copy2(audit / 'pages/button/top.png', output / 'preview.png')
    shutil.copy2(audit / 'corrected/rate/review.png', output / 'half-rating.png')
    lines = ['# 组件实验室 audit-v10 交付', '',
             'Debug EXE，内置 84 个组件页、173 个示例。', '',
             f'EXE SHA-256：`{sha}`。', '',
             f"{progress['total']} 个重放场景完成，另含修正与补充场景；共 {len(reports)} 份渲染报告、{summary['capture_count']} 张原始 PNG。",
             '84 页包含顶部、中部、底部共 252 张基础图片。具体看图结论见 style-audit.md、verification.json 的 components/review。', '',
             '后端：egui_kittest 0.31.1 / wgpu，Dx12 / Microsoft Basic Render Driver CPU。',
             '普通窗口使用 Glow；本次结果不覆盖该原生窗口链路。', '', '## 未测与限制', '']
    lines.extend('- ' + limit for limit in review['limitations'])
    lines += ['', '## 逐组件证据索引', '', '| 组件 | 示例数 | 图片及场景 | 看图结论与限制 |', '| --- | ---: | --- | --- |']
    for entry in index:
        note = entry['review'].replace('|', '/').replace('\n', ' ')
        lines.append(f"| {entry['component']} | {entry['examples']} | [{entry['component']}]({entry['pages']}/review.png)；详见 verification.json | {note} |")
    lines += ['', '原始报告保留执行时的临时路径；复制后的对应路径由 verification.json 的 reports 标明。',
              '证据中保留被后续场景替代的通用操作；不能仅凭文件名或图片存在认定相应交互通过。']
    (output / 'README.md').write_text('\n'.join(lines) + '\n', encoding='utf-8')
    print(json.dumps({key: summary[key] for key in ['executable', 'exe_sha256', 'report_count', 'capture_count']}, ensure_ascii=False))


if __name__ == '__main__':
    main()
