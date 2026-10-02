#!/usr/bin/env python3
"""Synthetic durable-job timing on Btrfs and tmpfs; no VMware operations."""
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

PHASES = ['create_ns', 'stage_ns', 'lease_journal_ns', 'recover_ns', 'cleanup_ns']


def plot(root):
    data = json.loads((root / 'measurements.json').read_text())
    summary = json.loads((root / 'summary.json').read_text())
    keys = [(r['filesystem'], r['phase'], r['round']) for r in data]
    assert len(set(keys)) == len(keys)
    assert len(data) == 6 + 3 * len(summary['repeated_filesystems'])
    for r in data:
        assert len(r['rows']) == r['jobs'] == (32 if r['phase'] == 'initial' else 64)
        assert r['exit_code'] == 0 and r['journal_commits_per_job'] == 10
        assert 0 < r['largest_record_bytes'] <= 8192
        for k in PHASES:
            assert all(math.isfinite(s[k]) and s[k] > 0 for s in r['rows'])
            assert r['median_ns'][k] == st.median(s[k] for s in r['rows'])
    expected = [fs for fs in ['btrfs', 'tmpfs'] if any(
        max(r['median_ns'][k] for r in data if r['filesystem'] == fs and r['phase'] == 'initial') /
        min(r['median_ns'][k] for r in data if r['filesystem'] == fs and r['phase'] == 'initial') > 1.05 for k in PHASES)]
    assert expected == summary['repeated_filesystems']
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    fig, axes = plt.subplots(1, 2, figsize=(12, 5))
    for ax, filesystem in zip(axes, ['btrfs', 'tmpfs']):
        for phase, shift, color in [('initial', -.08, '#126e82'), ('longer', .08, '#b3541e')]:
            rs = [r for r in data if r['filesystem'] == filesystem and r['phase'] == phase]
            for i, k in enumerate(PHASES):
                if rs:
                    ax.scatter([i + shift] * len(rs), [r['median_ns'][k] / 1000 for r in rs], color=color, label=phase if i == 0 else None)
        ax.set_xticks(range(len(PHASES)), ['create', 'prepare\nstage', 'five lease\nupdates', 'reopen /\nrecover', 'cleanup'])
        ax.set_yscale('log'); ax.set_ylabel('Microseconds (log scale)'); ax.set_title(filesystem); ax.legend()
    fig.suptitle('Synthetic ownership journal: every run median; filesystems kept separate')
    fig.tight_layout(); out = root / 'plots'; out.mkdir(exist_ok=True)
    for ext in ['png', 'svg']:
        fig.savefig(out / ('ownership.' + ext), dpi=160)
    plt.close(fig)
    svg = out / 'ownership.svg'; svg.write_text('\n'.join(s.rstrip() for s in svg.read_text().splitlines()) + '\n')
    files = [root / 'measurements.json', root / 'summary.json', out / 'ownership.png', out / 'ownership.svg']
    (out / 'audit.json').write_text(json.dumps(dict(runs=len(data), jobs=sum(r['jobs'] for r in data),
        sha256={str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in files}), indent=2) + '\n')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--report', type=Path, required=True)
    p.add_argument('--binary', type=Path)
    p.add_argument('--btrfs-parent', type=Path)
    p.add_argument('--tmpfs-parent', type=Path)
    p.add_argument('--plot-only', action='store_true')
    a = p.parse_args()
    if a.plot_only:
        plot(a.report); return
    if None in [a.binary, a.btrfs_parent, a.tmpfs_parent]:
        p.error('binary and both filesystem parents are required')
    a.report.mkdir(parents=True, exist_ok=True)
    assert not (a.report / 'measurements.json').exists()
    parents = dict(btrfs=a.btrfs_parent, tmpfs=a.tmpfs_parent)
    for fs, parent in parents.items():
        parent.mkdir(parents=True, exist_ok=True)
        assert subprocess.check_output(['stat', '-f', '-c', '%T', str(parent)], text=True).strip() == fs
    rows, commands = [], []
    env = dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(), kernel=platform.release(), cpu=0,
        conditions='Shared host, CPU 0 affinity, powersave, SMT not isolated; no cache flushing. Synthetic journal only. No builds/tests/plots overlap timing. Whole-process CPU/RSS include setup and fixture removal. tmpfs is volatile.', commands=commands)

    def snapshot():
        base = Path('/sys/devices/system/cpu/cpu0/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(), frequency={k:(base/k).read_text().strip() for k in ['scaling_cur_freq','scaling_governor']})

    def run(fs, phase, number):
        label = f'{phase}-{fs}-{number}'; fixture = parents[fs] / label
        assert not fixture.exists()
        count = 32 if phase == 'initial' else 64
        metrics = a.report / (label + '-time.json')
        command = ['/usr/bin/time', '-f', '{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
            '-o', str(metrics), 'taskset', '-c', '0', str(a.binary), str(fixture), str(count)]
        entry = dict(command=command, before=snapshot())
        with (a.report/(label+'.json')).open('w') as out, (a.report/(label+'.stderr')).open('w') as err:
            result = subprocess.run(command, stdout=out, stderr=err, timeout=600)
        entry.update(after=snapshot(), exit_code=result.returncode, metrics=json.loads(metrics.read_text()))
        commands.append(entry); (a.report/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
        assert result.returncode == 0
        row = dict(filesystem=fs, phase=phase, round=number, exit_code=result.returncode, **json.loads((a.report/(label+'.json')).read_text()))
        row['median_ns'] = {k:st.median(s[k] for s in row['rows']) for k in PHASES}
        rows.append(row); (a.report/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n')
        assert not fixture.exists()
        print(label, 'complete', flush=True)

    for n in range(1, 4):
        for fs in (['btrfs','tmpfs'] if n%2 else ['tmpfs','btrfs']):
            run(fs,'initial',n)
    repeated = [fs for fs in parents if any(max(r['median_ns'][k] for r in rows if r['filesystem']==fs) /
                min(r['median_ns'][k] for r in rows if r['filesystem']==fs) > 1.05 for k in PHASES)]
    for n in range(1, 4):
        for fs in (repeated if n%2 else list(reversed(repeated))):
            run(fs,'longer',n)
    (a.report/'summary.json').write_text(json.dumps(dict(repeated_filesystems=repeated),indent=2)+'\n')
    plot(a.report)


if __name__ == '__main__':
    main()
