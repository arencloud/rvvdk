#!/usr/bin/env python3
"""Validate PERF.1c sample coverage, hashes, oracles, and source provenance."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import shutil
from measure_transfer_hashes import summarize, validate_run, digest, write
from measure_transfer_hash_controls import summarize as control_summary


def check(report, source=None):
    provenance = json.loads((report / 'provenance.json').read_text())
    assert digest(report / 'plan.md') == provenance['plan_sha256']
    expected_index = {'environment.json', 'invocations.json', 'measurements.json', 'startup.json', 'completion.json'}
    total = 0
    for host, sha in [('non-sha-host', False), ('sha-host', True)]:
        root = report / host
        index = json.loads((root / 'index.json').read_text())
        assert set(index['sha256']) == expected_index
        saved_bytes = (root / 'summary.json').read_bytes()
        try:
            summary = summarize(root)
        finally:
            (root / 'summary.json').write_bytes(saved_bytes)
        assert json.loads(saved_bytes) == summary
        total += summary['timed_observations']
        env = json.loads((root / 'environment.json').read_text())
        assert ('sha_ni' in env['flags']) is sha
        assert env['affinity'] == [0]
        assert env['filesystems'] == {'disk': 'btrfs' if sha else 'xfs', 'tmpfs': 'tmpfs'}
        assert env['script_sha256'] == digest(report / 'measurement-driver.py')
        output_check = json.loads((report / 'startup-output-check.json').read_text())
        assert len({r['normalized_sha256'] for r in output_check.values()}) == 1
        for version, item in output_check.items():
            raw = item['stdout'].encode()
            assert hashlib.sha256(raw).hexdigest() == item['sha256']
            assert hashlib.sha256(raw.replace((version + '-cli').encode(), b'rvddk')).hexdigest() == item['normalized_sha256']
        for row in json.loads((root / 'startup.json').read_text()):
            assert row['stdout_sha256'] == output_check[row['version']]['sha256']
        for name, binary in provenance['binaries'].items():
            assert env['binary_sha256'][name] == binary['sha256']
            assert env['binary_bytes'][name] == binary['bytes']
        assert json.loads((root / 'completion.json').read_text()) == {
            'environment_and_binary_hashes_unchanged': True, 'fixture_parents_empty': True}
        runs = json.loads((root / 'measurements.json').read_text())
        logs = json.loads((root / 'invocations.json').read_text())
        assert len(logs) == 144
        assert {(r['group'], r['stage'], r['round'], r['version']) for r in logs} == {
            (r['group'], r['stage'], r['round'], v) for r in runs for v in ['reference', 'candidate']}
        assert all(r['exit_code'] == 0 and '1 passed; 0 failed;' in r['stdout'] and not r['stderr'] for r in logs)
    assert total == 1872
    for host in ['non-sha-host', 'sha-host']:
        root = report / host
        saved_bytes = (root / 'control-summary.json').read_bytes()
        try:
            controls = control_summary(root)
        finally:
            (root / 'control-summary.json').write_bytes(saved_bytes)
        assert json.loads(saved_bytes) == controls
        total += controls['timed_observations']
        env = json.loads((root / 'control-environment.json').read_text())
        for name, value in env['binary_sha256'].items():
            assert value == provenance['binaries'][name]['sha256']
        driver = report / 'control-driver-sha.py' if host == 'sha-host' else Path(__file__).with_name('measure_transfer_hash_controls.py')
        assert env['script_sha256'] == digest(driver)
        assert env['driver_sha256'] == digest(Path(__file__).with_name('measure_transfer_hashes.py'))
        assert env['affinity'] == [0]
        assert env['filesystem'] == ('btrfs' if host == 'sha-host' else 'xfs')
        logs = json.loads((root / 'control-invocations.json').read_text())
        runs = json.loads((root / 'controls.json').read_text())
        assert len(logs) == controls['timed_observations'] // 2
        assert {(r['group'], r['round'], r['mode'], r['pair'], r['version']) for r in logs} == {
            (r['group'], r['round'], r['mode'], pair, v) for r in runs for pair in range(8) for v in ['reference', 'candidate']}
        assert all(r['exit_code'] == 0 and '1 passed; 0 failed;' in r['stdout'] and not r['stderr'] for r in logs)
        assert all(r['actual_binary'] == ('reference' if r['mode'] == 'AA' else r['version']) for r in logs)
    plots = report / 'plot-index.json'
    if plots.exists():
        for name, expected in json.loads(plots.read_text())['files'].items():
            assert digest(report / name) == expected
    if source:
        for name, expected in provenance['sources']['candidate'].items():
            assert digest(source / name) == expected, name
    assert total == 2640
    return total


def self_test(report):
    rejected = 0
    for host in ['non-sha-host', 'sha-host']:
        original = json.loads((report / host / 'measurements.json').read_text())[0]
        mutations = [
            lambda r: r['versions']['candidate'][0].update(byte_comparison_passed=False),
            lambda r: r['versions']['candidate'][0].update(payload_allocated_bytes=0),
            lambda r: r['versions']['candidate'][0].update(cpu_ms=float('nan')),
            lambda r: r['versions']['candidate'][0].update(payload_bytes=1),
            lambda r: r['versions']['candidate'].pop(),
            lambda r: r['versions']['candidate'][2].update(pair=0),
        ]
        for mutate in mutations:
            value = copy.deepcopy(original); mutate(value)
            try: validate_run(value)
            except AssertionError: rejected += 1
            else: raise AssertionError('invalid row accepted')
        with tempfile.TemporaryDirectory(prefix='rvddk-perf1c-check-') as tmp:
            root = Path(tmp)
            for name in ['index.json', 'measurements.json', 'environment.json', 'invocations.json', 'startup.json', 'completion.json']:
                shutil.copy2(report / host / name, root / name)
            with (root / 'measurements.json').open('a') as f: f.write(' ')
            try: summarize(root)
            except AssertionError: rejected += 1
            else: raise AssertionError('altered raw bytes accepted')
    assert rejected == 14
    return rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--self-test', action='store_true')
    a = parser.parse_args()
    result = {'timed_observations': check(a.report, a.source)}
    if a.self_test: result['corrupt_copies_rejected'] = self_test(a.report)
    if a.self_test:
        write(a.report / 'evidence-check.json', result)
    print(json.dumps(result))


if __name__ == '__main__': main()
