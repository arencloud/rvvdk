#!/usr/bin/env python3
"""Plot saved R5.5 capacity evidence; no benchmarks or storage access."""
import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path
import platform
import statistics

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from plot import samples


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    out = args.output or args.report / 'scaling-plots'
    out.mkdir(parents=True, exist_ok=True)
    inputs = ['candidate-only-measurements.json', 'capacity-profiles.json']
    runs = json.loads((args.report / inputs[0]).read_text())
    profiles = sorted(json.loads((args.report / inputs[1]).read_text()),
                      key=lambda row: row['capacity_bytes'])
    values = {}
    for run in runs:
        values.setdefault(run['benchmark'].split('/')[1], []).append(statistics.median(samples(run)))
    for name, medians in values.items():
        if len(medians) != 3:
            raise ValueError(f'{name}: expected three independent runs')
    if [p['capacity_bytes'] for p in profiles] != [2**20, 2**30, 64 * 2**30]:
        raise ValueError('unexpected capacity profiles')
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10,
                         'axes.spines.top': False, 'axes.spines.right': False,
                         'svg.hashsalt': 'rvddk-sparse-scaling-v1', 'svg.fonttype': 'none'})
    fig, axes = plt.subplots(2, 2, figsize=(12, 8), layout='constrained')
    fig.suptitle('R5.5 · Sparse capacity and fragmentation costs', fontsize=17)

    def line(ax, names, xs, scale, label):
        ys = [statistics.median(values[name]) / scale for name in names]
        ax.plot(xs, ys, '-o', color='#2879A5', label=label)
        for x, name in zip(xs, names):
            ax.scatter([x] * 3, [v / scale for v in values[name]], color='#D87927', s=15, zorder=3)
        ax.grid(alpha=0.2)
        ax.set_ylim(bottom=0)

    caps = [p['capacity_bytes'] / 2**20 for p in profiles]
    ax = axes[0, 0]
    line(ax, ['open_zero_1m', 'open_zero_1g', 'open_zero_64g'], caps, 1e6, 'median of run medians')
    ax.set(xscale='log', xlabel='Virtual capacity (MiB, log scale)', ylabel='Open time (ms)',
           title='Load + validate + drop · zero maps · 64 KiB grains')
    ax = axes[0, 1]
    for key, label in [('reserved_memory_bytes', 'Loader reservation'),
                       ('metadata_read_bytes', 'Metadata read payload'),
                       ('fixture_metadata_bytes', 'Fixture bytes (outside loader budget)')]:
        ax.plot(caps, [p[key] / 2**20 for p in profiles], '-o', label=label)
    ax.set(xscale='log', xlabel='Virtual capacity (MiB, log scale)', ylabel='MiB',
           title='Deterministic payload counters · not process RSS', ylim=(0, None))
    ax.legend(fontsize=8); ax.grid(alpha=0.2)
    ax = axes[1, 0]
    line(ax, ['query_zero_1g', 'query_zero_64g'], [16384, 1048576], 1e6, 'zero map, one output')
    ax.set(xscale='log', xlabel='Grains scanned (log scale)', ylabel='Full query time (ms)',
           title='One zero extent still requires scanning the map')
    ax = axes[1, 1]
    names = ['query_alternating_256m', 'query_alternating_512m', 'query_alternating_limit']
    ys = [statistics.median(values[n]) / 1e6 for n in names]
    ax.bar(range(3), ys, color=['#2879A5', '#2879A5', '#999999'])
    for i, name in enumerate(names):
        ax.scatter([i] * 3, [v / 1e6 for v in values[name]], color='#D87927', s=15, zorder=3)
    ax.set_xticks(range(3), ['32,768 outputs', '65,536 outputs', '65,537 → error'])
    ax.set(ylabel='Query time (ms)', title='Alternating grains · count, then allocate/fill on success')
    ax.grid(axis='y', alpha=0.2)
    fig.supxlabel('Synthetic memory-backed metadata · 3 runs × 30 flat samples · dots: each run median\n'
                  'CPU 2–6 · shared host · fixture setup outside timing · no storage-throughput claim', fontsize=10)
    outputs = []
    for suffix in ['svg', 'png']:
        path = out / ('capacity.' + suffix)
        metadata = {'Creator': 'rvddk sparse scaling plots', 'Date': None} if suffix == 'svg' else {'Software': 'rvddk sparse scaling plots'}
        fig.savefig(path, dpi=160, metadata=metadata)
        outputs.append(path)
    plt.close(fig)
    manifest = dict(generator='scripts/benchmarks/plot_sparse_scaling.py', generator_sha256=digest(Path(__file__)),
                    sample_validator_sha256=digest(Path(__file__).with_name('plot.py')),
                    python=platform.python_version(), matplotlib=importlib.metadata.version('matplotlib'),
                    input_sha256={name: digest(args.report / name) for name in inputs},
                    output_sha256={path.name: digest(path) for path in outputs})
    (out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
