#!/usr/bin/env python3
"""Render saved Criterion evidence; never execute benchmarks or rewrite raw data."""

import argparse
import hashlib
import importlib.metadata
import json
import math
from pathlib import Path
import platform
import statistics
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
from matplotlib.patches import Patch

COLORS = {"baseline": "#2879A5", "candidate": "#D87927"}
VARIANTS = ("baseline", "candidate")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def close(actual, expected, label):
    require(math.isclose(actual, expected, rel_tol=1e-9, abs_tol=1e-9),
            f"{label}: computed {actual}, recorded {expected}")


def samples(run):
    sample = run["criterion"]["sample.json"]
    times, iters = sample["times"], sample["iters"]
    require(len(times) == len(iters) and len(times) > 0, "invalid sample lengths")
    require(all(math.isfinite(x) and x > 0 for x in times + iters),
            "sample times and iterations must be finite and positive")
    values = [time / count for time, count in zip(times, iters)]
    close(statistics.median(values), median(run), f"{run['tag']} sample median")
    return values


def median(run):
    return run["criterion"]["estimates.json"]["median"]["point_estimate"]


def compare(runs, summary):
    """Match by explicit pair IDs, not file order; validate published summaries."""
    require(bool(runs), "empty comparison")
    if isinstance(summary, dict):
        summary = [summary]
    require(len({r['tag'] for r in runs}) == len(runs), "duplicate run tag")
    for run in runs:
        samples(run)
    cases = list(dict.fromkeys(run["benchmark"] for run in runs))
    require(set(cases) == {r['benchmark'] for r in summary}, "summary workload mismatch")
    require(len(summary) == len(cases), "duplicate summary workload")
    result = []
    for case in cases:
        by_variant = {variant: {} for variant in VARIANTS}
        for run in (r for r in runs if r["benchmark"] == case):
            variant, pair = run["variant"], run["pair"]
            require(variant in VARIANTS, f"unknown variant {variant}")
            require(pair not in by_variant[variant], f"duplicate {case}/{variant}/{pair}")
            by_variant[variant][pair] = run
        pairs = sorted(by_variant["baseline"])
        require(pairs and pairs == sorted(by_variant["candidate"]), f"unmatched pairs: {case}")
        values = {v: [median(by_variant[v][p]) for p in pairs] for v in VARIANTS}
        aggregates = {v: statistics.median(values[v]) for v in VARIANTS}
        changes = [100 * (c / b - 1) for b, c in zip(values['baseline'], values['candidate'])]
        change = 100 * (aggregates['candidate'] / aggregates['baseline'] - 1)
        saved = next(row for row in summary if row['benchmark'] == case)
        for variant in VARIANTS:
            close(aggregates[variant], saved[variant + '_ns'], case)
        close(change, saved['change_percent'], case)
        require(len(changes) == len(saved['paired_changes_percent']), "paired summary length mismatch")
        for actual, expected in zip(changes, saved['paired_changes_percent']):
            close(actual, expected, case)
        result.append(dict(benchmark=case, pair_ids=pairs, run_medians_ns=values,
                           baseline_ns=aggregates['baseline'], candidate_ns=aggregates['candidate'],
                           change_percent=change, paired_changes_percent=changes))
    return result


class Report:
    def __init__(self, directory, output):
        self.directory, self.output = directory, output
        self.inputs, self.outputs = {}, []
        self.config = self.read('plot-config.json')
        self.environment = self.read('environment.json')
        self.identity = (f"Baseline {self.environment['baseline'][:7]} → candidate "
                         f"{self.config['candidate']} · lower time is better")

    def read(self, name):
        path = self.directory / name
        self.inputs[name] = digest(path)
        return json.loads(path.read_text())

    def label(self, case):
        return self.config['workloads'][case]['label']

    def phase(self, case):
        phase = self.config['workloads'][case]['phase']
        require(phase in ('planning', 'copy'), f"unknown phase {phase}")
        return phase

    def units(self, case):
        return (1000, 'µs') if self.phase(case) == 'planning' else (1e6, 'ms')

    def panels(self, count):
        columns = min(count, 2)
        rows = math.ceil(count / columns)
        fig, axes = plt.subplots(rows, columns, figsize=(12, 2.9 * rows + 2.5), squeeze=False)
        flat = list(axes.flat)
        for ax in flat[count:]:
            ax.set_visible(False)
        return fig, flat[:count]

    def save(self, fig, name, title, note, sampling=None, legend=True, candidate_only=False):
        height = fig.get_figheight()
        fig.suptitle(f"{self.config['title']}\n{title}", x=.055, y=1-.12/height,
                     ha='left', fontsize=17, fontweight='bold').set_in_layout(False)
        identity = self.identity if not candidate_only and name != 'unsupported-discovery' else (f"Candidate {self.config['candidate']} · no baseline timing comparison")
        fig.text(.055, 1-.98/height, identity, fontsize=10, color='#465568')
        if legend:
            fig.legend(handles=[Patch(facecolor=COLORS[v], label=v.title()) for v in VARIANTS],
                       loc='upper right', bbox_to_anchor=(.96, 1-.75/height), ncol=2, frameon=False)
        fig.text(.055, .15/height, self.config['conditions'] + '\n' +
                 (sampling or self.config['main_sampling']) + '\n' + note,
                 fontsize=9, color='#465568', linespacing=1.5)
        bottom = 1.5 if name in ('relative-change', 'followup-change') else .95
        fig.tight_layout(rect=(.025, bottom/height, .985, 1-1.3/height), h_pad=2.5, w_pad=3)
        for suffix in ('svg', 'png'):
            path = self.output / f'{name}.{suffix}'
            metadata = {'Creator': 'rvvdk benchmark plots', 'Date': None} if suffix == 'svg' else {'Software': 'rvvdk benchmark plots'}
            fig.savefig(path, dpi=160, metadata=metadata)
            self.outputs.append(path)
        plt.close(fig)

    def latency(self, rows, phase):
        rows = [r for r in rows if self.phase(r['benchmark']) == phase]
        if not rows:
            return
        fig, axes = self.panels(len(rows))
        for ax, row in zip(axes, rows):
            scale, unit = self.units(row['benchmark'])
            values = [row[v + '_ns'] / scale for v in VARIANTS]
            ax.bar(VARIANTS, values, color=list(COLORS.values()), width=.55)
            for index, variant in enumerate(VARIANTS):
                points = row['run_medians_ns'][variant]
                ax.text(index, max(points)/scale + max(values)*.065, f'{values[index]:.3f}', ha='center', fontsize=11)
                xs = [index + (i - (len(points)-1)/2) * .055 for i in range(len(points))]
                ax.scatter(xs, [v/scale for v in points], s=25, color='white', edgecolor='#293747', zorder=3)
            ax.set_ylim(0, max(max(row['run_medians_ns'][v]) / scale for v in VARIANTS) * 1.28)
            ax.set_ylabel(f'Elapsed time ({unit})')
            ax.set_title(f"{self.label(row['benchmark'])}\n{row['change_percent']:+.2f}% aggregate", fontsize=11)
        self.save(fig, f'{phase}-latency', f'{phase.title()} latency · baseline vs candidate',
                  'Bars: median of run medians. Dots: individual run medians. Zero-based axes; panel scales differ.')

    def changes(self, rows, name='relative-change', title='Elapsed-time change · every paired run', labels=None, sampling=None):
        fig, ax = plt.subplots(figsize=(12, 4 + len(rows) * .5))
        for i, row in enumerate(rows):
            delta = row['change_percent']
            ax.barh(i, delta, color='#C55A41' if delta > 0 else '#268477', height=.46)
            points = row['paired_changes_percent']
            ax.scatter(points, [i + (j-(len(points)-1)/2)*.1 for j in range(len(points))],
                       marker='o', s=33, color='#263445', edgecolor='white', linewidth=.6, zorder=3)
            ax.annotate(f'{delta:+.2f}%', (delta, i), xytext=(7 if delta >= 0 else -7, 14),
                        textcoords='offset points', ha='left' if delta >= 0 else 'right', fontsize=10)
        ax.set_yticks(range(len(rows)), labels or [self.label(r['benchmark']) for r in rows])
        ax.invert_yaxis()
        ax.axvline(0, color='#526274', linewidth=1)
        ax.axvline(5, color='#85919F', linestyle='--', linewidth=1)
        ax.margins(x=.18, y=.22)
        ax.set_xlabel('Candidate elapsed-time change (%) · positive = slower')
        ax.grid(axis='x')
        ax.grid(axis='y', visible=False)
        fig.legend(handles=[Patch(facecolor='#C55A41', label='Aggregate (ratio of medians)'),
                           Line2D([], [], marker='o', linestyle='', color='#263445', label='Matched run pair'),
                           Line2D([], [], linestyle='--', color='#85919F', label='+5% investigation threshold')],
                  loc='lower center', bbox_to_anchor=(.60, 1.03/fig.get_figheight()), ncol=3, fontsize=8, frameon=False)
        self.save(fig, name, title,
                  'Paired dots show observed variation, not confidence intervals. The threshold is policy, not statistical significance.',
                  sampling=sampling, legend=False)

    def distributions(self, rows, runs):
        fig, axes = self.panels(len(rows))
        for ax, row in zip(axes, rows):
            scale, unit = self.units(row['benchmark'])
            for index, pair in enumerate(row['pair_ids']):
                for variant, shift in [('baseline', -.18), ('candidate', .18)]:
                    run = next(r for r in runs if r['benchmark'] == row['benchmark'] and r['pair'] == pair and r['variant'] == variant)
                    ax.boxplot([[v/scale for v in samples(run)]], positions=[index+shift], widths=.28,
                               patch_artist=True, manage_ticks=False, showfliers=True,
                               boxprops={'facecolor': COLORS[variant], 'alpha': .8},
                               medianprops={'color': '#162334', 'linewidth': 1.5},
                               flierprops={'marker': '.', 'markersize': 4, 'markeredgecolor': COLORS[variant]})
            ax.set_xticks(range(len(row['pair_ids'])), [f'Pair {p}' for p in row['pair_ids']])
            ax.set_ylabel(f'Time / iteration ({unit})')
            ax.set_title(self.label(row['benchmark']), fontsize=11)
        self.save(fig, 'sample-distributions', 'Sample distributions · each run kept separate',
                  'Boxes: Q1–Q3; line: median; whiskers: 1.5×IQR; all outliers shown. Zoomed axes; scales differ. Not per-I/O latency.')

    def unsupported(self, runs, experiment=None):
        cases = list(dict.fromkeys(r['benchmark'] for r in runs))
        fig, axes = self.panels(len(cases))
        data = []
        for ax, case in zip(axes, cases):
            selected = [r for r in runs if r['benchmark'] == case]
            for run in selected:
                samples(run)
            values = [median(r) for r in selected]
            value = statistics.median(values)
            scale, unit = self.units(case)
            bar = ax.bar(['Candidate only'], [value/scale], color=COLORS['candidate'], width=.45)
            ax.bar_label(bar, labels=[f'{value/scale:.3f}'], padding=7)
            ax.scatter([(i-(len(values)-1)/2)*.04 for i in range(len(values))], [v/scale for v in values],
                       s=25, color='white', edgecolor='#293747', zorder=3)
            ax.set_ylim(0, max(values)/scale*1.3)
            ax.set_ylabel(f'Elapsed time ({unit})')
            ax.set_title(self.label(case), fontsize=11)
            data.append(dict(benchmark=case, candidate_ns=value, run_medians_ns=values))
        experiment = experiment or {
            'title': 'Injected unavailable discovery · candidate only',
            'note': 'No baseline speedup: baseline fails. LD_PRELOAD injects EINVAL; not real unsupported-filesystem seek latency.',
            'sampling': '3 candidate runs · 30 samples/run · 0.3 s warmup · 2 s target · sparse fixture copied as all Data',
        }
        self.save(fig, 'candidate-only' if 'candidate_only' in self.config else 'unsupported-discovery',
                  experiment['title'], experiment['note'], sampling=experiment['sampling'], legend=False,
                  candidate_only=True)
        return data


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path, help='Directory containing plot-config.json and recorded evidence')
    parser.add_argument('--output', type=Path, help='Default: REPORT/plots')
    args = parser.parse_args()
    output = args.output or args.report / 'plots'
    report = Report(args.report, output)
    runs = report.read('measurements.json')
    rows = compare(runs, report.read('summary.json'))
    followup = None
    if (args.report / 'followup-measurements.json').exists():
        followup = compare(report.read('followup-measurements.json'), report.read('followup-summary.json'))
    unsupported = report.read('unsupported-measurements.json') if (args.report / 'unsupported-measurements.json').exists() else None
    candidate_only = report.read('candidate-only-measurements.json') if (args.report / 'candidate-only-measurements.json').exists() else None
    if candidate_only is not None:
        require(unsupported is None, 'use one candidate-only experiment file per report')
        require('candidate_only' in report.config, 'candidate_only configuration is required')
        unsupported = candidate_only
    if followup:
        require({r['benchmark'] for r in followup} <= {r['benchmark'] for r in rows},
                'followup workloads must exist in the main comparison')
    if unsupported is not None:
        require(bool(unsupported), 'empty unsupported experiment')
        require(len({r['tag'] for r in unsupported}) == len(unsupported), 'duplicate unsupported run tag')
    for run in runs + (unsupported or []):
        samples(run)
        report.phase(run['benchmark'])
    plt.rcdefaults()
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10, 'axes.spines.top': False,
                         'axes.spines.right': False, 'axes.grid': True, 'axes.axisbelow': True,
                         'grid.color': '#E1E6ED', 'grid.alpha': .8, 'axes.titlepad': 13,
                         'svg.hashsalt': 'rvvdk-benchmark-plots-v1', 'svg.fonttype': 'none',
                         'figure.facecolor': 'white', 'axes.facecolor': 'white'})
    output.mkdir(parents=True, exist_ok=True)
    report.latency(rows, 'planning')
    report.latency(rows, 'copy')
    report.changes(rows)
    report.distributions(rows, runs)
    if followup:
        repeat_rows, labels = [], []
        for row in followup:
            repeat_rows.extend([next(r for r in rows if r['benchmark'] == row['benchmark']), row])
            labels.extend([report.label(row['benchmark']) + '\nMain comparison', report.label(row['benchmark']) + '\nLonger repeat'])
        report.changes(repeat_rows, 'followup-change', 'Main comparison vs targeted repeat', labels,
                       'Main: ' + report.config['main_sampling'] + '\nRepeat: ' + report.config['repeat_sampling'])
    computed = {'main': rows, 'followup': followup}
    if unsupported:
        computed['candidate_only' if candidate_only else 'unsupported'] = report.unsupported(unsupported, report.config.get('candidate_only'))
    computed_path = output / 'computed.json'
    computed_path.write_text(json.dumps(computed, indent=2) + '\n')
    report.outputs.append(computed_path)
    manifest = {'schema': 1, 'config': report.config, 'input_sha256': report.inputs,
                'generator_sha256': digest(Path(__file__)), 'python': platform.python_version(),
                'packages': {d.metadata['Name']: d.version for d in sorted(importlib.metadata.distributions(), key=lambda d: d.metadata['Name'])},
                'output_sha256': {p.name: digest(p) for p in report.outputs}}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Validated {len(runs)} main runs; wrote {len(report.outputs)-1} images to {output}')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError) as error:
        sys.exit(f'Plot generation failed: {error}')
