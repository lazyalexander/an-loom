"""Credential-free standalone acceptance with externally supplied candidate bytes."""
import argparse
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
    w.prepare(SimpleNamespace(output=root, loom=args.loom))
    (root/'DOMAIN.md').write_text('Synthetic test specification: follow materials.md.\n')
    settings = w.settings(root)
    w.review(SimpleNamespace(output=root))
    w.approve(SimpleNamespace(output=root, approve=w.digest(root/'review-plan.json')))
    shutil.copyfile(w.HERE/'reviewed-example/kit/transform.sh', root/'candidate.sh')
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
    print('Standalone acceptance passed: synthetic review and external bytes; no host, model or credentials')


if __name__ == '__main__':
    main()
