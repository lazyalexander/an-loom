"""Optional operator adapter; domain knowledge stays in selected example files."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
NATIVE = HERE.parent/'kimi-native'
spec = importlib.util.spec_from_file_location('native_harness', NATIVE/'qualify.py')
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)
digest, write = native.digest, native.write


def review_inputs(root):
    return {name: digest(root/name) for name in ('materials.md', 'DOMAIN.md', 'acceptance.sh')}


def draft(a):
    root = a.output.resolve()
    root.mkdir(mode=0o700)
    for name in ('materials.md', 'acceptance.sh'):
        shutil.copyfile(HERE/name, root/name)
    kimi, source_loom = a.kimi.resolve(strict=True), a.loom.resolve(strict=True)
    tools = root/'tools'
    tools.mkdir(mode=0o700)
    loom = tools/'an-loom'
    shutil.copy2(source_loom, loom)
    if digest(source_loom) != digest(loom):
        raise RuntimeError('Loom changed during tool snapshot')
    version = subprocess.run([str(kimi), '--version'], capture_output=True, text=True,
                             check=True, timeout=10).stdout.strip()
    if version != '2.3.1':
        raise RuntimeError('Only the existing Kimi 2.3.1 profile is qualified')
    settings = dict(kimi=str(kimi), loom=str(loom), model='kimi-code/k3')
    settings['sha256'] = dict(kimi=digest(kimi), loom=digest(loom),
                              profile=digest(NATIVE/'kimi-profile.md'))
    write(root/'settings.local.json', settings)
    prompt = ('Draft an English DOMAIN.md for a log-summary spool from these public materials. '
              'Cite S1/S2 for every rule, list interfaces, protected/kit paths, acceptance cases, '
              'effects, assumptions and unresolved questions. Preserve exact supplied rules. '
              'Return Markdown only. Do not generate implementation code, use tools, or grant '
              'authority. The developer must review it before coding.\n\n' + (root/'materials.md').read_text())
    (root/'draft-prompt.txt').write_text(prompt)
    write(root/'draft-intent.json', dict(prompt_sha256=digest(root/'draft-prompt.txt'),
          materials_sha256=digest(root/'materials.md'), automatic_retries=0))
    client = root/'draft-client'
    client.mkdir()
    subprocess.run(['git', 'init', '-q', str(client)], check=True, timeout=10)
    (client/'.kimi-code').mkdir()
    write(client/'.kimi-code/mcp.json', {'mcpServers': {}})
    skills = root/'draft-skills'
    skills.mkdir()
    env = os.environ.copy()
    env.update(KIMI_DISABLE_TELEMETRY='1', KIMI_CODE_TRUST_WORKSPACE='1')
    with (root/'draft-response.txt').open('x') as out, (root/'draft-stderr.txt').open('x') as err:
        try:
            result = subprocess.run([str(kimi), '-m', settings['model'], '--agent-file',
                str(NATIVE/'kimi-profile.md'), '--skills-dir', str(skills), '-p', prompt,
                '--output-format', 'text'], cwd=client, env=env, stdout=out, stderr=err, timeout=240)
        except subprocess.TimeoutExpired:
            write(root/'draft-result.json', dict(status='unknown', automatic_retries=0))
            raise RuntimeError('Unknown remote outcome; do not resubmit this directory')
    write(root/'draft-result.json', dict(status='completed' if result.returncode == 0 else 'failed',
                                        automatic_retries=0))
    if result.returncode:
        raise RuntimeError('Draft failed; retained evidence, no retry')
    raw = (root/'draft-response.txt').read_bytes()
    if len(raw) > 65536:
        raise RuntimeError('Draft exceeds review limit')
    text = raw.decode().strip().removeprefix('• ')
    lines = text.splitlines()
    if lines and lines[0] in ('```markdown', '```md') and lines[-1].strip() == '```':
        text = '\n'.join(lines[1:-1])
    (root/'DOMAIN.md').write_text(text+'\n')
    plan = dict(schema='weaver-log-review-v1', inputs=review_inputs(root),
                tools=settings['sha256'], model=settings['model'],
                editable=['kit/transform.sh'], automatic_retries=0)
    write(root/'review-plan.json', plan)
    print(json.dumps(dict(review_sha256=digest(root/'review-plan.json'), draft=str(root/'DOMAIN.md'))))


def review(a):
    root = a.output.resolve()
    if (root/'approval-intent.json').exists():
        raise RuntimeError('Submitted review is immutable; prepare a new task')
    settings = native.settings(root)
    plan = dict(schema='weaver-log-review-v1', inputs=review_inputs(root),
                tools=settings['sha256'], model=settings['model'],
                editable=['kit/transform.sh'], automatic_retries=0)
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
    settings = native.settings(root)
    plan = json.loads((root/'review-plan.json').read_text())
    if plan['inputs'] != review_inputs(root) or plan['tools'] != settings['sha256']:
        raise RuntimeError('Reviewed source/specification/check/tool changed; new review required')
    if approved:
        review = json.loads((root/'reviewed.json').read_text())
        if (review['sha256'] != digest(root/'review-plan.json') or
            review['prompt_sha256'] != digest(root/'prompt.txt') or
            review['generation_plan_sha256'] != digest(root/'generation-plan.json')):
            raise RuntimeError('Approval or dispatch context mismatch')
    return settings


def approve(a):
    root = a.output.resolve()
    settings = checked(root)
    if a.approve != digest(root/'review-plan.json'):
        raise RuntimeError('Explicit specification approval required')
    write(root/'approval-intent.json', dict(sha256=a.approve, automatic_retries=0))
    native.call(settings['loom'], 'init-native', root/'project')
    project = root/'project'
    for name in ('DOMAIN.md', 'materials.md', 'acceptance.sh'):
        shutil.copyfile(root/name, project/name)
    manifest = project/'loom.toml'
    manifest.write_text(manifest.read_text().replace('"AGENTS.md"]',
        '"AGENTS.md", "DOMAIN.md", "materials.md", "acceptance.sh"]'))
    config = project/'native-spool.toml'
    config.write_text(config.read_text().split('acceptance = [')[0] +
        'acceptance = [\'bash acceptance.sh "$PWD" "$AN_LOOM_WORK"\']\n')
    native.call(settings['loom'], 'task', project, root/'task')
    prompt = ('Implement only kit/transform.sh from the approved domain specification. '
              'Return exactly one JSON object with schema an-loom-kit-candidate-v1, '
              'path kit/transform.sh and content containing the complete script. '
              'No tools or delegation. Do not edit rules, silks, checks, permissions. '
              'Reuse awk.\nAPPROVED DOMAIN:\n' + (root/'DOMAIN.md').read_text() +
              '\nSELECTED SOURCES:\n' + (root/'materials.md').read_text() +
              '\nCURRENT KIT:\n' + (project/'kit/transform.sh').read_text())
    (root/'prompt.txt').write_text(prompt)
    write(root/'generation-plan.json', dict(schema='kimi-native-generation-v1', model=settings['model'],
        prompt_sha256=digest(root/'prompt.txt'), profile_sha256=settings['sha256']['profile'],
        kimi_sha256=settings['sha256']['kimi'], tools=[], automatic_retries=0))
    write(root/'reviewed.json', dict(sha256=a.approve,
        prompt_sha256=digest(root/'prompt.txt'),
        generation_plan_sha256=digest(root/'generation-plan.json'),
        authority='local operator; no signature claim'))
    print('Approved task prepared; generation remains a separate explicit stage')


def generate(a):
    checked(a.output.resolve(), approved=True)
    native.generate(SimpleNamespace(output=a.output,
        approve=digest(a.output.resolve()/'generation-plan.json')))


def receive(a):
    checked(a.output.resolve(), approved=True)
    native.receive(a)


def standalone(a):
    root = a.output.resolve()
    settings = checked(root, approved=True)
    write(root/'standalone-intent.json', dict(automatic_retries=0, isolation='operator native authority'))
    native.call(settings['loom'], 'check-task', root/'task')
    snapshot = root/'task/authority/snapshot'
    work = root/'standalone-work'
    work.mkdir()
    subprocess.run(['/bin/bash', str(snapshot/'acceptance.sh'), str(snapshot), str(work)],
                   check=True, timeout=30)
    native.call(settings['loom'], 'check-task', root/'task')
    archive = root/'task/authority/candidate.tar.gz'
    native.call(settings['loom'], 'verify', archive)
    shutil.copyfile(archive, root/'standalone-release.tar.gz')
    native.call(settings['loom'], 'export-native-spool-v2', archive, root/'standalone-spool')
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
          candidate_sha256=digest(root/'candidate.sh'), review_sha256=digest(root/'review-plan.json')))
    print((root/'standalone-result.json').read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['draft', 'review', 'approve', 'generate', 'receive', 'standalone'])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--loom', type=Path)
    parser.add_argument('--kimi', type=Path)
    parser.add_argument('--approve')
    parser.add_argument('--source-sha256')
    args = parser.parse_args()
    globals()[args.stage](args)


if __name__ == '__main__':
    main()
