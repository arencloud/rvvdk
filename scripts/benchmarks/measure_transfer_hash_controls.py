#!/usr/bin/env python3
"""Predeclared PERF.1c SHA-NI A/A and interleaved A/B CPU controls."""
import argparse
import json
import os
from pathlib import Path
import statistics as st
import subprocess
from measure_transfer_hashes import TESTS, digest, operation, validate_run, write

CASES = {'sha-host': [('transfer', 'small'), ('transfer', 'bulk'), ('worker', 'small')],
         'non-sha-host': [('artifact', 'small')]}


def summarize(report):
    index = json.loads((report / 'control-index.json').read_text())
    for name, expected in index.items(): assert digest(report / name) == expected
    runs = json.loads((report / 'controls.json').read_text())
    cases = CASES.get(json.loads((report / 'control-environment.json').read_text()).get('profile', 'sha-host'))
    assert cases is not None
    assert len(runs) == len(cases) * 6
    assert {(r['group'], r['round'], r['mode']) for r in runs} == {
        (f'{kind}/disk/{case}', rnd, mode) for kind, case in cases for rnd in [1, 2, 3] for mode in ['AA', 'AB']}
    result = {}; count = 0
    for run in runs:
        kind = run['group'].split('/')[0]
        for version, rows in run['versions'].items():
            assert len(rows) == 16
            count += len(rows)
            assert {r['pair'] for r in rows} == set(range(8))
            assert all(r['actual_binary'] == ('reference' if run['mode'] == 'AA' else version) for r in rows)
        for start in range(0, 8, 2):
            chunk = {'group': run['group'], 'stage': 'initial', 'pairs': 2, 'round': run['round'],
                'versions': {v: [dict(row, pair=row['pair'] - start) for row in rows if start <= row['pair'] < start + 2]
                    for v, rows in run['versions'].items()}}
            validate_run(chunk)
        for version, rows in run['versions'].items():
            for op in {operation(kind, row) for row in rows}:
                subset = [r for r in rows if operation(kind, r) == op]
                entry = result.setdefault(run['group'] + '/' + op, {}).setdefault(run['mode'], {}).setdefault(version, {})
                for metric in ['wall_ms', 'cpu_ms']:
                    entry.setdefault(metric, []).append(st.median(r[metric] for r in subset))
    assert count == len(cases) * 192
    summary = {'timed_observations': count, 'groups': {}}
    for group, modes in result.items():
        summary['groups'][group] = {}
        for mode, versions in modes.items():
            item = {v: {m: {'median': st.median(values), 'round_medians': values,
                'spread_percent': (max(values) - min(values)) / st.median(values) * 100}
                for m, values in metrics.items()} for v, metrics in versions.items()}
            item['change_percent'] = {m: (item['candidate'][m]['median'] / item['reference'][m]['median'] - 1) * 100 for m in ['wall_ms', 'cpu_ms']}
            summary['groups'][group][mode] = item
    write(report / 'control-summary.json', summary)
    return summary


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--report', type=Path, required=True)
    p.add_argument('--root', type=Path)
    p.add_argument('--summarize', action='store_true')
    p.add_argument('--profile', choices=list(CASES), default='sha-host')
    a = p.parse_args(); report = a.report.resolve()
    if a.summarize: summarize(report); return
    root = a.root.resolve(); parent = root / 'fixtures'; private = root / 'private'
    assert not (report / 'controls.json').exists() and not list(parent.iterdir())
    os.sched_setaffinity(0, {0})
    binaries = {f'{v}-{k}': root / f'{v}-{k}' for v in ['reference', 'candidate'] for k in ['library', 'integration']}
    hashes = {n: digest(path) for n, path in binaries.items()}
    write(report / 'control-environment.json', {'profile': a.profile, 'binary_sha256': hashes, 'affinity': [0], 'script_sha256': digest(__file__), 'driver_sha256': digest(Path(__file__).with_name('measure_transfer_hashes.py')), 'filesystem': subprocess.check_output(['stat', '-f', '-c', '%T', parent], text=True).strip()})
    runs, logs = [], []
    for kind, case in CASES[a.profile]:
        binary_kind, test, prefix = TESTS[kind]
        for rnd in [1, 2, 3]:
            for mode in (['AA', 'AB'] if rnd % 2 else ['AB', 'AA']):
                run = {'group': f'{kind}/disk/{case}', 'round': rnd, 'mode': mode, 'versions': {'reference': [], 'candidate': []}}
                for pair in range(8):
                    for version in (['reference', 'candidate'] if (rnd + pair) % 2 else ['candidate', 'reference']):
                        actual = 'reference' if mode == 'AA' else version
                        output = private / 'control-current.json'; assert not output.exists()
                        env = os.environ | {'RVVDK_HASH_CASE': case, f'RVVDK_{prefix}_PAIRS': '1', f'RVVDK_{prefix}_PARENT': str(parent), f'RVVDK_{prefix}_REPORT': str(output)}
                        result = subprocess.run([str(binaries[actual + '-' + binary_kind]), '--exact', test, '--ignored', '--test-threads=1'], env=env, capture_output=True, text=True, timeout=180)
                        assert result.returncode == 0 and '1 passed; 0 failed;' in result.stdout, result.stdout + result.stderr
                        logs.append({'group': run['group'], 'round': rnd, 'mode': mode, 'pair': pair, 'version': version, 'actual_binary': actual, 'stdout': result.stdout, 'stderr': result.stderr, 'exit_code': result.returncode})
                        rows = json.loads(output.read_text()); output.unlink()
                        assert all(r['pair'] == 0 for r in rows)
                        run['versions'][version].extend(dict(r, pair=pair, actual_binary=actual) for r in rows)
                        assert not list(parent.iterdir())
                runs.append(run); write(report / 'controls.json', runs); write(report / 'control-invocations.json', logs)
                print(run['group'], rnd, mode, flush=True)
    assert hashes == {n: digest(path) for n, path in binaries.items()}
    write(report / 'control-index.json', {name: digest(report / name) for name in ['controls.json', 'control-invocations.json', 'control-environment.json']})
    summarize(report)


if __name__ == '__main__': main()
