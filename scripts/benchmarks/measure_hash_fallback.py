#!/usr/bin/env python3
"""PERF.1b native fallback matrix; reuse exact digest/oracle and pairing rules."""
import argparse
import json
import os
from pathlib import Path
import subprocess

import measure_hash_backends as base

CASES = base.CASES + [
    ('dense512', 512 << 20, 1 << 20, 1, 'dense'),
    ('sparse512', 512 << 20, 1 << 20, 1, 'sparse'),
]


def summarize(report):
    index = base.load(report / 'hash-measurements.json')
    assert len(index) == len(CASES) * 6
    assert set(r['group'] for r in index) == {c[0] for c in CASES}
    for case in CASES:
        for stage, pairs in [('initial', 4), ('repeat', 8)]:
            runs = [r for r in index if r['group'] == case[0] and r['stage'] == stage]
            assert [r['round'] for r in runs] == [1, 2, 3]
            assert all(r['pairs'] == pairs for r in runs)
    return base.summarize(report, CASES)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    a = parser.parse_args()
    a.binary, a.report = a.binary.resolve(), a.report.resolve()
    assert not (a.report / 'hash-measurements.json').exists()
    a.report.mkdir(parents=True, exist_ok=True)
    affinity = sorted(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {affinity[0]})
    base.write(a.report / 'hash-environment.json', {
        'binary_sha256': base.sha(a.binary), 'affinity': [affinity[0]],
        'available_affinity': affinity, 'compiler_host': 'development host; native runner execution',
        'driver_sha256': base.sha(Path(__file__)), 'shared_driver_sha256': base.sha(Path(base.__file__)),
        'conditions': 'Serial timings on this host; ordinary caches; no cache drops or emulation. '
                      'CPU frequency/SMT/hypervisor/storage/competing host work not isolated.'})
    index = []
    for case in CASES:
        group, size, chunk, iterations, pattern = case
        for stage, pairs in [('initial', 4), ('repeat', 8)]:
            for round_ in range(1, 4):
                filename = f'hash-{group}-{stage}-{round_}.json'
                result = subprocess.run([a.binary, 'bench', str(size), str(chunk), str(iterations),
                                         str(pairs), pattern], capture_output=True, text=True,
                                        timeout=900, check=True)
                assert not result.stderr
                value = json.loads(result.stdout)
                # Persist before validation so a failed oracle remains reviewable.
                base.write(a.report / filename, value)
                base.validate(value, pairs, case)
                assert value['metadata']['sha'] is False, 'Runner configuration changed'
                index.append({'group': group, 'stage': stage, 'round': round_, 'pairs': pairs,
                              'file': filename, 'sha256': base.sha(a.report / filename)})
                base.write(a.report / 'hash-measurements.json', index)
                print(filename, flush=True)
    summarize(a.report)
    print('720 validated native hash observations', flush=True)


if __name__ == '__main__':
    main()
