#!/usr/bin/env python3
"""Assessment baselines and frozen unchanged-library regression controls."""
import argparse,os,pathlib,shutil,statistics as st
from measure_recovery_cli import load,write,digest,invoke,command_measure,PREFIX,spread,validate_aux_rows
import measure_acknowledgment as native
TESTS={
 'normal':'ownership::retained::output::publication::tests::benchmark::matched_publication',
 'recovery':'ownership::retained::output::publication::reconciliation::tests::benchmark::recovery_branches',
 'attestation':'ownership::retained::output::publication::attestation::tests::benchmark::attestation_histories',
 'acknowledgment':'ownership::retained::output::publication::acknowledgment::tests::benchmark::acknowledgment_histories'}
def summarize(report):
 runs=load(report/'supplement-measurements.json');groups={};operations=0
 for r in runs:
  if r['kind']=='assessment':
   assert digest(report/r['file'])==r['sha256'];rows=load(report/r['file']);validate_aux_rows(rows,r['pairs'],'assessment');modes={}
   for row in rows:assert row['oracle_passed'];modes.setdefault(row['mode'],[]).append(row['metrics'])
   medians={m:{k:st.median(v[k] for v in values) for k in values[0]} for m,values in modes.items()};operations+=len(rows)
  else:
   if r['kind']=='acknowledgment':
    # The inherited acknowledgment validator names only the history. Keep the
    # frozen binaries separate instead of pooling written/synced observations.
    medians={}
    for binary,filename in r['files'].items():
     _,values=native.load_run(report,dict(r,files={binary:filename}))
     medians.update({binary+'_'+mode:metrics for mode,metrics in values.items()})
   else:_,medians=native.load_run(report,r)
   assert len(medians)==4 and all(m.startswith(('reference_','candidate_')) for m in medians)
   operations+=4*r['pairs']
  g=groups.setdefault(r['group'],{'kind':r['kind'],'initial':{},'repeat':{}})
  for mode,metrics in medians.items():
   m=g[r['stage']].setdefault(mode,{})
   for k,v in metrics.items():m.setdefault(k,[]).append(v)
 out={name:{'kind':g['kind'],'modes':{m:{k:{'median':st.median(v),'round_medians':v,'spread_percent':spread(v)} for k,v in values.items()} for m,values in g['repeat'].items()}} for name,g in groups.items()}
 write(report/'supplement-summary.json',{'operations':operations,'groups':out});return out

def main():
 p=argparse.ArgumentParser()
 for k in ['reference','candidate','cli','report','private','btrfs-parent','tmpfs-parent']:p.add_argument('--'+k,type=pathlib.Path,required=True)
 a=p.parse_args()
 for k,v in vars(a).items():setattr(a,k,v.resolve())
 assert not (a.report/'supplement-measurements.json').exists();runs=[];os.sched_setaffinity(0,{min(os.sched_getaffinity(0))})
 def save(r):runs.append(r);write(a.report/'supplement-measurements.json',runs)
 for fs,parent in [('btrfs',a.btrfs_parent),('tmpfs',a.tmpfs_parent)]:
  for history in ['published','written']:
   handoff=a.private/f'assessment-{fs}-{history}.handoff';invoke(a.candidate,PREFIX+'prepare_fixture',os.environ|{'RVVDK_RETAINED_PARENT':str(parent),'RVDDK_CLI_HISTORY':history,'RVDDK_CLI_GRAINS':'1024','RVDDK_CLI_HANDOFF':str(handoff)},a.private/f'assessment-{fs}-{history}-prepare.txt');fixture=pathlib.Path(handoff.read_text());assert fixture.parent==parent and fixture.name.startswith('rvddk-retained-')
   before={str(f.relative_to(fixture)):(digest(f),f.stat().st_ino,f.stat().st_size,f.stat().st_blocks) for f in fixture.rglob('*') if f.is_file()}
   try:
    for stage,pairs in [('initial',4),('repeat',8)]:
     for round_ in range(1,4):
      group=f'assessment-{fs}-{history}';stem=f'{group}-{stage}-{round_}';rows=[]
      for pair in range(pairs):
       for mode in (['human','json'] if pair%2==0 else ['json','human']):
        args=['recover','assess',str(fixture/'recover.json')]+(['--json'] if mode=='json' else []);metrics=command_measure(a.cli,args,a.private);text=(a.private/'command.stdout').read_text();import json;data=json.loads(text[text.index('{'):]);assert data['status']=='assessed' and data['operation_succeeded']
        assert data['data']['publication']['output']['pending_transaction']==(history=='written');rows.append({'pair':pair,'mode':mode,'metrics':metrics,'oracle_passed':True})
      write(a.report/(stem+'.json'),rows);save({'kind':'assessment','group':group,'filesystem':fs,'history':history,'stage':stage,'round':round_,'pairs':pairs,'file':stem+'.json','sha256':digest(a.report/(stem+'.json'))});print(stem,flush=True)
    after={str(f.relative_to(fixture)):(digest(f),f.stat().st_ino,f.stat().st_size,f.stat().st_blocks) for f in fixture.rglob('*') if f.is_file()};assert after==before
   finally:shutil.rmtree(fixture)
 for kind,test in TESTS.items():
  for stage,pairs in [('initial',4),('repeat',8)]:
   for round_ in range(1,4):
    group='library-btrfs-stored_64-'+kind;stem=f'{group}-{stage}-{round_}';files={};hashes={}
    for binary in (['reference','candidate'] if round_%2 else ['candidate','reference']):
     filename=stem+'-'+binary+'.json';env=os.environ|{'RVVDK_RETAINED_PARENT':str(a.btrfs_parent),'RVVDK_RETAINED_PAIRS':str(pairs),'RVVDK_PROFILE_FIXTURE':'stored_64','RVVDK_RETAINED_REPORT':str(a.report/filename)}
     invoke(getattr(a,binary),test,env,a.report/(stem+'-'+binary+'.txt'));files[binary]=filename;hashes[filename]=digest(a.report/filename)
    r={'kind':kind,'group':group,'filesystem':'btrfs','fixture':'stored_64','stage':stage,'round':round_,'pairs':pairs,'files':files,'sha256':hashes};native.load_run(a.report,r);save(r);print(stem,flush=True)
 summarize(a.report)
 assert not list(a.btrfs_parent.iterdir()) and not list(a.tmpfs_parent.iterdir())
 print('Supplement complete and fixture teardown verified',flush=True)
if __name__=='__main__':main()
