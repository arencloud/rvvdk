#!/usr/bin/env python3
"""Matched owned RAW output and durable bundle publication: 64 MiB logical, 8 MiB data."""
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

TEST = 'ownership::retained::output::publication::tests::benchmark::matched_publication'
METRICS = ['wall_ms', 'cpu_ms']


def medians(rows):
    return {mode: {k: st.median(r[k] for r in rows if r['published'] == owned) for k in METRICS}
            for mode, owned in [('owned', False), ('published', True)]}


def repeat_needed(runs):
    return (any(r['median']['published'][k] / r['median']['owned'][k] > 1.05 for r in runs for k in METRICS)
            or any(max(r['median'][mode][k] for r in runs) / min(r['median'][mode][k] for r in runs) > 1.05
                   for mode in ['owned', 'published'] for k in METRICS))


def plot(root):
    runs = json.loads((root/'measurements.json').read_text())
    repeated = [fs for fs in ['btrfs', 'tmpfs'] if repeat_needed([r for r in runs if r['filesystem'] == fs and r['phase'] == 'initial'])]
    assert len(runs) == 6 + 3 * len(repeated)
    assert len({(r['filesystem'], r['phase'], r['round']) for r in runs}) == len(runs)
    summary = dict(repeated_filesystems=repeated, runs=len(runs), conversions=sum(len(r['rows']) for r in runs), results={})
    for run in runs:
        rows = run['rows']; count = 8 if run['phase'] == 'initial' else 16
        assert len(rows) == count*2 and run['median'] == medians(rows)
        for pair in range(count):
            values = rows[2*pair:2*pair+2]
            assert [r['published'] for r in values] == ([False, True] if pair % 2 == 0 else [True, False])
            assert all(r['pair'] == pair for r in values)
        for r in rows:
            assert all(math.isfinite(r[k]) and r[k] > 0 for k in METRICS)
            assert r['byte_comparison_passed'] and r['journal_unchanged']
            assert r['source_bytes'] == 8527360
            assert r['logical_bytes'] == 64 << 20
            assert r['read_bytes'] == 8 << 20
            assert r['written_bytes'] + r['zeroed_bytes'] == r['logical_bytes']
            assert 0 < r['output_allocated_bytes'] <= r['logical_bytes']
            assert 0 < r['source_allocated_bytes'] <= r['source_bytes'] + 65536
            assert r['logical_bytes_verified_in_timing'] == ((3 if r['published'] else 1) * (64 << 20))
            assert r['output_journal_commits'] == (9 if r['published'] else 7)
            assert r['cleanup_journal_commits'] == 2 and r['cleanup_passed']
            assert r['cleanup_ms'] > 0 and r['cleanup_cpu_ms'] > 0
            assert all(r[k] > 0 if r['published'] else r[k] == 0 for k in ['publication_admission_ms', 'publication_ms'])
            assert 0 < r['output_metadata_bytes'] <= 4096
            assert r['admission_ms'] > 0 and r['conversion_and_checks_ms'] > 0
    for fs in ['btrfs', 'tmpfs']:
        summary['results'][fs] = {}
        for phase in ['initial', 'longer']:
            selected = [r for r in runs if r['filesystem'] == fs and r['phase'] == phase]
            if not selected:
                continue
            result = summary['results'][fs][phase] = {}
            for k in METRICS:
                values = {m:st.median(r['median'][m][k] for r in selected) for m in ['owned', 'published']}
                result[k] = dict(**values, overhead_percent=(values['published']/values['owned']-1)*100,
                    spread_percent={m:(max(r['median'][m][k] for r in selected)/min(r['median'][m][k] for r in selected)-1)*100 for m in values})
            result['phase_ms'] = {mode:{k:st.median(st.median(r[k] for r in run['rows'] if r['published'] == retained) for run in selected) for k in ['admission_ms','conversion_and_checks_ms','publication_admission_ms','publication_ms','cleanup_ms','cleanup_cpu_ms']} for mode,retained in [('owned',False),('published',True)]}
    (root/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    fig, axes = plt.subplots(2, 2, figsize=(11, 7))
    for row, fs in zip(axes, ['btrfs', 'tmpfs']):
        for ax, k in zip(row, METRICS):
            for run in (r for r in runs if r['filesystem'] == fs):
                ax.plot([0, 1], [run['median'][m][k] for m in ['owned', 'published']],
                    marker='o' if run['phase'] == 'initial' else '^', label=f"{run['phase']} {run['round']}", alpha=.8)
            ax.set_xticks([0, 1], ['Owned RAW\nprivate staging', 'Owned RAW\n+ durable publication'])
            ax.set_ylim(bottom=0); ax.set_ylabel('Milliseconds'); ax.set_title(f'{fs}: {k}'); ax.legend(fontsize=7)
    fig.suptitle('Owned RAW / 64 MiB logical: fresh admission and durable bundle publication')
    fig.tight_layout(); out=root/'plots'; out.mkdir(exist_ok=True)
    for ext in ['png', 'svg']:
        fig.savefig(out/('publication.'+ext), dpi=160)
    plt.close(fig)
    svg=out/'publication.svg'; svg.write_text('\n'.join(s.rstrip() for s in svg.read_text().splitlines())+'\n')
    fig, axes = plt.subplots(1, 2, figsize=(10, 5))
    keys = ['admission_ms', 'conversion_and_checks_ms', 'publication_admission_ms', 'publication_ms']
    labels = ['Source admission', 'Owned conversion + checks', 'Output admission', 'Publish (incl. revalidation)']
    for ax, fs in zip(axes, ['btrfs', 'tmpfs']):
        result = summary['results'][fs].get('longer', summary['results'][fs]['initial'])
        base = [0., 0.]
        for key, label in zip(keys, labels):
            values = [result['phase_ms'][mode][key] for mode in ['owned', 'published']]
            ax.bar([0, 1], values, bottom=base, label=label)
            base = [a+b for a,b in zip(base, values)]
        ax.set_xticks([0, 1], ['Private staging', 'Published bundle'])
        ax.set_ylabel('Milliseconds'); ax.set_title(fs)
    axes[1].legend(fontsize=7)
    fig.suptitle('Median phase costs / longer runs when triggered; cleanup timed separately')
    fig.tight_layout()
    for ext in ['png', 'svg']:
        fig.savefig(out/('publication-phases.'+ext), dpi=160)
    plt.close(fig)
    svg=out/'publication-phases.svg'; svg.write_text('\n'.join(s.rstrip() for s in svg.read_text().splitlines())+'\n')
    paths=[root/'measurements.json', root/'summary.json', out/'publication.png', out/'publication.svg', out/'publication-phases.png', out/'publication-phases.svg']
    (out/'audit.json').write_text(json.dumps(dict(conversions=summary['conversions'], sha256={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}),indent=2)+'\n')


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
        conditions='Same release executable, alternating per-pair order, shared CPU0, powersave, SMT not isolated. No builds/tests/plots overlap timing. Authored VMDK: 128 stored-DEFLATE grains in 64 MiB logical capacity. Both paths freshly admit RetainedArtifact and convert_owned, including seven output journal commits, full logical readback, SHA256 and private metadata. Published path then opens VerifiedOutput (source still required), freshly verifies the entire RAW and source, and publishes after another verification, member syncs, durable intent, atomic no-replace bundle rename, both parent syncs and acknowledgment. Destination directory creation and initial locks precede timing; reopening the source store for publication is timed. Source fixture/journal preparation, independent logical/source/journal comparisons and allocation checks excluded. Both release capabilities before timer ends. Explicit checked output cleanup is timed separately after comparisons and excludes lock acquisition; its two commits are outside main timing. Per-call CPU includes workers; process CPU/RSS includes fixtures, oracle and cleanup. No live host, ordinary caches; tmpfs volatile.')

    def snapshot():
        base=Path('/sys/devices/system/cpu/cpu0/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(), frequency={k:(base/k).read_text().strip() for k in ['scaling_cur_freq', 'scaling_governor']})

    def run(fs, phase, number):
        label=f'{phase}-{fs}-{number}'; raw=a.report/(label+'.json'); metrics=a.report/(label+'-time.json')
        count=8 if phase=='initial' else 16
        command=['/usr/bin/time','-f','{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
            '-o',str(metrics),'taskset','-c','0',str(a.binary),'--exact',TEST,'--ignored','--test-threads=1']
        entry=dict(command=command,pairs=count,before=snapshot())
        process_env=dict(os.environ,RVVDK_RETAINED_PAIRS=str(count),RVVDK_RETAINED_PARENT=str(parents[fs].resolve()),RVVDK_RETAINED_REPORT=str(raw.resolve()))
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
