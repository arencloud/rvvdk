#!/usr/bin/env python3
"""Replay PERF.1a validators and reject corrupt copies, without rerunning timings."""
import argparse
import copy
import json
from pathlib import Path
import tempfile

import measure_hash_backends as micro
import measure_hash_workflows as workflow
from measure_recovery_cli import digest, load, validate_rows, write


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    a = parser.parse_args()
    report = a.report.resolve()
    index = load(report / 'hash-measurements.json')
    assert len(index) == 48
    assert set(r['group'] for r in index) == {c[0] for c in micro.CASES}
    for case in micro.CASES:
        for stage, pairs in [('initial', 4), ('repeat', 8)]:
            runs = [r for r in index if r['group'] == case[0] and r['stage'] == stage]
            assert [r['round'] for r in runs] == [1, 2, 3]
            assert all(r['pairs'] == pairs for r in runs)
    original = load(report / 'hash-summary.json')
    micro.summarize(report)
    assert load(report / 'hash-summary.json') == original
    original = load(report / 'workflow-summary.json')
    assert workflow.summarize(report) == original
    assert all(load(report / r['file'])['metadata']['sha'] for r in index)
    checks = []
    for name, sha in [('hash-check-native.json', True), ('hash-check-no-sha-emulated.json', False)]:
        data = load(report / name)
        assert data['metadata']['sha'] == sha and data['checks'] == 624 and len(data['cases']) == 36
        for case in data['cases']:
            assert case['sha256'] == micro.oracle(case['bytes'], case['sparse_pattern'])
        checks.append(name)
    rejected = []

    def reject(name, action):
        try:
            action()
        except (AssertionError, ValueError, KeyError):
            rejected.append(name)
        else:
            raise AssertionError('Corrupt evidence accepted: ' + name)

    run = index[0]
    good = load(report / run['file'])
    for field, value in [('pair', 3), ('backend', 'ring'), ('bytes_per_digest', 0),
                         ('chunk', 1), ('iterations', 0), ('sparse_pattern', True),
                         ('sha256', '0' * 64), ('oracle_passed', False),
                         ('wall_ms', float('nan')), ('cpu_ms', -1), ('peak_rss_kib', 0)]:
        bad = copy.deepcopy(good)
        bad['rows'][0][field] = value
        reject('hash:' + field, lambda: micro.validate(bad, run['pairs'], micro.CASES[0]))
    runs = load(report / 'workflow-measurements.json')
    cli = next(r for r in runs if r['kind'] == 'cli')
    good = load(report / cli['files']['reference'])
    for field, value in [('pair', 5), ('cli', True), ('fixture', 'unknown'),
                         ('oracle_passed', False), ('allocation_unchanged', False),
                         ('allocated_bytes', 0), ('logical_bytes', 1)]:
        bad = copy.deepcopy(good)
        bad[0][field] = value
        reject('cli:' + field, lambda: validate_rows(bad, cli['pairs'], cli['fixture'], cli['history']))
    bad = copy.deepcopy(good)
    bad[0]['report']['content_verified'] = False
    reject('cli:verification_report', lambda: validate_rows(bad, cli['pairs'], cli['fixture'], cli['history']))
    # Native validators also enforce the recorded work and persistence boundaries.
    with tempfile.TemporaryDirectory(prefix='perf1a-validation-') as tmp:
        root = Path(tmp)
        for kind, fields in {
            'normal': {'byte_comparison_passed': False, 'logical_bytes_verified_in_timing': 0, 'output_journal_commits': 0},
            'recovery': {'oracle_passed': False, 'explicit_sync_calls': 0, 'allocation_unchanged': False},
            'attestation': {'oracle_passed': False, 'allocation_unchanged': False},
            'acknowledgment': {'candidate_preserved': False, 'old_journal_replaced': False, 'explicit_sync_calls': 0},
        }.items():
            run = next(r for r in runs if r['kind'] == kind)
            filename = run['files']['reference']
            good = load(report / filename)
            write(root / filename, good)
            fake = dict(run, files={'reference': filename}, sha256={filename: digest(root / filename)})
            workflow.native.load_run(root, fake)
            for field, value in fields.items():
                bad = copy.deepcopy(good)
                bad[0][field] = value
                write(root / filename, bad)
                fake = dict(run, files={'reference': filename}, sha256={filename: digest(root / filename)})
                reject(kind + ':' + field, lambda: workflow.native.load_run(root, fake))
            fake['sha256'][filename] = '0' * 64
            reject(kind + ':file_hash', lambda: workflow.native.load_run(root, fake))
    provenance = load(report / 'candidate-provenance.json')
    assert digest(report / 'rejected-candidate.patch') == provenance['patch_sha256']
    result = {'hash_observations': 576, 'workflow_observations': original['timed_observations'],
              'independent_digest_cases': 72, 'correctness_files': checks,
              'corruption_rejections': rejected, 'production_patch_promoted': False,
              'method': 'Mutations use in-memory/private temporary copies; original timing samples are not modified.'}
    write(report / 'evidence-validation.json', result)
    print(json.dumps({'observations': 576 + original['timed_observations'], 'corruption_rejections': len(rejected)}))


if __name__ == '__main__':
    main()
