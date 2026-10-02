#!/usr/bin/env python3
"""Render sanitized composed-pipeline observations; no VMware or image access."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics as st

COLORS = {'export_seconds': '#1f77b4', 'conversion_seconds': '#ff7f0e',
          'publication_admission_seconds': '#2ca02c', 'publication_seconds': '#d62728'}

METRICS = ['wall_seconds', 'cpu_seconds', 'export_seconds', 'conversion_seconds',
           'publication_admission_seconds', 'publication_seconds', 'cleanup_seconds']


def read_rows(root):
    rows = []
    for path in sorted(root.glob('run-*.json')):
        r = json.loads(path.read_text())
        p = r['pipeline']; d = p['data']; t = d['transfer']; c = d['conversion']; u = d['publication']
        assert p['success'] and d['phase'] == 'completed' and not d['blocking_worker_failed']
        assert d['remote_error'] is None and d['local_error'] is None
        assert all(t[k] is None for k in ['primary_error', 'journal_error', 'payload_error', 'pagination_cleanup_error'])
        assert t['lease_cleanup'] == 'completed' and t['session_cleanup'] == 'logged_out'
        assert all(t[k] for k in ['manifest_verified', 'container_readback_verified', 'artifact_metadata_durable', 'native_admission_verified'])
        assert t['recovery']['state'] == 'completed_lease' and not t['recovery']['pending_transaction']
        assert t['received_encoded_bytes'] == t['written_encoded_bytes'] == t['durable_encoded_bytes'] == r['source_bytes']
        assert c['primary_error'] is None and c['assessment_error'] is None
        assert c['conversion_flushed'] and c['metadata_durable']
        assert c['recovery']['state'] == 'verified' and not c['recovery']['pending_transaction']
        assert c['logical_bytes_verified'] == r['output_logical_bytes'] == 30 << 30
        assert u['primary_error'] is None and u['assessment_error'] is None
        assert u['rename_observed'] and u['directories_synced']
        assert u['recovery']['output']['state'] == 'published'
        assert not u['recovery']['output']['pending_transaction']
        assert u['recovery']['staged'] == 'absent' and u['recovery']['published'] == 'owned'
        assert r['qemu_compare_passed'] and r['oracle']['matched']
        assert r['oracle']['verified_bytes'] == 8 << 20
        assert r['oracle']['disk_logical_bytes'] == r['output_logical_bytes']
        assert r['journal_states_before'] == ['completed_lease', 'published']
        assert r['journal_states_after'] == ['cleaned', 'cleaned'] and r['owned_payloads_removed']
        assert r['cleanup']['success'] and r['cleanup']['data']['output_cleaned'] and r['cleanup']['data']['source_cleaned']
        row = dict(run=r['run'], wall_seconds=d['elapsed_ms']/1000, cpu_seconds=p['cpu_seconds'],
                   peak_rss_mib=p['process_peak_rss_kib']/1024, cleanup_seconds=r['cleanup']['elapsed_ms']/1000,
                   cleanup_cpu_seconds=r['cleanup']['cpu_seconds'], source_bytes=r['source_bytes'],
                   source_allocated_bytes=r['source_allocated_bytes'], output_allocated_bytes=r['output_allocated_bytes'],
                   output_logical_bytes=r['output_logical_bytes'], qemu_compare_seconds=r['qemu_compare_ms']/1000)
        for key in ['export', 'conversion', 'publication_admission', 'publication']:
            row[key+'_seconds'] = d[key+'_ms']/1000
        assert all(math.isfinite(row[k]) and row[k] > 0 for k in METRICS)
        assert sum(row[k+'_seconds'] for k in ['export', 'conversion', 'publication_admission', 'publication']) <= row['wall_seconds'] + .001
        assert 0 < row['output_allocated_bytes'] <= row['output_logical_bytes']
        assert row['source_allocated_bytes'] > 0 and math.isfinite(row['peak_rss_mib']) and row['peak_rss_mib'] > 0
        row['export_phase_encoded_mib_per_second'] = row['source_bytes']/1024**2/row['export_seconds']
        rows.append(row)
    assert len(rows) in [1, 3, 6] and [r['run'] for r in rows] == list(range(1, len(rows)+1))
    assert len({r['output_logical_bytes'] for r in rows}) == 1
    return rows


def group(rows):
    keys = METRICS + ['peak_rss_mib', 'cleanup_cpu_seconds', 'export_phase_encoded_mib_per_second',
                      'output_allocated_bytes', 'source_allocated_bytes']
    return dict(medians={k: st.median(r[k] for r in rows) for k in keys},
                spread_percent=({k: (max(r[k] for r in rows)/min(r[k] for r in rows)-1)*100 for k in keys} if len(rows) > 1 else None))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path); a = parser.parse_args(); root = a.report
    rows = read_rows(root); initial = group(rows[:3])
    triggered = (any(initial['spread_percent'][k] > 5 for k in METRICS + ['cleanup_cpu_seconds'])
                 if len(rows) >= 3 else None)
    assert not triggered or len(rows) == 6, 'retain three additional observations for >5% spread'
    summary = dict(scope=('single_live_qualification_no_variance_estimate' if len(rows) == 1 else 'new_LAN_pipeline_baseline_not_a_matched_speedup'), rows=rows,
                   repeat_triggered=triggered, initial=initial, all=group(rows))
    if len(rows) == 6:
        summary['repeat'] = group(rows[3:])
    (root/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams.update({'svg.hashsalt': 'rvddk-composed-pipeline-v1', 'font.family': 'DejaVu Sans'})
    out = root/'plots'; out.mkdir(exist_ok=True)
    labels = (["Qualification 1"] if len(rows) == 1 else [f"{'Initial' if r['run'] <= 3 else 'Repeat'} {r['run']}" for r in rows])
    x = list(range(len(rows)))
    fig, axes = plt.subplots(2, 2, figsize=(12, 8), constrained_layout=True)
    bottom = [0.] * len(rows)
    for k, label in [('export_seconds', 'Export + admission'), ('conversion_seconds', 'Retain / convert / verify'),
                     ('publication_admission_seconds', 'Fresh output admission'), ('publication_seconds', 'Verify / publish / sync')]:
        values = [r[k] for r in rows]
        axes[0, 0].bar(x, values, bottom=bottom, label=label, color=COLORS[k])
        bottom = [b+v for b, v in zip(bottom, values)]
    axes[0, 0].set_ylabel('Pipeline phase seconds'); axes[0, 0].legend(fontsize=8)
    axes[0, 1].bar(x, [r['cpu_seconds'] for r in rows], color='#8056ac')
    axes[0, 1].set_ylabel('Process CPU seconds')
    axes[1, 0].bar(x, [r['peak_rss_mib'] for r in rows], color='#d38629')
    axes[1, 0].set_ylabel('Process lifetime peak RSS (MiB)')
    axes[1, 1].bar(x, [r['cleanup_seconds'] for r in rows], color='#218c74')
    axes[1, 1].set_ylabel('Separate checked cleanup seconds')
    for ax in axes.flat:
        ax.set_xticks(x, labels, rotation=15); ax.set_xlim(-.65, max(x)+.65); ax.set_ylim(bottom=0); ax.grid(axis='y', alpha=.2); ax.set_axisbelow(True)
    fig.suptitle('Composed Rust export → verified RAW publication / 30 GiB logical\nShared ESXi LAN runner; ordinary caches, no isolation / one run is not a variance estimate')
    save(fig, out, 'pipeline'); plt.close(fig)
    fig, axes = plt.subplots(1, 2, figsize=(12, 5), constrained_layout=True)
    bottom = [0.] * len(rows)
    for k, label in [('conversion_seconds', 'Retain / convert / verify'),
                     ('publication_admission_seconds', 'Fresh admission'), ('publication_seconds', 'Verify / publish / sync')]:
        values = [r[k] for r in rows]; axes[0].bar(x, values, bottom=bottom, label=label, color=COLORS[k])
        bottom = [b+v for b, v in zip(bottom, values)]
    axes[0].set_ylabel('Local phase seconds'); axes[0].legend(fontsize=8)
    for shift, key, label in [(-.25, 'source_allocated_bytes', 'Encoded source'), (0, 'output_allocated_bytes', 'Sparse RAW'), (.25, 'output_logical_bytes', 'Logical capacity')]:
        axes[1].bar([v+shift for v in x], [r[key]/1024**3 for r in rows], .25, label=label)
    axes[1].set_ylabel('GiB'); axes[1].legend(fontsize=8)
    for ax in axes:
        ax.set_xticks(x, labels, rotation=15); ax.set_xlim(-.65, max(x)+.65); ax.set_ylim(bottom=0); ax.grid(axis='y', alpha=.2); ax.set_axisbelow(True)
    fig.suptitle('Local verification/publication costs and allocated storage\nIndependent QEMU + mapped guest comparisons pass after timing; explicit cleanup follows')
    save(fig, out, 'local-and-allocation'); plt.close(fig)
    paths = sorted(root.glob('run-*.json')) + [root/'summary.json', Path(__file__)] + sorted(out.glob('*.svg')) + sorted(out.glob('*.png'))
    (out/'audit.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__, sha256={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}), indent=2)+'\n')


def save(fig, root, name):
    fig.savefig(root/(name+'.svg'), metadata={'Date': None})
    p = root/(name+'.svg'); p.write_text('\n'.join(line.rstrip() for line in p.read_text().splitlines())+'\n')
    fig.savefig(root/(name+'.png'), dpi=160, metadata={'Software': 'rvddk qualification'})


if __name__ == '__main__':
    main()
