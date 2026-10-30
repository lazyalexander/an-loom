#!/usr/bin/env python3
"""Operator qualification of a tool-free Kimi proposal and native spool delivery."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def write(path, value):
    with path.open('x') as stream:
        stream.write(json.dumps(value, indent=2) + '\n')


def call(executable, *args):
    p = subprocess.run([str(executable), *map(str, args)], capture_output=True, text=True)
    if p.returncode:
        raise RuntimeError(p.stderr + p.stdout)
    return json.loads(p.stdout)


def settings(root):
    s = json.loads((root / 'settings.local.json').read_text())
    for name, expected in s['sha256'].items():
        path = HERE / 'kimi-profile.md' if name == 'profile' else Path(s[name])
        if digest(path) != expected:
            raise RuntimeError('Selected tool/profile changed: ' + name)
    return s


def prepare(a):
    root = a.output.resolve()
    root.mkdir(mode=0o700)
    s = {name: str(getattr(a, name).resolve()) for name in ('loom', 'host', 'kimi')}
    version = subprocess.run([s['kimi'], '--version'], capture_output=True, text=True, check=True).stdout.strip()
    if version != '2.3.1':
        raise RuntimeError('This profile qualifies Kimi Code 2.3.1 only')
    s.update(model='kimi-code/k3', version=version)
    s['sha256'] = {name: digest(path) for name, path in s.items() if name in ('loom', 'host', 'kimi')}
    s['sha256']['profile'] = digest(HERE / 'kimi-profile.md')
    write(root / 'settings.local.json', s)
    call(s['loom'], 'init-native', root / 'project')
    config = root / 'project/native-spool.toml'
    text = config.read_text()
    start = text.index('acceptance = [')
    check = ('printf \'Beta\\nALPHA\\nbeta\\n\' > "$AN_LOOM_WORK/input"; '
             'bash kit/transform.sh "$AN_LOOM_WORK/input" > "$AN_LOOM_WORK/actual"; '
             'printf \'item:alpha\\nitem:beta\\n\' > "$AN_LOOM_WORK/expected"; '
             'cmp "$AN_LOOM_WORK/actual" "$AN_LOOM_WORK/expected"')
    config.write_text(text[:start] + 'acceptance = ["bash -n kit/transform.sh", ' + json.dumps(check) + ']\n')
    call(s['loom'], 'task', root / 'project', root / 'task')
    source = (root / 'project/kit/transform.sh').read_text()
    prompt = ('Adapt this public native spool kit. It receives an input filename in $1 and writes stdout. '
              'Lowercase ASCII input lines, sort and deduplicate with LC_ALL=C, then prefix each output '
              'line with item:. Prefer existing tr, sort and sed CLI tools. Return only JSON with schema '
              'an-loom-kit-candidate-v1, path kit/transform.sh and content containing the complete shell script. '
              'No tools, commands, file edits or delegation are available to you. Output is an inert proposal. '
              'Current source:\n' + source)
    (root / 'prompt.txt').write_text(prompt)
    write(root / 'generation-plan.json', {'schema': 'kimi-native-generation-v1', 'model': s['model'],
          'prompt_sha256': digest(root / 'prompt.txt'), 'profile_sha256': s['sha256']['profile'],
          'kimi_sha256': s['sha256']['kimi'], 'tools': [], 'automatic_retries': 0,
          'scope': 'trusted official client; no hostile-client sandbox claim'})
    print('Review prompt and profile. Generation approval: ' + digest(root / 'generation-plan.json'))


def generate(a):
    root = a.output.resolve()
    s = settings(root)
    plan = json.loads((root / 'generation-plan.json').read_text())
    if a.approve != digest(root / 'generation-plan.json') or digest(root / 'prompt.txt') != plan['prompt_sha256']:
        raise RuntimeError('Generation approval or prompt mismatch')
    # Exclusive intent fences every outcome, including launch error and timeout.
    write(root / 'generation-intent.json', plan)
    workspace = root / 'client'
    workspace.mkdir(mode=0o700)
    subprocess.run(['git', 'init', '-q', str(workspace)], check=True)
    (workspace / '.kimi-code').mkdir()
    write(workspace / '.kimi-code/mcp.json', {'mcpServers': {}})
    empty = root / 'empty-skills'
    empty.mkdir()
    env = os.environ.copy()
    env.update(KIMI_DISABLE_TELEMETRY='1', KIMI_CODE_TRUST_WORKSPACE='1')
    args = [s['kimi'], '-m', s['model'], '--agent-file', str(HERE / 'kimi-profile.md'),
            '--skills-dir', str(empty), '-p', (root / 'prompt.txt').read_text(), '--output-format', 'text']
    with (root / 'response.txt').open('x') as out, (root / 'client.stderr').open('x') as err:
        try:
            p = subprocess.run(args, cwd=workspace, env=env, stdout=out, stderr=err, timeout=240)
            outcome = {'status': 'completed' if p.returncode == 0 else 'failed', 'client_exit': p.returncode,
                       'model': s['model'], 'automatic_retries': 0}
        except subprocess.TimeoutExpired:
            write(root / 'generation-result.json', {'status': 'unknown', 'automatic_retries': 0,
                  'remote_termination_confirmed': False})
            raise RuntimeError('Client timeout; inspect retained evidence, do not replay')
    write(root / 'generation-result.json', outcome)
    if outcome['status'] != 'completed':
        raise RuntimeError('Client failed; inspect evidence, do not replay')
    extract(a)


def extract(a):
    root = a.output.resolve()
    settings(root)
    if json.loads((root / 'generation-result.json').read_text())['status'] != 'completed':
        raise RuntimeError('No completed generation; inspect evidence, do not replay')
    candidate = parse_candidate((root / 'response.txt').read_bytes())
    with (root / 'candidate.sh').open('x') as stream:
        stream.write(candidate['content'])
    print('Review candidate.sh. Receive with --source-sha256 ' + digest(root / 'candidate.sh'))


def parse_candidate(raw):
    if len(raw) > 65536:
        raise ValueError('Response exceeds limit')
    text = raw.decode().strip()
    # Accept only known render wrappers; never scrape arbitrary prose.
    if text.startswith('• '):
        text = text[2:]
    lines = text.splitlines()
    if len(lines) >= 3 and lines[0] == '```json' and lines[-1].strip() == '```':
        text = '\n'.join(lines[1:-1])
    obj = json.loads(text)
    if not isinstance(obj, dict) or set(obj) != {'schema', 'path', 'content'}:
        raise ValueError('Expected one closed candidate object')
    if obj['schema'] != 'an-loom-kit-candidate-v1' or obj['path'] != 'kit/transform.sh':
        raise ValueError('Candidate scope mismatch')
    if not isinstance(obj['content'], str) or not obj['content'] or len(obj['content'].encode()) > 65536:
        raise ValueError('Invalid candidate text')
    return obj


def receive(a):
    root = a.output.resolve()
    s = settings(root)
    if json.loads((root / 'generation-result.json').read_text())['status'] != 'completed':
        raise RuntimeError('No completed generation')
    candidate = parse_candidate((root / 'response.txt').read_bytes())
    if (root / 'candidate.sh').read_text() != candidate['content'] or a.source_sha256 != digest(root / 'candidate.sh'):
        raise RuntimeError('Reviewed candidate mismatch')
    call(s['loom'], 'receive-task', root / 'task', candidate['path'], root / 'candidate.sh')
    print('Candidate received. Use develop-native preview and approval; no code executed by reception.')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('stage', choices=['prepare', 'generate', 'extract', 'receive'])
    p.add_argument('--output', type=Path, required=True)
    for name in ('loom', 'host', 'kimi'):
        p.add_argument('--' + name, type=Path)
    p.add_argument('--approve')
    p.add_argument('--source-sha256')
    a = p.parse_args()
    if a.stage == 'prepare' and any(getattr(a, key) is None for key in ('loom', 'host', 'kimi')):
        p.error('prepare requires --loom, --host and --kimi')
    globals()[a.stage](a)


if __name__ == '__main__':
    main()
