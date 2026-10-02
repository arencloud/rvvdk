#!/usr/bin/env python3
"""Plot R5.11a grain-index measurements, retaining every regression pair."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def save(fig, out, name):
    fig.savefig(out / (name + '.svg'), metadata={'Date': None})
    svg = out / (name + '.svg')
    svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()).rstrip()+'\n')
    fig.savefig(out / (name + '.png'), dpi=180)
    plt.close(fig)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('report', type=Path)
    args = p.parse_args()
    source = args.report / 'measurements.json'
    rows = json.loads(source.read_text())
    keys = set()
    for r in rows:
        key = (r['benchmark'], r['variant'], r['phase'], r['round'])
        assert key not in keys
        keys.add(key)
        assert len(r['times']) == len(r['iters']) == 30
        assert all(x > 0 for x in r['times'] + r['iters'])
        median = statistics.median(t/n for t, n in zip(r['times'], r['iters']))
        assert abs(median - r['median_ns']) < 1e-9
    phases = ['initial'] + (['longer'] if any(r['phase'] == 'longer' for r in rows) else [])
    pairs = []
    for r in rows:
        if r['variant'] != 'candidate':
            continue
        base, = [b for b in rows if b['benchmark'] == r['benchmark'] and b['phase'] == r['phase'] and b['round'] == r['round'] and b['variant'] == 'before']
        pairs.append(dict(benchmark=r['benchmark'], phase=r['phase'], round=r['round'], change_percent=100*(r['median_ns']/base['median_ns']-1)))
    new = {}
    for r in rows:
        if r['variant'] == 'map':
            assert r['phase'] == 'new'
            new.setdefault(r['benchmark'].split('/')[1], []).append(r['median_ns'])
    assert len(new) == 9 and all(len(v) == 3 for v in new.values())
    assert len(pairs) == 21*len(phases)
    assert len(rows) == 27+42*len(phases)
    out = args.report / 'plots'
    out.mkdir(exist_ok=True)
    computed = dict(schema=1, pairs=pairs, map_run_medians_ns=new,
                    map_median_ns={k: statistics.median(v) for k, v in new.items()},
                    scope='Synthetic metadata in memory; no storage/decompression throughput or RSS claim')
    (out / 'computed.json').write_text(json.dumps(computed, indent=2)+'\n')
    plt.rcParams.update({'font.family': 'DejaVu Sans', 'font.size': 10,
                         'svg.hashsalt': 'rvddk-r511a-v1', 'svg.fonttype': 'none',
                         'axes.spines.top': False, 'axes.spines.right': False})
    fig, axes = plt.subplots(2, 4, figsize=(13, 7), layout='constrained')
    fig.suptitle('R5.11a · Existing stream admission against 64d81a7', fontsize=16)
    for ax, name in zip(axes.flat, sorted({r['benchmark'] for r in pairs})):
        for i, phase in enumerate(phases):
            values = [r['change_percent'] for r in pairs if r['benchmark'] == name and r['phase'] == phase]
            assert len(values) == 3
            ax.scatter([i-.12, i, i+.12], values, color='#c25b2a', s=26)
            ax.plot([i-.2,i+.2], [statistics.median(values)]*2, color='#174f70', lw=2)
        ax.set_title(name.split('/')[1].replace('_', ' '))
        ax.set_xticks(range(len(phases)), phases)
        ax.axhline(5, color='#a9323c', ls='--')
        ax.axhline(0, color='gray', lw=.7)
        ax.set_ylabel('Elapsed change (%) · lower is better')
        ax.grid(axis='y', alpha=.2)
    axes.flat[-1].axis('off')
    axes.flat[-1].text(0, .9, 'Dots: all three paired runs\nLine: median change\nDashed: +5% investigation gate\n\nShared host, CPU 0\nAlternating binary order\nNo blanket performance clearance', va='top')
    save(fig,out,'stream-regressions')
    fig, ax = plt.subplots(figsize=(11,6), layout='constrained')
    names = sorted(new)
    values = [statistics.median(new[n])/1000 for n in names]
    ax.barh(range(len(names)),values,color='#28688b',alpha=.8)
    for i, name in enumerate(names):
        ax.scatter([v/1000 for v in new[name]],[i]*3,color='#d46e2f',s=24,zorder=3)
    ax.set_yticks(range(len(names)),[n.replace('_',' ') for n in names])
    ax.set_xscale('log')
    ax.set_xlabel('Microseconds per operation · logarithmic scale · lower is better')
    ax.set_title('R5.11a · Bounded grain-index admission and lookup in memory', fontsize=15)
    ax.grid(axis='x',alpha=.2)
    fig.supxlabel('Three 30-sample runs · synthetic metadata · payloads skipped · no I/O throughput claim',fontsize=10)
    save(fig,out,'map-latency')
    audit = dict(schema=1, runs=len(rows), samples=sum(len(r['times']) for r in rows),
                 inputs={str(source.name):hashlib.sha256(source.read_bytes()).hexdigest()},
                 generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                 matplotlib=matplotlib.__version__,
                 outputs={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.name != 'audit.json'})
    (out/'audit.json').write_text(json.dumps(audit,indent=2)+'\n')
    print(f'Validated {len(rows)} runs; wrote two SVG/PNG charts and audit.')


if __name__ == '__main__':
    main()
