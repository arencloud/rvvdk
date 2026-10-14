#!/usr/bin/env python3
"""Serial, offline SHA-256 backend measurements with independent digest checks."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import statistics as st
import subprocess

CASES = [
    ('metadata64',64,64,262144,'dense'),
    ('metadata1024',1024,1024,16384,'dense'),
    ('chunk64k',65536,65536,256,'dense'),
    ('dense64',64<<20,1<<20,1,'dense'),
    ('dense64_chunk64k',64<<20,65536,1,'dense'),
    ('sparse64',64<<20,1<<20,1,'sparse'),
    ('dense256',256<<20,1<<20,1,'dense'),
    ('sparse256',256<<20,1<<20,1,'sparse'),
]
def write(path,value): path.write_text(json.dumps(value,indent=2,allow_nan=False)+'\n')
def load(path): return json.loads(path.read_text())
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def oracle(size,sparse):
    block=bytes((i*131+(i>>8)*17+29)&255 for i in range(65536))
    block=(block[:32768]+bytes((1<<20)-32768)) if sparse else block
    digest=hashlib.sha256()
    while size:
        n=min(size,len(block));digest.update(block[:n]);size-=n
    return digest.hexdigest()
def validate(value,pairs,case):
    _,size,chunk,iterations,pattern=case;rows=value['rows'];assert len(rows)==pairs*2
    expected=oracle(size,pattern=='sparse')
    for pair in range(pairs):
        group=rows[2*pair:2*pair+2]
        assert [r['pair'] for r in group]==[pair,pair]
        assert [r['backend'] for r in group]==(['sha2','ring'] if pair%2==0 else ['ring','sha2'])
    for r in rows:
        assert (r['bytes_per_digest'],r['chunk'],r['iterations'],r['sparse_pattern'])==(size,chunk,iterations,pattern=='sparse')
        assert r['sha256']==expected and r['oracle_passed'] is True
        assert all(math.isfinite(r[k]) and r[k]>0 for k in ['wall_ms','cpu_ms','peak_rss_kib'])
    return rows

def summarize(report):
    index=load(report/'hash-measurements.json');groups={}
    for run in index:
        assert sha(report/run['file'])==run['sha256']
        case=next(c for c in CASES if c[0]==run['group']);rows=validate(load(report/run['file']),run['pairs'],case)
        if run['stage']!='repeat':continue
        g=groups.setdefault(run['group'],{})
        for backend in ['sha2','ring']:
            values=[r for r in rows if r['backend']==backend];b=g.setdefault(backend,{})
            for k in ['wall_ms','cpu_ms']:
                b.setdefault(k,[]).append(st.median(r[k]/r['iterations'] for r in values))
    result={}
    for name,g in groups.items():
        result[name]={b:{k:{'round_medians_ms_per_digest':v,'median_ms_per_digest':st.median(v),'spread_percent':100*(max(v)/min(v)-1)} for k,v in m.items()} for b,m in g.items()}
        result[name]['ring_change_percent']={k:100*(st.median(g['ring'][k])/st.median(g['sha2'][k])-1) for k in ['wall_ms','cpu_ms']}
    write(report/'hash-summary.json',{'timed_observations':sum(r['pairs']*2 for r in index),'groups':result})
    return result

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args();a.binary=a.binary.resolve();a.report=a.report.resolve();assert not (a.report/'hash-measurements.json').exists()
    os.sched_setaffinity(0,{min(os.sched_getaffinity(0))});index=[]
    for case in CASES:
        group,size,chunk,iterations,pattern=case
        for stage,pairs in [('initial',4),('repeat',8)]:
            for round_ in range(1,4):
                filename=f'hash-{group}-{stage}-{round_}.json'
                out=subprocess.run([a.binary,'bench',str(size),str(chunk),str(iterations),str(pairs),pattern],capture_output=True,text=True,check=True,timeout=300)
                assert not out.stderr
                value=json.loads(out.stdout);validate(value,pairs,case);write(a.report/filename,value)
                index.append({'group':group,'stage':stage,'round':round_,'pairs':pairs,'file':filename,'sha256':sha(a.report/filename)});write(a.report/'hash-measurements.json',index);print(filename,flush=True)
    summarize(a.report)
    print('Hash campaign complete: 576 timed paired observations',flush=True)
if __name__=='__main__':main()
