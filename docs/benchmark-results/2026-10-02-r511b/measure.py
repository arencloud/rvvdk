"""Paired map controls and native-read baselines; preserve adverse repeats."""
import hashlib,json,os,platform,statistics,subprocess,time
from pathlib import Path
out=Path('target/r511b/observations');cpu=0;rows=[]
binaries={v:Path('target/r511b/reference/'+v) for v in ['map-before','map-candidate','reads']}
assert cpu in os.sched_getaffinity(0)
env={'baseline_commit':'ea94b1d','binary_sha256':{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()},'cpu_affinity':[cpu],'smt_siblings':Path('/sys/devices/system/cpu/cpu0/topology/thread_siblings_list').read_text().strip(),'governor':Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor').read_text().strip(),'rustc':subprocess.check_output(['rustc','-Vv'],text=True).strip(),'kernel':platform.release(),'sample_size':30,'warmup_seconds':0.5,'initial_measurement_seconds':2,'longer_measurement_seconds':4,'conditions':'Shared host, no CPU isolation/cache flushing/governor changes. Tests/builds/QEMU/live probes/plotting finish outside the timed matrix. Four existing map cases; seven new native read cases. Not blanket regression coverage.','commands':[]}
def run(variant,round_,phase,duration=2):
 group='stream_reads' if variant=='reads' else 'stream_map'
 key=f'r511b-{phase}-{variant}-{round_}'
 cmd=['taskset','-c',str(cpu),str(binaries[variant]),'--bench','--sample-size','30','--warm-up-time','0.5','--measurement-time',str(duration),'--save-baseline',key]
 if group=='stream_map':cmd+=['stream_map/(sparse_footer|dense_footer|empty_1t_footer|lookup_sparse)$']
 started=time.time()
 with (out/(key+'.txt')).open('w') as log:subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,check=True)
 files=sorted(Path('target/criterion',group).glob('*/'+key+'/sample.json'));assert len(files)==(7 if group=='stream_reads' else 4)
 for path in files:
  r=dict(benchmark=group+'/'+path.parent.parent.name,variant=variant,phase=phase,round=round_,**json.loads(path.read_text()))
  assert len(r['times'])==len(r['iters'])==30
  r['median_ns']=statistics.median(t/n for t,n in zip(r['times'],r['iters']));rows.append(r)
 env['commands'].append(dict(command=cmd,start_unix=started,end_unix=time.time()))
 (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n');(out/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n');print(key,'done',flush=True)
def pairs(phase):
 result=[]
 for r in rows:
  if r['variant']!='map-candidate' or r['phase']!=phase:continue
  b,=[v for v in rows if v['variant']=='map-before' and v['phase']==phase and v['round']==r['round'] and v['benchmark']==r['benchmark']]
  result.append(dict(benchmark=r['benchmark'],phase=phase,round=r['round'],change_percent=100*(r['median_ns']/b['median_ns']-1)))
 return result
for n in range(1,4):
 for variant in (['map-before','map-candidate'] if n%2 else ['map-candidate','map-before']):run(variant,n,'initial')
trigger=any(p['change_percent']>5 for p in pairs('initial'))
if trigger:
 print('Adverse >5% pair: repeat the four-case map group with longer measurements.',flush=True)
 for n in range(1,4):
  for variant in (['map-candidate','map-before'] if n%2 else ['map-before','map-candidate']):run(variant,n,'longer',4)
for n in range(1,4):run('reads',n,'new')
(out/'summary.json').write_text(json.dumps(dict(initial_pairs=pairs('initial'),longer_triggered=trigger,longer_pairs=pairs('longer')),indent=2)+'\n')
print('matrix complete',flush=True)
