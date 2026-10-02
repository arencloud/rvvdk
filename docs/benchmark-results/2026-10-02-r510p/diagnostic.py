"""Separate-core diagnostic with an identical-binary control; retain every run."""
import hashlib,json,os,statistics,subprocess,time
from pathlib import Path
out=Path('target/r510p/observations/diagnostic');out.mkdir(exist_ok=True)
cpu=4;assert cpu in os.sched_getaffinity(0)
paths={'before':Path('target/r510p/reference/stream-before'),'control':Path('target/r510p/reference/stream-before'),'candidate':Path('target/r510p/reference/stream-candidate')}
rows=[];commands=[]
(out/'environment.json').write_text(json.dumps({'cpu_affinity':[cpu],'smt_siblings':Path(f'/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list').read_text().strip(),'governor':Path(f'/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor').read_text().strip(),'binary_sha256':{k:hashlib.sha256(p.read_bytes()).hexdigest() for k,p in paths.items()},'reason':'Preserved adverse CPU0 observations; test a different core and a bit-identical before/control executable to quantify within-build variance. This does not replace CPU0 evidence.','conditions':'Shared host, no CPU isolation, unchanged governor/ASLR; compilation and test work outside timings.','warmup_seconds':0.5,'measurement_seconds':4,'sample_size':30},indent=2)+'\n')
for round_,order in enumerate([['before','control','candidate'],['control','candidate','before'],['candidate','before','control']],1):
 for variant in order:
  key=f'r510p-cpu4-{variant}-{round_}'
  command=['taskset','-c',str(cpu),str(paths[variant]),'--bench','--warm-up-time','0.5','--measurement-time','4','--sample-size','30','--save-baseline',key]
  started=time.time()
  with (out/(key+'.txt')).open('w') as log:subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,check=True)
  commands.append({'key':key,'command':command,'start_unix':started,'end_unix':time.time()})
  files=sorted(Path('target/criterion/stream_admission').glob('*/'+key+'/sample.json'));assert len(files)==7
  for path in files:
   r={'benchmark':'stream_admission/'+path.parent.parent.name,'phase':'diagnostic','variant':variant,'round':round_,**json.loads(path.read_text())}
   r['median_ns']=statistics.median(t/i for t,i in zip(r['times'],r['iters']));rows.append(r)
  (out/'commands.json').write_text(json.dumps(commands,indent=2)+'\n');(out/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n')
  print(key,'done',flush=True)
pairs=[]
for c in rows:
 if c['variant']=='before':continue
 b,=[r for r in rows if r['benchmark']==c['benchmark'] and r['round']==c['round'] and r['variant']=='before']
 pairs.append({'benchmark':c['benchmark'],'variant':c['variant'],'round':c['round'],'change_percent':100*(c['median_ns']/b['median_ns']-1)})
(out/'summary.json').write_text(json.dumps({'pairs':pairs,'scope':'diagnostic only; identical before/control executable; primary CPU0 observations retained'},indent=2)+'\n')
