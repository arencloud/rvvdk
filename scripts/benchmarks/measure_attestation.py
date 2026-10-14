#!/usr/bin/env python3
"""Local Rust attestation qualification with frozen normal/reconciliation controls."""
import argparse
import json
import math
import os
from pathlib import Path
import platform
import statistics as st
import subprocess
import time
import measure_reconciliation as previous
from measure_reconciliation import digest, write

ATTEST = 'ownership::retained::output::publication::attestation::tests::benchmark::attestation_histories'
PHASES = ['admission', 'verification', 'bundle_sync', 'parent_sync', 'published_record_sync', 'assessment_and_release']


def metrics(row, kind):
    result = {k: row[k] for k in previous.BASE}
    if kind == 'normal':
        for key in ['admission_ms', 'conversion_and_checks_ms', 'publication_admission_ms', 'publication_ms']:
            if row[key] > 0:
                result[key] = row[key]
    else:
        for phase, values in row['phases'].items():
            for metric, value in values.items():
                result[f'{phase}.{metric}'] = value
        if kind == 'recovery' and row['recording'] and not row['already_renamed']:
            result['rename_ms'] = row['rename_ms']
            result['rename_cpu_ms'] = row['rename_cpu_ms']
    # getrusage has finite resolution: a successful sub-microsecond phase can
    # measure zero CPU. Preserve zero, never manufacture an epsilon or drop it.
    for key, value in result.items():
        limited = key.endswith('.cpu_ms') or key == 'rename_cpu_ms'
        assert math.isfinite(value) and (value >= 0 if limited else value > 0)
    return result


def spread(values):
    return None if min(values) == 0 else 100 * (max(values) / min(values) - 1)


# Existing row schema, preserving every work/allocation gate while accepting
# resolution-limited CPU phases. The preceding campaign script stays unchanged.
def load_existing_run(root, run):
    modes = {}
    for binary, name in run['files'].items():
        path = root / name
        assert digest(path) == run['sha256'][name]
        rows = json.loads(path.read_text())
        assert len(rows) == 2 * run['pairs']
        for pair in range(run['pairs']):
            group = rows[2 * pair:2 * pair + 2]
            assert [r['pair'] for r in group] == [pair, pair]
            field = 'published' if run['kind'] == 'normal' else ('recording' if run['kind'] == 'controls' else 'already_renamed')
            assert [r[field] for r in group] == ([False, True] if pair % 2 == 0 else [True, False])
        for r in rows:
            assert 0 < r['output_allocated_bytes'] <= r['logical_bytes']
            assert r['cleanup_passed'] and r['cleanup_ms'] > 0 and r['cleanup_cpu_ms'] > 0
            if run['kind'] == 'normal':
                mode = binary + ('_published' if r['published'] else '_owned')
                assert r['logical_bytes'] == 64 << 20 and r['source_bytes'] == 8527360
                assert r['byte_comparison_passed'] and r['journal_unchanged']
                assert r['read_bytes'] == 8 << 20 and r['written_bytes'] + r['zeroed_bytes'] == 64 << 20
                assert r['logical_bytes_verified_in_timing'] == (3 if r['published'] else 1) * (64 << 20)
                assert r['output_journal_commits'] == (9 if r['published'] else 7)
                assert r['cleanup_journal_commits'] == 2
            else:
                mode = ('enabled' if r['recording'] else 'disabled') if run['kind'] == 'controls' else ('already_renamed' if r['already_renamed'] else 'staged')
                assert r['fixture'] == run['fixture'] and r['controls'] == (run['kind'] == 'controls')
                assert not r['counters'] and not r['regions']
                assert r['oracle_passed'] and r['source_unchanged'] and r['allocation_unchanged']
                assert r['initial_allocated_bytes'] == r['recovered_allocated_bytes'] == r['output_allocated_bytes']
                logical = (64 if run['fixture'] == 'stored_64' else 256) << 20
                assert r['logical_bytes_verified'] == r['logical_bytes'] == logical
                assert r['explicit_sync_calls'] == 8 and r['rename_calls'] == int(not r['already_renamed'])
                assert r['output_journal_commits'] == 1 and r['cleanup_journal_commits'] == 2
                assert set(r['phases']) == (set(previous.PHASES) if r['recording'] else set())
                if r['already_renamed'] or not r['recording']:
                    assert r['rename_ms'] == r['rename_cpu_ms'] == 0
                if run['kind'] == 'controls':
                    assert not r['already_renamed']
                else:
                    assert r['recording']
            metrics(r, run['kind'])
            modes.setdefault(mode, []).append(r)
    medians = {m: {k: st.median(metrics(r, run['kind'])[k] for r in rows)
                   for k in metrics(rows[0], run['kind'])} for m, rows in modes.items()}
    return modes, medians


def load_run(root, run):
    modes = {}
    for binary, name in run['files'].items():
        if run['kind'] in ['normal', 'recovery']:
            single = dict(run, files={binary: name})
            old, _ = load_existing_run(root, single)
            modes.update({(binary + '_' + mode if run['kind'] == 'recovery' else mode): rows for mode, rows in old.items()})
            continue
        path = root / name
        assert digest(path) == run['sha256'][name]
        rows = json.loads(path.read_text())
        assert len(rows) == 2 * run['pairs']
        controls = run['kind'] == 'controls'
        for pair in range(run['pairs']):
            group = rows[2 * pair:2 * pair + 2]
            assert [r['pair'] for r in group] == [pair, pair]
            field = 'recording' if controls else 'uncertain_acknowledgment'
            assert [r[field] for r in group] == ([False, True] if pair % 2 == 0 else [True, False])
        for r in rows:
            assert r['controls'] == controls and r['fixture'] == run['fixture']
            assert not r['counters'] and not r['regions']
            logical = (64 if run['fixture'] == 'stored_64' else 256) << 20
            assert r['logical_bytes_verified'] == r['logical_bytes'] == logical
            assert r['initial_allocated_bytes'] == r['recovered_allocated_bytes'] == r['output_allocated_bytes'] == 8 << 20
            assert r['explicit_sync_calls'] == 8 and r['rename_calls'] == r['output_journal_commits'] == 0
            assert r['cleanup_journal_commits'] == 2
            for flag in ['oracle_passed', 'source_unchanged', 'allocation_unchanged', 'cleanup_passed', 'journals_unchanged']:
                assert r[flag]
            assert set(r['phases']) == (set(PHASES) if r['recording'] else set())
            if controls:
                assert not r['uncertain_acknowledgment']
                mode = 'enabled' if r['recording'] else 'disabled'
            else:
                assert r['recording']
                mode = 'uncertain' if r['uncertain_acknowledgment'] else 'acknowledged'
            metrics(r, run['kind'])
            modes.setdefault(mode, []).append(r)
    medians = {m: {k: st.median(metrics(r, run['kind'])[k] for r in rows) for k in metrics(rows[0], run['kind'])} for m, rows in modes.items()}
    return modes, medians


def reasons(runs):
    result = []
    first = runs[0]
    for mode, values in first['medians'].items():
        for metric in values:
            measured = spread([r['medians'][mode][metric] for r in runs])
            if measured is None:
                result.append(f'{mode}:{metric}:resolution_limited')
            elif measured > 5:
                result.append(f'{mode}:{metric}:spread')
    if first['kind'] == 'normal':
        pairs = [('reference_' + m, 'candidate_' + m) for m in ['owned', 'published']]
    elif first['kind'] == 'recovery':
        pairs = [('reference_' + m, 'candidate_' + m) for m in ['staged', 'already_renamed']]
    elif first['kind'] == 'controls':
        pairs = [('disabled', 'enabled')]
    else:
        pairs = []
    for baseline, candidate in pairs:
        for key in ['wall_ms', 'cpu_ms']:
            if any(r['medians'][candidate][key] / r['medians'][baseline][key] > 1.05 for r in runs):
                label = (f'candidate/reference:{candidate.removeprefix("candidate_")}'
                         if first['kind'] == 'recovery' else f'{candidate}/{baseline}')
                result.append(f'{label}:{key}:adverse')
    return result


def summarize(root):
    runs = json.loads((root / 'measurements.json').read_text())
    decisions = json.loads((root / 'repeat-decisions.json').read_text())
    groups = {}; operations = 0
    for r in runs:
        modes, medians = load_run(root, r)
        assert medians == r['medians']
        expected = {'reference', 'candidate'} if r['kind'] in ['normal', 'recovery'] else {'candidate'}
        assert set(r['files']) == expected
        operations += sum(map(len, modes.values()))
        groups.setdefault(r['group'], []).append(r)
    assert len(groups) == 14 and set(groups) == set(decisions)
    summary = dict(operations=operations, groups={})
    for group, entries in groups.items():
        initial = [r for r in entries if r['phase'] == 'initial']
        repeat = [r for r in entries if r['phase'] == 'repeat']
        assert [r['round'] for r in initial] == [1, 2, 3]
        assert all(r['pairs'] == 4 for r in initial)
        assert decisions[group]['reasons'] == reasons(initial)
        assert [r['round'] for r in repeat] == ([1, 2, 3] if reasons(initial) else [])
        assert all(r['pairs'] == 8 for r in repeat)
        out = summary['groups'][group] = dict(kind=entries[0]['kind'], filesystem=entries[0]['filesystem'], fixture=entries[0]['fixture'], phases={})
        for phase, selected in [('initial', initial), ('repeat', repeat)]:
            if not selected:
                continue
            item = out['phases'][phase] = dict(modes={}, unresolved=reasons(selected))
            for mode in selected[0]['medians']:
                rows = [row for r in selected for row in load_run(root, r)[0][mode]]
                item['modes'][mode] = dict(metrics={k: dict(median=st.median(r['medians'][mode][k] for r in selected), spread_percent=spread([r['medians'][mode][k] for r in selected]), zero_resolution_samples=sum(metrics(row, entries[0]['kind'])[k] == 0 for row in rows)) for k in selected[0]['medians'][mode]}, peak_rss_kib=max(r['process_peak_rss_kib'] for r in rows), allocation_bytes=sorted({r['output_allocated_bytes'] for r in rows}))
    write(root / 'summary.json', summary)
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['reference', 'candidate', 'btrfs-parent', 'tmpfs-parent']:
        parser.add_argument('--' + name, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    parser.add_argument('--cpu', type=int, default=0)
    parser.add_argument('--summarize-only', action='store_true')
    parser.add_argument('--resume', action='store_true', help='Validate and retain complete existing observations; execute only missing rounds')
    a = parser.parse_args()
    if a.summarize_only:
        print(json.dumps(summarize(a.report), indent=2)); return
    assert a.cpu in os.sched_getaffinity(0)
    assert a.resume == (a.report / 'measurements.json').exists()
    binaries = {'reference': a.reference.resolve(), 'candidate': a.candidate.resolve()}
    parents = {'btrfs': a.btrfs_parent.resolve(), 'tmpfs': a.tmpfs_parent.resolve()}
    for fs, p in parents.items():
        p.mkdir(parents=True, exist_ok=True)
        assert subprocess.check_output(['stat', '-f', '-c', '%T', str(p)], text=True).strip() == fs
    a.report.mkdir(parents=True, exist_ok=True)
    environment = dict(binary_sha256={k: digest(v) for k, v in binaries.items()}, scripts={p.name: digest(p) for p in [Path(__file__), Path(previous.__file__)]}, kernel=platform.release(), cpu=a.cpu, affinity=sorted(os.sched_getaffinity(0)), lscpu=subprocess.check_output(['lscpu'], text=True), mounts={fs: subprocess.check_output(['findmnt', '-n', '-o', 'FSTYPE,OPTIONS', '-T', str(p)], text=True).strip() for fs, p in parents.items()}, conditions='Serial workloads; ordinary caches; fixed affinity; frequency/SMT/competing host work not isolated. Private tmpfs is volatile. Setup/oracles excluded, lifetime RSS includes setup and snapshots. No VMware access.')
    runs = []; decisions = {}; commands = []
    if a.resume:
        original = json.loads((a.report / 'environment.json').read_text())
        for key in ['binary_sha256', 'kernel', 'cpu', 'affinity', 'mounts']:
            assert original[key] == environment[key], key
        runs = json.loads((a.report / 'measurements.json').read_text())
        decisions = json.loads((a.report / 'repeat-decisions.json').read_text())
        commands = json.loads((a.report / 'commands.json').read_text())
        for run in runs:
            assert load_run(a.report, run)[1] == run['medians']
        write(a.report / 'resume.json', dict(unix_seconds=time.time(), previous_scripts=original['scripts'], current_scripts=environment['scripts'], retained_rounds=len(runs), reason='Accept zero resolution-limited CPU phases without modifying samples; preserve all completed work and execute only missing rounds.', preserved_numeric_sha256={p.name: digest(p) for p in a.report.glob('*-candidate.json')} | {p.name: digest(p) for p in a.report.glob('*-reference.json')}))
    write(a.report / 'environment.json', environment)
    def snapshot():
        base = Path(f'/sys/devices/system/cpu/cpu{a.cpu}/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(), frequency={k: (base / k).read_text().strip() for k in ['scaling_cur_freq', 'scaling_governor'] if (base / k).exists()})
    def run(kind, fs, fixture, phase, number):
        group = f'{kind}-{fs}-{fixture}'; label = f'{group}-{phase}-{number}'
        if any(r['group'] == group and r['phase'] == phase and r['round'] == number for r in runs):
            return
        entry = dict(group=group, kind=kind, filesystem=fs, fixture=fixture, phase=phase, round=number, pairs=4 if phase == 'initial' else 8, files={}, sha256={})
        selected = (['reference', 'candidate'] if number % 2 else ['candidate', 'reference']) if kind in ['normal', 'recovery'] else ['candidate']
        for binary in selected:
            name = label + '-' + binary + '.json'; output = a.report / name
            if output.exists():
                assert a.resume and any(c['group'] == group and c['phase'] == phase and c['round'] == number and c['binary'] == binary and c['exit_code'] == 0 for c in commands)
                entry['files'][binary] = name; entry['sha256'][name] = digest(output)
                continue
            env = dict(os.environ, RVVDK_RETAINED_PAIRS=str(entry['pairs']), RVVDK_RETAINED_PARENT=str(parents[fs]), RVVDK_RETAINED_REPORT=str(output.resolve()), RVVDK_PROFILE_FIXTURE=fixture, RVVDK_RECONCILE_CONTROLS='0', RVVDK_RECONCILE_COUNTERS='0', RVVDK_ATTEST_CONTROLS='1' if kind == 'controls' else '0', RVVDK_ATTEST_COUNTERS='0', RVVDK_ATTEST_AUDIT='0')
            test = previous.NORMAL if kind == 'normal' else previous.RECOVERY if kind == 'recovery' else ATTEST
            command = ['taskset', '-c', str(a.cpu), str(binaries[binary]), '--exact', test, '--ignored', '--test-threads=1']
            observation = dict(group=group, phase=phase, round=number, binary=binary, command=command, before=snapshot())
            log = a.report / (label + '-' + binary + '.txt')
            with log.open('w') as handle:
                result = subprocess.run(command, env=env, stdout=handle, stderr=subprocess.STDOUT, timeout=900)
            log.write_text(log.read_text().rstrip() + '\n')
            observation.update(after=snapshot(), exit_code=result.returncode)
            commands.append(observation); write(a.report / 'commands.json', commands)
            assert result.returncode == 0, str(log)
            entry['files'][binary] = name; entry['sha256'][name] = digest(output)
        entry['medians'] = load_run(a.report, entry)[1]
        runs.append(entry); write(a.report / 'measurements.json', runs)
        print(label, {m: round(v['wall_ms'], 3) for m, v in entry['medians'].items()}, flush=True)
    for kind in ['normal', 'recovery', 'attestation', 'controls']:
        for fs in parents:
            for fixture in (['stored_64'] if kind == 'normal' else ['stored_64', 'deflated_256']):
                group = f'{kind}-{fs}-{fixture}'
                for n in range(1, 4): run(kind, fs, fixture, 'initial', n)
                why = reasons([r for r in runs if r['group'] == group and r['phase'] == 'initial'])
                if group in decisions:
                    assert decisions[group]['reasons'] == why
                else:
                    decisions[group] = dict(reasons=why, decided_unix_seconds=time.time())
                    write(a.report / 'repeat-decisions.json', decisions)
                if decisions[group]['reasons']:
                    for n in range(1, 4): run(kind, fs, fixture, 'repeat', n)
    print('Complete:', summarize(a.report)['operations'], 'operations', flush=True)


if __name__ == '__main__':
    main()
