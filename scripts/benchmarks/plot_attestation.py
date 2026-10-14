#!/usr/bin/env python3
"""Revalidate attestation evidence and export standalone SVG/PNG comparison plots."""
import argparse
import json
from pathlib import Path
from measure_attestation import PHASES, digest, load_run, summarize, write


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    root = parser.parse_args().report
    summary = summarize(root)
    runs = json.loads((root / 'measurements.json').read_text())
    env = json.loads((root / 'environment.json').read_text())
    assert env['binary_sha256'] == json.loads((root / 'binaries.json').read_text())
    for name, value in env['scripts'].items():
        assert digest(Path(__file__).with_name(name)) == value
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams.update({'font.size': 9, 'svg.hashsalt': 'rvddk-attestation'})
    out = root / 'plots'; out.mkdir(exist_ok=True)
    def selected(group):
        p = summary['groups'][group]['phases']; return p.get('repeat', p['initial'])['modes']
    def value(group, mode, metric):
        return selected(group)[mode]['metrics'][metric]['median']
    def samples(group, mode):
        return [row for run in runs if run['group'] == group for row in load_run(root, run)[0][mode]]
    def dots(ax, x, group, mode, metric):
        for run in [r for r in runs if r['group'] == group]:
            rows = load_run(root, run)[0][mode]
            ax.scatter([x] * len(rows), [r[metric] for r in rows], s=9, color='black', alpha=.25, marker='o' if run['phase'] == 'initial' else '^')
    def save(fig, name):
        fig.tight_layout()
        fig.savefig(out / (name + '.png'), dpi=160)
        fig.savefig(out / (name + '.svg'), metadata={'Date': None})
        p = out / (name + '.svg'); p.write_text('\n'.join(l.rstrip() for l in p.read_text().splitlines()) + '\n')
        plt.close(fig)
    for kind, modes in [('normal', ['owned', 'published']), ('recovery', ['staged', 'already_renamed'])]:
        groups = [g for g in summary['groups'] if g.startswith(kind + '-')]
        fig, axes = plt.subplots(len(groups), 2, figsize=(12, 3.4 * len(groups)), squeeze=False)
        for row, group in enumerate(groups):
            for ax, metric in zip(axes[row], ['wall_ms', 'cpu_ms']):
                for i, mode in enumerate(modes):
                    for j, binary in enumerate(['reference', 'candidate']):
                        key = binary + '_' + mode; x = i * 3 + j
                        ax.bar(x, value(group, key, metric), color=['#60a5fa', '#f59e0b'][j], label=binary if i == 0 else None)
                        dots(ax, x, group, key, metric)
                ax.set_xticks([.5, 3.5], [m.replace('_', ' ') for m in modes]); ax.set_ylabel('Milliseconds'); ax.set_title(group + ' / ' + metric); ax.legend()
        fig.suptitle(kind.capitalize() + ' frozen reference/candidate — repeat group medians and all observations\nDots: initial; triangles: repeats. These compare the same existing operation.')
        save(fig, kind + '-regression')
    groups = [g for g in summary['groups'] if g.startswith('attestation-')]
    modes = ['acknowledged', 'uncertain']
    for metric in ['wall_ms', 'cpu_ms']:
        fig, axes = plt.subplots(2, 2, figsize=(13, 9))
        for ax, group in zip(axes.flat, groups):
            bottom = [0., 0.]
            for phase in PHASES:
                heights = [value(group, m, phase + '.' + metric) for m in modes]
                ax.bar(range(2), heights, bottom=bottom, label=phase.replace('_', ' '))
                bottom = [a + b for a, b in zip(bottom, heights)]
            for i, mode in enumerate(modes): dots(ax, i, group, mode, metric)
            ax.set_xticks(range(2), ['Acknowledged', 'Uncertain acknowledgment']); ax.set_ylabel('Milliseconds'); ax.set_title(group.removeprefix('attestation-'))
        handles, labels = axes.flat[0].get_legend_handles_labels()
        fig.legend(handles, labels, loc='lower center', ncol=3, bbox_to_anchor=(.5, -.025))
        fig.suptitle('Attestation ' + metric + ' — phase medians and all whole-operation observations\nPhase medians need not sum to whole median; histories are not a speedup comparison.')
        fig.subplots_adjust(bottom=.12)
        # Include the external legend in both exports.
        fig.tight_layout(rect=(0, .07, 1, .95))
        fig.savefig(out / ('attestation-' + metric + '.png'), dpi=160, bbox_inches='tight')
        fig.savefig(out / ('attestation-' + metric + '.svg'), metadata={'Date': None}, bbox_inches='tight')
        p = out / ('attestation-' + metric + '.svg'); p.write_text('\n'.join(l.rstrip() for l in p.read_text().splitlines()) + '\n'); plt.close(fig)
    fig, axes = plt.subplots(2, 2, figsize=(13, 9))
    labels = []; rss = []; alloc = []; clean = []; cpu = []
    for group in groups:
        for mode in modes:
            data = selected(group)[mode]; labels.append(group.removeprefix('attestation-') + '\n' + mode)
            rss.append(data['peak_rss_kib'] / 1024); alloc.append(data['allocation_bytes'][0] / (1 << 20)); clean.append(value(group, mode, 'cleanup_ms')); cpu.append(value(group, mode, 'cleanup_cpu_ms'))
    for ax, values, title, unit in zip(axes.flat, [rss, alloc, clean, cpu], ['Lifetime peak RSS (includes setup and journal/source snapshots)', 'RAW allocation (all three checkpoints equal)', 'Explicit output cleanup wall', 'Explicit output cleanup CPU'], ['MiB', 'MiB', 'Milliseconds', 'Milliseconds']):
        ax.bar(range(len(labels)), values); ax.set_xticks(range(len(labels)), labels, rotation=55, ha='right', fontsize=7); ax.set_title(title); ax.set_ylabel(unit)
    save(fig, 'resources-and-cleanup')
    fig, axes = plt.subplots(1, 2, figsize=(12, 5))
    for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
        for x, g in enumerate(groups):
            group = g.replace('attestation-', 'controls-')
            for run in [r for r in runs if r['group'] == group]:
                data = load_run(root, run)[0]
                deltas = [100 * (a[metric] / b[metric] - 1) for a, b in zip(data['enabled'], data['disabled'])]
                ax.scatter([x] * len(deltas), deltas, alpha=.4, s=13, color='#2563eb', marker='o' if run['phase'] == 'initial' else '^')
            ax.scatter([x], [100 * (value(group, 'enabled', metric) / value(group, 'disabled', metric) - 1)], marker='D', s=50, color='#b45309')
        ax.axhline(0, color='black', linewidth=.7); ax.axhline(5, color='red', linestyle='--', linewidth=.7)
        ax.set_xticks(range(4), [g.removeprefix('attestation-').replace('-', '\n', 1) for g in groups]); ax.set_ylabel('Change (%)'); ax.set_title(metric)
    fig.suptitle('Same-binary phase recording: all paired deltas and group-median ratios (diamonds)\nInactive checks remain in disabled mode; negative values do not prove a speedup.')
    save(fig, 'recording-overhead')
    paths = [root / n for run in runs for n in run['files'].values()]
    paths += [root / n for n in ['measurements.json', 'summary.json', 'environment.json', 'repeat-decisions.json', 'binaries.json']]
    paths += list(out.glob('*.svg')) + list(out.glob('*.png'))
    write(out / 'audit.json', dict(matplotlib=matplotlib.__version__, sha256={str(p.relative_to(root)): digest(p) for p in sorted(set(paths))}, plot_script_sha256=digest(Path(__file__))))
    print('Validated and plotted', summary['operations'], 'operations')


if __name__ == '__main__':
    main()
