#!/usr/bin/env python3
"""Validate and plot native fallback gains and separate SHA-NI regression controls."""
import argparse
from pathlib import Path
import platform
import statistics as st

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np

from check_hash_fallback import validate
import measure_hash_fallback as fallback
from measure_recovery_cli import digest, load, spread, write
from plot_hash_backends import pairs

COLORS = ['#167d9a', '#ce633e']


def compact(name):
    return name.replace('stored_64', '64 MiB').replace('deflated_256', '256 MiB').replace('_', ' ')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    a = parser.parse_args()
    report = a.report.resolve()
    validate(report)
    output = report / 'plots'
    output.mkdir(exist_ok=True)
    plt.rcParams.update({'font.size': 9, 'svg.hashsalt': 'perf1b', 'axes.spines.top': False,
                         'axes.spines.right': False, 'figure.facecolor': 'white'})
    files = []

    def save(fig, name, note):
        fig.text(.5, .015, note, ha='center', fontsize=8)
        fig.tight_layout(rect=(0, .045, 1, .96))
        for extension in ['png', 'svg']:
            path = output / (name + '.' + extension)
            fig.savefig(path, dpi=155, metadata={'Date': None} if extension == 'svg' else {})
            if extension == 'svg':
                path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines()) + '\n')
            files.append(path)
        plt.close(fig)

    hashes = load(report / 'non-sha-host/hash/hash-summary.json')['groups']
    index = load(report / 'non-sha-host/hash/hash-measurements.json')
    groups = {h: load(report / h / 'workflow-summary.json')['groups'] for h in ['non-sha-host', 'sha-host']}
    entries = {h: [(compact(name) + ' / ' + (suffix.replace('_', ' ') or 'command'), ref, cand)
                   for name, group in data.items() for suffix, ref, cand in pairs(group)]
               for h, data in groups.items()}
    computed = {'hash_changes': {name: value['ring_change_percent'] for name, value in hashes.items()},
                'workflows': {}}
    fig, axes = plt.subplots(1, 3, figsize=(16, 7))
    labels = ['64 B metadata', '1 KiB metadata', '64 KiB chunk', '64 MiB dense / 1 MiB',
              '64 MiB dense / 64 KiB', '64 MiB sparse pattern', '256 MiB dense',
              '256 MiB sparse pattern', '512 MiB dense', '512 MiB sparse pattern']
    y = np.arange(len(labels))
    for ax, metric in zip(axes[:2], ['wall_ms', 'cpu_ms']):
        changes = [value['ring_change_percent'][metric] for value in hashes.values()]
        ax.barh(y, changes, color=COLORS[1], alpha=.75)
        for i, value in enumerate(hashes.values()):
            ref, cand = [value[b][metric]['round_medians_ms_per_digest'] for b in ['sha2', 'ring']]
            ax.scatter([100 * (c / r - 1) for r, c in zip(ref, cand)], [i] * 3,
                       color='#343b42', s=13, zorder=3)
        ax.axvline(0, color='black', lw=.8)
        ax.set_yticks(y, labels)
        ax.invert_yaxis()
        ax.set_xlabel('ring / sha2 elapsed change (%) — negative is faster')
        ax.set_title('Wall time' if metric == 'wall_ms' else 'CPU time')
        ax.grid(axis='x', alpha=.15)
    for i, backend in enumerate(['sha2', 'ring']):
        rates = [case[1] / (1 << 20) * 1000 / hashes[case[0]][backend]['wall_ms']['median_ms_per_digest']
                 for case in fallback.CASES]
        axes[2].barh(y + (i - .5) * .35, rates, height=.35, color=COLORS[i], label=backend)
    axes[2].set_yticks(y, labels)
    axes[2].invert_yaxis()
    axes[2].set_xlabel('Memory hashing MiB/s; includes init/finalize')
    axes[2].set_title('Native Xeon guest, SHA-NI unavailable')
    axes[2].legend()
    fig.suptitle('PERF.1b — same-runner SHA-256 fallback comparisons', fontsize=14)
    save(fig, 'native-hash', '720 observations; all groups repeated. Bars: median of repeat-round medians. Dots: round ratios, not confidence intervals. No historical-host ratios.')

    for host, title in [('non-sha-host', 'Native non-SHA runner — conditional bulk fallback'),
                        ('sha-host', 'SHA-NI host — preserve the existing accelerated backend')]:
        fig, axes = plt.subplots(1, 2, figsize=(15, 9))
        computed['workflows'][host] = []
        for label, ref, cand in entries[host]:
            computed['workflows'][host].append({'label': label,
                'change_percent': {k: 100 * (cand[k]['median'] / ref[k]['median'] - 1) for k in ['wall_ms', 'cpu_ms']},
                'reference_wall_ms': ref['wall_ms']['median'], 'candidate_wall_ms': cand['wall_ms']['median']})
        for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
            changes = [100 * (c[metric]['median'] / r[metric]['median'] - 1) for _, r, c in entries[host]]
            ax.barh(np.arange(len(changes)), changes, color=COLORS[1], alpha=.75)
            for i, (_, ref, cand) in enumerate(entries[host]):
                dots = [100 * (c / r - 1) for r, c in zip(ref[metric]['round_medians'], cand[metric]['round_medians'])]
                ax.scatter(dots, [i] * 3, color='#343b42', s=13, zorder=3)
            ax.set_yticks(np.arange(len(changes)), [e[0] for e in entries[host]], fontsize=8)
            ax.invert_yaxis()
            ax.axvline(0, color='black', lw=.8)
            ax.axvline(5, color='#7d8790', ls='--', lw=.8)
            ax.set_xlabel('Candidate / reference change (%) — positive is slower')
            ax.set_title('Full-operation wall time' if metric == 'wall_ms' else 'Full-operation CPU time')
            ax.grid(axis='x', alpha=.15)
        fig.suptitle(title, fontsize=14)
        save(fig, host + '-workflows', '1,152 observations per host; three initial rounds of four pairs plus three rounds of eight pairs. Separate same-host comparisons; no cross-host speedup claim.')

    fig, axes = plt.subplots(1, 3, figsize=(19, 10))
    for panel, host in enumerate(['hash', 'non-sha-host', 'sha-host']):
        ax = axes[panel]
        labels_ = list(hashes) if host == 'hash' else [e[0] for e in entries[host]]
        for stage, marker in [('initial', 'o'), ('repeat', '^')]:
            for binary, color in zip(['reference', 'candidate'], COLORS):
                values = []
                if host == 'hash':
                    backend = 'sha2' if binary == 'reference' else 'ring'
                    for name in hashes:
                        medians = []
                        for run in index:
                            if run['group'] == name and run['stage'] == stage:
                                rows = load(report / 'non-sha-host/hash' / run['file'])['rows']
                                medians.append(st.median(r['wall_ms'] / r['iterations'] for r in rows if r['backend'] == backend))
                        values.append(spread(medians))
                else:
                    for group in groups[host].values():
                        for _, ref, cand in pairs(group, stage):
                            values.append((ref if binary == 'reference' else cand)['wall_ms']['spread_percent'])
                ax.scatter(values, np.arange(len(labels_)) + (-.12 if binary == 'reference' else .12),
                           marker=marker, color=color, s=22, label=binary + ' ' + stage)
        ax.set_yticks(np.arange(len(labels_)), labels_, fontsize=7)
        ax.invert_yaxis()
        ax.set_xlim(left=0)
        ax.axvline(5, color='#7d8790', ls='--', lw=.8)
        ax.set_xlabel('Wall round-median spread (%)')
        ax.set_title({'hash': 'Non-SHA hash matrix', 'non-sha-host': 'Non-SHA workflow controls', 'sha-host': 'SHA-NI workflow controls'}[host])
        ax.legend(fontsize=7, loc='best')
    fig.suptitle('All initial and repeat spread stays visible', fontsize=14)
    save(fig, 'variation', 'Spread = 100 × (max / min − 1), not a confidence interval. CPU frequency, host load and hypervisor scheduling are not isolated. +5% remains an investigation threshold.')

    fig, axes = plt.subplots(1, 2, figsize=(15, 7))
    for ax, host in zip(axes, ['non-sha-host', 'sha-host']):
        selected = [(n, r, c) for n, r, c in entries[host] if 'verification.cpu_ms' in r]
        for i, binary in enumerate(['reference', 'candidate']):
            values = [(r if binary == 'reference' else c)['verification.cpu_ms']['median'] for _, r, c in selected]
            ax.barh(np.arange(len(selected)) + (i - .5) * .35, values, height=.35, color=COLORS[i], label=binary)
        ax.set_yticks(np.arange(len(selected)), [n for n, _, _ in selected], fontsize=8)
        ax.invert_yaxis()
        ax.set_xlabel('Verification phase CPU (ms), median of repeat-round medians')
        ax.set_title('Non-SHA guest' if host == 'non-sha-host' else 'SHA-NI host')
        ax.legend()
    fig.suptitle('Fresh verification retains every byte comparison and hash', fontsize=14)
    save(fig, 'verification-cpu', 'Phase includes full logical-source read, RAW read, comparison, hash and checks. It is not an isolated hash timer. Compare reference/candidate within each panel.')

    provenance = load(report / 'candidate-provenance.json')
    fig, axes = plt.subplots(1, 3, figsize=(17, 6))
    axes[0].bar(['reference', 'candidate'], [provenance['binaries'][b + '-cli']['bytes'] / (1 << 20)
               for b in ['reference', 'candidate']], color=COLORS)
    axes[0].set_ylabel('Release executable size (MiB)')
    axes[0].set_title('Identical frozen binaries on both hosts')
    for ax, host in zip(axes[1:], ['non-sha-host', 'sha-host']):
        child = [(n, r, c) for n, r, c in entries[host] if n.endswith('/ cli') or n.startswith(('help ', 'assess '))]
        for i, binary in enumerate(['reference', 'candidate']):
            values = [(r if binary == 'reference' else c)['peak_rss_kib']['median'] / 1024 for _, r, c in child]
            ax.barh(np.arange(len(child)) + (i - .5) * .35, values, height=.35, color=COLORS[i], label=binary)
        ax.set_yticks(np.arange(len(child)), [n for n, _, _ in child], fontsize=7)
        ax.invert_yaxis()
        ax.set_xlabel('Child lifetime high-water RSS (MiB)')
        ax.set_title('Non-SHA guest' if host == 'non-sha-host' else 'SHA-NI host')
        ax.legend()
    fig.suptitle('Executable size and matched process memory controls', fontsize=14)
    save(fig, 'resources', 'Child peak RSS can include inherited launcher memory before exec (PERF.1a diagnostic). Native peaks include setup. These are not isolated CLI working sets.')
    supplemental = load(report / 'assessment-control-summary.json')['groups']
    original = groups['sha-host']['assess']['stages']
    fig, axes = plt.subplots(1, 2, figsize=(12, 5))
    labels_ = ['Original A/B initial', 'Original A/B repeat', 'Diagnostic A/A', 'Diagnostic A/B']
    computed['assessment_controls'] = {}
    for ax, metric in zip(axes, ['wall_ms', 'cpu_ms']):
        samples = [(original[stage]['reference'][metric]['round_medians'],
                    original[stage]['candidate'][metric]['round_medians']) for stage in ['initial', 'repeat']]
        samples += [(supplemental[g]['reference'][metric], supplemental[g]['candidate'][metric]) for g in ['identical', 'candidate']]
        changes = [100 * (st.median(c) / st.median(r) - 1) for r, c in samples]
        computed['assessment_controls'][metric] = dict(zip(labels_, changes))
        ax.barh(np.arange(4), changes, color=COLORS[1], alpha=.75)
        for i, (ref, cand) in enumerate(samples):
            ax.scatter([100 * (c / r - 1) for r, c in zip(ref, cand)], [i] * 3, color='#343b42', s=16, zorder=3)
        ax.set_yticks(np.arange(4), labels_)
        ax.invert_yaxis()
        ax.axvline(0, color='black', lw=.8)
        ax.axvline(5, color='#7d8790', ls='--', lw=.8)
        ax.set_xlabel('Second / first label elapsed change (%)')
        ax.set_title('Assessment wall time' if metric == 'wall_ms' else 'Assessment CPU time')
    fig.suptitle('Adverse assessment startup result retained alongside diagnostic controls', fontsize=13)
    save(fig, 'assessment-controls', '96 additional observations after the original SHA-host campaign. A/A uses the exact reference executable twice. Diagnostics do not replace original samples or clear variance.')
    write(output / 'computed.json', computed)
    inputs = {str(p.relative_to(report)): digest(p) for p in report.rglob('*.json')
              if output not in p.parents and p.name != 'artifact-manifest.json'}
    write(output / 'manifest.json', {'python': platform.python_version(), 'matplotlib': matplotlib.__version__,
        'numpy': np.__version__, 'generators': {n: digest(Path(__file__).parent / n) for n in
            ['plot_hash_fallback.py', 'plot_hash_backends.py', 'check_hash_fallback.py',
             'measure_hash_fallback.py', 'measure_hash_backends.py', 'measure_hash_workflows.py', 'measure_assessment_controls.py']},
        'inputs': inputs, 'outputs': {p.name: digest(p) for p in files}, 'computed_sha256': digest(output / 'computed.json')})
    print('Validated evidence and generated seven PNG/SVG pairs')


if __name__ == '__main__':
    main()
