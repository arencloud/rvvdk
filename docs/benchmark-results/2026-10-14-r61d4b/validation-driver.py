from pathlib import Path
import sys,json,hashlib,copy,statistics as st,types,tempfile
sys.path.insert(0,'scripts/benchmarks')
import measure_recovery_cli as m
import measure_recovery_cli_supplement as extra
root=Path.cwd();r=root/'docs/benchmark-results/2026-10-14-r61d4b';private=root/'target/r61d4b/private'
# Frozen code/binary identities, including every native source file.
for name,h in m.load(private/'source-hashes.json').items():assert m.digest(root/name)==h,name
for name,h in m.load(private/'binaries.json').items():assert m.digest(root/'target/r61d4b'/name)==h,name
runs=m.load(r/'measurements.json');supp=m.load(r/'supplement-measurements.json')
assert len(runs)==252 and len(supp)==48
for index in [runs,supp]:
 groups={}
 for row in index:groups.setdefault(row['group'],[]).append((row['stage'],row['round'],row['pairs']))
 expected=sorted([(stage,n,pairs) for stage,pairs in [('initial',4),('repeat',8)] for n in [1,2,3]])
 assert all(sorted(rows)==expected for rows in groups.values())
m.summary(types.SimpleNamespace(report=r));extra.summarize(r)
assert m.load(r/'summary.json')['operations']==3024 and m.load(r/'supplement-summary.json')['operations']==864
assert [v['exit'] for v in m.load(r/'io-fault-cases.json')]==[1,3,3,3]
checks=[]
def reject(name,fn):
 try:fn()
 except (AssertionError,KeyError,ValueError):checks.append(name)
 else:raise AssertionError('accepted invalid copy: '+name)
row=next(v for v in runs if v['kind']=='recovery');original=m.load(r/row['file'])
for name,mutate in [('pair order',lambda a:a.reverse()),('negative metric',lambda a:a[0]['metrics'].update(cpu_ms=-1)),('nonfinite metric',lambda a:a[0]['metrics'].update(wall_ms=float('nan'))),('allocation',lambda a:a[0].update(allocated_bytes=1)),('oracle',lambda a:a[0].update(oracle_passed=False)),('native mismatch',lambda a:a[0]['report'].update(content_verified=False)),('missing metrics',lambda a:a[0].update(metrics={})),('both reports fail',lambda a:[v['report'].update(bundle_synced=False) for v in a])]:
 a=copy.deepcopy(original);mutate(a);reject(name,lambda:m.validate_rows(a,row['pairs'],row['fixture'],row['history']))
for kind in ['existing','profile']:
 run=next(v for v in runs if v['kind']==kind);a=m.load(r/run['file']);a[0]['pair']=999;reject(kind+' pairing',lambda:m.validate_aux_rows(a,run['pairs'],kind))
run=next(v for v in supp if v['kind']=='assessment');a=m.load(r/run['file']);a[0]['oracle_passed']=False;reject('assessment oracle',lambda:m.validate_aux_rows(a,run['pairs'],'assessment'))
with tempfile.TemporaryDirectory(dir=private) as tmp:
 p=Path(tmp);m.write(p/'measurements.json',[row]);(p/row['file']).write_text('[]\n');reject('sample digest mismatch',lambda:m.summary(types.SimpleNamespace(report=p)))
phases=[]
for run in runs:
 if run['kind']!='profile':continue
 for row in m.load(r/run['file']):
  if not row['recording']:continue
  phases.append({'file':run['file'],'pair':row['pair'],'filesystem':run['filesystem'],'stage':run['stage'],'whole_minus_phases':{k:row[k]-sum(p[k] for p in row['phases'].values()) for k in ['wall_ms','cpu_ms']},'zero_cpu_phases':[k for k,v in row['phases'].items() if v['cpu_ms']==0]})
m.write(r/'phase-accounting.json',{'interpretation':'Signed whole-minus-phase clock differences; profile start precedes benchmark start and extraction follows timing. No forced equality or epsilon.','rows':phases})
m.write(r/'validation.json',{'source_hashes':len(m.load(private/'source-hashes.json')),'binary_hashes':len(m.load(private/'binaries.json')),'main_rounds':len(runs),'supplement_rounds':len(supp),'operations':3888,'sample_hashes_and_native_gates':True,'negative_copy_checks':checks,'phase_recordings':len(phases)})
print('All final sample/code/binary identities and gates passed;',len(checks),'negative copy checks passed')
