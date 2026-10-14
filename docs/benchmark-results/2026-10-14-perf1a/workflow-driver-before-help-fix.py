#!/usr/bin/env python3
"""Bounded frozen SHA-256 candidate controls; synthetic local fixtures only."""
import argparse
import json
import os
from pathlib import Path
import shutil
import statistics as st
import subprocess

import measure_acknowledgment as native
from measure_recovery_cli import (PREFIX, command_measure, digest, invoke, load,
                                  spread, validate_aux_rows, validate_rows, write)
from measure_recovery_cli_supplement import TESTS

CASES = [(kind, 'btrfs', 'stored_64', None) for kind in TESTS] + [
    ('attestation', 'tmpfs', 'deflated_256', None),
    ('cli', 'btrfs', 'stored_64', 'published'),
    ('cli', 'tmpfs', 'deflated_256', 'written'),
]


def validate(report, run):
    medians = {}
    count = 0
    for binary, filename in run['files'].items():
        assert digest(report / filename) == run['sha256'][filename]
        rows = load(report / filename)
        count += len(rows)
        if run['kind'] in TESTS:
            _, values = native.load_run(report, dict(run, files={binary: filename}))
            medians.update({(binary + '_' + mode if run['kind'] == 'acknowledgment' else mode): metrics
                            for mode, metrics in values.items()})
        elif run['kind'] == 'cli':
            validate_rows(rows, run['pairs'], run['fixture'], run['history'])
            for cli in [False, True]:
                values = [r['metrics'] for r in rows if r['cli'] == cli]
                medians[binary + ('_cli' if cli else '_direct')] = {
                    k: st.median(v[k] for v in values) for k in values[0]}
        else:
            validate_aux_rows(rows, run['pairs'], 'existing')
            for mode in ['reference', 'candidate']:
                values = [r['metrics'] for r in rows if r['binary'] == mode]
                medians[mode] = {k: st.median(v[k] for v in values) for k in values[0]}
    expected = 2 if run['kind'] == 'aux' else 4
    assert len(medians) == expected
    assert all(m.startswith(('reference', 'candidate')) for m in medians)
    return count, medians


def summarize(report):
    runs = load(report / 'workflow-measurements.json')
    groups = {}
    operations = 0
    for run in runs:
        count, medians = validate(report, run)
        operations += count
        g = groups.setdefault(run['group'], {'kind': run['kind'], 'initial': {}, 'repeat': {}})
        for mode, metrics in medians.items():
            m = g[run['stage']].setdefault(mode, {})
            for key, value in metrics.items():
                m.setdefault(key, []).append(value)
    expected = {f'{k}-{fs}-{fix}' + (f'-{hist}' if hist else '') for k, fs, fix, hist in CASES} | {'help', 'assess'}
    assert set(groups) == expected and len(runs) == 54 and operations == 1152
    for name in expected:
        for stage, pairs in [('initial', 4), ('repeat', 8)]:
            subset = [r for r in runs if r['group'] == name and r['stage'] == stage]
            assert [r['round'] for r in subset] == [1, 2, 3]
            assert all(r['pairs'] == pairs for r in subset)
    result = {'timed_observations': operations, 'groups': {}}
    for name, group in groups.items():
        result['groups'][name] = {'kind': group['kind'], 'stages': {
            stage: {mode: {key: {'median': st.median(values), 'round_medians': values,
                                'spread_percent': spread(values)} for key, values in metrics.items()}
                    for mode, metrics in group[stage].items()} for stage in ['initial', 'repeat']}}
    write(report / 'workflow-summary.json', result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['reference', 'candidate', 'reference-cli', 'candidate-cli', 'report',
                'private', 'btrfs-parent', 'tmpfs-parent']:
        parser.add_argument('--' + key, required=True, type=Path)
    a = parser.parse_args()
    for key, value in vars(a).items():
        setattr(a, key, value.resolve())
    assert not (a.report / 'workflow-measurements.json').exists()
    parents = {'btrfs': a.btrfs_parent, 'tmpfs': a.tmpfs_parent}
    for fs, parent in parents.items():
        assert subprocess.check_output(['stat', '-f', '-c', '%T', parent], text=True).strip() == fs
        assert not list(parent.iterdir())
    affinity = sorted(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {affinity[0]})
    write(a.report / 'workflow-environment.json', {
        'affinity': [affinity[0]], 'available_affinity': affinity,
        'kernel': list(os.uname()), 'binary_sha256': {k: digest(getattr(a, k)) for k in
            ['reference', 'candidate', 'reference_cli', 'candidate_cli']},
        'scripts': {Path(p).name: digest(Path(p)) for p in [__file__, native.__file__,
            'scripts/benchmarks/measure_attestation.py', 'scripts/benchmarks/measure_reconciliation.py',
            'scripts/benchmarks/measure_recovery_cli.py', 'scripts/benchmarks/measure_recovery_cli_supplement.py']},
        'mounts': {fs: subprocess.check_output(['findmnt', '-n', '-o', 'FSTYPE,OPTIONS', '-T', p], text=True).strip()
                   for fs, p in parents.items()},
        'conditions': 'Serial workload; normal page cache; setup/oracles excluded; no cold-cache claim. '
                      'Host load/SMT/frequency not isolated. Private tmpfs huge=never is volatile. '
                      'Native lifetime RSS includes fixture setup; CLI child RSS is a separate scope.'})
    runs = []

    def save(run):
        validate(a.report, run)
        runs.append(run)
        write(a.report / 'workflow-measurements.json', runs)
        print(run['group'], run['stage'], run['round'], flush=True)

    for kind, fs, fixture, history in CASES:
        group = f'{kind}-{fs}-{fixture}' + (f'-{history}' if history else '')
        for stage, pairs in [('initial', 4), ('repeat', 8)]:
            for round_ in range(1, 4):
                files, hashes = {}, {}
                for binary in (['reference', 'candidate'] if round_ % 2 else ['candidate', 'reference']):
                    stem = f'workflow-{group}-{stage}-{round_}-{binary}'
                    filename = stem + '.json'
                    env = os.environ | {'RVVDK_RETAINED_PARENT': str(parents[fs])}
                    if kind == 'cli':
                        test = PREFIX + 'command_benchmark'
                        env |= {'RVDDK_RECOVERY_CLI': str(getattr(a, binary + '_cli')),
                                'RVDDK_CLI_PAIRS': str(pairs), 'RVDDK_CLI_FIXTURE': fixture,
                                'RVDDK_CLI_HISTORY': history, 'RVDDK_CLI_REPORT': str(a.report / filename)}
                    else:
                        test = TESTS[kind]
                        env |= {'RVVDK_RETAINED_PAIRS': str(pairs), 'RVVDK_PROFILE_FIXTURE': fixture,
                                'RVVDK_RETAINED_REPORT': str(a.report / filename)}
                    invoke(getattr(a, binary), test, env, a.report / (stem + '.txt'))
                    files[binary] = filename
                    hashes[filename] = digest(a.report / filename)
                save({'kind': kind, 'group': group, 'filesystem': fs, 'fixture': fixture, 'history': history,
                      'stage': stage, 'round': round_, 'pairs': pairs, 'files': files, 'sha256': hashes})
    handoff = a.private / 'workflow-assess.handoff'
    invoke(a.reference, PREFIX + 'prepare_fixture', os.environ | {
        'RVVDK_RETAINED_PARENT': str(a.btrfs_parent), 'RVDDK_CLI_HISTORY': 'published',
        'RVDDK_CLI_GRAINS': '1024', 'RVDDK_CLI_HANDOFF': str(handoff)}, a.private / 'workflow-prepare.txt')
    fixture = Path(handoff.read_text())
    assert fixture.parent == a.btrfs_parent and fixture.name.startswith('rvddk-retained-')

    def snapshot():
        return {str(f.relative_to(fixture)): (digest(f), f.stat().st_ino, f.stat().st_size, f.stat().st_blocks)
                for f in fixture.rglob('*') if f.is_file()}

    before = snapshot()
    try:
        for group, args in [('help', ['--help']), ('assess', ['recover', 'assess', str(fixture / 'recover.json'), '--json'])]:
            expected = None
            for stage, pairs in [('initial', 4), ('repeat', 8)]:
                for round_ in range(1, 4):
                    rows = []
                    for pair in range(pairs):
                        for binary in (['reference', 'candidate'] if pair % 2 == 0 else ['candidate', 'reference']):
                            metrics = command_measure(getattr(a, binary + '_cli'), args, a.private)
                            output = (a.private / 'command.stdout').read_text()
                            if group == 'assess':
                                data = json.loads(output)
                                assert data['status'] == 'assessed' and data['operation_succeeded']
                                assert not data['data']['publication']['output']['pending_transaction']
                            if expected is None:
                                expected = output
                            assert output == expected
                            rows.append({'pair': pair, 'binary': binary, 'metrics': metrics, 'oracle_passed': True})
                    filename = f'workflow-{group}-{stage}-{round_}.json'
                    write(a.report / filename, rows)
                    save({'kind': 'aux', 'group': group, 'stage': stage, 'round': round_, 'pairs': pairs,
                          'files': {'paired': filename}, 'sha256': {filename: digest(a.report / filename)}})
        assert snapshot() == before
    finally:
        shutil.rmtree(fixture)
    assert all(not list(p.iterdir()) for p in parents.values())
    summarize(a.report)
    print('1152 workflow observations validated; fixture teardown passed', flush=True)


if __name__ == '__main__':
    main()
