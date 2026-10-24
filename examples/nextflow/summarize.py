#!/usr/bin/env python3
"""Export a closed portable summary; never publish native plans or transcripts."""
import argparse
import hashlib
import json
from pathlib import Path
import re


def summarize(source, revision):
    if not re.fullmatch('[a-f0-9]{40}', revision):
        raise ValueError('Use the exact host source revision selected by the operator')
    result = {key: source[key] for key in ['schema', 'nextflow', 'distribution_sha256',
                                          'upstream_commit', 'host_binary_sha256', 'model_calls']}
    result['host_source_revision'] = revision
    result['cases'] = {}
    for name in ['upstream', 'normal', 'interrupted']:
        case = source['cases'][name]
        phases = {}
        for phase in ['before', 'during', 'after', 'interrupted', 'resumed']:
            if phase in case:
                entries = case[phase]
                phases[phase] = {'entries': len(entries),
                                 'files': sum(v['kind'] == 'file' for v in entries.values()),
                                 'symlinks': sum(v['kind'] == 'symlink' for v in entries.values()),
                                 'bytes': sum(v.get('bytes', 0) for v in entries.values())}
        row = {'phases': phases, 'artifacts': case['artifacts'],
               'scratch_collected': case['scratch_collected'],
               'accepted_revisions_survived': case['accepted_revisions_survived']}
        for key in ['active_reclaim_refused', 'cancelled_outputs_refused',
                    'cache_preserved_for_resume', 'resume_after_collection_refused']:
            if key in case:
                row[key] = case[key]
        if name == 'interrupted':
            row['resume_trace'] = case['resume_trace']
        result['cases'][name] = row
    result['fixture_sha256'] = {name: hashlib.sha256((Path(__file__).parent / name).read_bytes()).hexdigest()
                               for name in ['upstream-hello.nf', 'files.nf', 'observe.py']}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('observation', type=Path)
    parser.add_argument('host_revision')
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    result = summarize(json.loads(args.observation.read_text()), args.host_revision)
    # Exclusive publication: never replace a different acceptance record.
    with args.output.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')


if __name__ == '__main__':
    main()
