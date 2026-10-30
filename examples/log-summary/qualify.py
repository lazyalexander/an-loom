"""Trusted operator acceptance; no model invocation and no automatic retry."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--loom', type=Path, required=True)
    parser.add_argument('--host', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--execute', action='store_true', required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(mode=0o700)  # Existing/unknown attempts cannot be resubmitted.
    loom, host = args.loom.resolve(strict=True), args.host.resolve(strict=True)
    events = []

    def call(binary, *argv, expected=0):
        result = subprocess.run([str(binary), *map(str, argv)], capture_output=True,
                                text=True, timeout=90)
        events.append(dict(command=str(argv[0]), code=result.returncode,
                           stdout=result.stdout, stderr=result.stderr))
        (root/'commands.local.json').write_text(json.dumps(events, indent=2))
        if result.returncode != expected:
            raise RuntimeError(result.stderr + result.stdout)
        return json.loads(result.stdout) if result.stdout.strip() else None

    def approved(binary, *argv):
        preview = call(binary, *argv)
        return call(binary, *argv, '--approve', preview['approval_sha256'])

    project, task = root/'project', root/'task'
    call(loom, 'init-native', project)
    config = project/'native-spool.toml'
    text = config.read_text()
    check = ('w="$AN_LOOM_WORK"; printf \'INFO start\\nERROR failed\\nINFO done\\n\' > "$w/i"; '
             'bash kit/transform.sh "$w/i" > "$w/a"; '
             'printf \'total=3\\nINFO=2\\nWARN=0\\nERROR=1\\n\' > "$w/e"; '
             'cmp "$w/a" "$w/e"')
    config.write_text(text[:text.index('acceptance = [')] + 'acceptance = [' +
                      json.dumps('bash -n kit/transform.sh') + ', ' + json.dumps(check) + ']\n')
    call(loom, 'task', project, task)
    candidate = root/'candidate.sh'
    shutil.copyfile(Path(__file__).with_name('transform.sh'), candidate)
    call(loom, 'receive-task', task, 'kit/transform.sh', candidate)
    approved(loom, 'develop-native', task, host, root/'check-job', root/'delivery')
    instance = root/'delivery/instance'
    source = root/'events.log'
    reports = []
    for name, text, expected in [
        ('first', 'INFO start\nERROR failed\nINFO done\n', 'Report\ntotal=3\nINFO=2\nWARN=0\nERROR=1\n'),
        ('second', 'WARN changed\n', 'Report\ntotal=1\nINFO=0\nWARN=1\nERROR=0\n')]:
        source.write_text(text)
        run = approved(host, 'spool-start', instance, name, source, '15')
        assert run['status'] == 'succeeded'
        assert run['workspace']['termination_confirmed'] is True
        approved(host, 'spool-accept', instance, name)
        binding = call(host, 'spool-artifacts', instance, name)
        target = root/(name+'.txt')
        revision = binding['outputs'][0]['binding']['revision']
        call(host, 'artifact-checkout', binding['catalog'], revision, target)
        assert target.read_text() == expected
        reports.append((name, binding, expected))
        approved(host, 'spool-reclaim', instance, name)
        assert not (instance/'runs'/name/'job/work').exists()
    assert reports[0][1]['outputs'] != reports[1][1]['outputs']
    source.write_text('UNKNOWN rejected\n')
    preview = call(host, 'spool-start', instance, 'invalid', source, '15')
    failed = call(host, 'spool-start', instance, 'invalid', source, '15',
                  '--approve', preview['approval_sha256'], expected=1)
    assert failed['status'] == 'failed'
    assert failed['workspace']['termination_confirmed'] is True
    call(host, 'spool-accept', instance, 'invalid', expected=1)
    call(host, 'spool-artifacts', instance, 'invalid', expected=1)
    assert not (instance/'runs/invalid/accepted.json').exists()
    assert not (instance/'runs/invalid/job/artifacts').exists()
    approved(host, 'spool-reclaim', instance, 'invalid')
    assert not (instance/'runs/invalid/job/work').exists()
    approved(host, 'spool-unload', instance)
    for name, binding, expected in reports:
        target = root/(name+'-retained.txt')
        call(host, 'artifact-checkout', binding['catalog'],
             binding['outputs'][0]['binding']['revision'], target)
        assert target.read_text() == expected
    summary = dict(schema='log-summary-qualification-v1', passed=True, model_invoked=False,
                   runs=3, accepted_runs=2, failed_run_rejected=True, fixed_versions_survive_input_edit_reclaim_unload=True,
                   candidate_sha256=hashlib.sha256(candidate.read_bytes()).hexdigest())
    (root/'acceptance.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
