#!/usr/bin/env python3
"""Replay PERF.1b evidence and reject corrupt copies without running workloads."""
import argparse
import copy
from pathlib import Path
import tempfile

import measure_hash_backends as micro
import measure_hash_fallback as fallback
import measure_hash_workflows as workflow
from measure_recovery_cli import digest, load, validate_rows, write
import measure_assessment_controls as assessment


def validate(report):
    hashes = report / 'non-sha-host' / 'hash'
    old = load(hashes / 'hash-summary.json')
    fallback.summarize(hashes)
    assert load(hashes / 'hash-summary.json') == old and old['timed_observations'] == 720
    for run in load(hashes / 'hash-measurements.json'):
        assert load(hashes / run['file'])['metadata']['sha'] is False
    for host, sha in [('sha-host', True), ('non-sha-host', False)]:
        root = report / host
        qualification = load(root / 'functional/qualification.json')
        assert qualification['full_library'] == (not sha)
        assert qualification['cli_fixture_groups'] == 6
        assert qualification['separate_filesystems'] and qualification['fixture_parents_empty']
        normalized = load(report / 'text-log-normalization.json') if (report / 'text-log-normalization.json').exists() else {}
        for filename, original_hash in qualification['logs_sha256'].items():
            path = root / 'functional' / filename
            record = normalized.get(str(path.relative_to(report)))
            if record:
                assert record['original_sha256'] == original_hash
                assert digest(path) == record['committed_sha256']
            else:
                assert digest(path) == original_hash
        old = load(root / 'workflow-summary.json')
        assert workflow.summarize(root) == old and old['timed_observations'] == 1152
        check = load((root if sha else hashes) / 'hash-check-native.json')
        assert check['metadata']['sha'] == sha and check['checks'] == 624
        assert len(check['cases']) == 36
        for case in check['cases']:
            assert case['sha256'] == micro.oracle(case['bytes'], case['sparse_pattern'])
        env = load(root / 'workflow-environment.json')
        assert env['disk_filesystem'] == ('btrfs' if sha else 'xfs')
        provenance = load(report / 'candidate-provenance.json')
        assert qualification['binary_sha256']['tests'] == provenance['binaries']['candidate-tests']['sha256']
        assert qualification['binary_sha256']['cli'] == provenance['binaries']['candidate-cli']['sha256']
        for binary, entry in provenance['binaries'].items():
            key = binary.replace('-cli', '_cli').replace('-tests', '')
            assert env['binary_sha256'][key] == entry['sha256']
    assert digest(report / 'candidate.patch') == load(report / 'candidate-provenance.json')['patch_sha256']
    old = load(report / 'assessment-control-summary.json')
    assert assessment.summarize(report) == old and old['timed_observations'] == 96
    provenance = load(report / 'candidate-provenance.json')['binaries']
    for run in load(report / 'assessment-control-measurements.json'):
        assert run['binary_sha256']['reference'] == provenance['reference-cli']['sha256']
        expected = 'reference-cli' if run['comparison'] == 'identical' else 'candidate-cli'
        assert run['binary_sha256']['candidate'] == provenance[expected]['sha256']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    a = parser.parse_args()
    report = a.report.resolve()
    validate(report)
    rejected = []

    def reject(name, action):
        try:
            action()
        except (AssertionError, ValueError, KeyError):
            rejected.append(name)
        else:
            raise AssertionError('Corrupt evidence accepted: ' + name)

    hashes = report / 'non-sha-host' / 'hash'
    run = load(hashes / 'hash-measurements.json')[-1]
    case = next(c for c in fallback.CASES if c[0] == run['group'])
    good = load(hashes / run['file'])
    for field, value in [('pair', -1), ('backend', 'ring'), ('bytes_per_digest', 0),
                         ('iterations', 0), ('sha256', '0' * 64), ('oracle_passed', False),
                         ('wall_ms', float('nan')), ('cpu_ms', -1)]:
        bad = copy.deepcopy(good)
        bad['rows'][0][field] = value
        reject('512MiB:' + field, lambda: micro.validate(bad, run['pairs'], case))
    for host in ['sha-host', 'non-sha-host']:
        root = report / host
        runs = load(root / 'workflow-measurements.json')
        with tempfile.TemporaryDirectory(prefix='perf1b-validation-') as tmp:
            temp = Path(tmp)
            for kind, fields in {
                'normal': {'byte_comparison_passed': False, 'logical_bytes_verified_in_timing': 0},
                'recovery': {'oracle_passed': False, 'explicit_sync_calls': 0, 'allocation_unchanged': False},
                'attestation': {'oracle_passed': False, 'allocation_unchanged': False},
                'acknowledgment': {'candidate_preserved': False, 'old_journal_replaced': False, 'explicit_sync_calls': 0},
            }.items():
                run = next(r for r in runs if r['kind'] == kind)
                filename = run['files']['candidate']
                good = load(root / filename)
                write(temp / filename, good)
                fake = dict(run, files={'candidate': filename}, sha256={filename: digest(temp / filename)})
                workflow.native.load_run(temp, fake)
                for field, value in fields.items():
                    bad = copy.deepcopy(good)
                    bad[0][field] = value
                    write(temp / filename, bad)
                    fake = dict(run, files={'candidate': filename}, sha256={filename: digest(temp / filename)})
                    reject(host + ':' + kind + ':' + field, lambda: workflow.native.load_run(temp, fake))
                fake['sha256'][filename] = '0' * 64
                reject(host + ':' + kind + ':file_hash', lambda: workflow.native.load_run(temp, fake))
        run = next(r for r in runs if r['kind'] == 'cli')
        good = load(root / run['files']['candidate'])
        for field, value in [('pair', -1), ('allocation_unchanged', False), ('logical_bytes', 1)]:
            bad = copy.deepcopy(good)
            bad[0][field] = value
            reject(host + ':cli:' + field, lambda: validate_rows(bad, run['pairs'], run['fixture'], run['history']))
    write(report / 'evidence-validation.json', {
        'native_hash_observations': 720, 'workflow_observations_per_host': 1152,
        'primary_timed_observations': 3024, 'assessment_diagnostic_observations': 96,
        'total_validated_timed_observations': 3120, 'native_digest_checks_per_host': 624,
        'independent_python_digest_cases': 72, 'corruption_rejections': rejected,
        'scope': 'Checks use in-memory or temporary copies; original timing samples remain unchanged.'})
    print('3120 validated timing observations;', len(rejected), 'corrupt-copy rejections')


if __name__ == '__main__':
    main()
