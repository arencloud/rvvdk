"""Paired CLI sparse controls and stream conversion baselines; preserve adverse repeats."""
import hashlib,json,os,platform,statistics,subprocess,time
from pathlib import Path
out=Path('docs/benchmark-results/2026-10-02-r512');cpu=0;rows=[]
binaries={v:Path('target/r512/reference/'+v) for v in ['sparse-before','sparse-candidate','stream']}
assert cpu in os.sched_getaffinity(0)
env={'baseline_commit':'d65b098','binary_sha256':{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()},'cpu_affinity':[cpu],'smt_siblings':Path('/sys/devices/system/cpu/cpu0/topology/thread_siblings_list').read_text().strip(),'governor':Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor').read_text().strip(),'rustc':subprocess.check_output(['rustc','-Vv'],text=True).strip(),'kernel':platform.release(),'sample_size':30,'warmup_seconds':0.5,'initial_measurement_seconds':2,'longer_measurement_seconds':4,'conditions':'Shared host, no CPU isolation/cache flushing/governor changes. Tests/builds/QEMU/live probes/plotting finish outside the timed matrix. Five existing sparse CLI cases; six new stream CLI cases. Not blanket regression coverage.','commands':[]}
def run(variant,round_,phase,duration=2):
 group='cli_stream' if variant=='stream' else 'cli_sparse'
 key=f'r512-{phase}-{variant}-{round_}'
 cmd=['taskset','-c',str(cpu),str(binaries[variant]),'--bench','--sample-size','30','--warm-up-time','0.5','--measurement-time',str(duration),'--save-baseline',key]
 started=time.time()
 with (out/(key+'.txt')).open('w') as log:subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,check=True)
 files=sorted(Path('target/criterion',group).glob('*/'+key+'/sample.json'));assert len(files)==(6 if group=='cli_stream' else 5)
 for path in files:
  r=dict(benchmark=group+'/'+path.parent.parent.name,variant=variant,phase=phase,round=round_,**json.loads(path.read_text()))
  assert len(r['times'])==len(r['iters'])==30
  r['median_ns']=statistics.median(t/n for t,n in zip(r['times'],r['iters']));rows.append(r)
 env['commands'].append(dict(command=cmd,start_unix=started,end_unix=time.time()))
 (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n');(out/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n');print(key,'done',flush=True)
def pairs(phase):
 result=[]
 for r in rows:
  if r['variant']!='sparse-candidate' or r['phase']!=phase:continue
  b,=[v for v in rows if v['variant']=='sparse-before' and v['phase']==phase and v['round']==r['round'] and v['benchmark']==r['benchmark']]
  result.append(dict(benchmark=r['benchmark'],phase=phase,round=r['round'],change_percent=100*(r['median_ns']/b['median_ns']-1)))
 return result
for n in range(1,4):
 for variant in (['sparse-before','sparse-candidate'] if n%2 else ['sparse-candidate','sparse-before']):run(variant,n,'initial')
trigger=any(p['change_percent']>5 for p in pairs('initial'))
if trigger:
 print('Adverse >5% pair: repeat the five-case sparse CLI group with longer measurements.',flush=True)
 for n in range(1,4):
  for variant in (['sparse-candidate','sparse-before'] if n%2 else ['sparse-before','sparse-candidate']):run(variant,n,'longer',4)
for n in range(1,4):run('stream',n,'new')
(out/'summary.json').write_text(json.dumps(dict(initial_pairs=pairs('initial'),longer_triggered=trigger,longer_pairs=pairs('longer')),indent=2)+'\n')
print('matrix complete',flush=True)
