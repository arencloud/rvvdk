"""Bounded paired CLI controls, adverse repeats and prior/identical-binary checks."""
import hashlib,json,os,platform,statistics,subprocess,time
from pathlib import Path
out=Path('docs/benchmark-results/2026-10-02-r512p');ref=Path('target/r512p/reference');rows=[];cpu=0
selected={'sparse':['copy_mono_verify','copy_split_verify'],'vmdk':['copy_flat_verify','copy_mixed_verify'],'transfer':['threaded_new_verify'],'stream':['copy_dense','copy_dense_verify','copy_sparse_verify']}
binaries={f'{g}-{v}':ref/f'{g}-{v}' for g in selected for v in ['before','candidate']}
binaries.update({'sparse-historical':ref/'sparse-historical','sparse-identical_a':ref/'sparse-before','sparse-identical_b':ref/'sparse-before'})
assert cpu in os.sched_getaffinity(0)
def snapshot():
 return {'frequency_khz':Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq').read_text().strip(),'load':Path('/proc/loadavg').read_text().split()[:3],'cpu_ticks':[v for v in Path('/proc/stat').read_text().splitlines() if v.startswith(('cpu0 ','cpu8 '))]}
env=dict(baseline_commit='0ed0a09',historical_commit='d65b098',cpu_affinity=[cpu],smt_siblings=Path('/sys/devices/system/cpu/cpu0/topology/thread_siblings_list').read_text().strip(),governor=Path('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor').read_text().strip(),kernel=platform.release(),rustc=subprocess.check_output(['rustc','-Vv'],text=True).strip(),binary_sha256={k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()},conditions='Shared development host, tmpfs fixtures, CPU 0 affinity; no isolation, governor changes or cache flushing. Frequency/load/ticks sampled around each process, not continuous telemetry. No builds/tests/QEMU/live work/plots concurrent with matrix. Identical checks use exactly the same executable path.',commands=[])
def run(group,variant,round_,phase,cases,duration):
 key=f'r512p-{phase}-{group}-{variant}-{round_}'
 cmd=['taskset','-c',str(cpu),str(binaries[group+'-'+variant]),'--bench',f"cli_{group}/({'|'.join(cases)})$",'--sample-size','30','--warm-up-time','0.5','--measurement-time',str(duration),'--save-baseline',key]
 entry=dict(command=cmd,start_unix=time.time(),before=snapshot())
 with (out/(key+'.txt')).open('w') as log:subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT,check=True)
 entry.update(end_unix=time.time(),after=snapshot());env['commands'].append(entry)
 files=sorted(Path('target/criterion','cli_'+group).glob('*/'+key+'/sample.json'));assert len(files)==len(cases)
 for path in files:
  r=dict(benchmark='cli_'+group+'/'+path.parent.parent.name,variant=variant,phase=phase,round=round_,**json.loads(path.read_text()))
  assert len(r['times'])==len(r['iters'])==30
  r['median_ns']=statistics.median(t/n for t,n in zip(r['times'],r['iters']));rows.append(r)
 (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n');(out/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n');print(key,'done',flush=True)
def pairs(phase,before='before',candidate='candidate'):
 results=[]
 for r in rows:
  if r['phase']!=phase or r['variant']!=candidate:continue
  b,=[b for b in rows if b['phase']==phase and b['variant']==before and b['round']==r['round'] and b['benchmark']==r['benchmark']]
  results.append(dict(phase=phase,round=r['round'],benchmark=r['benchmark'],change_percent=100*(r['median_ns']/b['median_ns']-1)))
 return results
for n in range(1,4):
 for group,cases in selected.items():
  for v in (['before','candidate'] if n%2 else ['candidate','before']):run(group,v,n,'initial',cases,2)
adverse={r['benchmark'] for r in pairs('initial') if r['change_percent']>5}
print('longer cases',sorted(adverse),flush=True)
for n in range(1,4):
 for group,cases in selected.items():
  cases=[c for c in cases if 'cli_'+group+'/'+c in adverse]
  if cases:
   for v in (['candidate','before'] if n%2 else ['before','candidate']):run(group,v,n,'longer',cases,4)
for n in range(1,4):
 for v in (['historical','before'] if n%2 else ['before','historical']):run('sparse',v,n,'historical',['copy_mono_verify'],4)
 for v in (['identical_a','identical_b'] if n%2 else ['identical_b','identical_a']):run('sparse',v,n,'identical',['copy_mono_verify'],4)
summary=dict(initial=pairs('initial'),longer=pairs('longer'),adverse_cases=sorted(adverse),historical=pairs('historical','historical','before'),identical=pairs('identical','identical_a','identical_b'))
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print('matrix complete',flush=True)
