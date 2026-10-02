#!/usr/bin/env python3
"""Validate and plot R5.11b native reads and all retained map regression pairs."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def save(fig,root,name):
    svg=root/(name+'.svg')
    fig.savefig(svg,metadata={'Date':None})
    svg.write_text('\n'.join(line.rstrip() for line in svg.read_text().splitlines()).rstrip()+'\n')
    fig.savefig(root/(name+'.png'),dpi=180)
    plt.close(fig)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('report',type=Path);root=p.parse_args().report
    sample=root/'measurements.json';live_path=root/'live.json'
    rows=json.loads(sample.read_text());live=json.loads(live_path.read_text());seen=set()
    for r in rows:
        key=(r['benchmark'],r['variant'],r['phase'],r['round']);assert key not in seen;seen.add(key)
        assert len(r['times'])==len(r['iters'])==30 and all(x>0 for x in r['times']+r['iters'])
        assert abs(statistics.median(t/n for t,n in zip(r['times'],r['iters']))-r['median_ns'])<1e-9
    phases=['initial']+(['longer'] if any(r['phase']=='longer' for r in rows) else [])
    pairs=[];new={}
    for r in rows:
        if r['variant']=='map-candidate':
            b,=[x for x in rows if x['benchmark']==r['benchmark'] and x['variant']=='map-before' and x['phase']==r['phase'] and x['round']==r['round']]
            pairs.append(dict(benchmark=r['benchmark'],phase=r['phase'],round=r['round'],change_percent=100*(r['median_ns']/b['median_ns']-1)))
        elif r['variant']=='reads':new.setdefault(r['benchmark'].split('/')[1],[]).append(r['median_ns'])
    assert len(pairs)==12*len(phases) and len(rows)==24*len(phases)+21
    assert len(new)==7 and all(len(v)==3 for v in new.values())
    live_rows={mode:[r for r in live['rows'] if r['report']['mode']==mode] for mode in ['sequential','random']}
    assert all(len(v)==3 for v in live_rows.values()) and all(r['exit_code']==0 for r in live['rows'])
    checks=[r for r in live['rows'] if r['report']['mode'] in ['ranges','compare']]
    assert len(checks)==2 and all(r['report']['reference_matched'] for r in checks)
    metrics={
        'read_mib_s':lambda r:r['report']['logical_bytes']/2**20/r['report']['read_seconds'],
        'read_seconds':lambda r:r['report']['read_seconds'],
        'admission_seconds':lambda r:r['report']['admission_seconds'],
        'process_cpu_seconds':lambda r:r['metrics']['user_seconds']+r['metrics']['system_seconds'],
        'process_rss_mib':lambda r:r['metrics']['max_rss_kib']/1024,
    }
    live_values={mode:{key:[fn(r) for r in group] for key,fn in metrics.items()} for mode,group in live_rows.items()}
    out=root/'plots';out.mkdir(exist_ok=True)
    computed=dict(schema=1,pairs=pairs,native_run_medians_ns=new,native_median_ns={k:statistics.median(v) for k,v in new.items()},live_values=live_values,scope='In-memory read costs; buffered live allocated-grain sequential/random reads. No zero-capacity throughput inflation or whole-source speedup claim.')
    (out/'computed.json').write_text(json.dumps(computed,indent=2)+'\n')
    plt.rcParams.update({'font.family':'DejaVu Sans','font.size':10,'svg.hashsalt':'rvddk-r511b-v1','svg.fonttype':'none','axes.spines.top':False,'axes.spines.right':False})
    fig,axes=plt.subplots(2,2,figsize=(11,7),layout='constrained');fig.suptitle('R5.11b · Map admission controls against ea94b1d',fontsize=15)
    names=sorted({r['benchmark'] for r in pairs});assert len(names)==4
    for ax,name in zip(axes.flat,names):
        for i,phase in enumerate(phases):
            values=[r['change_percent'] for r in pairs if r['benchmark']==name and r['phase']==phase];assert len(values)==3
            ax.scatter([i-.12,i,i+.12],values,color='#c25b2a')
            ax.plot([i-.2,i+.2],[statistics.median(values)]*2,color='#174f70',lw=2)
        ax.axhline(5,color='#a9323c',ls='--');ax.axhline(0,color='gray',lw=.7)
        ax.set_xticks(range(len(phases)),phases);ax.set_title(name.split('/')[1].replace('_',' '));ax.set_ylabel('Elapsed change (%) · lower is better');ax.grid(axis='y',alpha=.2)
    fig.supxlabel('All paired run medians retained · +5% investigation threshold · shared host, CPU 0',fontsize=10)
    save(fig,out,'map-controls')
    fig,ax=plt.subplots(figsize=(11,5.5),layout='constrained');names=sorted(new)
    for i,name in enumerate(names):
        values=[v/1000 for v in new[name]];ax.scatter(values,[i]*3,color='#c25b2a',zorder=3)
        median=statistics.median(values);ax.plot([median,median],[i-.25,i+.25],color='#174f70',lw=3)
    ax.set_xscale('log');ax.set_yticks(range(len(names)),[n.replace('_',' ') for n in names]);ax.grid(axis='x',alpha=.2)
    ax.set_xlabel('Microseconds per operation · logarithmic scale · lower is better');ax.set_title('R5.11b · Native logical reads in memory',fontsize=15)
    fig.supxlabel('Three 30-sample runs · 1 MiB sequential/zero requests; 4 KiB random/cached/cross-grain requests',fontsize=9)
    save(fig,out,'native-read-latency')
    fig,axes=plt.subplots(2,3,figsize=(13,7.5),layout='constrained');fig.suptitle('R5.11b · Repeated native reads of the retained ESXi export',fontsize=15)
    titles=['Delivered read MiB/s · higher is better','Read phase seconds','Map acquisition seconds','Whole-process CPU seconds','Whole-process peak RSS (MiB)']
    for ax,(key,_),title in zip(axes.flat,metrics.items(),titles):
        for i,mode in enumerate(['sequential','random']):
            values=live_values[mode][key];ax.bar(i,statistics.median(values),color='#28688b',alpha=.75)
            ax.scatter([i-.12,i,i+.12],values,color='#d46e2f',s=26,zorder=3)
        ax.set_xticks([0,1],['Allocated-grain\nsequential','4 KiB\nrandom']);ax.set_title(title);ax.grid(axis='y',alpha=.2)
    axes.flat[-1].axis('off');axes.flat[-1].text(0,.95,'Three runs per workload\n57,295 allocated 64 KiB grains\n4,096 random requests per run\n\nRead time excludes map admission\nCPU/RSS include whole process\nShared VM, buffered I/O\nNo cache flushing\n\nNo VM power changes or new export',va='top')
    save(fig,out,'live-native-reads')
    audit=dict(schema=1,runs=len(rows),samples=sum(len(r['times']) for r in rows),inputs={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [sample,live_path]},generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),matplotlib=matplotlib.__version__,outputs={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.name!='audit.json'})
    (out/'audit.json').write_text(json.dumps(audit,indent=2)+'\n');print(f'Validated {len(rows)} runs and live checks; wrote three SVG/PNG plots.')


if __name__=='__main__':main()
