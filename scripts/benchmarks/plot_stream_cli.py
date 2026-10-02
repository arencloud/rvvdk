#!/usr/bin/env python3
"""Plot retained R5.12 paired controls, new CLI samples and live conversion costs."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import statistics
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('report',type=Path)
    args=p.parse_args();root=args.report;out=root/'plots';out.mkdir(exist_ok=True)
    rows=json.loads((root/'measurements.json').read_text())
    for r in rows:
        assert len(r['times'])==len(r['iters'])==30
        assert all(math.isfinite(v) and v>0 for v in r['times']+r['iters'])
        assert math.isclose(statistics.median(t/n for t,n in zip(r['times'],r['iters'])),r['median_ns'])
    plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False})
    outputs=[]
    def save(fig,name):
        fig.tight_layout()
        for ext in ['svg','png']:
            path=out/(name+'.'+ext);fig.savefig(path,dpi=160)
            if ext=='svg':path.write_text('\n'.join(v.rstrip() for v in path.read_text().splitlines())+'\n')
            outputs.append(path)
        plt.close(fig)
    summary=json.loads((root/'summary.json').read_text())
    keys=[(r['benchmark'],r['variant'],r['phase'],r['round']) for r in rows]
    assert len(keys)==len(set(keys))
    assert len(rows)==(78 if summary['longer_triggered'] else 48)
    for phase in ['initial','longer']:
        candidates=[r for r in rows if r['variant']=='sparse-candidate' and r['phase']==phase]
        assert len(candidates)==len(summary[phase+'_pairs'])
        for r in candidates:
            b,=[b for b in rows if b['variant']=='sparse-before' and
                (b['benchmark'],b['phase'],b['round'])==(r['benchmark'],r['phase'],r['round'])]
            pair,=[p for p in summary[phase+'_pairs'] if (p['benchmark'],p['round'])==(r['benchmark'],r['round'])]
            assert math.isclose(pair['change_percent'],100*(r['median_ns']/b['median_ns']-1))
    fig,ax=plt.subplots(figsize=(11,5))
    names=sorted({r['benchmark'] for r in summary['initial_pairs']})
    for phase,color,shift in [('initial','#126e82',-.12),('longer','#b3541e',.12)]:
        values=summary[phase+'_pairs']
        for i,name in enumerate(names):
            ys=[r['change_percent'] for r in values if r['benchmark']==name]
            if ys:
                ax.scatter([i+shift]*len(ys),ys,color=color,label=phase if i==0 else None)
                ax.plot([i+shift-.07,i+shift+.07],[statistics.median(ys)]*2,color=color,lw=3)
    ax.axhline(0,color='gray',lw=1);ax.axhline(5,color='red',ls='--',lw=1)
    ax.set_xticks(range(len(names)),[n.split('/')[-1] for n in names],rotation=15)
    ax.set_ylabel('Candidate time change (%) — higher is slower')
    ax.set_title('Existing sparse CLI controls: every paired round retained')
    ax.legend();save(fig,'controls')
    fig,ax=plt.subplots(figsize=(11,5));names=sorted({r['benchmark'] for r in rows if r['variant']=='stream'})
    for i,name in enumerate(names):
        group=[r for r in rows if r['variant']=='stream' and r['benchmark']==name]
        for j,r in enumerate(group):
            samples=[t/n/1e6 for t,n in zip(r['times'],r['iters'])]
            ax.scatter([i+(j-1)*.1]*30,samples,s=9,alpha=.35,color='#126e82')
            ax.scatter(i+(j-1)*.1,r['median_ns']/1e6,color='#b3541e',s=25)
    ax.set_xticks(range(len(names)),[n.split('/')[-1] for n in names],rotation=15)
    ax.set_yscale('log');ax.set_ylabel('Complete CLI operation (ms, log scale)')
    ax.set_title('New stream CLI baselines — 4 MiB images, 30 samples × 3 rounds')
    save(fig,'stream-cli')
    live=json.loads((root/'live.json').read_text());runs=[r for r in live['rows'] if r['case'].startswith('copy-verify-')]
    assert len(runs)==3 and all(r['exit_code']==0 and r['verification']['bytes_verified']==r['logical_bytes'] for r in runs)
    assert all(r['exit_code']==0 for r in live['rows'])
    assert live['checks']==[
        {'case':'qemu_full_logical','exit_code':0,'logical_bytes':runs[0]['logical_bytes']},
        {'case':'independent_guest_oracle','matched':True,'logical_bytes':8388608}]
    fig,axes=plt.subplots(1,4,figsize=(15,4));x=list(range(1,len(runs)+1))
    for ax,ys,label in [
        (axes[0],[r['logical_bytes']/1048576/r['elapsed_seconds'] for r in runs],'Logical MiB/s (includes holes)'),
        (axes[1],[r['metrics']['user_seconds']+r['metrics']['system_seconds'] for r in runs],'Process CPU seconds'),
        (axes[2],[r['metrics']['max_rss_kib']/1024 for r in runs],'Peak RSS (MiB)'),
        (axes[3],[r['output_allocated_bytes']/1024**3 for r in runs],'Output allocation (GiB)')]:
        ax.bar(x,ys,color='#126e82');ax.set_xticks(x);ax.set_xlabel('Run');ax.set_ylabel(label)
    fig.suptitle('Retained 30 GiB export: native CLI copy + full readback + durability')
    save(fig,'live-conversion')
    audit=dict(rows=len(rows),samples=sum(len(r['times']) for r in rows),inputs={n:hashlib.sha256((root/n).read_bytes()).hexdigest() for n in ['measurements.json','summary.json','live.json']},outputs={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in outputs})
    (out/'audit.json').write_text(json.dumps(audit,indent=2)+'\n')
    print(json.dumps(dict(rows=audit['rows'],samples=audit['samples'],plots=len(outputs))))


if __name__=='__main__':main()
