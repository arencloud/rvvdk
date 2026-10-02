import hashlib,json,os,platform,statistics,subprocess,time
from pathlib import Path
out=Path('target/r510p/observations');runs=[];cpu=0
binaries={
 'original':Path('target/r510-reference/descriptor-baseline'),
 'before':Path('target/r510p/reference/descriptor-before'),
 'candidate':Path('target/r510p/reference/descriptor-candidate'),
 'stream_before':Path('target/r510p/reference/stream-before'),
 'stream_candidate':Path('target/r510p/reference/stream-candidate'),
}
assert cpu in os.sched_getaffinity(0)
env={'original_commit':'f756fb1','before_commit':'05b5fba','cpu_affinity':[cpu],
 'binary_sha256':{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()},
 'rustc':subprocess.check_output(['rustc','-Vv'],text=True).strip(),
 'kernel':platform.release(),'governor':Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor').read_text().strip(),
 'smt_siblings':Path('/sys/devices/system/cpu/cpu0/topology/thread_siblings_list').read_text().strip(),
 'conditions':'Shared host; same CPU as R5.10; no CPU isolation, cache flushing or governor changes. Compilation and QEMU outside timed matrix.',
 'sample_size':30,'warmup_seconds':0.5,'measurement_seconds':2,'order':[]}
(out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
def run(group,variant,round_,phase='initial',duration=2):
 key=f'r510p-{phase}-{group}-{variant}-{round_}'
 binary=binaries[('stream_' if group=='stream_admission' else '')+variant]
 command=['taskset','-c',str(cpu),str(binary),'--bench','--warm-up-time','0.5','--measurement-time',str(duration),'--sample-size','30','--save-baseline',key]
 started=time.time()
 with (out/(key+'.txt')).open('w') as log:
  subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,check=True)
 paths=sorted(Path('target/criterion',group).glob('*/'+key+'/sample.json'))
 assert len(paths)==(4 if group=='descriptor' else 7)
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
  for reference in (['original','before'] if row['benchmark'].startswith('descriptor/') else ['before']):
   base,=[x for x in runs if x['benchmark']==row['benchmark'] and x['round']==row['round'] and x['phase']==phase and x['variant']==reference]
   result.append({'benchmark':row['benchmark'],'reference':reference,'round':row['round'],'phase':phase,'change_percent':100*(row['median_ns']/base['median_ns']-1)})
 return result
for round_,order in enumerate([['original','before','candidate'],['candidate','before','original'],['before','original','candidate']],1):
 for variant in order:run('descriptor',variant,round_)
for round_ in range(1,4):
 for variant in (['before','candidate'] if round_%2 else ['candidate','before']):run('stream_admission',variant,round_)
pairs=comparisons('initial');trigger=any(row['change_percent']>5 for row in pairs)
if trigger:
 print('Adverse >5% observation: longer repeats for affected groups.',flush=True)
 groups=sorted({row['benchmark'].split('/')[0] for row in pairs if row['change_percent']>5})
 for group in groups:
  orders=([['candidate','before','original'],['original','before','candidate'],['candidate','original','before']] if group=='descriptor' else [['candidate','before'],['before','candidate'],['candidate','before']])
  for round_,order in enumerate(orders,1):
   for variant in order:run(group,variant,round_,'longer',4)
(out/'summary.json').write_text(json.dumps({'initial_comparisons':pairs,'longer_triggered':trigger,'longer_comparisons':comparisons('longer'),'scope':'same-harness old descriptor and new stream metadata costs; no disk/decompression timing'},indent=2)+'\n')
print('matrix complete',flush=True)
