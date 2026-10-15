#!/usr/bin/env python3
"""Replay PERF.1c evidence checks and plot both CPU profiles without cross-host ratios."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from measure_transfer_hashes import digest, write
from check_transfer_hashes import check


def label(group):
    if group == 'startup': return 'CLI --help (whole process)'
    kind, fs, case, op = group.split('/')
    return f'{op} / {case} / {fs}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    report = parser.parse_args().report
    check(report)
    data = {host: json.loads((report / host / 'summary.json').read_text())['groups'] for host in ['non-sha-host', 'sha-host']}
    out = report / 'plots'; out.mkdir(exist_ok=True)
    plt.rcParams.update({'font.size': 9, 'svg.hashsalt': 'perf1c', 'axes.spines.top': False, 'axes.spines.right': False})
    files = []

    def save(fig, name, note):
        fig.text(.5, .012, note, ha='center', fontsize=8)
        fig.tight_layout(rect=(0, .04, 1, .97))
        for ext in ['png', 'svg']:
            path = out / f'{name}.{ext}'
            fig.savefig(path, dpi=150, metadata={'Date': None} if ext == 'svg' else {})
            if ext == 'svg': path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines()) + '\n')
            files.append(path)
        plt.close(fig)

    for host, groups in data.items():
        names = list(groups)
        fig, axes = plt.subplots(1, 2, figsize=(15, 11), sharey=True)
        for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
            for i, name in enumerate(names):
                g = groups[name]
                ref = g['repeat']['reference'][metric]['round_medians']
                cand = g['repeat']['candidate'][metric]['round_medians']
                changes = [(c / r - 1) * 100 for r, c in zip(ref, cand)]
                value = g['repeat']['change_percent'][metric]
                ax.barh(i, value, color='#187f99' if value <= 0 else '#ca6347', alpha=.8)
                ax.plot([min(changes), max(changes)], [i, i], color='#293942', lw=1.2)
                ax.scatter(g['initial']['change_percent'][metric], i, marker='x', color='#6b4a95', s=24, zorder=3)
            ax.axvline(0, color='black', lw=.8)
            ax.axvline(5, color='gray', lw=.8, ls=':')
            ax.set_xlabel(('Wall' if metric == 'wall_ms' else 'CPU') + ' change (%) — negative is less time')
            ax.grid(axis='x', alpha=.2)
        axes[0].set_yticks(range(len(names)), [label(n) for n in names]); axes[0].invert_yaxis()
        fig.suptitle(('Native non-SHA runner' if host == 'non-sha-host' else 'SHA-NI host') + ': complete operations and worker phases')
        save(fig, host + '-changes', 'Bars: repeat median change; lines: range of matched round-median changes; crosses: initial change. Normal cache; loopback fixtures, not live ESXi.')

    selected = ['transfer/disk/bulk/proof-export', 'transfer/disk/bulk/owned-transfer', 'artifact/disk/bulk/artifact-admission', 'worker/disk/bulk/worker-seal', 'worker/disk/bulk/worker-admit']
    fig, axes = plt.subplots(2, 2, figsize=(14, 8))
    for row, (host, groups) in enumerate(data.items()):
        for ax, metric in zip(axes[row], ['wall_ms', 'cpu_ms']):
            y = np.arange(len(selected))
            for offset, version, color in [(-.18, 'reference', '#768995'), (.18, 'candidate', '#187f99')]:
                values = [groups[n]['repeat'][version][metric]['median'] for n in selected]
                ax.barh(y + offset, values, height=.34, color=color, label=version)
            ax.set_yticks(y, [n.split('/')[-1] for n in selected]); ax.invert_yaxis()
            ax.set_title(host + ' / ' + ('wall' if metric == 'wall_ms' else 'CPU'))
            ax.set_xlabel('milliseconds per operation'); ax.legend(); ax.grid(axis='x', alpha=.2)
    save(fig, 'bulk-times', '8 MiB-class encoded payloads, primary filesystem, repeat medians. Worker phases are separately timed; do not sum them into the TLS operations.')

    fig, axes = plt.subplots(1, 3, figsize=(15, 5))
    for ax, metric in zip(axes[:2], ['wall_ms', 'cpu_ms']):
        for host_index, (host, groups) in enumerate(data.items()):
            for offset, version, color in [(-.18, 'reference', '#768995'), (.18, 'candidate', '#187f99')]:
                g = groups['startup']['repeat'][version][metric]
                ax.bar(host_index + offset, g['median'], width=.34, color=color, label=version if host_index == 0 else None)
                ax.scatter([host_index + offset] * 3, g['round_medians'], color='#293942', s=15)
        ax.set_xticks([0, 1], ['non-SHA', 'SHA-NI']); ax.set_ylabel(metric); ax.set_title('CLI --help startup'); ax.legend()
    names = ['transfer/disk/bulk/owned-transfer', 'artifact/disk/bulk/artifact-admission', 'worker/disk/bulk/worker-admit', 'startup']
    y = np.arange(len(names))
    for offset, (host, groups) in zip([-.18, .18], data.items()):
        values = [groups[n]['repeat']['candidate']['process_peak_rss_kib']['median'] / 1024 for n in names]
        axes[2].barh(y + offset, values, height=.34, label=host)
    axes[2].set_yticks(y, ['transfer', 'artifact', 'worker', 'startup']); axes[2].set_xlabel('candidate lifetime peak RSS (MiB)'); axes[2].legend()
    save(fig, 'process-costs', 'Startup dots: three repeat round medians. RSS includes fixture setup/TLS; startup child RSS may include inherited Python memory. No isolated working-set claim.')
    entries = [(host, name, group) for host in data
               for name, group in json.loads((report / host / 'control-summary.json').read_text())['groups'].items()]
    fig, axes = plt.subplots(1, 2, figsize=(15, 7), sharey=True)
    for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
        for i, (host, name, g) in enumerate(entries):
            for offset, mode, color in [(-.23, 'original', '#ca6347'), (0, 'AA', '#768995'), (.23, 'AB', '#187f99')]:
                value = data[host][name]['repeat']['change_percent'][metric] if mode == 'original' else g[mode]['change_percent'][metric]
                ax.barh(i + offset, value, height=.21, color=color, label=mode if i == 0 else None)
        ax.axvline(0, color='black', lw=.8); ax.axvline(5, color='gray', lw=.8, ls=':')
        ax.set_xlabel(metric.replace('_ms', '') + ' change (%)'); ax.legend(); ax.grid(axis='x', alpha=.2)
    axes[0].set_yticks(range(len(entries)), [host + ' / ' + label(name) for host, name, _ in entries]); axes[0].invert_yaxis()
    fig.suptitle('Original adverse results retained beside supplemental adjacent-pair controls')
    save(fig, 'regression-controls', 'Original: main repeat comparison. AA: identical reference binary. AB: unchanged candidate versus reference. Three rounds of eight adjacent pairs; variation remains open.')
    write(report / 'plot-index.json', {'files': {str(p.relative_to(report)): digest(p) for p in files}, 'matplotlib': matplotlib.__version__})


if __name__ == '__main__': main()
