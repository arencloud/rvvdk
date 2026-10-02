#!/usr/bin/env python3
"""Audit and plot R5.12p zero output, paired controls and retained-export allocation."""
import argparse,hashlib,json,math,statistics
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('report',type=Path);args=p.parse_args();root=args.report
    rows=json.loads((root/'measurements.json').read_text());summary=json.loads((root/'summary.json').read_text());out=root/'plots';out.mkdir(exist_ok=True)
    keys=[(r['benchmark'],r['variant'],r['phase'],r['round']) for r in rows];assert len(keys)==len(set(keys))
    for r in rows:
        assert len(r['times'])==len(r['iters'])==30
        assert all(math.isfinite(x) and x>0 for x in r['times']+r['iters'])
        assert math.isclose(r['median_ns'],statistics.median(t/n for t,n in zip(r['times'],r['iters'])))
    for phase,before,candidate in [('initial','before','candidate'),('longer','before','candidate'),('historical','historical','before'),('identical','identical_a','identical_b')]:
        matches=[r for r in rows if r['phase']==phase and r['variant']==candidate];assert len(matches)==len(summary[phase])
        for r in matches:
            b,=[v for v in rows if v['phase']==phase and v['variant']==before and (v['round'],v['benchmark'])==(r['round'],r['benchmark'])]
            pair,=[v for v in summary[phase] if (v['round'],v['benchmark'])==(r['round'],r['benchmark'])]
            assert math.isclose(pair['change_percent'],100*(r['median_ns']/b['median_ns']-1))
    assert len(rows)==48+6*len(summary['adverse_cases'])+12
    plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False});outputs=[]
    def save(fig,name):
        fig.tight_layout()
        for ext in ['svg','png']:
            path=out/(name+'.'+ext);fig.savefig(path,dpi=160)
            if ext=='svg':path.write_text('\n'.join(v.rstrip() for v in path.read_text().splitlines())+'\n')
            outputs.append(path)
        plt.close(fig)
    fig,ax=plt.subplots(figsize=(15,6));names=sorted({r['benchmark'] for r in summary['initial']})
    for phase,color,shift in [('initial','#126e82',-.12),('longer','#b3541e',.12)]:
        first=True
        for i,name in enumerate(names):
            ys=[r['change_percent'] for r in summary[phase] if r['benchmark']==name]
            if ys:
                ax.scatter([i+shift]*len(ys),ys,color=color,label=phase if first else None);first=False
                ax.plot([i+shift-.07,i+shift+.07],[statistics.median(ys)]*2,color=color,lw=3)
    ax.axhline(0,color='gray',lw=1);ax.axhline(5,color='red',ls='--',lw=1)
    ax.set_xticks(range(len(names)),[n.replace('cli_','').replace('/','\n') for n in names],rotation=20,ha='right')
    ax.set_ylabel('Candidate time change (%) — higher is slower');ax.set_title('Zero-output change: every paired control and adverse repeat');ax.legend();save(fig,'controls')
    fig,axes=plt.subplots(1,2,figsize=(10,4))
    for ax,phase,title in zip(axes,['historical','identical'],['Prior CLI integration (d65b098 → 0ed0a09)','Same 0ed0a09 executable, paired with itself']):
        points=summary[phase];ax.scatter([r['round'] for r in points],[r['change_percent'] for r in points],color='#126e82',s=60)
        ax.axhline(0,color='gray');ax.axhline(5,color='red',ls='--');ax.set_xticks([1,2,3]);ax.set_xlabel('Paired round');ax.set_ylabel('Time change (%)');ax.set_title(title)
    fig.suptitle('Monolithic sparse copy + verify: separate causality controls');save(fig,'historical-controls')
    local=json.loads((root/'live-local.json').read_text());runner=json.loads((root/'live-runner.json').read_text())
    assert len(local['rows'])==18 and len(runner['rows'])==6
    for data in [local,runner]:
        for r in data['rows']:
            assert r['exit_code']==0 and r['logical_bytes']==32212254720
            assert r['stats']['bytes_zeroed']==28457369600 and r['stats']['bytes_discarded']==0
            if r['mode']=='copy_verify':assert r['verification']['bytes_verified']==r['logical_bytes']
        for check in data['checks']:
            assert check.get('exit_code',0)==0 and check.get('matched',True)
            assert check['logical_bytes']==(8388608 if check['case']=='independent_guest_oracle' else 32212254720)
    fig,axes=plt.subplots(1,4,figsize=(16,5));groups=[('Local before',local,'before'),('Local candidate',local,'candidate'),('XFS runner candidate',runner,'candidate')]
    for ax,metric,label in zip(axes,['elapsed','cpu','rss','allocation'],['CLI operation seconds','Process CPU seconds','Peak RSS (MiB)','Output allocation (GiB)']):
        for i,(name,data,variant) in enumerate(groups):
            for mode,color,shift in [('copy','#126e82',-.13),('copy_verify','#b3541e',.13)]:
                values=[]
                for r in data['rows']:
                    if r['variant']!=variant or r['mode']!=mode:continue
                    m=r['metrics'];values.append({'elapsed':r['elapsed_seconds'],'cpu':m['user_seconds']+m['system_seconds'],'rss':m['max_rss_kib']/1024,'allocation':r['output_allocated_bytes']/1024**3}[metric])
                ax.scatter([i+shift]*len(values),values,color=color,label=mode if i==0 else None)
                ax.plot([i+shift-.08,i+shift+.08],[statistics.median(values)]*2,color=color,lw=3)
        ax.set_xticks(range(3),[g[0].replace(' ','\n') for g in groups]);ax.set_ylabel(label);ax.set_ylim(bottom=0)
    axes[0].legend();fig.suptitle('Retained 30 GiB export — local pairs and separate unchanged XFS runner');save(fig,'retained-export')
    fig,ax=plt.subplots(figsize=(10,4))
    paired=[]
    for mode,color,marker in [('copy','#126e82','o'),('copy_verify','#b3541e','s')]:
        values=[]
        for r in local['rows']:
            if r['variant']!='candidate' or r['mode']!=mode:continue
            b,=[v for v in local['rows'] if v['variant']=='before' and v['mode']==mode and v['round']==r['round']]
            pair=dict(mode=mode,round=r['round'],change_percent=100*(r['elapsed_seconds']/b['elapsed_seconds']-1))
            paired.append(pair);values.append(pair)
        ax.scatter([v['round'] for v in values],[v['change_percent'] for v in values],color=color,marker=marker,s=60,label=mode)
    assert len(paired)==9
    ax.axhline(0,color='gray');ax.axhline(5,color='red',ls='--');ax.axvline(3.5,color='gray',ls=':')
    ax.set_xticks(range(1,7));ax.set_xlabel('Paired round (4–6 are additional copy-only repeats)');ax.set_ylabel('Candidate elapsed change (%)')
    ax.set_title('Local retained export: preserve the adverse full-copy result');ax.legend();save(fig,'local-latency-pairs')
    path=out/'local-pairs.json';path.write_text(json.dumps(paired,indent=2)+'\n');outputs.append(path)
    inputs=['measurements.json','summary.json','live-local.json','live-runner.json']
    audit=dict(rows=len(rows),samples=len(rows)*30,inputs={n:hashlib.sha256((root/n).read_bytes()).hexdigest() for n in inputs},outputs={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in outputs})
    (out/'audit.json').write_text(json.dumps(audit,indent=2)+'\n');print(json.dumps({'rows':len(rows),'samples':len(rows)*30,'plots':sum(p.suffix in ['.png','.svg'] for p in outputs)}))


if __name__=='__main__':main()
