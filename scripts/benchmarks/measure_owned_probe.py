#!/usr/bin/env python3
"""Synthetic matched acquire/abort probes with and without durable ownership."""
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

TEST = 'export_tests::owned_probe::benchmark::matched_owned_probe'
METRICS = ['wall_ms', 'cpu_ms']


def medians(rows):
    return {mode: {k: st.median(r[k] for r in rows if r['owned'] == owned) for k in METRICS}
            for mode, owned in [('proof', False), ('owned', True)]}


def repeat_needed(runs):
    return (any(r['median']['owned'][k] / r['median']['proof'][k] > 1.05 for r in runs for k in METRICS)
            or any(max(r['median'][mode][k] for r in runs) / min(r['median'][mode][k] for r in runs) > 1.05
                   for mode in ['proof', 'owned'] for k in METRICS))


def plot(root):
    runs = json.loads((root/'measurements.json').read_text())
    repeated = [fs for fs in ['btrfs', 'tmpfs'] if repeat_needed([r for r in runs if r['filesystem'] == fs and r['phase'] == 'initial'])]
    assert len(runs) == 6 + 3 * len(repeated)
    assert len({(r['filesystem'], r['phase'], r['round']) for r in runs}) == len(runs)
    summary = dict(repeated_filesystems=repeated, runs=len(runs), probes=sum(len(r['rows']) for r in runs), results={})
    for run in runs:
        rows = run['rows']; count = 16 if run['phase'] == 'initial' else 32
        assert len(rows) == count*2 and run['median'] == medians(rows)
        for pair in range(count):
            values = rows[2*pair:2*pair+2]
            assert [r['owned'] for r in values] == ([False, True] if pair % 2 == 0 else [True, False])
            assert all(r['pair'] == pair for r in values)
        for r in rows:
            assert all(math.isfinite(r[k]) and r[k] > 0 for k in METRICS)
            assert r['journal_commits'] == (7 if r['owned'] else 0)
            assert r['empty_payload_verified']
            if r['owned']:
                assert r['cleanup_ms'] > 0 and 16 < r['file_bytes'] <= 8192+16
            else:
                assert r['cleanup_ms'] == r['file_bytes'] == r['allocated_bytes'] == 0
    for fs in ['btrfs', 'tmpfs']:
        summary['results'][fs] = {}
        for phase in ['initial', 'longer']:
            selected = [r for r in runs if r['filesystem'] == fs and r['phase'] == phase]
            if not selected:
                continue
            result = summary['results'][fs][phase] = {}
            for k in METRICS:
                values = {m:st.median(r['median'][m][k] for r in selected) for m in ['proof', 'owned']}
                result[k] = dict(**values, overhead_percent=(values['owned']/values['proof']-1)*100,
                    spread_percent={m:(max(r['median'][m][k] for r in selected)/min(r['median'][m][k] for r in selected)-1)*100 for m in values})
            result['cleanup_ms'] = st.median(st.median(r['cleanup_ms'] for r in run['rows'] if r['owned']) for run in selected)
    (root/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    fig, axes = plt.subplots(2, 2, figsize=(11, 7))
    for row, fs in zip(axes, ['btrfs', 'tmpfs']):
        for ax, k in zip(row, METRICS):
            for run in (r for r in runs if r['filesystem'] == fs):
                ax.plot([0, 1], [run['median'][m][k] for m in ['proof', 'owned']],
                    marker='o' if run['phase'] == 'initial' else '^', label=f"{run['phase']} {run['round']}", alpha=.8)
            ax.set_xticks([0, 1], ['Explicit proof', 'Durable probe'])
            ax.set_ylim(bottom=0); ax.set_ylabel('Milliseconds'); ax.set_title(f'{fs}: {k}'); ax.legend(fontsize=7)
    fig.suptitle('Synthetic local TLS acquire/abort: every run median; no payload transfer')
    fig.tight_layout(); out=root/'plots'; out.mkdir(exist_ok=True)
    for ext in ['png', 'svg']:
        fig.savefig(out/('owned-probe.'+ext), dpi=160)
    plt.close(fig)
    svg=out/'owned-probe.svg'; svg.write_text('\n'.join(s.rstrip() for s in svg.read_text().splitlines())+'\n')
    paths=[root/'measurements.json', root/'summary.json', out/'owned-probe.png', out/'owned-probe.svg']
    (out/'audit.json').write_text(json.dumps(dict(probes=summary['probes'], sha256={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}),indent=2)+'\n')


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path); p.add_argument('--report', type=Path, required=True)
    p.add_argument('--btrfs-parent', type=Path); p.add_argument('--tmpfs-parent', type=Path)
    p.add_argument('--plot-only', action='store_true'); a=p.parse_args()
    if a.plot_only:
        plot(a.report); return
    if None in [a.binary, a.btrfs_parent, a.tmpfs_parent]:
        p.error('binary and both fixture parents required')
    parents=dict(btrfs=a.btrfs_parent, tmpfs=a.tmpfs_parent)
    for fs, path in parents.items():
        path.mkdir(parents=True, exist_ok=True)
        assert subprocess.check_output(['stat', '-f', '-c', '%T', str(path)],text=True).strip() == fs
    a.report.mkdir(parents=True, exist_ok=True); assert not (a.report/'measurements.json').exists()
    runs, commands=[], []
    env=dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), kernel=platform.release(), cpu=0, commands=commands,
        conditions='Same release executable; alternating per-pair order, shared CPU0, powersave, SMT not isolated. Synthetic local TLS server TCP_NODELAY. No builds/tests/plots overlap timing. Per-probe CPU includes server and journal worker; setup/readback/cleanup excluded. Whole-process CPU/RSS includes all setup/cleanup. No payload, live host or throughput claim. tmpfs is volatile.')

    def snapshot():
        base=Path('/sys/devices/system/cpu/cpu0/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(), frequency={k:(base/k).read_text().strip() for k in ['scaling_cur_freq', 'scaling_governor']})

    def run(fs, phase, number):
        label=f'{phase}-{fs}-{number}'; raw=a.report/(label+'.json'); metrics=a.report/(label+'-time.json')
        count=16 if phase=='initial' else 32
        command=['/usr/bin/time','-f','{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
            '-o',str(metrics),'taskset','-c','0',str(a.binary),'--exact',TEST,'--ignored','--test-threads=1']
        entry=dict(command=command,pairs=count,before=snapshot())
        process_env=dict(os.environ,RVVDK_PROBE_PAIRS=str(count),RVVDK_PROBE_PARENT=str(parents[fs].resolve()),RVVDK_PROBE_REPORT=str(raw.resolve()))
        with (a.report/(label+'.txt')).open('w') as log:
            result=subprocess.run(command,env=process_env,stdout=log,stderr=subprocess.STDOUT,timeout=120)
        entry.update(after=snapshot(),exit_code=result.returncode,metrics=json.loads(metrics.read_text()));commands.append(entry)
        (a.report/'environment.json').write_text(json.dumps(env,indent=2)+'\n');assert result.returncode==0
        rows=json.loads(raw.read_text());runs.append(dict(filesystem=fs,phase=phase,round=number,rows=rows,median=medians(rows)))
        (a.report/'measurements.json').write_text(json.dumps(runs,indent=2)+'\n')
        print(label,runs[-1]['median'],flush=True)
    for n in range(1,4):
        for fs in (['btrfs','tmpfs'] if n%2 else ['tmpfs','btrfs']):
            run(fs,'initial',n)
    repeated=[fs for fs in parents if repeat_needed([r for r in runs if r['filesystem']==fs])]
    for n in range(1,4):
        for fs in (repeated if n%2 else list(reversed(repeated))):
            run(fs,'longer',n)
    plot(a.report)


if __name__=='__main__':
    main()
