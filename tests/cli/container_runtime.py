#!/usr/bin/env python3
"""Paired offline packaging/import acceptance; no runtime services or task execution."""
import argparse
import copy
import hashlib
import io
import json
from pathlib import Path
import shlex
import subprocess
import tarfile

COMPONENTS = ('container', 'container_apiserver', 'container_runtime_linux',
              'container_network_vmnet', 'container_images', 'container_kernel',
              'container_init', 'container_image', 'container_swift_lock')


def sha(value):
    return hashlib.sha256(value).hexdigest()


def describe(payloads, components):
    return {'schema': 'an-offline-runtime-v1', 'name': 'container-fixture', 'version': '1',
            'platform': 'macos-aarch64', 'licenses': ['NOTICE'], 'components': components,
            'files': {name: {'sha256': sha(data), 'bytes': len(data),
                            'executable': name in COMPONENTS[:5]} for name, data in payloads.items()}}


def archive(path, manifest, payloads):
    with tarfile.open(path, 'w:gz', format=tarfile.USTAR_FORMAT) as target:
        for name, data in [('runtime.json', json.dumps(manifest).encode()), *sorted(payloads.items())]:
            header = tarfile.TarInfo(name)
            header.size = len(data)
            header.mode = 0o400
            target.addfile(header, io.BytesIO(data))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('loom', 'host', 'evidence'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    loom, host = args.loom.resolve(strict=True), args.host.resolve(strict=True)
    root = args.evidence.resolve()
    root.mkdir(mode=0o700, parents=True)
    marker = root / 'EXECUTED'
    executable = ('#!/bin/sh\ntouch ' + shlex.quote(str(marker)) + '\nexit 97\n').encode()
    payloads = {name: executable if i < 5 else name.encode() for i, name in enumerate(COMPONENTS)}
    payloads['NOTICE'] = b'Synthetic test fixture; no upstream binary or image.'
    declaration = {'schema': 'an-container-backend-v1', 'backend': 'apple-container',
                   'image': 'local/fixture@sha256:' + 'a' * 64, 'cpus': 2,
                   'memory_mib': 1024, 'storage_bytes': 1073741824}
    source = {'schema': 'an-container-provenance-v1',
              'repository': 'https://github.com/lazyalexander/container.git',
              'commit': 'a15b46097aa4c2325da13c1fb5192b9274aab083',
              'swift_lock_sha256': sha(payloads['container_swift_lock']),
              'files': {name: sha(data) for name, data in payloads.items()}}
    payloads.update(backend=json.dumps(declaration).encode(), container_provenance=json.dumps(source).encode())
    components = {name: name for name in (*COMPONENTS, 'backend', 'container_provenance')}
    commands = []

    def call(binary, *arguments, ok=True):
        result = subprocess.run([str(binary), *map(str, arguments)], capture_output=True, text=True, timeout=30)
        commands.append({'arguments': list(map(str, arguments)), 'returncode': result.returncode,
                         'stdout': result.stdout, 'stderr': result.stderr})
        (root / 'commands.json').write_text(json.dumps(commands, indent=2))
        assert (result.returncode == 0) == ok, (arguments, result.stderr)
        assert not marker.exists(), 'Import or packaging executed a payload'
        return json.loads(result.stdout) if ok else None

    def pack_case(name, data, refs, valid):
        directory = root / name
        directory.mkdir()
        for path, value in data.items():
            (directory / path).write_bytes(value)
            (directory / path).chmod(0o500 if path in COMPONENTS[:5] else 0o400)
        manifest = describe(data, refs)
        selection = directory / 'selection.json'
        selection.write_text(json.dumps(manifest))
        packed = root / (name + '.tar.gz')
        call(loom, 'pack-runtime', directory, selection, packed, ok=valid)
        if not valid:
            assert not packed.exists()
            archive(packed, manifest, data)
        result = call(host, 'app', 'check-environment', packed, ok=valid)
        return packed, result

    valid, preview = pack_case('valid', payloads, components, True)
    assert preview['executed'] is False
    stage_selection = root / 'stage-selection.json'
    stage_selection.write_text(json.dumps({'name': 'container-fixture', 'version': '1',
        'platform': 'macos-aarch64', 'licenses': ['NOTICE'], 'components': components,
        'sources': {name: str(root / 'valid' / name) for name in payloads}}))
    staged = root / 'staged'
    call(loom, 'stage-runtime', stage_selection, staged)
    staged_archive = root / 'staged.tar.gz'
    call(loom, 'pack-runtime', staged, staged / 'runtime.json', staged_archive)
    assert call(host, 'app', 'check-environment', staged_archive)['approval_sha256'] == preview['approval_sha256']
    installed = root / 'installed'
    call(host, 'app', 'import-environment', installed, valid, '--approve', 'b' * 64, ok=False)
    result = call(host, 'app', 'import-environment', installed, valid, '--approve', preview['approval_sha256'])
    assert result['installed'] and not result['reused']
    assert call(host, 'app', 'import-environment', installed, valid, '--approve', preview['approval_sha256'])['reused']
    for field, value in [('image', 'local/fixture:latest'), ('backend', 'shell'), ('cpus', 0), ('network', True)]:
        data = copy.deepcopy(payloads)
        changed = dict(declaration, **{field: value})
        data['backend'] = json.dumps(changed).encode()
        pack_case('invalid-' + field, data, components, False)
    data = copy.deepcopy(payloads)
    changed_source = copy.deepcopy(source)
    changed_source['files']['container'] = 'f' * 64
    data['container_provenance'] = json.dumps(changed_source).encode()
    pack_case('invalid-provenance', data, components, False)
    pack_case('mixed', payloads, dict(components, cocoon='container'), False)
    orphan = dict(components)
    del orphan['backend']
    pack_case('orphan', payloads, orphan, False)
    legacy, legacy_preview = pack_case('legacy', {'NOTICE': b'legacy'}, {'domain': 'NOTICE'}, True)
    call(host, 'app', 'import-environment', installed, legacy, '--approve', legacy_preview['approval_sha256'])
    changed = installed / preview['approval_sha256'] / 'backend'
    changed.chmod(0o600)
    changed.write_bytes(b'changed')
    call(host, 'app', 'import-environment', installed, valid, '--approve', preview['approval_sha256'], ok=False)
    (root / 'report.json').write_text(json.dumps({'passed': True, 'payload_executed': False,
        'legacy_compatible': True, 'runtime_qualified': False,
        'host_sha256': sha(host.read_bytes()), 'loom_sha256': sha(loom.read_bytes())}, indent=2))
    print('Paired Container offline package/import checks passed; execution unqualified.')


if __name__ == '__main__':
    main()
