#!/usr/bin/env python3
"""Validate and render PERF.1a evidence; never executes a measured workload."""
import argparse
import json
from pathlib import Path
import platform
import statistics as st

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np

import measure_hash_backends as micro
import measure_hash_workflows as workflow
from measure_recovery_cli import digest, load, spread, write

COLORS = ['#167d9a', '#ce633e']


def collect(report):
    old_hash = load(report / 'hash-summary.json')
    old_work = load(report / 'workflow-summary.json')
    micro.summarize(report)
    workflow.summarize(report)
    assert load(report / 'hash-summary.json') == old_hash
    assert load(report / 'workflow-summary.json') == old_work
    index = load(report / 'hash-measurements.json')
    assert len(index) == 48 and old_hash['timed_observations'] == 576
    groups = {}
    for case in micro.CASES:
        name = case[0]
        groups[name] = {}
        for stage, pairs in [('initial', 4), ('repeat', 8)]:
            runs = [r for r in index if r['group'] == name and r['stage'] == stage]
            assert [r['round'] for r in runs] == [1, 2, 3]
            assert all(r['pairs'] == pairs for r in runs)
            groups[name][stage] = {b: {k: [] for k in ['wall_ms', 'cpu_ms']} for b in ['sha2', 'ring']}
            for run in runs:
                rows = micro.validate(load(report / run['file']), pairs, case)
                for backend in ['sha2', 'ring']:
                    for metric in ['wall_ms', 'cpu_ms']:
                        groups[name][stage][backend][metric].append(st.median(
                            r[metric] / r['iterations'] for r in rows if r['backend'] == backend))
    assert set(r['group'] for r in index) == set(groups)
    return groups, old_work['groups']


def pairs(group, stage='repeat'):
    modes = group['stages'][stage]
    for ref in modes:
        if ref.startswith('reference'):
            candidate = ref.replace('reference', 'candidate', 1)
            assert candidate in modes
            yield ref.removeprefix('reference').lstrip('_'), modes[ref], modes[candidate]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    a = parser.parse_args()
    report = a.report.resolve()
    hashes, workflows = collect(report)
    output = report / 'plots'
    output.mkdir(exist_ok=True)
    plt.rcParams.update({'font.size': 9, 'svg.hashsalt': 'perf1a', 'axes.spines.top': False,
                         'axes.spines.right': False, 'figure.facecolor': 'white'})
    outputs = []

    def save(fig, name, note):
        fig.text(.5, .015, note, ha='center', fontsize=8)
        fig.tight_layout(rect=(0, .045, 1, .96))
        for ext in ['png', 'svg']:
            path = output / (name + '.' + ext)
            fig.savefig(path, dpi=155, metadata={'Date': None} if ext == 'svg' else {})
            if ext == 'svg':
                path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines()) + '\n')
            outputs.append(path)
        plt.close(fig)

    fig, axes = plt.subplots(1, 3, figsize=(16, 6))
    labels = ['64 B metadata', '1 KiB metadata', '64 KiB chunk', '64 MiB dense / 1 MiB',
              '64 MiB dense / 64 KiB', '64 MiB sparse pattern', '256 MiB dense', '256 MiB sparse pattern']
    y = np.arange(len(labels))
    for ax, metric in zip(axes[:2], ['wall_ms', 'cpu_ms']):
        values, rounds = [], []
        for g in hashes.values():
            ref, cand = [g['repeat'][b][metric] for b in ['sha2', 'ring']]
            values.append(100 * (st.median(cand) / st.median(ref) - 1))
            rounds.append([100 * (b / a - 1) for a, b in zip(ref, cand)])
        ax.barh(y, values, color=COLORS[1], alpha=.75)
        for i, values_ in enumerate(rounds):
            ax.scatter(values_, [i] * 3, color='#343b42', s=13, zorder=3)
        ax.axvline(0, color='black', lw=.8)
        ax.axvline(5, color='#7d8790', ls='--', lw=.8)
        ax.set_yticks(y, labels)
        ax.invert_yaxis()
        ax.set_xlabel('ring / sha2 change (%) — positive is slower')
        ax.set_title('Wall time' if metric == 'wall_ms' else 'CPU time')
        ax.grid(axis='x', alpha=.15)
    for i, backend in enumerate(['sha2', 'ring']):
        rates = [case[1] / (1 << 20) * 1000 / st.median(hashes[case[0]]['repeat'][backend]['wall_ms'])
                 for case in micro.CASES]
        axes[2].barh(y + (i - .5) * .35, rates, height=.35, label=backend, color=COLORS[i])
    axes[2].set_yticks(y, labels)
    axes[2].invert_yaxis()
    axes[2].set_xlabel('Memory hashing MiB/s (zero-based axis)')
    axes[2].set_title('Effective hashing rate, includes init/finalize')
    axes[2].legend()
    fig.suptitle('PERF.1a — existing sha2 wins every measured SHA-NI hash group', fontsize=14)
    save(fig, 'hash-backends', '576 observations; all groups repeated. Bars: median of three repeat-round medians. Dots: round ratios, not confidence intervals.')

    entries = [(name + ' / ' + (suffix or 'command'), ref, cand)
               for name, group in workflows.items() for suffix, ref, cand in pairs(group)]
    fig, axes = plt.subplots(1, 2, figsize=(15, 9))
    for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
        changes = [100 * (c[metric]['median'] / r[metric]['median'] - 1) for _, r, c in entries]
        ax.barh(np.arange(len(entries)), changes, color=COLORS[1], alpha=.75)
        for i, (_, ref, cand) in enumerate(entries):
            dots = [100 * (c / r - 1) for r, c in zip(ref[metric]['round_medians'], cand[metric]['round_medians'])]
            ax.scatter(dots, [i] * 3, color='#343b42', s=13, zorder=3)
        ax.set_yticks(np.arange(len(entries)), [e[0] for e in entries], fontsize=8)
        ax.invert_yaxis()
        ax.axvline(0, color='black', lw=.8)
        ax.axvline(5, color='#7d8790', ls='--', lw=.8)
        ax.set_xlabel('Candidate / reference change (%) — positive is slower')
        ax.set_title('Full-operation wall time' if metric == 'wall_ms' else 'Full-operation CPU time')
        ax.grid(axis='x', alpha=.15)
    fig.suptitle('Frozen retained-payload candidate — all repeat groups, including adverse results', fontsize=14)
    save(fig, 'workflow-changes', '1,152 observations; 3 × 4 initial pairs plus 3 × 8 repeat pairs per group. No CPU/storage/cache equivalence beyond the recorded host.')

    fig, axes = plt.subplots(1, 2, figsize=(15, 9))
    for ax, family in zip(axes, ['hash', 'workflow']):
        labels_ = list(hashes) if family == 'hash' else [e[0] for e in entries]
        for stage, marker in [('initial', 'o'), ('repeat', '^')]:
            for backend, color in zip(['reference', 'candidate'], COLORS):
                values = []
                if family == 'hash':
                    b = 'sha2' if backend == 'reference' else 'ring'
                    values = [spread(g[stage][b]['wall_ms']) for g in hashes.values()]
                else:
                    for group in workflows.values():
                        for _, ref, cand in pairs(group, stage):
                            values.append((ref if backend == 'reference' else cand)['wall_ms']['spread_percent'])
                offset = (-.12 if backend == 'reference' else .12) + (-.04 if stage == 'initial' else .04)
                ax.scatter(values, np.arange(len(labels_)) + offset, color=color, marker=marker,
                           label=backend + ' ' + stage, s=24, alpha=.85)
        ax.set_yticks(np.arange(len(labels_)), labels_, fontsize=8)
        ax.invert_yaxis()
        ax.set_xlim(left=0)
        ax.axvline(5, color='#7d8790', ls='--', lw=.8)
        ax.set_xlabel('Spread of round wall medians: 100 × (max / min − 1) %')
        ax.set_title('Hash microbenchmarks' if family == 'hash' else 'Full operations and commands')
        ax.legend(fontsize=8, loc='best')
    fig.suptitle('Timing variation stays open — initial observations and all longer repeats', fontsize=14)
    save(fig, 'round-variation', 'Spread is descriptive, not a confidence interval. Fixed CPU affinity; SMT, frequency and competing host activity not isolated.')

    provenance = load(report / 'candidate-provenance.json')
    fig, axes = plt.subplots(1, 2, figsize=(13, 5))
    axes[0].bar(['reference CLI', 'candidate CLI'], [provenance['binaries'][b + '-cli']['bytes'] / (1 << 20)
                for b in ['reference', 'candidate']], color=COLORS)
    axes[0].set_ylabel('Release executable size (MiB)')
    axes[0].set_title('Exact frozen binaries (unstripped)')
    child = [(n, r, c) for n, r, c in entries if n.endswith('/ cli') or n.startswith(('help ', 'assess '))]
    for i, b in enumerate(['reference', 'candidate']):
        vals = [(r if b == 'reference' else c)['peak_rss_kib']['median'] / 1024 for _, r, c in child]
        axes[1].barh(np.arange(len(child)) + (i - .5) * .35, vals, height=.35, color=COLORS[i], label=b)
    axes[1].set_yticks(np.arange(len(child)), [n for n, _, _ in child], fontsize=8)
    axes[1].invert_yaxis()
    axes[1].set_xlabel('Child lifetime peak RSS, including launch memory (MiB)')
    axes[1].set_title('Recorded child high-water marks; not isolated CLI working sets')
    axes[1].legend()
    fig.suptitle('Linkage and process memory controls', fontsize=14)
    save(fig, 'cli-resources', 'Child RSS can include inherited launcher memory (diagnostic confirmed). Native/probe peaks include setup. Compare only matched scopes.')
    inputs = {p.name: digest(p) for p in report.glob('*.json') if p.name != 'artifact-manifest.json'}
    write(output / 'manifest.json', {'python': platform.python_version(), 'matplotlib': matplotlib.__version__,
        'numpy': np.__version__, 'generator_sha256': digest(Path(__file__)), 'inputs': inputs,
        'outputs': {p.name: digest(p) for p in outputs}})
    print('Validated inputs; generated four PNG/SVG pairs')


if __name__ == '__main__':
    main()
