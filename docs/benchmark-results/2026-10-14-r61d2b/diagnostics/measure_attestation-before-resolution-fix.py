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
    if kind in ['normal', 'recovery']:
        return previous.metrics(row, kind)
    result = {k: row[k] for k in previous.BASE}
    for phase, values in row['phases'].items():
        for metric, value in values.items():
            result[f'{phase}.{metric}'] = value
    assert all(math.isfinite(v) and v > 0 for v in result.values())
    return result


def load_run(root, run):
    modes = {}
    for binary, name in run['files'].items():
        if run['kind'] in ['normal', 'recovery']:
            single = dict(run, files={binary: name})
            old, _ = previous.load_run(root, single)
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
    result = previous.reasons(runs)
    if runs[0]['kind'] == 'recovery':
        for mode in ['staged', 'already_renamed']:
            for metric in ['wall_ms', 'cpu_ms']:
                if any(r['medians']['candidate_' + mode][metric] / r['medians']['reference_' + mode][metric] > 1.05 for r in runs):
                    result.append(f'candidate/reference:{mode}:{metric}:adverse')
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
                item['modes'][mode] = dict(metrics={k: dict(median=st.median(r['medians'][mode][k] for r in selected), spread_percent=100 * (max(r['medians'][mode][k] for r in selected) / min(r['medians'][mode][k] for r in selected) - 1)) for k in selected[0]['medians'][mode]}, peak_rss_kib=max(r['process_peak_rss_kib'] for r in rows), allocation_bytes=sorted({r['output_allocated_bytes'] for r in rows}))
    write(root / 'summary.json', summary)
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['reference', 'candidate', 'btrfs-parent', 'tmpfs-parent']:
        parser.add_argument('--' + name, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    parser.add_argument('--cpu', type=int, default=0)
    parser.add_argument('--summarize-only', action='store_true')
    a = parser.parse_args()
    if a.summarize_only:
        print(json.dumps(summarize(a.report), indent=2)); return
    assert a.cpu in os.sched_getaffinity(0)
    assert not (a.report / 'measurements.json').exists()
    binaries = {'reference': a.reference.resolve(), 'candidate': a.candidate.resolve()}
    parents = {'btrfs': a.btrfs_parent.resolve(), 'tmpfs': a.tmpfs_parent.resolve()}
    for fs, p in parents.items():
        p.mkdir(parents=True, exist_ok=True)
        assert subprocess.check_output(['stat', '-f', '-c', '%T', str(p)], text=True).strip() == fs
    a.report.mkdir(parents=True, exist_ok=True)
    write(a.report / 'environment.json', dict(binary_sha256={k: digest(v) for k, v in binaries.items()}, scripts={p.name: digest(p) for p in [Path(__file__), Path(previous.__file__)]}, kernel=platform.release(), cpu=a.cpu, affinity=sorted(os.sched_getaffinity(0)), lscpu=subprocess.check_output(['lscpu'], text=True), mounts={fs: subprocess.check_output(['findmnt', '-n', '-o', 'FSTYPE,OPTIONS', '-T', str(p)], text=True).strip() for fs, p in parents.items()}, conditions='Serial workloads; ordinary caches; fixed affinity; frequency/SMT/competing host work not isolated. Private tmpfs is volatile. Setup/oracles excluded, lifetime RSS includes setup and snapshots. No VMware access.'))
    runs = []; decisions = {}; commands = []
    def snapshot():
        base = Path(f'/sys/devices/system/cpu/cpu{a.cpu}/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(), frequency={k: (base / k).read_text().strip() for k in ['scaling_cur_freq', 'scaling_governor'] if (base / k).exists()})
    def run(kind, fs, fixture, phase, number):
        group = f'{kind}-{fs}-{fixture}'; label = f'{group}-{phase}-{number}'
        entry = dict(group=group, kind=kind, filesystem=fs, fixture=fixture, phase=phase, round=number, pairs=4 if phase == 'initial' else 8, files={}, sha256={})
        selected = (['reference', 'candidate'] if number % 2 else ['candidate', 'reference']) if kind in ['normal', 'recovery'] else ['candidate']
        for binary in selected:
            name = label + '-' + binary + '.json'; output = a.report / name
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
                decisions[group] = dict(reasons=reasons([r for r in runs if r['group'] == group]), decided_unix_seconds=time.time())
                write(a.report / 'repeat-decisions.json', decisions)
                if decisions[group]['reasons']:
                    for n in range(1, 4): run(kind, fs, fixture, 'repeat', n)
    print('Complete:', summarize(a.report)['operations'], 'operations', flush=True)


if __name__ == '__main__':
    main()
