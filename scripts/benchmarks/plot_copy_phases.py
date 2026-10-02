#!/usr/bin/env python3
"""Validate and plot the fixed R5.12q phase investigation; no private inputs needed."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics as st

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('report', type=Path)
    root = p.parse_args().report
    source = root / 'measurements.json'
    data = json.loads(source.read_text())
    rows = data['rows']
    assert len(rows) == 28 and len(data['reads']) == 6
    assert data['source_stamp_unchanged'] and data['binaries_unchanged']
    assert len({(r['group'], r['variant'], r['round']) for r in rows}) == 28
    assert len(data['checks']) == 28
    assert {c['run'] for c in data['checks']} == {
        f"{r['group']}-{r['variant']}-{r['round']}" for r in rows}
    assert all(c['exit_code'] == 0 for c in data['checks'])
    for r in rows:
        assert r['exit_code'] == 0 and r['logical_bytes'] == 30 * 1024**3
        assert r['durability'] == 'file_and_directory_synced' and r['backend'] == 'threaded'
        assert r['stats'] == rows[0]['stats']
        assert r['allocated_bytes'] == (30 * 1024**3 if r['binary'] == 'before' else 3754885120)
        assert math.isfinite(r['elapsed_seconds']) and r['elapsed_seconds'] > 0
        if r['progress']:
            events = r['events']
            assert all(y['elapsed_seconds'] >= x['elapsed_seconds'] for x, y in zip(events, events[1:]))
            assert all(math.isfinite(t) and t >= 0 for t in r['phases_seconds'].values())
            t = {e['phase']: e['elapsed_seconds'] for e in events}
            assert math.isclose(sum(r['phases_seconds'].values()), t['completed'] - t['planning'])
            # CLI report and terminal progress have slightly different boundaries.
            assert abs(sum(r['phases_seconds'].values()) - r['elapsed_seconds']) < .05
    for r in data['reads']:
        assert r['exit_code'] == 0 and r['mode'] == 'sequential'
        assert r['allocated_grains'] == r['read_calls'] == 57295
        assert r['logical_bytes'] == 3754885120 and r['decode_memory_bytes'] == 141679

    pairs = {}
    for group, count, first, second in [('policy', 6, 'before', 'candidate'),
            ('identical', 3, 'a', 'b'), ('progress_control', 3, 'a', 'b'),
            ('syscalls', 2, 'before', 'candidate')]:
        pairs[group] = []
        for n in range(1, count + 1):
            a, = [r for r in rows if (r['group'], r['round'], r['variant']) == (group, n, first)]
            b, = [r for r in rows if (r['group'], r['round'], r['variant']) == (group, n, second)]
            pair = dict(round=n, elapsed_delta_seconds=b['elapsed_seconds'] - a['elapsed_seconds'],
                        change_percent=100 * (b['elapsed_seconds'] / a['elapsed_seconds'] - 1))
            if a['progress'] and b['progress']:
                pair['phase_delta_seconds'] = {k: b['phases_seconds'][k] - v for k, v in a['phases_seconds'].items()}
            pairs[group].append(pair)
    summary = dict(pairs=pairs, policy_medians={})
    summary['syscall_wall_seconds'] = {}
    trace_hashes = {}
    expected = dict(pread64=114977, pwrite64=3596, fallocate=22, fdatasync=1, fsync=2)
    for path in sorted(root.glob('syscalls-*.strace')):
        totals = {}
        for line in path.read_text().splitlines():
            fields = line.split()
            if fields and fields[-1] in expected:
                assert len(fields) == 5, 'unexpected syscall error column'
                assert int(fields[3]) == expected[fields[-1]]
                totals[fields[-1]] = float(fields[1])
        assert set(totals) == set(expected)
        summary['syscall_wall_seconds'][path.stem] = totals
        trace_hashes[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    assert len(trace_hashes) == 4
    for variant in ['before', 'candidate']:
        rs = [r for r in rows if r['group'] == 'policy' and r['variant'] == variant]
        summary['policy_medians'][variant] = dict(
            elapsed_seconds=st.median(r['elapsed_seconds'] for r in rs),
            cpu_seconds=st.median(r['metrics']['user_seconds'] + r['metrics']['system_seconds'] for r in rs),
            rss_kib=st.median(r['metrics']['max_rss_kib'] for r in rs),
            physical_extents=st.median(r['physical_extents'] for r in rs),
            phases_seconds={k: st.median(r['phases_seconds'][k] for r in rs) for k in rs[0]['phases_seconds']})
    summary['read_medians'] = {k: st.median(r[k] for r in data['reads']) for k in ['admission_seconds', 'read_seconds']}
    summary_path = root / 'summary.json'
    summary_path.write_text(json.dumps(summary, indent=2) + '\n')
    out = root / 'plots'
    out.mkdir(exist_ok=True)
    outputs = [summary_path]
    plt.rcParams.update({'font.size': 10, 'axes.spines.top': False, 'axes.spines.right': False})

    def save(fig, name):
        fig.tight_layout()
        for ext in ['png', 'svg']:
            path = out / (name + '.' + ext)
            fig.savefig(path, dpi=160)
            if ext == 'svg':
                path.write_text('\n'.join(s.rstrip() for s in path.read_text().splitlines()) + '\n')
            outputs.append(path)
        plt.close(fig)

    fig, axes = plt.subplots(1, 2, figsize=(13, 5))
    policy = [r for r in rows if r['group'] == 'policy']
    labels = [f"{r['round']}{'B' if r['variant'] == 'before' else 'S'}" for r in policy]
    bottom = [0.] * len(policy)
    for key, title in [('admission_plan', 'Admission / planning'), ('transfer', 'Transfer'),
                       ('engine_flush', 'Engine flush')]:
        values = [r['phases_seconds'][key] for r in policy]
        axes[0].bar(range(len(policy)), values, bottom=bottom, label=title)
        bottom = [a + b for a, b in zip(bottom, values)]
    values = [sum(r['phases_seconds'].values()) - b for r, b in zip(policy, bottom)]
    axes[0].bar(range(len(policy)), values, bottom=bottom, label='Revalidate / sync / publish')
    axes[0].set_xticks(range(len(labels)), labels)
    axes[0].set_ylabel('Elapsed seconds'); axes[0].legend(fontsize=8)
    axes[0].set_title('Every untraced pair, in execution order\nB = allocating baseline, S = sparse output')
    for key, label in [('transfer', 'Transfer'), ('engine_flush', 'Engine flush')]:
        axes[1].plot(range(1, 7), [r['phase_delta_seconds'][key] for r in pairs['policy']], 'o-', label=label)
    axes[1].plot(range(1, 7), [r['elapsed_delta_seconds'] for r in pairs['policy']], 'o--', label='Complete copy')
    axes[1].axhline(0, color='gray'); axes[1].set_xlabel('Paired round')
    axes[1].set_ylabel('Sparse − baseline seconds'); axes[1].legend()
    axes[1].set_title('Where the measured difference occurs')
    save(fig, 'phases')

    fig, axes = plt.subplots(1, 3, figsize=(13, 4))
    for ax, group, title in zip(axes, ['policy', 'identical', 'progress_control'],
            ['Sparse vs allocating output', 'Same sparse binary: B vs A', 'Same sparse binary: progress on vs off']):
        ps = pairs[group]
        ax.scatter([r['round'] for r in ps], [r['change_percent'] for r in ps])
        ax.axhline(0, color='gray'); ax.axhline(5, color='red', ls='--')
        ax.axhline(st.median(r['change_percent'] for r in ps), color='#126e82', ls=':')
        ax.set_title(title, fontsize=10); ax.set_xlabel('Paired round'); ax.set_ylabel('Time change (%)')
    save(fig, 'controls')

    fig, axes = plt.subplots(1, 2, figsize=(11, 4))
    for variant, label in [('before', 'Allocating baseline'), ('candidate', 'Sparse output')]:
        rs = [r for r in policy if r['variant'] == variant]
        axes[0].scatter([r['physical_extents'] for r in rs], [r['phases_seconds']['engine_flush'] for r in rs], label=label)
    axes[0].set_xscale('log'); axes[0].set_xlabel('Physical extents reported by filefrag (log scale)')
    axes[0].set_ylabel('Engine flush seconds'); axes[0].legend()
    axes[0].set_title('Layout association; causality is not established')
    axes[1].plot(range(1, 7), [r['read_seconds'] for r in data['reads']], 'o-', label='Allocated-grain reads + decode')
    axes[1].plot(range(1, 7), [r['admission_seconds'] for r in data['reads']], 'o-', label='Map admission')
    axes[1].set_xlabel('Read-only control run'); axes[1].set_ylabel('Seconds'); axes[1].legend()
    axes[1].set_title('Separate frozen native probe; no destination')
    save(fig, 'layout-and-read')
    audit = dict(measurements_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                 syscall_trace_sha256=trace_hashes,
                 copy_runs=len(rows), full_readbacks=len(data['checks']), read_controls=len(data['reads']),
                 output_sha256={str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in outputs})
    (out / 'audit.json').write_text(json.dumps(audit, indent=2) + '\n')


if __name__ == '__main__':
    main()
