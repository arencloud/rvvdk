#!/usr/bin/env python3
"""Plot V0.3.1 baseline/candidate discovery evidence; no VMware access."""
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
    phases = ['initial']
    if (args.input / 'repeat-1-baseline-1.json').exists():
        phases.append('repeat')
    reference = None
    hashes = {}
    results = {}
    plt.rcParams.update({'svg.hashsalt': 'rvddk-v031', 'font.family': 'DejaVu Sans', 'font.size': 10})
    fig, axes = plt.subplots(len(phases), 3, figsize=(14, 4.8 * len(phases)), squeeze=False)
    for row, phase in enumerate(phases):
        blocks = []
        all_runs = {'baseline': [], 'candidate': []}
        for pair in range(1, 4):
            runs = {}
            for arm in ('baseline', 'candidate'):
                names = ([f'{phase}-{pair}-{arm}.json'] if phase == 'initial' else
                         [f'{phase}-{pair}-{arm}-{part}.json' for part in (1, 2)])
                runs[arm] = []
                for name in names:
                    raw = (args.input / name).read_bytes()
                    hashes[name] = hashlib.sha256(raw).hexdigest()
                    data = json.loads(raw)
                    assert data['scope'] == 'rust_read_only_inventory' and len(data['runs']) == 3
                    for run in data['runs']:
                        r = run['report']
                        assert r['primary_error'] is None and r['pagination_cleanup_error'] is None
                        assert r['cleanup'] == 'logged_out' and r['connection_policy'] == 'reuse'
                        assert r['certificate_checks'] == 1 and len(r['requests']) == 14
                        assert all(call['error'] is None for call in r['requests'])
                        if reference is None:
                            reference = r['inventory']
                        assert r['inventory'] == reference
                        runs[arm].append(run)
                all_runs[arm].extend(runs[arm])
            elapsed = {arm: [r['report']['elapsed_ms'] for r in runs[arm]] for arm in runs}
            changes = [(c / b - 1) * 100 for b, c in zip(elapsed['baseline'], elapsed['candidate'])]
            medians = {arm: statistics.median(values) for arm, values in elapsed.items()}
            blocks.append({'pair': pair, 'order': ['baseline', 'candidate'] if pair != 2 else ['candidate', 'baseline'],
                           'elapsed_ms': elapsed, 'individual_change_percent': changes,
                           'median_change_percent': (medians['candidate'] / medians['baseline'] - 1) * 100})
            axes[row, 0].plot([0, 1], [medians[a] / 1000 for a in ('baseline', 'candidate')], 'o-',
                              label=f'Pair {pair}: {blocks[-1]["median_change_percent"]:+.1f}%')
        median = {a: statistics.median(r['report']['elapsed_ms'] for r in runs) for a, runs in all_runs.items()}
        aggregate = (median['candidate'] / median['baseline'] - 1) * 100
        adverse = aggregate > 5 or any(b['median_change_percent'] > 5 or any(c > 5 for c in b['individual_change_percent']) for b in blocks)
        results[phase] = {'blocks': blocks, 'median_ms': median, 'aggregate_change_percent': aggregate,
                          'exceeds_five_percent': adverse, 'sessions': sum(map(len, all_runs.values()))}
        results[phase]['resources'] = {
            arm: {'median_cpu_ms': statistics.median(r['cpu_seconds'] * 1000 for r in runs),
                  'rss_range_kib': [min(r['process_peak_rss_kib'] for r in runs), max(r['process_peak_rss_kib'] for r in runs)],
                  'request_position_median_ms': [statistics.median(r['report']['requests'][i]['elapsed_ms'] for r in runs) for i in range(14)]}
            for arm, runs in all_runs.items()}
        maxima = [0, 0, 0]
        for i, arm in enumerate(('baseline', 'candidate')):
            runs = all_runs[arm]
            jitter = [i + (n - (len(runs) - 1) / 2) * 0.012 for n in range(len(runs))]
            color = '#d97706' if i == 0 else '#2563eb'
            for col, values in enumerate((
                [r['report']['elapsed_ms'] / 1000 for r in runs],
                [r['cpu_seconds'] * 1000 for r in runs],
                [r['process_peak_rss_kib'] / 1024 for r in runs],
            )):
                maxima[col] = max(maxima[col], max(values))
                axes[row, col].scatter(jitter, values, color=color, alpha=0.55, s=22)
        for col, (title, unit) in enumerate((('Discovery elapsed', 'Seconds'), ('Process CPU per session', 'Milliseconds'), ('Process lifetime high-water RSS', 'MiB'))):
            ax = axes[row, col]
            ax.set(title=f'{phase.title()} · {title}', ylabel=unit, xticks=[0, 1],
                   xticklabels=['V0.2 baseline', 'V0.3.1 candidate'], xlim=(-0.3, 1.3), ylim=(0, maxima[col] * 1.12))
            ax.grid(axis='y', alpha=0.2)
            ax.spines[['top', 'right']].set_visible(False)
        axes[row, 0].legend(frameon=False, fontsize=9)
    fig.suptitle('V0.3.1 · Shared-session regression check · 2026-10-01', fontsize=15, fontweight='bold')
    fig.text(0.5, 0.01, 'All observations retained · Lines join pair medians · Identical 14-call inventory sessions · Uncontrolled host/network load · No disk transfer', ha='center', fontsize=9)
    fig.tight_layout(rect=[0, 0.035, 1, 0.95])
    args.output.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output / 'discovery.svg', metadata={'Date': None})
    fig.savefig(args.output / 'discovery.png', dpi=150, metadata={'Software': 'rvddk V0.3.1 plot'})
    plt.close(fig)
    computed = {'schema': 1, 'scope': 'shared-session discovery comparison; not transfer throughput',
                'inputs_sha256': hashes, 'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                'python': platform.python_version(), 'matplotlib': matplotlib.__version__, 'phases': results,
                'longer_repeat_triggered': results['initial']['exceeds_five_percent']}
    (args.output / 'computed.json').write_text(json.dumps(computed, indent=2) + '\n')


if __name__ == '__main__':
    main()
