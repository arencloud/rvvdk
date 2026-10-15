#!/usr/bin/env python3
"""PERF.1c frozen-binary comparisons; authored local/loopback fixtures only."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics as st
import subprocess
import tempfile
import time

TESTS = {
    'transfer': ('integration', 'export_tests::owned_transfer::benchmark::matched_owned_transfer', 'TRANSFER'),
    'artifact': ('integration', 'export_tests::owned_artifact::benchmark::matched_owned_artifact', 'ARTIFACT'),
    'worker': ('library', 'journal_worker::admission::tests::benchmark::worker_hash_phases', 'WORKER'),
}
SCHEDULE = [('initial', 2), ('repeat', 4)]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


def groups():
    return [(kind, 'disk', case) for kind in TESTS for case in ['small', 'bulk', 'zeros']] + [
        (kind, 'tmpfs', 'bulk') for kind in TESTS]


def operation(kind, row):
    if kind == 'transfer':
        return 'owned-transfer' if row['owned'] else 'proof-export'
    if kind == 'artifact':
        return 'artifact-admission' if row['admitted'] else 'owned-export'
    return 'worker-' + row['phase']


def validate_run(run):
    kind, _, case = run['group'].split('/')
    pairs = dict(SCHEDULE)[run['stage']]
    assert run['round'] in [1, 2, 3]
    assert run['pairs'] == pairs and set(run['versions']) == {'reference', 'candidate'}
    all_ops = set()
    for version, rows in run['versions'].items():
        assert len(rows) == pairs * 2
        seen = set()
        for row in rows:
            assert row['case'] == case and row['byte_comparison_passed'] is True
            op = operation(kind, row)
            assert (row['pair'], op) not in seen and row['pair'] in range(pairs)
            seen.add((row['pair'], op)); all_ops.add(op)
            for key in ['wall_ms', 'cpu_ms', 'process_peak_rss_kib', 'payload_bytes', 'payload_allocated_bytes']:
                assert isinstance(row[key], (int, float)) and math.isfinite(row[key]) and row[key] >= 0
            assert row['wall_ms'] > 0 and row['process_peak_rss_kib'] > 0
            size = row['payload_bytes']
            assert size == (65536 if case == 'small' else 8 << 20) if kind == 'transfer' else (
                65536 < size < 200000 if case == 'small' else 8 << 20 < size < 9 << 20)
            assert row['payload_allocated_bytes'] >= size
            if kind != 'worker':
                owned = row.get('owned', True)
                assert row['journal_commits'] == (8 if owned else 0)
                assert row['requests'] > 0
                assert row['cleanup_ms'] >= 0
            if kind == 'artifact':
                assert row['decoded_oracle_passed'] is True
                assert row['present_grains_verified'] == ((1 if case == 'small' else 128) if row['admitted'] else 0)
        assert len({op for _, op in seen}) == 2
    assert len(all_ops) == 2


def summarize(report):
    index = json.loads((report / 'index.json').read_text())
    for name, expected in index['sha256'].items():
        assert digest(report / name) == expected, name
    runs = json.loads((report / 'measurements.json').read_text())
    expected = {(f'{k}/{fs}/{case}', stage, rnd) for k, fs, case in groups()
                for stage, _ in SCHEDULE for rnd in [1, 2, 3]}
    assert len(runs) == len(expected) == 72
    assert {(r['group'], r['stage'], r['round']) for r in runs} == expected
    collected = {}
    count = 0
    for run in runs:
        validate_run(run)
        for version, rows in run['versions'].items():
            count += len(rows)
            for op in {operation(run['group'].split('/')[0], row) for row in rows}:
                subset = [r for r in rows if operation(run['group'].split('/')[0], r) == op]
                name = run['group'] + '/' + op
                metrics = ['wall_ms', 'cpu_ms', 'process_peak_rss_kib', 'payload_bytes', 'payload_allocated_bytes']
                if 'cleanup_ms' in subset[0]: metrics.append('cleanup_ms')
                entry = collected.setdefault(name, {}).setdefault(run['stage'], {}).setdefault(version, {})
                for key in metrics:
                    entry.setdefault(key, []).append(st.median(r[key] for r in subset))
    assert count == 864 and len(collected) == 24
    startup = json.loads((report / 'startup.json').read_text())
    assert len(startup) == 72
    for stage, pairs in [('initial', 4), ('repeat', 8)]:
        for rnd in [1, 2, 3]:
            rows = [r for r in startup if r['stage'] == stage and r['round'] == rnd]
            assert len(rows) == pairs * 2
            assert {(r['pair'], r['version']) for r in rows} == {(p, v) for p in range(pairs) for v in ['reference', 'candidate']}
            for version in ['reference', 'candidate']:
                subset = [r for r in rows if r['version'] == version]
                assert all(r['exit_code'] == 0 and r['wall_ms'] > 0 and r['cpu_ms'] >= 0 for r in subset)
                entry = collected.setdefault('startup', {}).setdefault(stage, {}).setdefault(version, {})
                for key in ['wall_ms', 'cpu_ms', 'process_peak_rss_kib']:
                    entry.setdefault(key, []).append(st.median(r[key] for r in subset))
    # Clap prints argv[0]; frozen reference/candidate filenames differ.
    for version in ['reference', 'candidate']:
        assert len({r['stdout_sha256'] for r in startup if r['version'] == version}) == 1
    summary = {'timed_observations': count + len(startup), 'groups': {}}
    for group, stages in collected.items():
        summary['groups'][group] = {}
        for stage, versions in stages.items():
            output = {}
            for version, metrics in versions.items():
                output[version] = {key: {'median': st.median(values), 'round_medians': values,
                    'spread_percent': (max(values) - min(values)) / st.median(values) * 100 if st.median(values) else 0}
                    for key, values in metrics.items()}
            output['change_percent'] = {key: (output['candidate'][key]['median'] / metric['median'] - 1) * 100
                for key, metric in output['reference'].items() if metric['median']}
            summary['groups'][group][stage] = output
    write(report / 'summary.json', summary)
    return summary


def environment(binaries, parents, cpu):
    info = Path('/proc/cpuinfo').read_text().split('\n\n')[0]
    fields = dict(line.split(':', 1) for line in info.splitlines() if ':' in line)
    fields = {k.strip(): v.strip() for k, v in fields.items()}
    return {'kernel': platform.release(), 'architecture': platform.machine(),
        'glibc': list(platform.libc_ver()), 'cpu_model': fields['model name'],
        'flags': sorted(fields['flags'].split()), 'affinity': [cpu], 'visible_cpus': os.cpu_count(),
        'memory_kib': int(Path('/proc/meminfo').read_text().splitlines()[0].split()[1]),
        'binary_sha256': {name: digest(path) for name, path in binaries.items()},
        'binary_bytes': {name: path.stat().st_size for name, path in binaries.items()},
        'filesystems': {name: subprocess.check_output(['stat', '-f', '-c', '%T', path], text=True).strip() for name, path in parents.items()},
        'tmpfs_options': ','.join(v for v in subprocess.check_output(['findmnt', '-n', '-o', 'OPTIONS', '-T', parents['tmpfs']], text=True).strip().split(',') if not v.startswith(('uid=', 'gid='))),
        'shmem_huge_policy': Path('/sys/kernel/mm/transparent_hugepage/shmem_enabled').read_text().strip(),
        'script_sha256': digest(__file__), 'conditions': 'CPU 0; serial; normal page cache; local TLS TCP_NODELAY; setup/oracles outside operation timers. Private tmpfs is volatile. Frequency/SMT/host activity not isolated. RSS is process lifetime including setup; startup child RSS can include inherited launcher memory.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--summarize', action='store_true')
    for name in ['reference-library', 'candidate-library', 'reference-integration', 'candidate-integration', 'reference-cli', 'candidate-cli', 'disk-parent', 'tmpfs-parent', 'private']:
        parser.add_argument('--' + name, type=Path)
    a = parser.parse_args(); a.report = a.report.resolve()
    if a.summarize:
        summarize(a.report); return
    binaries = {f'{v}-{kind}': getattr(a, f'{v}_{kind}').resolve() for v in ['reference', 'candidate'] for kind in ['library', 'integration', 'cli']}
    parents = {'disk': a.disk_parent.resolve(), 'tmpfs': a.tmpfs_parent.resolve()}
    a.private = a.private.resolve(); a.private.mkdir(parents=True, exist_ok=True)
    assert not (a.report / 'measurements.json').exists()
    a.report.mkdir(parents=True, exist_ok=True)
    assert 0 in os.sched_getaffinity(0)
    os.sched_setaffinity(0, {0})
    for path in parents.values(): assert not list(path.iterdir())
    env_before = environment(binaries, parents, 0)
    assert env_before['filesystems']['tmpfs'] == 'tmpfs'
    # Linux can omit huge=never from mount options because it is the default.
    assert 'huge=never' in env_before['tmpfs_options'] or (
        'huge=' not in env_before['tmpfs_options'] and '[never]' in env_before['shmem_huge_policy'])
    write(a.report / 'environment.json', env_before)
    runs, logs = [], []
    for kind, fs, case in groups():
        binary_kind, test, prefix = TESTS[kind]
        for stage, pairs in SCHEDULE:
            for rnd in [1, 2, 3]:
                run = {'group': f'{kind}/{fs}/{case}', 'stage': stage, 'round': rnd, 'pairs': pairs, 'versions': {}}
                for version in (['reference', 'candidate'] if rnd % 2 else ['candidate', 'reference']):
                    out = a.private / 'current.json'
                    assert not out.exists()
                    env = os.environ | {'RVVDK_HASH_CASE': case, f'RVVDK_{prefix}_PAIRS': str(pairs), f'RVVDK_{prefix}_PARENT': str(parents[fs]), f'RVVDK_{prefix}_REPORT': str(out)}
                    result = subprocess.run([str(binaries[version + '-' + binary_kind]), '--exact', test, '--ignored', '--test-threads=1'], env=env, capture_output=True, text=True, timeout=180)
                    logs.append({'group': run['group'], 'stage': stage, 'round': rnd, 'version': version, 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
                    write(a.report / 'invocations.json', logs)
                    assert result.returncode == 0, result.stdout + result.stderr
                    assert '1 passed; 0 failed;' in result.stdout
                    run['versions'][version] = json.loads(out.read_text()); out.unlink()
                    assert not list(parents[fs].iterdir())
                validate_run(run); runs.append(run)
                write(a.report / 'measurements.json', runs)
                print(run['group'], stage, rnd, flush=True)
    startup = []
    for stage, pairs in [('initial', 4), ('repeat', 8)]:
        for rnd in [1, 2, 3]:
            for pair in range(pairs):
                for version in (['reference', 'candidate'] if (rnd + pair) % 2 else ['candidate', 'reference']):
                    with tempfile.TemporaryFile() as output:
                        start = time.perf_counter_ns()
                        child = subprocess.Popen([str(binaries[version + '-cli']), '--help'], stdout=output, stderr=output)
                        _, status, usage = os.wait4(child.pid, 0)
                        wall_ms = (time.perf_counter_ns() - start) / 1e6
                        child.returncode = os.waitstatus_to_exitcode(status)
                        assert child.returncode == 0
                        output.seek(0); stdout = output.read()
                    startup.append({'stage': stage, 'round': rnd, 'pair': pair, 'version': version, 'wall_ms': wall_ms, 'cpu_ms': (usage.ru_utime + usage.ru_stime) * 1000, 'process_peak_rss_kib': usage.ru_maxrss, 'exit_code': child.returncode, 'stdout_sha256': hashlib.sha256(stdout).hexdigest()})
    write(a.report / 'startup.json', startup)
    env_after = environment(binaries, parents, 0)
    assert env_before == env_after
    assert all(not list(p.iterdir()) for p in parents.values())
    write(a.report / 'completion.json', {'environment_and_binary_hashes_unchanged': True, 'fixture_parents_empty': True})
    write(a.report / 'index.json', {'sha256': {name: digest(a.report / name) for name in ['environment.json', 'invocations.json', 'measurements.json', 'startup.json', 'completion.json']}})
    summarize(a.report)


if __name__ == '__main__':
    main()
