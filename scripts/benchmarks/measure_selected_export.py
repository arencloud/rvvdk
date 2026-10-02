#!/usr/bin/env python3
"""Matched synthetic TLS exports: legacy capacity proof versus explicit identity."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics as st
import subprocess
import time

TEST = 'export_tests::selected_benchmark::matched_selected_export'
METRICS = ['wall_ms', 'cpu_ms']


def medians(rows):
    return {mode: {k: st.median(r[k] for r in rows if r['explicit'] == explicit)
                   for k in METRICS}
            for mode, explicit in [('legacy', False), ('explicit', True)]}


def repeat_needed(runs):
    return (any(r['median']['explicit'][k] / r['median']['legacy'][k] > 1.05
                for r in runs for k in METRICS)
            or any(max(r['median'][mode][k] for r in runs) /
                   min(r['median'][mode][k] for r in runs) > 1.05
                   for mode in ['legacy', 'explicit'] for k in METRICS))


def plot(root):
    runs = json.loads((root / 'measurements.json').read_text())
    repeated = repeat_needed([r for r in runs if r['phase'] == 'initial'])
    assert len(runs) == (6 if repeated else 3)
    assert len({(r['phase'], r['round']) for r in runs}) == len(runs)
    for run in runs:
        rows = run['rows']
        count = 32 if run['phase'] == 'initial' else 64
        assert len(rows) == 2 * count
        assert run['median'] == medians(rows)
        for pair in range(count):
            values = rows[2 * pair:2 * pair + 2]
            assert [r['explicit'] for r in values] == ([False, True] if pair % 2 == 0 else [True, False])
            assert all(r['pair'] == pair for r in values)
        for r in rows:
            assert all(math.isfinite(r[k]) and r[k] > 0 for k in METRICS)
            assert r['byte_comparison_passed'] and r['encoded_bytes'] == 65536
            assert r['allocated_bytes'] == 65536
            assert r['vm_reads'] == (5 if r['explicit'] else 2)
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    fig, axes = plt.subplots(1, 2, figsize=(11, 4.5))
    for ax, k in zip(axes, METRICS):
        for phase, marker in [('initial', 'o'), ('longer', '^')]:
            for run in (r for r in runs if r['phase'] == phase):
                y = [run['median'][m][k] for m in ['legacy', 'explicit']]
                ax.plot([0, 1], y, marker=marker, alpha=.8,
                        label=f"{phase} {run['round']}")
        ax.set_xticks([0, 1], ['Capacity proof', 'Explicit identity'])
        ax.set_ylabel('Milliseconds'); ax.set_title(k.replace('_ms', '').upper())
        ax.set_ylim(bottom=0); ax.legend(fontsize=8)
    nodelay = json.loads((root/'environment.json').read_text()).get('server_nodelay', False)
    fig.suptitle(f'Local TLS / 64 KiB / tmpfs / server TCP_NODELAY={nodelay}: every run median')
    fig.tight_layout(); out = root / 'plots'; out.mkdir(exist_ok=True)
    for ext in ['png', 'svg']:
        fig.savefig(out / ('selected-export.' + ext), dpi=160)
    plt.close(fig)
    svg = out / 'selected-export.svg'
    svg.write_text('\n'.join(s.rstrip() for s in svg.read_text().splitlines()) + '\n')
    summary = {'repeated': repeated, 'runs': len(runs), 'exports': sum(len(r['rows']) for r in runs)}
    for phase in ['initial', 'longer']:
        values = [r for r in runs if r['phase'] == phase]
        if not values:
            continue
        summary[phase] = {}
        for k in METRICS:
            legacy = st.median(r['median']['legacy'][k] for r in values)
            explicit = st.median(r['median']['explicit'][k] for r in values)
            summary[phase][k] = dict(legacy=legacy, explicit=explicit,
                overhead_percent=(explicit/legacy-1)*100,
                spread_percent={m:(max(r['median'][m][k] for r in values) /
                                   min(r['median'][m][k] for r in values)-1)*100
                                for m in ['legacy', 'explicit']})
    (root/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    paths = [root/'measurements.json', root/'summary.json', out/'selected-export.svg', out/'selected-export.png']
    (out/'audit.json').write_text(json.dumps(dict(exports=summary['exports'],
        sha256={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}), indent=2)+'\n')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path)
    p.add_argument('--report', type=Path, required=True)
    p.add_argument('--plot-only', action='store_true')
    p.add_argument('--server-nodelay', action='store_true')
    a = p.parse_args()
    if a.plot_only:
        plot(a.report); return
    if a.binary is None:
        p.error('--binary required')
    assert subprocess.check_output(['stat', '-f', '-c', '%T', '/tmp'], text=True).strip() == 'tmpfs'
    a.report.mkdir(parents=True, exist_ok=True)
    assert not (a.report/'measurements.json').exists()
    runs, commands = [], []
    environment = dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), kernel=platform.release(), cpu=0, server_nodelay=a.server_nodelay,
        conditions='Same release executable, alternating per-pair legacy/explicit order. CPU 0 affinity; shared host; SMT not isolated. /tmp tmpfs; no real VMDK decode or VMware host. Per-export CPU includes mock server threads, excludes fixture construction and readback/removal. Peak RSS is process lifetime. Builds/tests/plots outside timed matrix.', commands=commands)

    def snapshot():
        base = Path('/sys/devices/system/cpu/cpu0/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(),
            frequency={k:(base/k).read_text().strip() for k in ['scaling_cur_freq', 'scaling_governor']})

    def run(phase, number):
        label = f'{phase}-{number}'
        raw = a.report/(label+'.json'); metrics = a.report/(label+'-time.json')
        count = 32 if phase == 'initial' else 64
        command = ['/usr/bin/time', '-f', '{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
            '-o', str(metrics), 'taskset', '-c', '0', str(a.binary), '--exact', TEST, '--ignored', '--test-threads=1']
        entry = dict(command=command, pairs=count, before=snapshot())
        env = dict(os.environ, RVVDK_SELECTION_PAIRS=str(count), RVVDK_SELECTION_NODELAY='1' if a.server_nodelay else '0', RVVDK_SELECTION_REPORT=str(raw.resolve()), TMPDIR='/tmp')
        with (a.report/(label+'.txt')).open('w') as log:
            result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=120)
        entry.update(after=snapshot(), exit_code=result.returncode, metrics=json.loads(metrics.read_text()))
        commands.append(entry)
        (a.report/'environment.json').write_text(json.dumps(environment, indent=2)+'\n')
        assert result.returncode == 0
        rows = json.loads(raw.read_text())
        runs.append(dict(phase=phase, round=number, rows=rows, median=medians(rows)))
        (a.report/'measurements.json').write_text(json.dumps(runs, indent=2)+'\n')
        print(label, runs[-1]['median'], flush=True)

    for n in range(1, 4):
        run('initial', n)
    if repeat_needed(runs):
        for n in range(1, 4):
            run('longer', n)
    plot(a.report)


if __name__ == '__main__':
    main()
