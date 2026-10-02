import hashlib,json,os,statistics,subprocess
from pathlib import Path
out=Path('target/r510-observations/final-bench');out.mkdir(exist_ok=True);rows=[];cpu=min(os.sched_getaffinity(0))
base=Path('target/r510-reference/descriptor-baseline');candidate=Path('target/release/deps/descriptor-5da4a7465d2c7315');stream=Path('target/release/deps/stream-831300028eecf8ad')
(out/'benchmark-environment.json').write_text(json.dumps({'cpu_affinity':[cpu],'baseline_commit':'f756fb1','binary_sha256':{n:hashlib.sha256(p.read_bytes()).hexdigest() for n,p in [('baseline',base),('candidate',candidate),('stream',stream)]},'conditions':'same pinned CPU; shared host, caches/governor uncontrolled; synthetic memory inputs; no storage/decompression throughput claim'},indent=2)+'\n')
def run(binary,group,variant,round_,duration,phase):
 key=f'r510-final-{phase}-{variant}-{round_}'
 with (out/(key+'.txt')).open('w') as log:
  subprocess.run(['taskset','-c',str(cpu),str(binary),'--bench','--warm-up-time','0.2','--measurement-time',str(duration),'--sample-size','30','--save-baseline',key],stdout=log,stderr=subprocess.STDOUT,check=True)
 for path in sorted(Path('target/criterion',group).glob('*/'+key+'/sample.json')):
  sample=json.loads(path.read_text());r={'benchmark':group+'/'+path.parent.parent.name,'variant':variant,'round':round_,'phase':phase,**sample};r['median_ns']=statistics.median(t/n for t,n in zip(r['times'],r['iters']));rows.append(r)
 (out/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n')
 print(key,'done',flush=True)
for i in range(1,4):
 for variant,b in ([('baseline',base),('candidate',candidate)] if i%2 else [('candidate',candidate),('baseline',base)]):run(b,'descriptor',variant,i,.5,'initial')
def pairs(phase):
 found=[]
 for a in rows:
  if a['phase']==phase and a['variant']=='baseline':
   b=next(x for x in rows if x['phase']==phase and x['variant']=='candidate' and x['round']==a['round'] and x['benchmark']==a['benchmark'])
   found.append({'benchmark':a['benchmark'],'round':a['round'],'change_percent':(b['median_ns']/a['median_ns']-1)*100})
 return found
initial=pairs('initial');trigger=any(x['change_percent']>5 for x in initial)
if trigger:
 print('Adverse >5% observation: running three longer alternating pairs.',flush=True)
 for i in range(1,4):
  for variant,b in ([('candidate',candidate),('baseline',base)] if i%2 else [('baseline',base),('candidate',candidate)]):run(b,'descriptor',variant,i,2,'longer')
for i in range(1,4):run(stream,'stream_admission','new',i,.5,'new')
(out/'performance-summary.json').write_text(json.dumps({'initial_pairs':initial,'longer_triggered':trigger,'longer_pairs':pairs('longer'),'scope':'descriptor regression check and new metadata microbenchmarks'},indent=2)+'\n')
