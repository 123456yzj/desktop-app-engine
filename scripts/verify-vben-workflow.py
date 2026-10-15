"""Exercise the embedded PowerShell runner and actual Bash login-shell exit behavior.

Uses mocked SSH/Git/pnpm only. No remote connections or native GUI interaction.
"""
import json
import os
import shlex
import subprocess
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
state = json.loads((root / 'examples/vben-package-legacy.json').read_text(encoding='utf-8'))['state']
bash = r'C:\Program Files\Git\bin\bash.exe'
remotes = {}


def execute(operation, lines, code=0, target='demo-host', branch='feature/tms', prior=None):
    outputs = '; '.join("Write-Output '" + line.replace("'", "''") + "'" for line in lines)
    mock = "function ssh.exe { [Console]::Error.WriteLine('__ARGS__'+(ConvertTo-Json -InputObject @($args) -Compress)); " + outputs + f'; $global:LASTEXITCODE={code}' + ' }; '
    previous = prior or {**state['view'], 'currentBranch': 'dev', 'project': 'demo-host /tmp/repo', 'branches': 'dev\nfeature/tms'}
    result = subprocess.run(['pwsh.exe', '-NoLogo', '-NoProfile', '-NonInteractive', '-CommandWithArgs', mock + state['commandScript'], state['runner'], operation, target, '/tmp/repo', branch, json.dumps(previous, ensure_ascii=False)], capture_output=True, text=True, encoding='utf-8', timeout=20)
    assert result.returncode == (code if target else 2), (result.stdout, result.stderr)
    response = json.loads(result.stdout)
    assert set(response) == set(state['view']), response
    args_line = next((line for line in result.stderr.splitlines() if line.startswith('__ARGS__')), None)
    if args_line:
        args = json.loads(args_line[len('__ARGS__'):])
        command = shlex.split(args[-1])
        assert command[:2] == ['bash', '-lc']
        remote = command[2]
        remotes[operation] = remote
        syntax = subprocess.run([bash, '-n', '-c', remote], capture_output=True, timeout=10)
        assert syntax.returncode == 0, syntax.stderr
        assert 'status --porcelain' not in remote
        if operation == 'switch':
            assert 'git checkout ' in remote and 'git switch ' not in remote
        if operation == 'pull':
            assert 'git pull;' in remote and '--ff-only' not in remote
        if operation == 'build':
            assert 'pnpm build:dev:tms;' in remote
            assert remote.index('exit 4;') < remote.index('rm -rf -- ./.turbo;') < remote.index('pnpm build:dev:tms;')
        assert f'[vben] operation={operation} exit={code}' in result.stderr
    return response


r = execute('refresh', ['__VBEN_BRANCH__dev', '__VBEN_BRANCH__feature/tms', '__VBEN_CURRENT__dev'])
assert r['currentBranch'] == 'dev' and r['branches'] == 'dev\nfeature/tms' and r['ready'] and r['hasBranches'] and r['success']
r = execute('switch', ['Switched to branch feature/tms', '__VBEN_CURRENT__feature/tms'])
assert r['currentBranch'] == 'feature/tms' and r['success']
r = execute('switch', ['error: local changes would be overwritten', '__VBEN_CURRENT__dev'], 1)
assert r['currentBranch'] == 'dev' and r['failure'] and not r['success'] and '冲突' in r['detail']
r = execute('pull', ['no such ref was fetched', '__VBEN_CURRENT__dev'], 1)
assert r['failure'] and '跟踪分支不存在' in r['detail']
r = execute('build', ['Build complete', '__VBEN_CURRENT__dev'])
assert r['buildResult'] == '打包成功' and r['buildBranch'] == 'dev' and r['buildTime'] != '—'
r = execute('build', ['pnpm failed', '__VBEN_CURRENT__dev'], 2)
assert r['buildResult'] == '打包失败' and r['failure']
r = execute('refresh', ['Permission denied'], 255)
assert not r['ready'] and not r['hasBranches'] and r['currentBranch'] == '尚未读取'
r = execute('refresh', [], target='')
assert r['failure'] and 'SSH' in r['detail']
r = execute('refresh', ['__VBEN_BRANCH__dev', '__VBEN_CURRENT__'])
assert not r['ready'] and r['hasBranches'] and '游离' in r['currentBranch']
r = execute('refresh', [], 7)
assert r['failure'] and '退出码 7' in r['detail'] and '实时日志' in r['detail']

# Run the generated remote script in a real login Bash with a failing logout.
# HOME is isolated to this disposable subprocess, never the user's shell.
fake_commands = r'''
git() {
    case "$1" in
        rev-parse) return 0 ;;
        symbolic-ref) printf '%s\n' "${TEST_CURRENT_BRANCH:-dev}" ;;
        fetch) return "${TEST_FETCH_CODE:-0}" ;;
        for-each-ref) if [ "${TEST_LIST_CODE:-0}" != 0 ]; then return "$TEST_LIST_CODE"; fi; printf 'dev\nfeature/tms\n' ;;
        check-ref-format|checkout|pull) return 0 ;;
        *) return 99 ;;
    esac
}
rm() {
    if [ "${TEST_CLEAN_CODE:-0}" != 0 ]; then return "$TEST_CLEAN_CODE"; fi
    command rm "$@"
}
pnpm() {
    if [ -e ./.turbo ]; then return 96; fi
    printf 'called\n' > .build-called
    return "${TEST_BUILD_CODE:-0}"
}
'''
with tempfile.TemporaryDirectory(prefix='vben-logout-') as directory:
    sandbox = Path(directory)
    (sandbox / '.bash_logout').write_text('false\n', encoding='utf-8')

    def login(operation, *, original=False, missing=False, **overrides):
        (sandbox / '.build-called').unlink(missing_ok=True)
        remote = remotes[operation].replace("'/tmp/repo'", shlex.quote(sandbox.as_posix() + ('/missing' if missing else '')))
        if original:
            remote = remote.replace('rc=$?; set +e;', 'rc=$?;')
        env = {**os.environ, 'HOME': sandbox.as_posix(), **overrides}
        return subprocess.run([bash, '--noprofile', '--norc', '-lc', fake_commands + remote], env=env, capture_output=True, text=True, encoding='utf-8', timeout=10)

    before = login('refresh', original=True)
    assert before.returncode == 1 and '__VBEN_BRANCH__dev' in before.stdout, before
    after = login('refresh')
    assert after.returncode == 0 and '__VBEN_BRANCH__dev' in after.stdout and '__VBEN_CURRENT__dev' in after.stdout, after
    assert '[vben] stage=branches exit=0' in after.stderr, after.stderr
    failed = login('refresh', TEST_FETCH_CODE='7')
    assert failed.returncode == 7 and '[vben] stage=fetch exit=7' in failed.stderr, failed
    assert '__VBEN_BRANCH__' not in failed.stdout and '__VBEN_CURRENT__dev' in failed.stdout
    failed = login('refresh', TEST_LIST_CODE='8')
    assert failed.returncode == 8 and '[vben] stage=branches exit=8' in failed.stderr, failed
    failed = login('build', TEST_BUILD_CODE='9')
    assert failed.returncode == 9 and '[vben] stage=build exit=9' in failed.stderr, failed
    failed = login('refresh', missing=True)
    assert failed.returncode != 0 and '[vben] stage=repository exit=' in failed.stderr, failed
    assert '__VBEN_CURRENT__' not in failed.stdout

    turbo = sandbox / '.turbo'
    turbo.mkdir()
    (turbo / 'cache').write_text('stale', encoding='utf-8')
    (sandbox / 'keep.txt').write_text('keep', encoding='utf-8')
    cleaned = login('build')
    assert cleaned.returncode == 0 and not turbo.exists(), cleaned
    assert (sandbox / '.build-called').exists() and (sandbox / 'keep.txt').read_text() == 'keep'
    assert '[vben] cleaning project .turbo' in cleaned.stderr
    absent = login('build')
    assert absent.returncode == 0 and (sandbox / '.build-called').exists(), absent
    turbo.mkdir()
    (turbo / 'cache').write_text('stale', encoding='utf-8')
    failed = login('build', TEST_CLEAN_CODE='13')
    assert failed.returncode == 13 and '[vben] stage=clean-turbo exit=13' in failed.stderr, failed
    assert turbo.exists() and not (sandbox / '.build-called').exists()
    changed = login('build', TEST_CURRENT_BRANCH='other-branch')
    assert changed.returncode == 4 and '[vben] stage=branch-check exit=4' in changed.stderr, changed
    assert (turbo / 'cache').read_text() == 'stale' and not (sandbox / '.build-called').exists()
    assert '[vben] cleaning' not in changed.stderr

print('PASS 10 mocked SSH workflows and 10 actual Bash checks including pre-build .turbo cleanup, missing cache, cleanup failure and branch mismatch. No real SSH or native GUI.')
