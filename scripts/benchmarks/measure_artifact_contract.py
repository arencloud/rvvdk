#!/usr/bin/env python3
"""Synthetic R6.1a baseline and repeatability observations; no VMware access."""
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


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path)
    p.add_argument('--plot-only', action='store_true')
    p.add_argument('--report', type=Path, required=True)
    a = p.parse_args()
    if a.plot_only:
        plot(a.report)
        return
    if a.binary is None:
        p.error("--binary is required for measurements")
    a.report.mkdir(parents=True, exist_ok=True)
    assert not (a.report / 'measurements.json').exists()
    rows, commands = [], []
    env = dict(kernel=platform.release(), cpu=0, binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),
               conditions='Synthetic in-memory fixtures; CPU 0; shared host, powersave, SMT sibling not isolated. No network or disk payload. CPU/RSS cover Criterion including warmup and analysis, not per-operation resource use.', commands=commands)

    def snapshot():
        cpu = Path('/sys/devices/system/cpu/cpu0/cpufreq')
        return dict(unix_seconds=time.time(), load=os.getloadavg(),
                    frequency={k: (cpu / k).read_text().strip() for k in ['scaling_governor', 'scaling_cur_freq']},
                    cpu_ticks=[s for s in Path('/proc/stat').read_text().splitlines() if s.startswith(('cpu ', 'cpu0 '))])

    def run(phase, number, cases, duration):
        label = f'{phase}-{number}'
        filt = 'export_contract/' + ('(' + '|'.join(cases) + ')$' if cases else '')
        metrics = a.report / (label + '-time.json')
        command = ['/usr/bin/time', '-f', '{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
                   '-o', str(metrics), 'taskset', '-c', '0', str(a.binary), filt, '--bench',
                   '--sample-size', '30', '--warm-up-time', '0.5', '--measurement-time', str(duration),
                   '--save-baseline', 'r61a-' + label]
        entry = dict(command=command, before=snapshot())
        with (a.report / (label + '.txt')).open('w') as log:
            result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
        entry.update(after=snapshot(), exit_code=result.returncode, metrics=json.loads(metrics.read_text()))
        commands.append(entry)
        (a.report / 'environment.json').write_text(json.dumps(env, indent=2) + '\n')
        assert result.returncode == 0
        files = sorted(Path('target/criterion/export_contract').glob('*/r61a-' + label + '/sample.json'))
        assert len(files) == (len(cases) if cases else 8)
        for path in files:
            r = dict(case=path.parent.parent.name, phase=phase, round=number, **json.loads(path.read_text()))
            assert len(r['times']) == len(r['iters']) == 30
            r['median_ns'] = st.median(t / n for t, n in zip(r['times'], r['iters']))
            rows.append(r)
        (a.report / 'measurements.json').write_text(json.dumps(rows, indent=2) + '\n')
        print(label, 'complete', flush=True)

    for n in range(1, 4):
        run('initial', n, [], 2)
    names = sorted({r['case'] for r in rows})
    # No old implementation exists. Repeat a >5% spread, never call it a regression.
    unstable = [name for name in names if max(r['median_ns'] for r in rows if r['case'] == name)
                / min(r['median_ns'] for r in rows if r['case'] == name) > 1.05]
    if unstable:
        for n in range(1, 4):
            run('longer', n, unstable, 4)
    summary = dict(repeated_cases=unstable, cases={})
    for name in names:
        summary['cases'][name] = {phase: [r['median_ns'] for r in rows if r['case'] == name and r['phase'] == phase]
                                 for phase in ['initial', 'longer']}
    (a.report / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')

    plot(a.report)


def plot(root):
    rows = json.loads((root / 'measurements.json').read_text())
    summary = json.loads((root / 'summary.json').read_text())
    names = sorted(summary['cases'])
    assert len(names) == 8
    assert len(rows) == 24 + 3 * len(summary['repeated_cases'])
    assert len({(r['case'], r['phase'], r['round']) for r in rows}) == len(rows)
    for r in rows:
        assert r['round'] in [1, 2, 3]
        assert len(r['times']) == len(r['iters']) == 30
        assert all(math.isfinite(x) and x > 0 for x in r['times'] + r['iters'])
        assert math.isclose(r['median_ns'], st.median(t / n for t, n in zip(r['times'], r['iters'])))
    for name in names:
        for phase in ['initial', 'longer']:
            assert summary['cases'][name][phase] == [r['median_ns'] for r in rows if r['case'] == name and r['phase'] == phase]
        assert len(summary['cases'][name]['initial']) == 3
    repeated = [name for name in names if max(summary['cases'][name]['initial']) / min(summary['cases'][name]['initial']) > 1.05]
    assert repeated == summary['repeated_cases']
    assert all(len(summary['cases'][name]['longer']) == (3 if name in repeated else 0) for name in names)
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    fig, ax = plt.subplots(figsize=(11, 5))
    for phase, shift, color in [('initial', -.08, '#126e82'), ('longer', .08, '#b3541e')]:
        first = True
        for i, name in enumerate(names):
            values = summary['cases'][name][phase]
            if values:
                ax.scatter([i + shift] * len(values), values, color=color, label=phase if first else None)
                first = False
    ax.set_yscale('log'); ax.set_ylabel('Nanoseconds per operation (log scale)')
    ax.set_xticks(range(len(names)), [s.replace('_', '\n') for s in names]); ax.legend()
    ax.set_title('Artifact contract: synthetic baseline, every 30-sample run median')
    fig.tight_layout()
    out = root / 'plots'; out.mkdir(exist_ok=True)
    for ext in ['png', 'svg']:
        fig.savefig(out / ('contract.' + ext), dpi=160)
    plt.close(fig)
    svg = out / 'contract.svg'
    svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()) + '\n')
    paths = [root / 'measurements.json', root / 'summary.json', out / 'contract.svg', out / 'contract.png']
    (out / 'audit.json').write_text(json.dumps(dict(runs=len(rows), samples=30 * len(rows),
        sha256={str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}), indent=2) + '\n')


if __name__ == '__main__':
    main()
