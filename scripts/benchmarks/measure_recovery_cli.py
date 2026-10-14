#!/usr/bin/env python3
"""Serial authored CLI/native recovery comparisons; never accepts live credentials."""
import argparse,hashlib,json,math,os,pathlib,shutil,statistics as st,subprocess,tempfile,time
PREFIX='ownership::retained::output::publication::tests::cli::'
HISTORIES=['staged','renamed','published','uncertain','written','synced']
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,v):p.write_text(json.dumps(v,indent=2,allow_nan=False)+'\n')
def load(p):return json.loads(p.read_text())
def spread(v):return None if min(v)==0 else 100*(max(v)/min(v)-1)
def invoke(binary,test,env,log):
 with log.open('w') as f:subprocess.run([binary,'--exact',test,'--ignored','--test-threads=1'],env=env,stdout=f,stderr=subprocess.STDOUT,check=True,timeout=900)
def prepare(a,parent,label):
 handoff=a.private/(label+'.handoff');invoke(a.library,PREFIX+'prepare_fixture',os.environ|{'RVVDK_RETAINED_PARENT':str(parent),'RVDDK_CLI_HISTORY':'published','RVDDK_CLI_GRAINS':'1024','RVDDK_CLI_HANDOFF':str(handoff)},a.private/(label+'-prepare.txt'))
 p=pathlib.Path(handoff.read_text());assert p.parent==parent and p.name.startswith('rvddk-retained-');return p

def validate_rows(rows,pairs,fixture,history):
 assert len(rows)==2*pairs
 for pair in range(pairs):
  group=rows[2*pair:2*pair+2];assert [r['pair'] for r in group]==[pair,pair]
  assert [r['cli'] for r in group]==([False,True] if pair%2==0 else [True,False])
  assert group[0]['report']==group[1]['report']
  for r in group:
   assert r['fixture']==fixture and r['history']==history and r['oracle_passed'] and r['allocation_unchanged']
   assert r['logical_bytes']==(64<<20 if fixture=='stored_64' else 256<<20)
   assert r['allocated_bytes']==8<<20
   assert r['report']['logical_bytes_verified']==r['logical_bytes'] and r['report']['primary_error'] is None and r['report']['assessment_error'] is None
   assert r['report']['recovery']['output']=={'state':'published','sequence':8,'pending_transaction':False}
   assert r['report']['recovery']['published']=='owned' and r['report']['recovery']['staged']=='absent'
   assert set(r['metrics'])=={'wall_ms','cpu_ms','peak_rss_kib'}
   flags=['content_verified','bundle_synced','directories_synced']
   if history in ['staged','renamed']:
    flags+=['intent_synced','acknowledgment_durable']
    assert r['report']['origin']==('staged' if history=='staged' else 'already_renamed')
    assert r['report']['rename_observed']==(history=='staged')
   elif history in ['published','uncertain']:flags+=['published_record_synced','bindings_rechecked']
   else:flags+=['intent_synced','transaction_synced','transaction_promoted','acknowledgment_durable','bindings_rechecked']
   assert all(r['report'][k] is True for k in flags)
   for k,v in r['metrics'].items():assert math.isfinite(v) and v>0,k

def validate_aux_rows(rows,pairs,kind):
 assert kind in ['profile','existing','assessment'] and len(rows)==2*pairs
 field,order={'profile':('recording',[False,True]),'existing':('binary',['reference','candidate']),'assessment':('mode',['human','json'])}[kind]
 for pair in range(pairs):
  group=rows[2*pair:2*pair+2]
  assert [r['pair'] for r in group]==[pair,pair]
  assert [r[field] for r in group]==(order if pair%2==0 else order[::-1])
 for row in rows:
  if kind=='profile':
   assert set(row['phases'])==({'parse','relay_start','config','locks','operation','relay_join','report'} if row['recording'] else set())
   for phase in row['phases'].values():
    assert set(phase)=={'wall_ms','cpu_ms'}
    assert all(math.isfinite(v) and v>=0 for v in phase.values())
   metrics={k:row[k] for k in ['wall_ms','cpu_ms','peak_rss_kib']}
  else:
   assert row['oracle_passed'] is True
   metrics=row['metrics']
   assert set(metrics)=={'wall_ms','cpu_ms','peak_rss_kib','output_bytes'}
  assert all(math.isfinite(v) and v>=0 for v in metrics.values())
  assert metrics['wall_ms']>0 and metrics['peak_rss_kib']>0

def command_measure(binary,args,private):
 # Capture into regular private files to avoid pipe backpressure; wait4 gives
 # per-child CPU/RSS. Parent wait wall includes process launch and report flush.
 with (private/'command.stdout').open('wb') as out,(private/'command.stderr').open('wb') as err:
  start=time.perf_counter();p=subprocess.Popen([binary]+args,stdout=out,stderr=err)
  _,status,usage=os.wait4(p.pid,0);p.returncode=os.waitstatus_to_exitcode(status);wall=(time.perf_counter()-start)*1000
 assert p.returncode==0
 assert not (private/'command.stderr').read_bytes()
 return {'wall_ms':wall,'cpu_ms':1000*(usage.ru_utime+usage.ru_stime),'peak_rss_kib':usage.ru_maxrss,'output_bytes':(private/'command.stdout').stat().st_size}

def summary(a):
 runs=load(a.report/'measurements.json');groups={}
 for run in runs:
  assert digest(a.report/run['file'])==run['sha256']
  rows=load(a.report/run['file'])
  if run['kind']=='recovery':validate_rows(rows,run['pairs'],run['fixture'],run['history'])
  else:validate_aux_rows(rows,run['pairs'],run['kind'])
  group=groups.setdefault(run['group'],{'kind':run['kind'],'initial':{},'repeat':{}})
  modes={}
  for row in rows:
   if run['kind']=='recovery':mode='cli' if row['cli'] else 'direct';metrics=row['metrics']
   elif run['kind']=='profile':mode='recorded' if row['recording'] else 'disabled';metrics={k:row[k] for k in ['wall_ms','cpu_ms','peak_rss_kib']}
   else:mode=row['binary'];metrics=row['metrics']
   for k,v in metrics.items():assert math.isfinite(v) and v>=0,(run['file'],k,v)
   modes.setdefault(mode,[]).append(metrics)
  for mode,values in modes.items():
   entry=group[run['stage']].setdefault(mode,{})
   for k in values[0]:entry.setdefault(k,[]).append(st.median(v[k] for v in values))
 result={}
 for name,g in groups.items():
  result[name]={'kind':g['kind'],'modes':{mode:{metric:{'round_medians':v,'median':st.median(v),'spread_percent':spread(v)} for metric,v in metrics.items()} for mode,metrics in g['repeat'].items()}}
 write(a.report/'summary.json',{'operations':sum(2*r['pairs'] for r in runs),'groups':result})
 return result

def main():
 p=argparse.ArgumentParser();
 for key in ['reference','candidate','library','profile-tests','private','report','btrfs-parent','tmpfs-parent']:p.add_argument('--'+key,type=pathlib.Path,required=True)
 p.add_argument('--resume',action='store_true')
 a=p.parse_args();
 for k,v in vars(a).items():
  if isinstance(v,pathlib.Path):setattr(a,k,v.resolve())
 assert a.resume or not (a.report/'measurements.json').exists(),'refuse overwriting a campaign'
 a.report.mkdir(parents=True,exist_ok=True);runs=load(a.report/'measurements.json') if a.resume else [];decisions=load(a.report/'repeat-decisions.json') if a.resume and (a.report/'repeat-decisions.json').exists() else []
 for run in runs:assert digest(a.report/run['file'])==run['sha256']
 affinity=sorted(os.sched_getaffinity(0));os.sched_setaffinity(0,{affinity[0]})
 write(a.report/('environment-resumed.json' if a.resume else 'environment.json'),{'affinity':[affinity[0]],'available_affinity':affinity,'uname':list(os.uname()),'cache_policy':'normal page cache; no cold-cache claim','cpu':subprocess.check_output(['lscpu'],text=True),'filesystems':subprocess.check_output(['stat','-f','-c','%T',a.btrfs_parent,a.tmpfs_parent],text=True).splitlines(),'mount_options':{'tmpfs':'size=512m,mode=0700,huge=never; private mount namespace'},'timed_workloads':'serial; no concurrent tests/builds/audits/plots'})
 def save(run,rows):
  prior=next((r for r in runs if r['file']==run['file']),None)
  if prior:
   assert rows==load(a.report/run['file']);return
  if (a.report/run['file']).exists():assert rows==load(a.report/run['file'])
  else:write(a.report/run['file'],rows)
  run['sha256']=digest(a.report/run['file']);runs.append(run);write(a.report/'measurements.json',runs)
 for fs,parent in [('btrfs',a.btrfs_parent),('tmpfs',a.tmpfs_parent)]:
  for fixture in ['stored_64','deflated_256']:
   for history in HISTORIES:
    group=f'recovery-{fs}-{fixture}-{history}';initial=[]
    for stage,pairs in [('initial',4),('repeat',8)]:
     for round_ in range(1,4):
      stem=f'{group}-{stage}-{round_}';file=stem+'.json';env=os.environ|{'RVDDK_RECOVERY_CLI':str(a.candidate),'RVVDK_RETAINED_PARENT':str(parent),'RVDDK_CLI_PAIRS':str(pairs),'RVDDK_CLI_FIXTURE':fixture,'RVDDK_CLI_HISTORY':history,'RVDDK_CLI_REPORT':str(a.report/file)}
      if not (a.report/file).exists():invoke(a.library,PREFIX+'command_benchmark',env,a.report/(stem+'.txt'))
      rows=load(a.report/file);validate_rows(rows,pairs,fixture,history)
      run={'kind':'recovery','group':group,'filesystem':fs,'fixture':fixture,'history':history,'stage':stage,'round':round_,'pairs':pairs,'file':file};save(run,rows)
      if stage=='initial':initial.append(rows)
      print(stem,'operations',2*pairs,flush=True)
     if stage=='initial':
      reasons=[]
      for cli in [False,True]:
       for metric in ['wall_ms','cpu_ms']:
        v=[st.median(r['metrics'][metric] for r in rows if r['cli']==cli) for rows in initial]
        if spread(v)>5:reasons.append(f'{cli}:{metric}:spread>5%')
      # The whole CLI vs direct API is a new adapter-cost observation, never an
      # implementation speedup/regression. Longer repeats characterize that cost.
      reasons.append('predeclared longer characterization of new whole-command/direct-API costs')
      decisions.append({'group':group,'repeat':True,'reasons':reasons});write(a.report/'repeat-decisions.json',decisions)
  # Repeated Published attestation permits matched recording controls on one
  # fixture; no record is rewritten and independent work was qualified above.
  fixture=prepare(a,parent,'profile-'+fs)
  try:
   for stage,pairs in [('initial',4),('repeat',8)]:
    for round_ in range(1,4):
     group='profile-'+fs;stem=f'{group}-{stage}-{round_}';file=stem+'.json'
     if not (a.report/file).exists():invoke(a.profile_tests,'recovery::tests::adapter_profile',os.environ|{'RVDDK_CLI_CONFIG':str(fixture/'recover.json'),'RVDDK_CLI_PAIRS':str(pairs),'RVDDK_CLI_REPORT':str(a.report/file)},a.report/(stem+'.txt'))
     rows=load(a.report/file);assert len(rows)==2*pairs
     for r in rows:
      assert set(r['phases'])==({'parse','relay_start','config','locks','operation','relay_join','report'} if r['recording'] else set())
      for phase in r['phases'].values():assert all(math.isfinite(v) and v>=0 for v in phase.values())
     save({'kind':'profile','group':group,'filesystem':fs,'stage':stage,'round':round_,'pairs':pairs,'file':file},rows)
     print(stem,flush=True)
  finally:shutil.rmtree(fixture)
  # Existing CLI regressions use one authored 64 MiB RAW and equivalent FLAT.
  with tempfile.TemporaryDirectory(prefix='cli-existing-',dir=parent) as tmp:
   d=pathlib.Path(tmp);source=d/'source.raw';dest=d/'copy.raw';descriptor=d/'disk.vmdk'
   with source.open('wb') as f:f.truncate(64<<20);f.seek(0);f.write(bytes(range(256))*(32768));f.flush();os.fsync(f.fileno())
   descriptor.write_text('version=1\nCID=12345678\nparentCID=ffffffff\ncreateType="monolithicFlat"\nRW 131072 FLAT "source.raw" 0\n')
   verify_target=d/'verify.raw';shutil.copyfile(source,verify_target)
   source_hash=digest(source);assert digest(verify_target)==source_hash
   cases=[('help',None),('inspect','raw'),('plan','raw'),('copy','raw'),('verify','raw'),('inspect','vmdk'),('copy','vmdk'),('verify','vmdk')]
   for command,fmt in cases:
    group=f'existing-{fs}-{command}-{fmt or "text"}'
    for stage,pairs in [('initial',4),('repeat',8)]:
     for round_ in range(1,4):
      stem=f'{group}-{stage}-{round_}'
      if (a.report/(stem+'.json')).exists():continue
      rows=[]
      for pair in range(pairs):
       for binary in (['reference','candidate'] if pair%2==0 else ['candidate','reference']):
        args=['--help'] if command=='help' else [command,str(source if fmt=='raw' else descriptor)]
        if command in ['copy','plan']:args.append(str(dest))
        if command=='verify':args.append(str(verify_target))
        if fmt:args+=['--format',fmt,'--json']
        if command=='copy':args+=['--verify']
        metrics=command_measure(getattr(a,binary),args,a.private)
        if command=='copy':assert digest(dest)==source_hash;dest.unlink()
        rows.append({'pair':pair,'binary':binary,'metrics':metrics,'oracle_passed':True})
      stem=f'{group}-{stage}-{round_}';save({'kind':'existing','group':group,'filesystem':fs,'stage':stage,'round':round_,'pairs':pairs,'file':stem+'.json'},rows);print(stem,flush=True)
    decisions.append({'group':group,'repeat':True,'reasons':['predeclared longer characterization of linkage/startup and existing-command variability']});write(a.report/'repeat-decisions.json',decisions)
  assert not list(parent.iterdir()),'fixture teardown incomplete'
 summary(a);print('Campaign completed and fixture teardown verified',flush=True)
if __name__=='__main__':main()
