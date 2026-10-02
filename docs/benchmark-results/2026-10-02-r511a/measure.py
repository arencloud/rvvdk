import hashlib,json,os,platform,statistics,subprocess,time
from pathlib import Path
out=Path('target/r511a/observations');runs=[];cpu=0
binaries={
 'before':Path('target/r511a/reference/stream-before'),
 'candidate':Path('target/r511a/reference/stream-candidate'),
 'map':Path('target/r511a/reference/stream-map'),
}
assert cpu in os.sched_getaffinity(0)
env={'before_commit':'64d81a7','cpu_affinity':[cpu],
 'binary_sha256':{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()},
 'rustc':subprocess.check_output(['rustc','-Vv'],text=True).strip(),
 'kernel':platform.release(),'governor':Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor').read_text().strip(),
 'smt_siblings':Path('/sys/devices/system/cpu/cpu0/topology/thread_siblings_list').read_text().strip(),
 'conditions':'Shared host; same CPU as R5.10; no CPU isolation, cache flushing or governor changes. Compilation, tests, QEMU, live probes and plotting outside timed matrix.',
 'sample_size':30,'warmup_seconds':0.5,'measurement_seconds':2,'order':[]}
(out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
def run(group,variant,round_,phase='initial',duration=2):
 key=f'r511a-{phase}-{group}-{variant}-{round_}'
 binary=binaries[variant]
 command=['taskset','-c',str(cpu),str(binary),'--bench','--warm-up-time','0.5','--measurement-time',str(duration),'--sample-size','30','--save-baseline',key]
 started=time.time()
 with (out/(key+'.txt')).open('w') as log:
  subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,check=True)
 paths=sorted(Path('target/criterion',group).glob('*/'+key+'/sample.json'))
 assert len(paths)==(9 if group=='stream_map' else 7)
 for path in paths:
  sample=json.loads(path.read_text());row={'benchmark':group+'/'+path.parent.parent.name,'variant':variant,'round':round_,'phase':phase,**sample}
  assert len(row['times'])==len(row['iters'])==30
  row['median_ns']=statistics.median(t/n for t,n in zip(row['times'],row['iters']));runs.append(row)
 env['order'].append({'key':key,'command':command,'start_unix':started,'end_unix':time.time()})
 (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
 (out/'measurements.json').write_text(json.dumps(runs,indent=2)+'\n')
 print(key,'done',flush=True)
def comparisons(phase):
 result=[]
 for row in runs:
  if row['variant']!='candidate' or row['phase']!=phase:continue
  for reference in ['before']:
   base,=[x for x in runs if x['benchmark']==row['benchmark'] and x['round']==row['round'] and x['phase']==phase and x['variant']==reference]
   result.append({'benchmark':row['benchmark'],'reference':reference,'round':row['round'],'phase':phase,'change_percent':100*(row['median_ns']/base['median_ns']-1)})
 return result
for round_ in range(1,4):
 for variant in (['before','candidate'] if round_%2 else ['candidate','before']):run('stream_admission',variant,round_)
pairs=comparisons('initial');trigger=any(row['change_percent']>5 for row in pairs)
if trigger:
 print('Adverse >5% observation: longer repeats of the stream group.',flush=True)
 for round_ in range(1,4):
  for variant in (['candidate','before'] if round_%2 else ['before','candidate']):run('stream_admission',variant,round_,'longer',4)
for round_ in range(1,4):run('stream_map','map',round_,'new')
(out/'summary.json').write_text(json.dumps({'initial_comparisons':pairs,'longer_triggered':trigger,'longer_comparisons':comparisons('longer'),'scope':'In-memory stream admission regression pairs and new grain-map costs; no disk/decompression timing'},indent=2)+'\n')
print('matrix complete',flush=True)
