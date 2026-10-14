#!/usr/bin/env python3
"""Offline Rust reconciliation qualification: frozen normal path, branches and clock controls."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics as st
import subprocess
import time

NORMAL = 'ownership::retained::output::publication::tests::benchmark::matched_publication'
RECOVERY = 'ownership::retained::output::publication::reconciliation::tests::benchmark::recovery_branches'
BASE = ['wall_ms', 'cpu_ms', 'cleanup_ms', 'cleanup_cpu_ms']
PHASES = ['admission', 'verification', 'bundle_sync', 'intent_sync', 'rename_and_checks',
          'parent_sync', 'acknowledgment', 'assessment_and_release']


def write(path, obj):
    path.write_text(json.dumps(obj, indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def metrics(row, kind):
    result = {k: row[k] for k in BASE}
    if kind == 'normal':
        for key in ['admission_ms', 'conversion_and_checks_ms', 'publication_admission_ms', 'publication_ms']:
            if row[key] > 0:
                result[key] = row[key]
    else:
        for phase, values in row['phases'].items():
            for metric, value in values.items():
                result[f'{phase}.{metric}'] = value
        if row['recording'] and not row['already_renamed']:
            result['rename_ms'] = row['rename_ms']
            result['rename_cpu_ms'] = row['rename_cpu_ms']
    assert all(math.isfinite(v) and v > 0 for v in result.values()), result
    return result


def load_run(root, run):
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
                assert set(r['phases']) == (set(PHASES) if r['recording'] else set())
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


def reasons(runs):
    first = runs[0]
    found = []
    for mode, values in first['medians'].items():
        for metric in values:
            samples = [r['medians'][mode][metric] for r in runs]
            if max(samples) / min(samples) > 1.05:
                found.append(f'{mode}:{metric}:spread')
    pairs = []
    if first['kind'] == 'normal':
        pairs = [('reference_' + m, 'candidate_' + m) for m in ['owned', 'published']]
    elif first['kind'] == 'controls':
        pairs = [('disabled', 'enabled')]
    for baseline, candidate in pairs:
        for key in ['wall_ms', 'cpu_ms']:
            if any(r['medians'][candidate][key] / r['medians'][baseline][key] > 1.05 for r in runs):
                found.append(f'{candidate}/{baseline}:{key}:adverse')
    return found


def summarize(root):
    runs = json.loads((root / 'measurements.json').read_text())
    decisions = json.loads((root / 'repeat-decisions.json').read_text())
    groups = {}
    operations = 0
    for run in runs:
        modes, medians = load_run(root, run)
        assert medians == run['medians']
        operations += sum(map(len, modes.values()))
        groups.setdefault(run['group'], []).append(run)
    assert len(groups) == 10 and set(groups) == set(decisions)
    summary = dict(operations=operations, groups={})
    for group, entries in groups.items():
        initial = [r for r in entries if r['phase'] == 'initial']
        assert [r['round'] for r in initial] == [1, 2, 3]
        assert all(r['pairs'] == 4 for r in initial)
        assert decisions[group]['reasons'] == reasons(initial)
        repeat = [r for r in entries if r['phase'] == 'repeat']
        assert [r['round'] for r in repeat] == ([1, 2, 3] if reasons(initial) else [])
        assert all(r['pairs'] == 8 for r in repeat)
        result = summary['groups'][group] = dict(kind=entries[0]['kind'], filesystem=entries[0]['filesystem'], fixture=entries[0]['fixture'], phases={})
        for phase, selected in [('initial', initial), ('repeat', repeat)]:
            if not selected:
                continue
            out = result['phases'][phase] = dict(modes={}, unresolved=reasons(selected))
            for mode in selected[0]['medians']:
                rows = [r for run in selected for r in load_run(root, run)[0][mode]]
                out['modes'][mode] = dict(metrics={k: dict(median=st.median(run['medians'][mode][k] for run in selected),
                    spread_percent=100 * (max(run['medians'][mode][k] for run in selected) / min(run['medians'][mode][k] for run in selected) - 1))
                    for k in selected[0]['medians'][mode]}, peak_rss_kib=max(r['process_peak_rss_kib'] for r in rows),
                    allocation_bytes=sorted({r['output_allocated_bytes'] for r in rows}))
    write(root / 'summary.json', summary)
    return summary


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--reference', type=Path)
    p.add_argument('--candidate', type=Path)
    p.add_argument('--report', required=True, type=Path)
    p.add_argument('--btrfs-parent', type=Path)
    p.add_argument('--tmpfs-parent', type=Path)
    p.add_argument('--cpu', default=0, type=int)
    p.add_argument('--summarize-only', action='store_true')
    a = p.parse_args()
    if a.summarize_only:
        print(json.dumps(summarize(a.report), indent=2)); return
    assert a.cpu in os.sched_getaffinity(0)
    assert not (a.report / 'measurements.json').exists()
    binaries = {'reference': a.reference.resolve(), 'candidate': a.candidate.resolve()}
    parents = {'btrfs': a.btrfs_parent.resolve(), 'tmpfs': a.tmpfs_parent.resolve()}
    for fs, path in parents.items():
        path.mkdir(exist_ok=True, parents=True)
        assert subprocess.check_output(['stat', '-f', '-c', '%T', str(path)], text=True).strip() == fs
    a.report.mkdir(exist_ok=True, parents=True)
    env = dict(binary_sha256={k: digest(v) for k, v in binaries.items()}, harness_sha256=digest(Path(__file__)),
               kernel=platform.release(), cpu=a.cpu, affinity=sorted(os.sched_getaffinity(0)),
               lscpu=subprocess.check_output(['lscpu'], text=True),
               mounts={fs: subprocess.check_output(['findmnt', '-n', '-o', 'FSTYPE,OPTIONS', '-T', str(path)], text=True).strip() for fs, path in parents.items()},
               conditions='One workload at a time, ordinary caches, fixed CPU affinity; frequency/SMT and competing host work not isolated. tmpfs is volatile. Fixture setup and independent oracles excluded from operation timing; lifetime RSS includes setup. Hook clocks measured by separate same-binary disabled/enabled controls. No VMware access.')
    write(a.report / 'environment.json', env)
    runs, decisions, commands = [], {}, []
    def snapshot():
        base = Path(f'/sys/devices/system/cpu/cpu{a.cpu}/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(), frequency={k: (base / k).read_text().strip() for k in ['scaling_cur_freq', 'scaling_governor'] if (base / k).exists()})
    def run(kind, fs, fixture, phase, number):
        group = f'{kind}-{fs}-{fixture}'
        label = f'{group}-{phase}-{number}'
        entry = dict(group=group, kind=kind, filesystem=fs, fixture=fixture, phase=phase, round=number,
                     pairs=4 if phase == 'initial' else 8, files={}, sha256={})
        for binary in (['reference', 'candidate'] if number % 2 else ['candidate', 'reference']) if kind == 'normal' else ['candidate']:
            name = label + '-' + binary + '.json'
            output = a.report / name
            process_env = dict(os.environ, RVVDK_RETAINED_PAIRS=str(entry['pairs']), RVVDK_RETAINED_PARENT=str(parents[fs]),
                               RVVDK_RETAINED_REPORT=str(output.resolve()), RVVDK_PROFILE_FIXTURE=fixture,
                               RVVDK_RECONCILE_CONTROLS='1' if kind == 'controls' else '0', RVVDK_RECONCILE_COUNTERS='0')
            command = ['taskset', '-c', str(a.cpu), str(binaries[binary]), '--exact', NORMAL if kind == 'normal' else RECOVERY, '--ignored', '--test-threads=1']
            observation = dict(group=group, phase=phase, round=number, binary=binary, command=command, before=snapshot())
            log = a.report / (label + '-' + binary + '.txt')
            with log.open('w') as handle:
                result = subprocess.run(command, env=process_env, stdout=handle, stderr=subprocess.STDOUT, timeout=900)
            log.write_text(log.read_text().rstrip() + '\n')
            observation.update(after=snapshot(), exit_code=result.returncode)
            commands.append(observation); write(a.report / 'commands.json', commands)
            assert result.returncode == 0, str(log)
            entry['files'][binary] = name; entry['sha256'][name] = digest(output)
        entry['medians'] = load_run(a.report, entry)[1]
        runs.append(entry); write(a.report / 'measurements.json', runs)
        print(label, {m: round(v['wall_ms'], 3) for m, v in entry['medians'].items()}, flush=True)
    groups = [(kind, fs, fixture) for kind in ['normal', 'recovery', 'controls'] for fs in parents
              for fixture in (['stored_64'] if kind == 'normal' else ['stored_64', 'deflated_256'])]
    for kind, fs, fixture in groups:
        group = f'{kind}-{fs}-{fixture}'
        for n in range(1, 4):
            run(kind, fs, fixture, 'initial', n)
        decisions[group] = dict(reasons=reasons([r for r in runs if r['group'] == group]), decided_unix_seconds=time.time())
        write(a.report / 'repeat-decisions.json', decisions)
        if decisions[group]['reasons']:
            for n in range(1, 4):
                run(kind, fs, fixture, 'repeat', n)
    print('Complete:', summarize(a.report)['operations'], 'operations', flush=True)


if __name__ == '__main__':
    main()
