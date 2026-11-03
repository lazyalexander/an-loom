"""Review and receive domain work; the operator's harness owns generation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
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
    result = subprocess.run([str(executable), *map(str, args)], capture_output=True,
                            text=True, check=True, timeout=30)
    return json.loads(result.stdout)


def settings(root):
    selected = json.loads((root/'settings.local.json').read_text())
    if set(selected) != {'loom', 'loom_sha256'}:
        raise RuntimeError('Legacy harness-owned review; use its original adapter or prepare a new directory')
    if digest(selected['loom']) != selected['loom_sha256']:
        raise RuntimeError('Selected Loom binary changed')
    return selected


def review_inputs(root):
    return {name: digest(root/name) for name in ('materials.md', 'DOMAIN.md', 'acceptance.sh')}


def prepare(a):
    root = a.output.resolve()
    root.mkdir(mode=0o700)
    for name in ('materials.md', 'acceptance.sh'):
        shutil.copyfile(HERE/name, root/name)
    source = a.loom.resolve(strict=True)
    tools = root/'tools'
    tools.mkdir()
    loom = tools/'an-loom'
    shutil.copy2(source, loom)
    if digest(source) != digest(loom):
        raise RuntimeError('Loom changed during tool snapshot')
    write(root/'settings.local.json', dict(loom=str(loom), loom_sha256=digest(loom)))
    prompt = ('Draft an English DOMAIN.md for a log-summary spool from these public materials. '
              'Cite S1/S2 for every rule, list interfaces, protected/kit paths, acceptance cases, '
              'effects, assumptions and unresolved questions. Preserve exact supplied rules. '
              'Return Markdown only. Do not generate implementation code, use tools, or grant '
              'authority. The developer must review it before coding.\n\n' + (root/'materials.md').read_text())
    (root/'draft-prompt.txt').write_text(prompt)
    (root/'client').mkdir()
    print('Use your native harness to draft DOMAIN.md from draft-prompt.txt, then review it.')


def review(a):
    root = a.output.resolve()
    if (root/'approval-intent.json').exists():
        raise RuntimeError('Submitted review is immutable; prepare a new task')
    selected = settings(root)
    plan = dict(schema='weaver-log-review-v2', inputs=review_inputs(root),
                loom_sha256=selected['loom_sha256'], editable=['kit/transform.sh'])
    history = root/'review-history'
    history.mkdir(exist_ok=True)
    prior = root/'review-plan.json'
    if prior.exists():
        archived = history/(digest(prior)+'.json')
        if not archived.exists():
            shutil.copyfile(prior, archived)
    prior.write_text(json.dumps(plan, indent=2)+'\n')
    print(json.dumps(dict(review_sha256=digest(prior), draft=str(root/'DOMAIN.md'))))


def checked(root, approved=False):
    selected = settings(root)
    plan = json.loads((root/'review-plan.json').read_text())
    if (plan.get('schema') != 'weaver-log-review-v2' or plan['inputs'] != review_inputs(root)
            or plan['loom_sha256'] != selected['loom_sha256']):
        raise RuntimeError('Reviewed source/specification/check/tool changed; new review required')
    if approved:
        review = json.loads((root/'reviewed.json').read_text())
        if (review['sha256'] != digest(root/'review-plan.json') or
            review['prompt_sha256'] != digest(root/'prompt.txt') or
            review['task_sha256'] != digest(root/'task/authority/task.json')):
            raise RuntimeError('Approval or task context mismatch')
    return selected


def approve(a):
    root = a.output.resolve()
    selected = checked(root)
    if a.approve != digest(root/'review-plan.json'):
        raise RuntimeError('Explicit specification approval required')
    write(root/'approval-intent.json', dict(sha256=a.approve, automatic_retries=0))
    call(selected['loom'], 'init-native', root/'project')
    project = root/'project'
    for name in ('DOMAIN.md', 'materials.md', 'acceptance.sh'):
        shutil.copyfile(root/name, project/name)
    manifest = project/'loom.toml'
    manifest.write_text(manifest.read_text().replace('"AGENTS.md"]',
        '"AGENTS.md", "DOMAIN.md", "materials.md", "acceptance.sh"]'))
    config = project/'native-spool.toml'
    config.write_text(config.read_text().split('acceptance = [')[0] +
        'acceptance = [\'bash acceptance.sh "$PWD" "$AN_LOOM_WORK"\']\n')
    call(selected['loom'], 'task', project, root/'task')
    prompt = ('Implement only kit/transform.sh from the approved domain specification. '
              'Return the complete shell script only, without Markdown fences. '
              'Do not use tools, execute commands or delegate. Do not edit rules, silks, '
              'checks or permissions. Reuse awk. This is an inert proposal; receiving it '
              'and executing checks are separate operator actions.\nAPPROVED DOMAIN:\n' +
              (root/'DOMAIN.md').read_text() + '\nSELECTED SOURCES:\n' +
              (root/'materials.md').read_text() + '\nCURRENT KIT:\n' +
              (project/'kit/transform.sh').read_text())
    (root/'prompt.txt').write_text(prompt)
    write(root/'reviewed.json', dict(sha256=a.approve, prompt_sha256=digest(root/'prompt.txt'),
          task_sha256=digest(root/'task/authority/task.json'),
          authority='local operator; no signature claim'))
    print('Task prepared. Use your native harness with prompt.txt; review its candidate.sh separately.')


def receive(a):
    root = a.output.resolve()
    selected = checked(root, approved=True)
    if a.source_sha256 != digest(root/'candidate.sh'):
        raise RuntimeError('Reviewed candidate mismatch')
    call(selected['loom'], 'receive-task', root/'task', 'kit/transform.sh', root/'candidate.sh')
    print('Candidate received as untrusted bytes; no model completion or execution claim.')


def standalone(a):
    root = a.output.resolve()
    selected = checked(root, approved=True)
    write(root/'standalone-intent.json', dict(automatic_retries=0, isolation='operator native authority'))
    call(selected['loom'], 'check-task', root/'task')
    snapshot = root/'task/authority/snapshot'
    work = root/'standalone-work'
    work.mkdir()
    subprocess.run(['/bin/bash', str(snapshot/'acceptance.sh'), str(snapshot), str(work)],
                   check=True, timeout=30)
    call(selected['loom'], 'check-task', root/'task')
    archive = root/'task/authority/candidate.tar.gz'
    call(selected['loom'], 'verify', archive)
    shutil.copyfile(archive, root/'standalone-release.tar.gz')
    call(selected['loom'], 'export-native-spool-v2', archive, root/'standalone-spool')
    (work/'input').write_text('INFO first\nERROR second\n')
    env = os.environ.copy()
    env.update(AN_SPOOL_INPUT=str(work/'input'),
               AN_SPOOL_KIT_DIR=str(root/'standalone-spool/kit'),
               AN_SPOOL_SILK_DIR=str(root/'standalone-spool/silk'))
    subprocess.run(['/bin/bash', str(root/'standalone-spool/silk/main.sh')],
                   cwd=work, env=env, check=True, timeout=15)
    expected = 'Report\ntotal=2\nINFO=1\nWARN=0\nERROR=1\n'
    if (work/'report.txt').read_text() != expected:
        raise RuntimeError('Standalone output mismatch')
    write(root/'standalone-result.json', dict(passed=True, host_invoked=False,
          host_acceptance=False, archive_sha256=digest(archive),
          candidate_sha256=digest(snapshot/'kit/transform.sh'), review_sha256=digest(root/'review-plan.json')))
    print((root/'standalone-result.json').read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['prepare', 'review', 'approve', 'receive', 'standalone'])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--loom', type=Path)
    parser.add_argument('--approve')
    parser.add_argument('--source-sha256')
    args = parser.parse_args()
    if args.stage == 'prepare' and args.loom is None:
        parser.error('prepare requires --loom')
    globals()[args.stage](args)


if __name__ == '__main__':
    main()
