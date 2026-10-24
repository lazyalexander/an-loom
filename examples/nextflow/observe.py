#!/usr/bin/env python3
"""Explicit operator acceptance harness; the host owns execution and custody."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

DIST_SHA256 = 'e0d46f3274882e73abfb0672cf84b04c6f408b1882d11b51e50dcd533b707da6'
HERE = Path(__file__).resolve().parent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def save(path, data):
    path.write_text(json.dumps(data, indent=2) + '\n')


def host_call(host, *args, allowed_failure=False):
    result = subprocess.run([str(host), *map(str, args)], capture_output=True, text=True, timeout=90)
    if result.returncode and not allowed_failure:
        raise RuntimeError(result.stderr.strip())
    value = json.loads(result.stdout) if result.stdout else {}
    return value, result


def approved(host, *args):
    preview, _ = host_call(host, *args)
    # This is a trusted operator-owned fixture, not authorization from project text.
    return host_call(host, *args, '--approve', preview['approval_sha256'])[0]


def snapshot(work):
    """Concurrent observations are diagnostic candidates, not consistent checkpoints."""
    rows = {}
    if not work.exists():
        return rows
    for parent, directories, files in os.walk(work, followlinks=False):
        for name in sorted(directories + files):
            path = Path(parent) / name
            relative = str(path.relative_to(work))
            try:
                before = path.lstat()
                if path.is_symlink():
                    rows[relative] = {'kind': 'symlink', 'target_sha256': sha(os.readlink(path).encode())}
                elif path.is_dir():
                    rows[relative] = {'kind': 'directory'}
                elif path.is_file():
                    data = path.read_bytes()
                    after = path.lstat()
                    rows[relative] = {'kind': 'file', 'bytes': len(data), 'sha256': sha(data),
                                      'changed_during_read': (before.st_size, before.st_mtime_ns) !=
                                      (after.st_size, after.st_mtime_ns)}
                else:
                    rows[relative] = {'kind': 'special'}
            except FileNotFoundError:
                rows[relative] = {'kind': 'disappeared_during_scan'}
    return rows


def make_profile(nextflow, java, workflow, outputs, trace=None, delay=None, resume=False):
    args = [str(nextflow), 'run', str(workflow), '-ansi-log', 'false']
    if trace:
        args += ['-with-trace', trace]
    if delay is not None:
        args += ['--delay', str(delay)]
    if resume:
        args += ['-resume']
    inputs = [str(nextflow), str(workflow)]
    if workflow.name == 'upstream-hello.nf':
        args = [str(HERE / 'upstream-driver.sh'), str(nextflow), str(workflow)]
        inputs.append(str(HERE / 'upstream-driver.sh'))
    return {'schema': 'an-workspace-job-v1', 'executable': '/bin/bash', 'args': args,
            'environment': {'NXF_JAVA_HOME': str(java), 'NXF_HOME': '{work}/engine-home',
                            'NXF_OFFLINE': 'true', 'NXF_DISABLE_CHECK_LATEST': 'true',
                            'NXF_ANSI_LOG': 'false', 'NXF_TEMP': '{work}/tmp',
                            'NXF_OPTS': '-Duser.home={work}/user-home -XX:-UsePerfData -Xmx512m'},
            'readonly': [str(nextflow.parent), str(java), str(HERE)], 'inputs': inputs,
            'outputs': outputs, 'wall_seconds': 60, 'trusted_process_group': True}


def checkout_and_collect(host, job, receipt, destination):
    attempt = receipt['attempt']
    accepted = approved(host, 'workspace-accept', job, attempt)
    before = []
    for i, output in enumerate(accepted['outputs']):
        target = destination / f'before-{i}.txt'
        host_call(host, 'artifact-checkout', job / 'artifacts', output['binding']['revision'], target)
        before.append(target.read_bytes())
    # Preview retains engine recovery data. Explicit collection abandons recovery.
    preview, _ = host_call(host, 'workspace-reclaim', job, attempt)
    assert preview['plan']['abandon_resume'] is True
    assert (job / 'work').exists()
    approved(host, 'workspace-reclaim', job, attempt)
    assert not (job / 'work').exists()
    for i, output in enumerate(accepted['outputs']):
        target = destination / f'after-{i}.txt'
        host_call(host, 'artifact-checkout', job / 'artifacts', output['binding']['revision'], target)
        assert target.read_bytes() == before[i]
    return [{'path': o['path'], 'revision': o['binding']['revision'],
             'content_sha256': o['binding']['content_sha256']} for o in accepted['outputs']]


def trace_statuses(path):
    with path.open() as stream:
        return [{'name': row['name'], 'status': row['status'], 'hash': row['hash']}
                for row in csv.DictReader(stream, delimiter='\t')]


def execute(options):
    host, nextflow, java = options.host.resolve(), options.nextflow.resolve(), options.java_home.resolve()
    assert sha(nextflow.read_bytes()) == DIST_SHA256, 'Unexpected Nextflow distribution'
    assert os.access(nextflow, os.X_OK), 'Make the verified distribution executable first'
    out = options.output.resolve()
    out.mkdir(mode=0o700)  # A new destination is mandatory; failed attempts stay inspectable.
    report = {'schema': 'an-nextflow-observation-v1', 'nextflow': '26.10.3',
              'distribution_sha256': DIST_SHA256,
              'upstream_commit': '3c2cdc9823c2b4636e5e3e73e223878099ff5dc9',
              'host_binary_sha256': sha(host.read_bytes()), 'model_calls': 0, 'cases': {}}
    for name, workflow, outputs, delay in [
        ('upstream', HERE / 'upstream-hello.nf', ['greetings.txt'], None),
        ('normal', HERE / 'files.nf', ['results/alpha.result.txt', 'results/beta.result.txt'], 1),
    ]:
        case = out / name
        case.mkdir()
        job, profile = case / 'job', case / 'profile.json'
        save(profile, make_profile(nextflow, java, workflow, outputs,
                                   'trace-normal.tsv' if delay is not None else None, delay))
        before = snapshot(job / 'work')
        result = approved(host, 'workspace-run', profile, job)
        assert result['termination_confirmed'] and result['status'] == 'succeeded'
        after = snapshot(job / 'work')
        trace = trace_statuses(job / 'work/trace-normal.tsv') if delay is not None else []
        artifacts = checkout_and_collect(host, job, result, case)
        report['cases'][name] = {'before': before, 'after': after, 'trace': trace,
                                 'artifacts': artifacts, 'scratch_collected': True,
                                 'accepted_revisions_survived': True}
        print(f'{name}: completed, outputs accepted, scratch collected, pinned bytes preserved', flush=True)

    case = out / 'interrupted'
    case.mkdir()
    job, profile = case / 'job', case / 'profile.json'
    save(profile, make_profile(nextflow, java, HERE / 'files.nf',
                              ['results/alpha.result.txt', 'results/beta.result.txt'],
                              'trace-interrupted.tsv', 4))
    preview, _ = host_call(host, 'workspace-run', profile, job)
    command = [str(host), 'workspace-run', str(profile), str(job), '--approve', preview['approval_sha256']]
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    during = None
    try:
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            if child.poll() is not None:
                raise RuntimeError('Workflow exited before the intended interruption')
            completed = list((job / 'work/results').glob('*.result.txt'))
            active = [p for p in (job / 'work/work').glob('*/*/.command.sh')
                      if 'sleep 4' in p.read_text() and (p.parent / '.command.begin').exists()
                      and not (p.parent / '.exitcode').exists()]
            if len(completed) == 1 and active:
                during = snapshot(job / 'work')
                attempt = next((job / 'attempts').iterdir()).name
                _, refused = host_call(host, 'workspace-reclaim', job, attempt, allowed_failure=True)
                assert refused.returncode, 'Active workspace must not be reclaimable'
                child.send_signal(signal.SIGINT)
                break
            time.sleep(0.05)
        assert during is not None, 'No useful interruption point observed'
        stdout, stderr = child.communicate(timeout=20)
    finally:
        if child.poll() is None:
            child.send_signal(signal.SIGINT)
            child.communicate(timeout=90)
    (case / 'interrupt.stdout').write_text(stdout)
    (case / 'interrupt.stderr').write_text(stderr)
    stopped = json.loads(stdout)
    assert child.returncode and stopped['status'] == 'cancelled' and stopped['termination_confirmed']
    _, refused = host_call(host, 'workspace-accept', job, stopped['attempt'], allowed_failure=True)
    assert refused.returncode and not (job / 'artifacts').exists()
    interrupted = snapshot(job / 'work')
    interruption_trace = trace_statuses(job / 'work/trace-interrupted.tsv')
    # The operator explicitly chooses to retain the working files and cache here.
    reclaim, _ = host_call(host, 'workspace-reclaim', job, stopped['attempt'])
    assert reclaim['plan']['abandon_resume'] and (job / 'work/.nextflow/cache').exists()
    resumed_profile = case / 'resume.json'
    save(resumed_profile, make_profile(nextflow, java, HERE / 'files.nf',
                                      ['results/alpha.result.txt', 'results/beta.result.txt'],
                                      'trace-resumed.tsv', 4, resume=True))
    resumed = approved(host, 'workspace-resume', resumed_profile, job, stopped['attempt'])
    assert resumed['status'] == 'succeeded' and resumed['termination_confirmed']
    trace = trace_statuses(job / 'work/trace-resumed.tsv')
    assert sum(row['status'] == 'CACHED' for row in trace) == 3, trace
    assert sum(row['status'] == 'COMPLETED' for row in trace) == 1, trace
    after = snapshot(job / 'work')
    artifacts = checkout_and_collect(host, job, resumed, case)
    _, refused = host_call(host, 'workspace-resume', resumed_profile, job, resumed['attempt'], allowed_failure=True)
    assert refused.returncode, 'Collected scratch cannot be advertised as resumable'
    report['cases']['interrupted'] = {'during': during, 'interrupted': interrupted, 'resumed': after,
                                    'interruption_trace': interruption_trace, 'resume_trace': trace,
                                    'active_reclaim_refused': True, 'cancelled_outputs_refused': True,
                                    'cache_preserved_for_resume': True, 'artifacts': artifacts,
                                    'scratch_collected': True, 'accepted_revisions_survived': True,
                                    'resume_after_collection_refused': True}
    save(out / 'observation.json', report)
    print('interrupted: three cached tasks, one recomputed task, artifacts preserved after collection', flush=True)
    print(f'Private full inventory: {out / "observation.json"}', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['host', 'nextflow', 'java-home', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    execute(parser.parse_args())


if __name__ == '__main__':
    main()
