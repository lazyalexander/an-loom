"""Credential-free standalone acceptance; simulated review/generator, no model."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
from types import SimpleNamespace
import weaver as w


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--loom', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.parent.mkdir(parents=True, exist_ok=True)
    root.mkdir()
    for name in ('materials.md', 'acceptance.sh'):
        shutil.copyfile(w.HERE/name, root/name)
    (root/'DOMAIN.md').write_text('Synthetic test specification: follow materials.md.\n')
    fake = root/'fake-client'
    fake.write_text('Fixture only: no agent or model invocation')
    settings = dict(loom=str(args.loom.resolve(strict=True)), kimi=str(fake), model='fixture-no-model')
    settings['sha256'] = dict(loom=w.digest(settings['loom']), kimi=w.digest(fake),
                              profile=w.digest(w.NATIVE/'kimi-profile.md'))
    w.write(root/'settings.local.json', settings)
    w.review(SimpleNamespace(output=root))
    w.approve(SimpleNamespace(output=root, approve=w.digest(root/'review-plan.json')))
    code = (w.HERE.parent/'log-summary/transform.sh').read_text()
    w.write(root/'response.txt', dict(schema='an-loom-kit-candidate-v1', path='kit/transform.sh', content=code))
    (root/'candidate.sh').write_text(code)
    w.write(root/'generation-result.json', dict(status='completed', model_invoked=False))
    for forbidden in ('DOMAIN.md', 'acceptance.sh', 'silk/report.sh'):
        result = subprocess.run([settings['loom'], 'receive-task', str(root/'task'),
            forbidden, str(root/'candidate.sh')], capture_output=True, timeout=10)
        if result.returncode == 0:
            raise RuntimeError('Protected path was accepted: '+forbidden)
    w.receive(SimpleNamespace(output=root, source_sha256=w.digest(root/'candidate.sh')))
    w.standalone(SimpleNamespace(output=root))
    result = subprocess.run([settings['loom'], 'pack-task', str(root/'task'),
        str(root/'not-a-host-release')], capture_output=True, timeout=10)
    if result.returncode == 0 or (root/'not-a-host-release').exists():
        raise RuntimeError('Standalone success was accepted as host evidence')
    print('Standalone acceptance passed: synthetic review/generator; no host, model or credentials')


if __name__ == '__main__':
    main()
