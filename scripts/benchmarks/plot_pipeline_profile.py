#!/usr/bin/env python3
"""Validate and plot paired, disjoint local profiling regions; offline only."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics as st
from measure_pipeline_profile import FIXTURES, METRICS, medians, repeat_needed, validate

REGIONS = {
    'container_read': 'Encoded read', 'container_hash': 'Encoded SHA-256',
    'native_map': 'Native metadata', 'present_grain_read': 'Present grain reads',
    'copy_engine_including_flush': 'Copy engine + flush', 'logical_source_read': 'Logical source reads',
    'raw_read': 'RAW reads', 'logical_compare': 'Byte comparison', 'logical_hash': 'Logical SHA-256',
    'journal_commit_including_sync': 'Journal + sync', 'payload_sync': 'Other sync',
}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('report', type=Path); a = p.parse_args(); root = a.report
    summaries = {}; inputs = []; environments = []
    for directory in sorted(root.glob('profile-*')):
        if not directory.is_dir() or directory.name == 'profile-plots': continue
        runs = json.loads((directory/'measurements.json').read_text())
        assert {r['fixture'] for r in runs} == set(FIXTURES)
        assert all(r['phase'] in ['initial', 'repeat'] for r in runs)
        env = json.loads((directory/'environment.json').read_text()); environments.append(env)
        fs = env['filesystem']; assert fs not in summaries
        inputs += [directory/'measurements.json', directory/'environment.json']
        results = {}
        for fixture in FIXTURES:
            selected = [r for r in runs if r['fixture'] == fixture]
            initial = [r for r in selected if r['phase'] == 'initial']
            assert len(initial) == 3 and [r['round'] for r in initial] == [1, 2, 3]
            triggered = repeat_needed(initial)
            assert len(selected) == (6 if triggered else 3)
            result = dict(repeat_triggered=triggered, phases={})
            for phase in ['initial', 'repeat']:
                group = [r for r in selected if r['phase'] == phase]
                if not group: continue
                assert [r['round'] for r in group] == [1, 2, 3]
                pairs = 4 if phase == 'initial' else 8
                for run in group:
                    tag = f"{fixture}-{phase}-{run['round']}"
                    assert run['tag'] == tag
                    raw = directory/(tag+'.json')
                    assert json.loads(raw.read_text()) == run['rows']
                    inputs.append(raw)
                    validate(run['rows'], pairs, fixture)
                    assert run['medians'] == medians(run['rows'])
                    assert run['pairs'] == pairs
                control = [r for run in group for r in run['rows'] if not r['instrumented']]
                profiled = [r for run in group for r in run['rows'] if r['instrumented']]
                keys = set(REGIONS)
                assert all(set(r['regions']) == keys for r in profiled)
                estimates = {m: {k: st.median(r['medians'][m][k] for r in group) for k in METRICS} for m in ['disabled', 'instrumented']}
                spread = {m: {k: (max(r['medians'][m][k] for r in group)/min(r['medians'][m][k] for r in group)-1)*100 for k in METRICS} for m in estimates}
                regions = {key: {k: st.mean(r['regions'][key][k] for r in profiled) for k in ['wall_ms', 'cpu_ms', 'bytes', 'calls']} for key in keys}
                whole_means = {k: st.mean(r[k] for r in profiled) for k in ['wall_ms', 'cpu_ms']}
                residual = {k: whole_means[k]-sum(v[k] for v in regions.values()) for k in whole_means}
                assert residual['wall_ms'] >= 0 and residual['cpu_ms'] >= -.1
                result['phases'][phase] = dict(pairs=len(control), medians=estimates, spread_percent=spread,
                    profiling_change_percent={k:(estimates['instrumented'][k]/estimates['disabled'][k]-1)*100 for k in ['wall_ms','cpu_ms']},
                    paired_changes_percent={k:[(i[k]/c[k]-1)*100 for c,i in zip(control,profiled)] for k in ['wall_ms','cpu_ms']},
                    mean_regions=regions, instrumented_whole_means=whole_means, mean_unattributed=residual,
                    allocated_bytes=sorted({r['output_allocated_bytes'] for r in control+profiled}),
                    peak_rss_kib=sorted({r['process_peak_rss_kib'] for r in control+profiled}))
            results[fixture] = result
        summaries[fs] = results
    assert summaries and len({e['binary_sha256'] for e in environments}) == 1
    assert len({e['harness_sha256'] for e in environments}) == 1
    binaries = root/'binaries.json'
    if binaries.exists():
        assert environments[0]['binary_sha256'] == json.loads(binaries.read_text())['profile_test_sha256']
        inputs.append(binaries)
    assert len({(e['kernel'],e['cpu_model'],e['cpu'],e['sha_ni_visible']) for e in environments}) == 1
    summary = dict(scope='paired_instrumentation_overhead_and_authored_local_costs_not_a_production_speedup', filesystems=summaries)
    (root/'profile-summary.json').write_text(json.dumps(summary, indent=2, sort_keys=True)+'\n')
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    plt.rcParams.update({'svg.hashsalt':'rvddk-pipeline-profile-v1','font.family':'DejaVu Sans'})
    out = root/'profile-plots'; out.mkdir(exist_ok=True)
    fig, axes = plt.subplots(len(summaries), 2, figsize=(13,5*len(summaries)), squeeze=False, constrained_layout=True)
    labels = ['Stored / 64 MiB','Deflated / 64 MiB','Deflated / 256 MiB']
    color = plt.get_cmap('tab20')
    selected = {fs: [results[f]['phases'].get('repeat',results[f]['phases']['initial']) for f in FIXTURES] for fs,results in summaries.items()}
    for row,(fs,values) in enumerate(selected.items()):
        for col,key in enumerate(['wall_ms','cpu_ms']):
            ax=axes[row,col];bottom=[0.]*3
            for i,(region,label) in enumerate(REGIONS.items()):
                heights=[r['mean_regions'][region][key] for r in values]
                ax.bar(range(3),heights,bottom=bottom,label=label,color=color(i));bottom=[b+h for b,h in zip(bottom,heights)]
            residual=[max(0,r['mean_unattributed'][key]) for r in values]
            ax.bar(range(3),residual,bottom=bottom,label='Unattributed / profiling overhead',color='#777777')
            ax.set_xticks(range(3),labels,rotation=12);ax.set_ylim(bottom=0);ax.set_ylabel('Mean milliseconds')
            ax.set_title(f'{fs}: instrumented {"wall" if col==0 else "thread/process CPU"}')
            ax.grid(axis='y',alpha=.2);ax.set_axisbelow(True)
    axes[0,1].legend(fontsize=7,loc='upper left',bbox_to_anchor=(1.01,1))
    fig.suptitle('Disjoint measured regions / authored data = 8 MiB, logical capacity varies\nRepeat groups when triggered; one thread, shared CPU, ordinary caches; filesystems measured separately')
    save(fig,out,'regions');plt.close(fig)
    fig,axes=plt.subplots(len(summaries),2,figsize=(12,4.5*len(summaries)),squeeze=False,constrained_layout=True)
    for row,(fs,results) in enumerate(summaries.items()):
        for col,key in enumerate(['wall_ms','cpu_ms']):
            ax=axes[row,col]
            for phase,shift,marker in [('initial',-.12,'o'),('repeat',.12,'^')]:
                for i,f in enumerate(FIXTURES):
                    r=results[f]['phases'].get(phase)
                    if r is None:continue
                    ax.scatter([i+shift]*len(r['paired_changes_percent'][key]),r['paired_changes_percent'][key],s=15,alpha=.4,color='C0' if phase=='initial' else 'C1')
                    ax.scatter(i+shift,r['profiling_change_percent'][key],s=65,marker=marker,color='C0' if phase=='initial' else 'C1',label=phase if i==0 else None)
            ax.axhline(0,color='#555555',lw=.8);ax.axhline(5,color='#bb4444',ls='--',lw=.8)
            ax.set_xticks(range(3),labels,rotation=12);ax.set_ylabel('Instrumented / disabled change (%)');ax.set_title(f'{fs}: {key}');ax.legend()
    fig.suptitle('Profiling overhead controls / positive means slower; +5% investigation threshold\nLarge markers = ratio of median run medians; small dots = individual pairs, not confidence intervals')
    save(fig,out,'overhead');plt.close(fig)
    paths=inputs+[root/'profile-summary.json',Path(__file__),Path(__file__).with_name('measure_pipeline_profile.py')]+sorted(out.glob('*.svg'))+sorted(out.glob('*.png'))
    (out/'audit.json').write_text(json.dumps(dict(matplotlib=matplotlib.__version__,sha256={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}),indent=2)+'\n')


def save(fig, root, name):
    fig.savefig(root/(name+'.svg'),metadata={'Date':None})
    p=root/(name+'.svg');p.write_text('\n'.join(s.rstrip() for s in p.read_text().splitlines())+'\n')
    fig.savefig(root/(name+'.png'),dpi=160,metadata={'Software':'rvddk qualification'})


if __name__=='__main__':
    main()
