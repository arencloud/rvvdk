#!/usr/bin/env python3
"""Reproduce the V0.2 Rust connection-policy comparison; no disk throughput claims."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    raw = args.input.read_bytes()
    data = json.loads(raw)
    assert data['schema'] == 1 and data['scope'] == 'rust_read_only_inventory'
    assert data['expected_inventory_limit'] is False
    runs = data['runs']
    assert len(runs) == 6
    assert [r['report']['connection_policy'] for r in runs] == ['fresh', 'reuse', 'reuse', 'fresh', 'fresh', 'reuse']
    fresh, reuse, changes = [], [], []
    for run in runs:
        report = run['report']
        assert report['primary_error'] is None and report['pagination_cleanup_error'] is None
        assert report['cleanup'] == 'logged_out'
        assert report['inventory'] == runs[0]['report']['inventory']
        assert len(report['requests']) == 14
        assert report['certificate_checks'] == (14 if report['connection_policy'] == 'fresh' else 1)
        assert [r['method'] for r in report['requests']] == ['RetrieveServiceContent', 'Login'] + ['RetrievePropertiesEx'] * 11 + ['Logout']
        assert all(r['error'] is None for r in report['requests'])
    for a, b in zip(runs[::2], runs[1::2]):
        pair = {r['report']['connection_policy']: r['report']['elapsed_ms'] for r in (a, b)}
        fresh.append(pair['fresh'])
        reuse.append(pair['reuse'])
        changes.append((pair['reuse'] / pair['fresh'] - 1) * 100)
    aggregate = (statistics.median(reuse) / statistics.median(fresh) - 1) * 100
    args.output.mkdir(parents=True, exist_ok=True)
    plt.rcParams.update({'svg.hashsalt': 'rvddk-v02', 'font.family': 'DejaVu Sans', 'font.size': 10})
    fig, axes = plt.subplots(1, 3, figsize=(14, 4.8))
    for i, (f, r, change) in enumerate(zip(fresh, reuse, changes), 1):
        axes[0].plot([0, 1], [f / 1000, r / 1000], 'o-', label=f'Pair {i}: {change:+.1f}%', linewidth=1.8)
    axes[0].set(xticks=[0, 1], xticklabels=['Fresh TLS', 'Reuse'], ylabel='Complete discovery (s)', title='Same Rust binary, 14 requests')
    axes[0].set_xlim(-0.2, 1.2)
    axes[0].set_ylim(bottom=0)
    axes[0].legend(frameon=False, fontsize=9)
    colors = ['#d97706' if r['report']['connection_policy'] == 'fresh' else '#2563eb' for r in runs]
    axes[1].bar(range(1, 7), [r['cpu_seconds'] * 1000 for r in runs], color=colors)
    axes[1].set(xlabel='Run order', ylabel='Process CPU time (ms)', title='CPU cost per session', xticks=range(1, 7))
    axes[2].plot(range(1, 7), [r['process_peak_rss_kib'] / 1024 for r in runs], 'o-', color='#059669')
    axes[2].set(xlabel='Run order', ylabel='Process lifetime high-water RSS (MiB)', title='RSS includes runtime and TLS', xticks=range(1, 7), ylim=(0, 7))
    for ax in axes:
        ax.grid(axis='y', alpha=0.2)
        ax.set_axisbelow(True)
        ax.spines[['top', 'right']].set_visible(False)
    fig.suptitle('V0.2 · Rust ESXi discovery · 2026-10-01', fontsize=15, fontweight='bold')
    fig.text(0.5, 0.015, 'CPU bars: orange = fresh, blue = reuse · Login + inventory + parsing + Logout · Network and host load uncontrolled · No disk transfer', ha='center', fontsize=9)
    fig.tight_layout(rect=[0, 0.05, 1, 0.93])
    fig.savefig(args.output / 'discovery.svg', metadata={'Date': None})
    fig.savefig(args.output / 'discovery.png', dpi=150, metadata={'Software': 'rvddk V0.2 plot'})
    plt.close(fig)
    computed = {'schema': 1, 'input_sha256': hashlib.sha256(raw).hexdigest(),
                'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                'python': platform.python_version(), 'matplotlib': matplotlib.__version__,
                'fresh_ms': fresh, 'reuse_ms': reuse, 'paired_elapsed_change_percent': changes,
                'median_elapsed_change_percent': aggregate,
                'longer_repeat_triggered': aggregate > 5 or any(c > 5 for c in changes),
                'requests': 84, 'scope': 'same-source connection-policy comparison; not disk throughput'}
    (args.output / 'computed.json').write_text(json.dumps(computed, indent=2) + '\n')


if __name__ == '__main__':
    main()
