#!/usr/bin/env python3
"""Plot every recorded discovery request; never interpret it as disk throughput."""
import argparse
import hashlib
import json
from pathlib import Path
import platform

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    raw = args.input.read_bytes()
    data = json.loads(raw)
    assert data['schema'] == 1 and data['scope'] == 'read_only_inventory'
    reports = data['reports']
    assert len(reports) == 3
    methods = ['RetrieveServiceContent', 'Login', 'RetrievePropertiesEx', 'Logout']
    totals = []
    for report in reports:
        assert report['logout'] is True
        assert len(report['requests']) == 14
        assert report['about'] == reports[0]['about']
        assert report['vms'] == reports[0]['vms']
        assert all(r['method'] in methods and r['elapsed_ms'] > 0 for r in report['requests'])
        assert [r['method'] for r in report['requests']] == methods[:2] + [methods[2]] * 11 + methods[3:]
        totals.append(round(sum(r['elapsed_ms'] for r in report['requests']), 3))

    args.output.mkdir(parents=True, exist_ok=True)
    plt.rcParams.update({'svg.hashsalt': 'rvddk-v01', 'font.family': 'DejaVu Sans', 'font.size': 10})
    fig, axes = plt.subplots(1, 2, figsize=(12, 4.7), gridspec_kw={'width_ratios': [1, 2]})
    colors = ['#2563eb', '#059669', '#d97706']
    axes[0].bar(range(1, 4), [v / 1000 for v in totals], color=colors, width=0.6)
    axes[0].set(xticks=[1, 2, 3], xlabel='Recorded session', ylabel='Sum of request times (s)', title='Complete discovery + logout')
    axes[0].set_ylim(0, max(totals) / 1000 * 1.2)
    for i, value in enumerate(totals, 1):
        axes[0].text(i, value / 1000 + 0.08, f'{value / 1000:.3f}', ha='center')
    for run, report in enumerate(reports):
        for index, method in enumerate(methods):
            values = [r['elapsed_ms'] for r in report['requests'] if r['method'] == method]
            xs = [index + (run - 1) * 0.21 + (i - (len(values) - 1) / 2) * 0.012 for i in range(len(values))]
            axes[1].scatter(xs, values, s=25, alpha=0.8, color=colors[run], label=f'Session {run + 1}' if index == 0 else None)
    axes[1].set(xticks=range(4), xticklabels=['Service content', 'Login', 'Properties', 'Logout'], ylabel='Request latency (ms)', title='All 42 requests, including fresh TLS')
    axes[1].set_ylim(bottom=0)
    axes[1].legend(frameon=False)
    for ax in axes:
        ax.grid(axis='y', alpha=0.2)
        ax.set_axisbelow(True)
        ax.spines[['top', 'right']].set_visible(False)
    fig.suptitle('V0.1 · ESXi discovery observations · 2026-10-01', fontsize=15, fontweight='bold')
    fig.text(0.5, 0.015, 'Python qualification probe · one TLS connection per request · no disk transfer or Rust performance comparison', ha='center', fontsize=9)
    fig.tight_layout(rect=[0, 0.045, 1, 0.93])
    fig.savefig(args.output / 'discovery.svg', metadata={'Date': None})
    fig.savefig(args.output / 'discovery.png', dpi=150, metadata={'Software': 'rvddk V0.1 plot'})
    plt.close(fig)
    computed = {'schema': 1, 'input_sha256': hashlib.sha256(raw).hexdigest(),
                'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                'python': platform.python_version(), 'matplotlib': matplotlib.__version__,
                'requests': 42, 'session_request_sum_ms': totals,
                'scope': 'control-plane observations; no disk throughput or before/after comparison'}
    (args.output / 'computed.json').write_text(json.dumps(computed, indent=2) + '\n')


if __name__ == '__main__':
    main()
