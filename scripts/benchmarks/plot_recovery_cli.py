#!/usr/bin/env python3
"""Validate frozen local-recovery samples and export standalone PNG/SVG figures."""
import argparse,hashlib,json,pathlib,statistics as st,sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from measure_recovery_cli import load,write,summary

def main():
 p=argparse.ArgumentParser();p.add_argument('report',type=pathlib.Path);a=p.parse_args();a.report=a.report.resolve();groups=summary(a);out=a.report/'plots';out.mkdir(exist_ok=True)
 plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False,'svg.hashsalt':'rvddk-r61d4b'})
 generated=[]
 def save(fig,name):
  fig.tight_layout()
  for ext in ['png','svg']:
   path=out/(name+'.'+ext);fig.savefig(path,dpi=160,bbox_inches='tight',metadata={'Date':None} if ext=='svg' else {'Software':'rvddk benchmark plots'})
   if ext=='svg':path.write_text('\n'.join(line.rstrip() for line in path.read_text().splitlines())+'\n')
   generated.append(path)
  plt.close(fig)
 existing=[(k,g) for k,g in groups.items() if g['kind']=='existing']
 fig,axes=plt.subplots(1,2,figsize=(14,7),sharey=True)
 labels=[k.removeprefix('existing-') for k,g in existing];y=np.arange(len(labels))
 for ax,metric,title in zip(axes,['wall_ms','cpu_ms'],['Wall time','Process CPU']):
  changes=[100*(g['modes']['candidate'][metric]['median']/g['modes']['reference'][metric]['median']-1) for k,g in existing]
  ax.barh(y,changes,color=['#c16b31' if v>0 else '#238d91' for v in changes]);ax.axvline(0,color='#555',lw=0.8);ax.set_xlabel('Candidate/reference median change (%)');ax.set_title(title);ax.grid(axis='x',alpha=.2)
 axes[0].set_yticks(y,labels);fig.suptitle('Existing CLI commands — longer-repeat medians; positive means slower',y=1.02);save(fig,'existing-cli')
 recovery=[(k,g) for k,g in groups.items() if g['kind']=='recovery']
 for metric,title in [('wall_ms','Wall time'),('cpu_ms','CPU time')]:
  fig,axes=plt.subplots(2,2,figsize=(14,9))
  for ax,(fs,fixture) in zip(axes.flat,[(f,s) for f in ['btrfs','tmpfs'] for s in ['stored_64','deflated_256']]):
   selected=[(k,g) for k,g in recovery if k.startswith(f'recovery-{fs}-{fixture}-')];x=np.arange(len(selected))
   for mode,delta,color,label in [('direct',-.19,'#238d91','Direct API (test process)'),('cli',.19,'#d18b48','Whole CLI child')]:
    med=[g['modes'][mode][metric]['median'] for k,g in selected];rounds=[g['modes'][mode][metric]['round_medians'] for k,g in selected];ax.bar(x+delta,med,.36,label=label,color=color);ax.errorbar(x+delta,med,yerr=[[m-min(v) for m,v in zip(med,rounds)],[max(v)-m for m,v in zip(med,rounds)]],fmt='none',ecolor='#444',capsize=3)
   ax.set_xticks(x,[k.rsplit('-',1)[1] for k,g in selected],rotation=20);ax.set_ylabel('Milliseconds');ax.set_title(f'{fs} / {fixture}');ax.grid(axis='y',alpha=.2)
  axes.flat[0].legend();fig.suptitle(title+' — different measurement scopes; bars are medians, whiskers round ranges',y=1.02);save(fig,'recovery-'+metric)
 runs=load(a.report/'measurements.json');profile={}
 for r in runs:
  if r['kind']=='profile' and r['stage']=='repeat':profile.setdefault(r['filesystem'],[]).extend(load(a.report/r['file']))
 phases=['parse','relay_start','config','locks','operation','relay_join','report']
 fig,axes=plt.subplots(1,2,figsize=(13,5));colors=plt.get_cmap('tab10').colors
 for ax,include,title in [(axes[0],phases,'Full embedded command'),(axes[1],[p for p in phases if p!='operation'],'Adapter stages only (operation excluded)')]:
  bottom=np.zeros(2)
  for i,phase in enumerate(include):
   v=[st.median(r['phases'][phase]['wall_ms'] for r in profile[fs] if r['recording']) for fs in ['btrfs','tmpfs']];ax.bar(['btrfs','tmpfs'],v,bottom=bottom,label=phase,color=colors[phases.index(phase)]);bottom+=v
  ax.set_ylabel('Milliseconds');ax.set_title(title);ax.grid(axis='y',alpha=.2)
 handles,labels=axes[0].get_legend_handles_labels();axes[1].legend(handles,labels,loc='upper left',bbox_to_anchor=(1.02,1));fig.suptitle('Test-only adapter spans — stored 64 MiB Published attestation',y=1.02);save(fig,'adapter-phases')
 fig,axes=plt.subplots(1,2,figsize=(12,5));x=np.arange(2)
 for mode,delta,color in [('disabled',-.2,'#238d91'),('recorded',.2,'#d18b48')]:
  for ax,metric in zip(axes,['wall_ms','cpu_ms']):ax.bar(x+delta,[groups['profile-'+fs]['modes'][mode][metric]['median'] for fs in ['btrfs','tmpfs']],.38,label=mode,color=color);ax.set_xticks(x,['btrfs','tmpfs']);ax.set_ylabel('Milliseconds');ax.set_title(metric);ax.grid(axis='y',alpha=.2)
 axes[0].legend();fig.suptitle('Recording controls — identical verification; no causal overhead claim',y=1.02);save(fig,'recording-controls')
 fig,axes=plt.subplots(1,2,figsize=(14,6));labels=[k.removeprefix('recovery-') for k,g in recovery];x=np.arange(len(labels))
 axes[0].bar(x,[g['modes']['cli']['peak_rss_kib']['median']/1024 for k,g in recovery],color='#238d91');axes[0].set_ylabel('MiB');axes[0].set_title('CLI child lifetime peak RSS (wait4)');axes[0].set_xticks(x,labels,rotation=90)
 allocation=[]
 for k,g in recovery:
  r=next(r for r in runs if r['group']==k);allocation.append(load(a.report/r['file'])[0]['allocated_bytes']/(1<<20))
 axes[1].bar(x,allocation,color='#d18b48');axes[1].set_title('RAW allocation preserved across recovery and oracle');axes[1].set_ylabel('MiB');axes[1].set_xticks(x,labels,rotation=90);save(fig,'resources')
 from measure_recovery_cli_supplement import summarize as supplementary_summary
 supplement=supplementary_summary(a.report)
 fig,ax=plt.subplots(figsize=(14,6));labels=[];values=[]
 for k,g in (groups|supplement).items():
  for mode,m in g['modes'].items():
   s=m['wall_ms']['spread_percent']
   if s is not None and s>5:labels.append(k+' / '+mode);values.append(s)
 if values:ax.barh(np.arange(len(values)),values,color='#c16b31');ax.set_yticks(np.arange(len(values)),labels);fig.set_size_inches(14,max(6,len(values)*.24))
 ax.set_xlabel('Range of longer-round wall medians / minimum (%)');ax.set_title('Unresolved wall variation above 5% — observations retained');ax.axvline(5,color='#555',ls='--');save(fig,'variation')
 fig,axes=plt.subplots(1,2,figsize=(12,5))
 assessments=[(k,g) for k,g in supplement.items() if g['kind']=='assessment'];x=np.arange(len(assessments))
 for mode,delta,color in [('human',-.19,'#238d91'),('json',.19,'#d18b48')]:
  for ax,metric in zip(axes,['wall_ms','cpu_ms']):
   ax.bar(x+delta,[g['modes'][mode][metric]['median'] for k,g in assessments],.36,label=mode,color=color);ax.set_xticks(x,[k.removeprefix('assessment-') for k,g in assessments],rotation=20);ax.set_ylabel('Milliseconds');ax.set_title(metric)
 axes[0].legend();fig.suptitle('Observational assessment — whole process, no payload scan',y=1.02);save(fig,'assessment')
 fig,axes=plt.subplots(1,2,figsize=(13,6),sharey=True);labels=[];changes={'wall_ms':[],'cpu_ms':[]}
 for k,g in supplement.items():
  if g['kind']=='assessment':continue
  for mode,m in g['modes'].items():
   if not mode.startswith('candidate_'):continue
   reference=g['modes'][mode.replace('candidate_','reference_',1)];labels.append(g['kind']+' / '+mode.removeprefix('candidate_'))
   for metric in changes:changes[metric].append(100*(m[metric]['median']/reference[metric]['median']-1))
 y=np.arange(len(labels))
 for ax,metric in zip(axes,['wall_ms','cpu_ms']):ax.barh(y,changes[metric],color=['#c16b31' if v>0 else '#238d91' for v in changes[metric]]);ax.axvline(0,color='#555',lw=.8);ax.set_title(metric);ax.set_xlabel('Candidate/reference median change (%)')
 axes[0].set_yticks(y,labels);fig.suptitle('Frozen library controls — unchanged production source, Btrfs stored 64 MiB',y=1.02);save(fig,'library-controls')
 signals=load(a.report/'signal-cases.json');verification=load(a.report/'verification-signals.json')
 fig,axes=plt.subplots(1,2,figsize=(13,6));histories=list(dict.fromkeys(r['history'] for r in signals));labels=[h+' / '+s for h in histories for s in ['SIGINT','SIGTERM']]
 for i,label in enumerate(labels):
  h,s=label.split(' / ')
  for r in signals:
   if r['history']==h and r['signal']==s:axes[0].scatter(r['fsync_boundary'],i,color='#238d91' if r['exit'] else '#d18b48',marker='o' if r['exit'] else 's',s=55)
 axes[0].scatter([],[],color='#238d91',label='Cancellation observed');axes[0].scatter([],[],color='#d18b48',marker='s',label='Completed before token observation');axes[0].set_yticks(range(len(labels)),labels);axes[0].set_xlabel('Injected fsync boundary (ordinal)');axes[0].set_title('Release binary signal outcomes');axes[0].legend(loc='upper center',bbox_to_anchor=(.5,-.17),fontsize=8,ncol=2)
 rows=[r for r in verification if r['signal'] in ['SIGINT','SIGTERM']];axes[1].bar([r['signal'] for r in rows],[r['data']['logical_bytes_verified']/(1<<20) for r in rows],color='#238d91');axes[1].set_ylabel('MiB verified before cancellation');axes[1].set_title('Partial RAW verification, 256 MiB fixture');fig.suptitle('Actual signals — boundary observations, not cancellation latency guarantees',y=1.02);save(fig,'signals')
 extra=load(a.report/'supplement-measurements.json')
 inputs=[a.report/'measurements.json',a.report/'summary.json',a.report/'supplement-measurements.json',a.report/'supplement-summary.json',a.report/'signal-cases.json',a.report/'verification-signals.json']+[a.report/r['file'] for r in runs]
 for r in extra:
  inputs.extend([a.report/r['file']] if r['kind']=='assessment' else [a.report/v for v in r['files'].values()])
 helpers=['plot_recovery_cli.py','measure_recovery_cli.py','measure_recovery_cli_supplement.py','measure_acknowledgment.py','measure_attestation.py','measure_reconciliation.py']
 write(out/'audit.json',{'inputs':{str(p.relative_to(a.report)):hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs},'artifacts':{str(p.relative_to(a.report)):hashlib.sha256(p.read_bytes()).hexdigest() for p in generated},'generators':{name:hashlib.sha256(pathlib.Path(__file__).with_name(name).read_bytes()).hexdigest() for name in helpers},'matplotlib':matplotlib.__version__,'numpy':np.__version__,'python':sys.version})
 print('Validated samples and generated',len(generated)//2,'plots')
if __name__=='__main__':main()
