#!/usr/bin/env python3
"""Recompute and plot R5.10p comparisons from all saved Criterion samples."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def compute(path):
    rows = json.loads(path.read_text())
    index = {}
    for row in rows:
        key = (row['benchmark'], row['phase'], row['variant'], row['round'])
        if key in index or len(row['times']) != 30 or len(row['iters']) != 30:
            raise ValueError(f'duplicate or incomplete run: {key}')
        if any(v <= 0 for v in row['times'] + row['iters']):
            raise ValueError(f'nonpositive sample: {key}')
        median = statistics.median(t / i for t, i in zip(row['times'], row['iters']))
        if median != row['median_ns']:
            raise ValueError(f'median mismatch: {key}')
        index[key] = median
    pairs = []
    medians = {}
    for benchmark, phase in sorted({(r['benchmark'], r['phase']) for r in rows}):
        variants = ['original', 'before', 'candidate'] if benchmark.startswith('descriptor/') else ['before', 'candidate']
        if (benchmark, phase, 'control', 1) in index:
            variants.append('control')
        for variant in variants:
            values = [index[(benchmark, phase, variant, n)] for n in range(1, 4)]
            medians[f'{benchmark}/{phase}/{variant}'] = values
        comparisons = [('candidate', 'before')]
        if 'control' in variants:
            comparisons.append(('control', 'before'))
        if benchmark.startswith('descriptor/'):
            comparisons += [('candidate', 'original'), ('before', 'original')]
        for candidate, reference in comparisons:
            for n in range(1, 4):
                change = 100 * (index[(benchmark, phase, candidate, n)] /
                                index[(benchmark, phase, reference, n)] - 1)
                pairs.append(dict(benchmark=benchmark, phase=phase, candidate=candidate,
                                  reference=reference, round=n, change_percent=change))
    return dict(sample_count=len(rows) * 30, comparisons=pairs, run_medians_ns=medians,
                scope='Synthetic metadata in memory; shared host, one pinned CPU')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    args = parser.parse_args()
    root = args.report
    source = root / 'measurements.json'
    computed = compute(source)
    out = root / 'plots'
    out.mkdir(exist_ok=True)
    (out / 'computed.json').write_text(json.dumps(computed, indent=2) + '\n')
    pairs = computed['comparisons']
    phases = sorted({p['phase'] for p in pairs})
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10,
                         'svg.hashsalt': 'rvddk-r510p-v1', 'svg.fonttype': 'none',
                         'axes.spines.top': False, 'axes.spines.right': False})
    fig, axes = plt.subplots(2, 2, figsize=(12, 8), layout='constrained')
    fig.suptitle('R5.10p · Fixed-key comparison specialization', fontsize=17)
    names = sorted({p['benchmark'] for p in pairs if p['benchmark'].startswith('descriptor/')})
    series = [('before', 'original', 'R5.10 vs\noriginal'),
              ('candidate', 'before', 'Inline vs\nR5.10'),
              ('candidate', 'original', 'Inline vs\noriginal')]
    colors = ['#246486', '#d37327']
    for ax, name in zip(axes.flat, names):
        for phase_index, phase in enumerate(phases):
            for i, (candidate, reference, _) in enumerate(series):
                values = [p['change_percent'] for p in pairs if p['benchmark'] == name
                          and p['phase'] == phase and p['candidate'] == candidate
                          and p['reference'] == reference]
                if not values:
                    continue
                center = i + (phase_index - (len(phases) - 1) / 2) * .3
                ax.scatter([center - .06, center, center + .06], values,
                           color=colors[phase_index], s=25, label=phase if i == 0 else None)
                ax.plot([center - .12, center + .12], [statistics.median(values)] * 2,
                        color=colors[phase_index], lw=2)
        ax.axhline(5, color='#b23740', ls='--', label='+5% investigation threshold')
        ax.axhline(0, color='gray', lw=.7)
        ax.set_xticks(range(3), [s[2] for s in series])
        ax.set_title(name.split('/')[1])
        ax.set_ylabel('Elapsed change (%) · lower is better')
        ax.grid(axis='y', alpha=.2)
    axes[0, 0].legend(fontsize=8)
    fig.supxlabel('Original: f756fb1 · R5.10: 05b5fba · dots: three run pairs · line: median', fontsize=10)
    save(fig, out, 'descriptor-comparisons')

    fig, ax = plt.subplots(figsize=(10, 6), layout='constrained')
    names = sorted({p['benchmark'] for p in pairs if p['benchmark'].startswith('stream_admission/')})
    for phase_index, phase in enumerate(phases):
        for i, name in enumerate(names):
            values = [p['change_percent'] for p in pairs if p['benchmark'] == name and p['phase'] == phase]
            if not values:
                continue
            center = i + (phase_index - (len(phases) - 1) / 2) * .3
            ax.scatter(values, [center - .06, center, center + .06], s=30,
                       color=colors[phase_index], label=phase if i == 0 else None)
            ax.plot([statistics.median(values)] * 2, [center - .12, center + .12],
                    color=colors[phase_index], lw=2)
    ax.axvline(5, color='#b23740', ls='--', label='+5% investigation threshold')
    ax.axvline(0, color='gray', lw=.7)
    ax.set_yticks(range(len(names)), [n.split('/')[1].replace('_', ' ') for n in names])
    ax.set_xlabel('Inline vs R5.10 elapsed change (%) · lower is better')
    ax.set_title('R5.10p · Stream metadata regression checks', fontsize=16)
    ax.grid(axis='x', alpha=.2)
    ax.legend(fontsize=9)
    fig.supxlabel('Three pairs per phase · 30 samples/run · no storage or decompression timing', fontsize=10)
    save(fig, out, 'stream-comparisons')
    inputs = {'measurements.json': digest(source), str(Path(__file__)): digest(Path(__file__))}
    diagnostic = root / 'diagnostic' / 'measurements.json'
    if diagnostic.exists():
        detail = compute(diagnostic)
        (out / 'diagnostic-computed.json').write_text(json.dumps(detail, indent=2) + '\n')
        inputs['diagnostic/measurements.json'] = digest(diagnostic)
        fig, axes = plt.subplots(1, 2, figsize=(13, 6), layout='constrained')
        fig.suptitle('R5.10p · Separate-core diagnostic with an identical-binary control', fontsize=16)
        span = max(6, max(abs(p['change_percent']) for p in detail['comparisons']) * 1.12)
        for ax, variant, title in zip(axes, ['candidate', 'control'],
                                     ['Inline vs baseline', 'Identical control vs baseline']):
            for i, name in enumerate(names):
                values = [p['change_percent'] for p in detail['comparisons']
                          if p['benchmark'] == name and p['candidate'] == variant]
                ax.scatter(values, [i - .08, i, i + .08], color='#246486', s=30)
                ax.plot([statistics.median(values)] * 2, [i - .15, i + .15], color='#d37327', lw=2)
            ax.axvline(5, color='#b23740', ls='--')
            ax.axvline(0, color='gray', lw=.7)
            ax.set_xlim(-span, span)
            ax.set_yticks(range(len(names)), [n.split('/')[1].replace('_', ' ') for n in names])
            ax.set_title(title)
            ax.set_xlabel('Elapsed change (%) · lower is better')
            ax.grid(axis='x', alpha=.2)
        fig.supxlabel('CPU4 · three pairs · shared host · does not replace the retained CPU0 observations', fontsize=10)
        save(fig, out, 'diagnostic-controls')
    (out / 'manifest.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__,
        input_sha256=inputs,
        output_sha256={p.name: digest(p) for p in sorted(out.iterdir()) if p.name != 'manifest.json'}), indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(fig, out, name):
    fig.savefig(out / (name + '.png'), dpi=160)
    path = out / (name + '.svg')
    fig.savefig(path, metadata={'Date': None})
    path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines()) + '\n')
    plt.close(fig)


if __name__ == '__main__':
    main()
