#!/usr/bin/env python3
"""Validate retained reconciliation samples and export standalone diagnostic figures."""
import argparse
import json
from pathlib import Path
import statistics as st
from measure_reconciliation import PHASES, digest, load_run, summarize, write


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('report', type=Path)
    a = p.parse_args()
    root = a.report
    summary = summarize(root)
    runs = json.loads((root / 'measurements.json').read_text())
    env = json.loads((root / 'environment.json').read_text())
    assert env['binary_sha256'] == json.loads((root / 'binaries.json').read_text())
    assert env['harness_sha256'] == digest(Path(__file__).with_name('measure_reconciliation.py'))
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams.update({'font.size': 9, 'svg.hashsalt': 'rvddk-reconciliation'})
    out = root / 'plots'
    out.mkdir(exist_ok=True)

    def save(fig, name):
        fig.tight_layout()
        for ext in ['svg', 'png']:
            fig.savefig(out / f'{name}.{ext}', dpi=160)
        plt.close(fig)
        svg = out / f'{name}.svg'
        svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()) + '\n')

    def values(group, mode, metric):
        return [(r['phase'], row[metric]) for r in runs if r['group'] == group
                for row in load_run(root, r)[0][mode]]

    def selected(group):
        phases = summary['groups'][group]['phases']
        return phases.get('repeat', phases['initial'])

    fig, axes = plt.subplots(2, 2, figsize=(12, 8))
    modes = ['reference_owned', 'candidate_owned', 'reference_published', 'candidate_published']
    for row, fs in zip(axes, ['btrfs', 'tmpfs']):
        group = f'normal-{fs}-stored_64'
        for ax, metric in zip(row, ['wall_ms', 'cpu_ms']):
            ax.bar(range(4), [selected(group)['modes'][m]['metrics'][metric]['median'] for m in modes], color=['#9ca3af', '#60a5fa'] * 2, alpha=.65)
            for x, mode in enumerate(modes):
                for phase, marker, color in [('initial', 'o', '#555555'), ('repeat', '^', '#145faa')]:
                    samples = [v for ph, v in values(group, mode, metric) if ph == phase]
                    ax.scatter([x] * len(samples), samples, s=9, marker=marker, color=color, alpha=.35)
            ax.set_xticks(range(4), ['Reference\nowned', 'Candidate\nowned', 'Reference\npublished', 'Candidate\npublished'])
            ax.set_ylabel('Milliseconds'); ax.set_title(f'{fs}: {metric}'); ax.set_ylim(bottom=0)
    fig.suptitle('Normal path regression comparison — 64 MiB logical / 8 MiB data\nAll operation samples: circles initial, triangles repeat; bars use repeated medians where available')
    save(fig, 'normal-publication')

    groups = [f'recovery-{fs}-{fixture}' for fs in ['btrfs', 'tmpfs'] for fixture in ['stored_64', 'deflated_256']]
    phase_labels = ['Source admission', 'Full verification', 'Bundle sync', 'Intent sync', 'Rename / checks', 'Parent sync', 'Acknowledgment', 'Assessment / release']
    for metric in ['wall_ms', 'cpu_ms']:
        fig, axes = plt.subplots(2, 2, figsize=(13, 9))
        for ax, group in zip(axes.flat, groups):
            data = selected(group)['modes']; base = [0., 0.]; modes = ['staged', 'already_renamed']
            for key, label in zip(PHASES, phase_labels):
                v = [data[m]['metrics'][f'{key}.{metric}']['median'] for m in modes]
                ax.bar([0, 1], v, bottom=base, label=label)
                base = [b + x for b, x in zip(base, v)]
            for x, mode in enumerate(modes):
                for phase, marker in [('initial', 'o'), ('repeat', '^')]:
                    samples = [v for ph, v in values(group, mode, metric) if ph == phase]
                    ax.scatter([x] * len(samples), samples, s=9, marker=marker, color='black', alpha=.3)
            ax.set_xticks([0, 1], ['Staged intent', 'Already renamed'])
            ax.set_ylabel('Milliseconds'); ax.set_title(group.removeprefix('recovery-')); ax.set_ylim(bottom=0)
        handles, labels = axes[0, 0].get_legend_handles_labels()
        fig.legend(handles, labels, loc='outside lower center', ncol=4, fontsize=8)
        fig.suptitle(f'Reconciliation {metric} — median phases and all whole-operation samples\nPhase medians need not sum to the whole median; branch costs are not a matched speedup')
        fig.subplots_adjust(bottom=.12)
        fig.tight_layout(rect=(0, .07, 1, .94))
        # Preserve the explicit space reserved for the external legend.
        for ext in ['svg', 'png']:
            fig.savefig(out / f'recovery-{metric}.{ext}', dpi=160)
        plt.close(fig)
        svg = out / f'recovery-{metric}.svg'
        svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()) + '\n')

    fig, axes = plt.subplots(2, 2, figsize=(13, 8))
    labels, rss, allocated, cleanup, cpu = [], [], [], [], []
    for group in groups:
        for mode, data in selected(group)['modes'].items():
            labels.append(group.removeprefix('recovery-') + '\n' + mode.replace('_', ' '))
            rss.append(data['peak_rss_kib'] / 1024)
            assert len(data['allocation_bytes']) == 1
            allocated.append(data['allocation_bytes'][0] / (1 << 20))
            cleanup.append(data['metrics']['cleanup_ms']['median'])
            cpu.append(data['metrics']['cleanup_cpu_ms']['median'])
    for ax, vals, title, unit in zip(axes.flat, [rss, allocated, cleanup, cpu], ['Lifetime peak RSS (includes setup)', 'Unchanged RAW allocation', 'Explicit cleanup wall', 'Explicit cleanup CPU'], ['MiB', 'MiB', 'ms', 'ms']):
        ax.bar(range(len(labels)), vals, color=['#2563eb', '#059669'] * 4)
        ax.set_xticks(range(len(labels)), labels, rotation=35, ha='right', fontsize=7)
        ax.set_ylabel(unit); ax.set_title(title); ax.set_ylim(bottom=0)
    fig.suptitle('Recovery memory, sparse allocation and separately timed cleanup')
    save(fig, 'resources-and-cleanup')

    fig, axes = plt.subplots(1, 2, figsize=(12, 5))
    control_groups = [g.replace('recovery-', 'controls-') for g in groups]
    for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
        for x, group in enumerate(control_groups):
            for run in [r for r in runs if r['group'] == group]:
                modes = load_run(root, run)[0]
                deltas = [100 * (enabled[metric] / disabled[metric] - 1) for enabled, disabled in zip(modes['enabled'], modes['disabled'])]
                ax.scatter([x] * len(deltas), deltas, marker='o' if run['phase'] == 'initial' else '^', s=13, alpha=.4, color='#2563eb')
            modes = selected(group)['modes']
            delta = 100 * (modes['enabled']['metrics'][metric]['median'] / modes['disabled']['metrics'][metric]['median'] - 1)
            ax.scatter([x], [delta], marker='D', s=50, color='#b45309')
        ax.axhline(0, color='black', linewidth=.7); ax.axhline(5, color='red', linestyle='--', linewidth=.7)
        ax.set_xticks(range(4), [g.removeprefix('controls-').replace('-', '\n', 1) for g in control_groups])
        ax.set_ylabel('Change (%)'); ax.set_title(metric)
    fig.suptitle('Phase-recording controls — all paired deltas; diamonds are group median ratios\nSame executable, staged branch; inactive hook/counter checks remain in disabled mode')
    save(fig, 'recording-overhead')
    paths = [root / name for run in runs for name in run['files'].values()]
    paths += [root / n for n in ['measurements.json', 'repeat-decisions.json', 'environment.json', 'binaries.json', 'summary.json']]
    paths += list(out.glob('*.svg')) + list(out.glob('*.png'))
    write(out / 'audit.json', dict(matplotlib=matplotlib.__version__, sha256={str(p.relative_to(root)): digest(p) for p in sorted(set(paths))}, scripts={str(Path(__file__)): digest(Path(__file__)), str(Path(__file__).with_name('measure_reconciliation.py')): digest(Path(__file__).with_name('measure_reconciliation.py'))}))
    print('Validated and plotted', summary['operations'], 'operations')


if __name__ == '__main__':
    main()
