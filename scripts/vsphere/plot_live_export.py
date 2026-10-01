#!/usr/bin/env python3
"""Plot completed powered-off export measurements; never contacts VMware."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics


def measurements(paths):
    rows = []
    for index, path in enumerate(paths, 1):
        raw = path.read_bytes()
        report = json.loads(raw)
        if (report['primary_error'] is not None or report['lease_cleanup'] != 'completed'
                or report['session_cleanup'] != 'logged_out' or not report['manifest_verified']
                or not report['artifact_published'] or report['artifact_cleanup_error'] is not None
                or report['lease_cleanup_error'] is not None or report['pagination_cleanup_error'] is not None
                or report['shutdown_requested'] or report['initial_power_state'] != 'poweredOff'
                or len(report['files']) != 1):
            raise ValueError('only completed, verified, powered-off runs are comparable')
        file = report['files'][0]
        if report['received_encoded_bytes'] != file['encoded_bytes']:
            raise ValueError('byte counts disagree')
        if file['elapsed_ms'] <= 0 or report['elapsed_ms'] < file['elapsed_ms']:
            raise ValueError('invalid timing boundaries')
        if report['cpu_seconds'] is None or report['process_peak_rss_kib'] is None:
            raise ValueError('CPU/RSS observations required')
        rows.append({'run': index, 'report_sha256': hashlib.sha256(raw).hexdigest(),
                     'encoded_bytes': file['encoded_bytes'],
                     'logical_capacity_bytes': report['selected_capacity_bytes'],
                     'transfer_seconds': file['elapsed_ms'] / 1000,
                     'operation_seconds': report['elapsed_ms'] / 1000,
                     'encoded_mib_per_second': file['encoded_bytes'] / 1024**2 / (file['elapsed_ms'] / 1000),
                     'cpu_seconds': report['cpu_seconds'],
                     'peak_rss_mib': report['process_peak_rss_kib'] / 1024})
    if len(rows) < 3 or len({r['logical_capacity_bytes'] for r in rows}) != 1:
        raise ValueError('at least three runs of the same disk capacity required')
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('reports', nargs='+', type=Path)
    args = parser.parse_args()
    rows = measurements(args.reports)
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams.update({'svg.hashsalt': 'rvddk-live-export-v1', 'font.family': 'DejaVu Sans'})
    fig, axes = plt.subplots(2, 2, figsize=(11, 7), constrained_layout=True)
    x = list(range(len(rows)))
    labels = [f"Run {r['run']}" for r in rows]
    for ax in axes.flat:
        ax.set_xticks(x, labels)
        ax.grid(axis='y', alpha=.2)
        ax.set_axisbelow(True)
    axes[0, 0].bar([v-.18 for v in x], [r['transfer_seconds'] for r in rows], .36, label='Receive + hash + file sync')
    axes[0, 0].bar([v+.18 for v in x], [r['operation_seconds'] for r in rows], .36, label='Whole operation')
    axes[0, 0].set_ylabel('Seconds')
    axes[0, 0].legend(fontsize=8)
    axes[0, 1].bar(x, [r['encoded_mib_per_second'] for r in rows], color='#218c74')
    axes[0, 1].set_ylabel('Encoded MiB/s (includes file sync)')
    axes[1, 0].bar(x, [r['cpu_seconds'] for r in rows], color='#7c4dba')
    axes[1, 0].set_ylabel('Process CPU seconds')
    axes[1, 1].bar(x, [r['peak_rss_mib'] for r in rows], color='#c77924')
    axes[1, 1].set_ylabel('Process lifetime peak RSS (MiB)')
    fig.suptitle('Independent Rust disk export — repeated live observations\nPowered-off source; network and host load uncontrolled', fontsize=13)
    args.output.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output/'live-export.svg', metadata={'Date': None})
    svg = args.output/'live-export.svg'
    svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()) + '\n')
    fig.savefig(args.output/'live-export.png', dpi=160, metadata={'Software': 'rvddk qualification'})
    plt.close(fig)
    result = {'schema': 1, 'scope': 'live_export_observations_not_before_after_tuning',
              'rows': rows, 'median_encoded_mib_per_second': statistics.median(r['encoded_mib_per_second'] for r in rows),
              'median_operation_seconds': statistics.median(r['operation_seconds'] for r in rows),
              'matplotlib': matplotlib.__version__}
    (args.output/'computed.json').write_text(json.dumps(result, indent=2)+'\n')


if __name__ == '__main__':
    main()
