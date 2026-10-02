#!/usr/bin/env python3
"""Plot retained R5.10 measurements; validate samples and preserve adverse pairs."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def read_runs(path):
    rows = json.loads(path.read_text())
    seen = set()
    for row in rows:
        key = (row['benchmark'], row['variant'], row['phase'], row['round'])
        if key in seen:
            raise ValueError(f'duplicate run: {key}')
        seen.add(key)
        times, iters = row['times'], row['iters']
        if len(times) != 30 or len(iters) != 30 or any(x <= 0 for x in times + iters):
            raise ValueError(f'invalid samples: {key}')
        median = statistics.median(t / n for t, n in zip(times, iters))
        if abs(median - row['median_ns']) > 1e-9:
            raise ValueError(f'median mismatch: {key}')
    if len(rows) != 69:
        raise ValueError('expected 48 descriptor and 21 stream runs per design')
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    args = parser.parse_args()
    root = args.report
    inputs = [root / design / 'measurements.json' for design in ('initial', 'final')]
    runs = [read_runs(path) for path in inputs]
    pairs = []
    for design, rows in zip(('initial', 'final'), runs):
        for phase in ('initial', 'longer'):
            for base in rows:
                if base['phase'] != phase or base['variant'] != 'baseline':
                    continue
                candidate, = [r for r in rows if r['benchmark'] == base['benchmark']
                              and r['phase'] == phase and r['round'] == base['round']
                              and r['variant'] == 'candidate']
                pairs.append(dict(design=design, phase=phase, benchmark=base['benchmark'],
                                  round=base['round'], change_percent=100 *
                                  (candidate['median_ns'] / base['median_ns'] - 1)))
    new = {}
    for row in runs[1]:
        if row['phase'] == 'new':
            new.setdefault(row['benchmark'].split('/')[1], []).append(row['median_ns'])
    if len(pairs) != 48 or len(new) != 7 or any(len(v) != 3 for v in new.values()):
        raise ValueError('incomplete comparison matrix')
    out = root / 'plots'
    out.mkdir(exist_ok=True)
    computed = dict(schema=1, descriptor_pairs=pairs, final_stream_run_medians_ns=new,
                    final_stream_median_ns={k: statistics.median(v) for k, v in new.items()},
                    disposition='Open: repeated >5% small-descriptor regression; no clearance',
                    scope='Synthetic metadata in memory; no disk or decompression throughput')
    (out / 'computed.json').write_text(json.dumps(computed, indent=2) + '\n')
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10,
                         'svg.hashsalt': 'rvddk-r510-v1', 'svg.fonttype': 'none',
                         'axes.spines.top': False, 'axes.spines.right': False})
    fig, axes = plt.subplots(2, 2, figsize=(12, 8), layout='constrained')
    fig.suptitle('R5.10 · Metadata admission and retained descriptor regressions', fontsize=16)
    benchmarks = sorted({p['benchmark'] for p in pairs})
    for ax, benchmark in zip(axes.flat, benchmarks):
        for x, (design, phase) in enumerate([('initial', 'initial'), ('initial', 'longer'),
                                            ('final', 'initial'), ('final', 'longer')]):
            values = [p['change_percent'] for p in pairs if p['benchmark'] == benchmark
                      and p['design'] == design and p['phase'] == phase]
            ax.scatter([x - .12, x, x + .12], values, color='#c25b2a', s=27)
            ax.plot([x - .2, x + .2], [statistics.median(values)] * 2, color='#174f70', lw=2)
        ax.axhline(5, color='#b23740', ls='--', label='+5% investigation threshold')
        ax.axhline(0, color='gray', lw=.7)
        ax.set_title(benchmark.split('/')[1])
        ax.set_xticks(range(4), ['First\nshort', 'First\nlonger', 'Final\nshort', 'Final\nlonger'])
        ax.set_ylabel('Candidate elapsed change (%) · lower is better')
        ax.grid(axis='y', alpha=.2)
    axes[0, 0].legend(fontsize=8)
    fig.supxlabel('Dots: three alternating pairs · line: median · shared host, one pinned CPU', fontsize=10)
    save(fig, out, 'descriptor-regressions')
    fig, axes = plt.subplots(1, 2, figsize=(12, 4.5), layout='constrained')
    fig.suptitle('R5.10 · Final metadata costs in memory', fontsize=16)
    for ax, names, divisor, unit in [
        (axes[0], [k for k in sorted(new) if not k.startswith('envelope')], 1, 'ns'),
        (axes[1], ['envelope_front', 'envelope_footer'], 1000, 'µs'),
    ]:
        ys = [statistics.median(new[k]) / divisor for k in names]
        ax.barh(range(len(names)), ys, color='#28688b', alpha=.8)
        for i, name in enumerate(names):
            ax.scatter([v / divisor for v in new[name]], [i] * 3, color='#d46e2f', s=22, zorder=3)
            ax.text(max(new[name]) / divisor + max(ys) * .04, i,
                    f'{ys[i]:.2f}', va='center', fontsize=9)
        ax.set_yticks(range(len(names)), [k.replace('_', ' ') for k in names])
        ax.set_xlim(0, max(ys) * 1.3)
        ax.set_xlabel(f'Elapsed time ({unit}) · median of three runs')
        ax.grid(axis='x', alpha=.2)
    fig.supxlabel('30 samples/run · dots: run medians · no payload decoding or storage I/O', fontsize=10)
    save(fig, out, 'metadata-costs')
    artifacts = inputs + [Path(__file__)]
    (out / 'manifest.json').write_text(json.dumps(dict(
        matplotlib=matplotlib.__version__,
        input_sha256={str(p.relative_to(root)) if p.is_relative_to(root) else str(p):
                      hashlib.sha256(p.read_bytes()).hexdigest() for p in artifacts},
        output_sha256={p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                       for p in sorted(out.iterdir()) if p.name != 'manifest.json'}), indent=2) + '\n')


def save(fig, out, name):
    fig.savefig(out / (name + '.png'), dpi=160)
    path = out / (name + '.svg')
    fig.savefig(path, metadata={'Date': None})
    path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines()) + '\n')
    plt.close(fig)


if __name__ == '__main__':
    main()
