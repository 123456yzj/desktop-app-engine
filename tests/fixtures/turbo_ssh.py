"""SSH transport double: execute the production Bash/Node code against disposable local repos."""
import os
import shlex
import subprocess
import sys
from pathlib import Path

root = Path.cwd()
remote = shlex.split(sys.argv[-1])
assert remote[:2] == ['bash', '-lc']
assert sys.argv[-2] == 'fixture-host', sys.argv
functions = r'''
rm() {
    printf 'clean\n' >> .clean-invocations
    if [ -f .fail-clean ]; then return 13; fi
    command rm "$@"
}
pnpm() {
    printf '%s\n' "$1" >> .build-invocations
    printf 'OUTPUT-BEGIN %s\n' "$1"
    sleep 0.12
    if [ -f .fail-build ] && [ "$(cat .fail-build)" = "$1" ]; then
        printf 'BUILD-ERROR %s\n' "$1" >&2
        return 9
    fi
    mkdir -p .turbo
    printf 'new-cache\n' > .turbo/generated
    printf 'OUTPUT-END %s\n' "$1"
}
'''
environment = {**os.environ, 'HOME': root.as_posix(), 'MSYS_NO_PATHCONV': '1'}
result = subprocess.run([r'C:\Program Files\Git\bin\bash.exe', '--noprofile', '--norc', '-lc', functions + remote[2]], env=environment)
sys.exit(result.returncode)
