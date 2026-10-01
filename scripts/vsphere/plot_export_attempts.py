#!/usr/bin/env python3
"""Plot every supplied real transfer attempt, including incomplete/failing ones."""
import argparse
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('reports', nargs='+', type=Path)
    args = parser.parse_args()
    rows = []
    for index, path in enumerate(args.reports, 1):
        raw = path.read_bytes()
        report = json.loads(raw)
        if report['probe_only'] or report['cpu_seconds'] is None or report['process_peak_rss_kib'] is None:
            raise ValueError('instrumented full-mode attempts required')
        completed = (report['primary_error'] is None and report['lease_cleanup'] == 'completed'
                     and report['session_cleanup'] == 'logged_out' and report['manifest_verified']
                     and report['artifact_published'] and report['pagination_cleanup_error'] is None
                     and report['lease_cleanup_error'] is None and report['artifact_cleanup_error'] is None)
        outcome = 'completed' if completed else (report['primary_error'] or 'cleanup_unconfirmed')
        rows.append({'attempt': index, 'report': path.name, 'sha256': hashlib.sha256(raw).hexdigest(),
                     'outcome': outcome, 'completed': completed, 'elapsed_seconds': report['elapsed_ms']/1000,
                     'accepted_encoded_mib': report['received_encoded_bytes']/1024**2,
                     'cpu_seconds': report['cpu_seconds'], 'peak_rss_mib': report['process_peak_rss_kib']/1024})
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams.update({'svg.hashsalt': 'rvddk-export-attempts-v1', 'font.family': 'DejaVu Sans'})
    fig, axes = plt.subplots(2, 2, figsize=(12, 8), constrained_layout=True)
    x = list(range(len(rows)))
    colors = ['#218c74' if r['completed'] else '#b45f35' for r in rows]
    for ax, field, title in zip(axes.flat,
                               ['elapsed_seconds', 'accepted_encoded_mib', 'cpu_seconds', 'peak_rss_mib'],
                               ['Whole-operation seconds', 'Accepted encoded MiB — partial on failure',
                                'Process CPU seconds', 'Process lifetime peak RSS (MiB)']):
        ax.bar(x, [r[field] for r in rows], color=colors)
        ax.set_xticks(x, [f"A{r['attempt']}\n{r['outcome']}" for r in rows], fontsize=8)
        ax.set_ylabel(title)
        ax.grid(axis='y', alpha=.2)
        ax.set_axisbelow(True)
        for i, row in enumerate(rows):
            ax.annotate(f"{row[field]:.2f}", (i, row[field]), xytext=(0, 4), textcoords='offset points', ha='center', fontsize=8)
        ax.margins(y=.18)
    fig.suptitle('Live Rust export attempts — every supplied outcome retained\nDifferent stages/builds; no tuning comparison or completed-throughput claim', fontsize=13)
    args.output.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output/'attempts.svg', metadata={'Date': None})
    svg = args.output/'attempts.svg'
    svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()) + '\n')
    fig.savefig(args.output/'attempts.png', dpi=160, metadata={'Software': 'rvddk qualification'})
    plt.close(fig)
    (args.output/'attempts-computed.json').write_text(json.dumps({'schema':1,'scope':'diagnostic_transfer_attempts',
                                                              'rows':rows,'completed_count':sum(r['completed'] for r in rows),
                                                              'matplotlib':matplotlib.__version__},indent=2)+'\n')


if __name__ == '__main__':
    main()
