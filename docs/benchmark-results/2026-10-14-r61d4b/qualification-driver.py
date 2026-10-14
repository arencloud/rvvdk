import hashlib,json,os,pathlib,re,shutil,signal,subprocess,sys
root=pathlib.Path.cwd();base=root/'target/r61d4b';private=base/'private';report=root/'docs/benchmark-results/2026-10-14-r61d4b';lib=base/'candidate-tests';cli=base/'candidate-cli'
prefix='ownership::retained::output::publication::tests::cli::'
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def prepare(mode,parent,label,grains=32):
 handoff=private/(label+'.handoff');env=os.environ|{'RVVDK_RETAINED_PARENT':str(parent),'RVDDK_CLI_HISTORY':mode,'RVDDK_CLI_GRAINS':str(grains),'RVDDK_CLI_HANDOFF':str(handoff)}
 with (private/(label+'-prepare.txt')).open('w') as f:subprocess.run([lib,'--exact',prefix+'prepare_fixture','--ignored','--test-threads=1'],env=env,stdout=f,stderr=subprocess.STDOUT,check=True,timeout=60)
 p=pathlib.Path(handoff.read_text());assert p.parent==parent and p.name.startswith('rvddk-retained-');return p
def call(p,verb,label,inject=None,full=False):
 trace=private/(label+'.strace');args=['strace','-f','-yy','-o',str(trace),'-e','trace=%file,%network,read,write,writev,pwrite64,fsync,fdatasync,ftruncate,fallocate']
 if inject:args+=['-e','inject='+inject]
 args+=[str(cli),'recover',verb,str(p/'recover.json'),'--json']
 with (open('/dev/full','wb') if full else (private/(label+'.stdout')).open('wb')) as out,(private/(label+'.stderr')).open('wb') as err:
  proc=subprocess.Popen(args,stdout=out,stderr=err,start_new_session=True)
  try:code=proc.wait(timeout=90)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait();raise
 data=None
 if not inject or 'SIGKILL' not in inject:
  data=json.loads((private/(label+('.stdout' if code==0 else '.stderr'))).read_text())
 return code,data,trace.read_text()
def snapshot(p):return {str(f.relative_to(p)):(hashlib.sha256(f.read_bytes()).hexdigest(),f.stat().st_ino,f.stat().st_size,f.stat().st_blocks) for f in p.rglob('*') if f.is_file()}
results=[];signals=[];faults=[];kills=[];verification=[];assessments=[]
parent=pathlib.Path(sys.argv[1]).resolve()
for mode,verb,syncs,renames in [('written','assess',0,0),('staged','reconcile-intent',10,2),('renamed','reconcile-intent',10,1),('published','attest-published',8,0),('written','acknowledge-published',10,1),('synced','acknowledge-published',10,1)]:
 label='audit-'+mode+'-'+verb;p=prepare(mode,parent,label);before=snapshot(p);code,data,trace=call(p,verb,label);assert code==0,(label,code,data)
 calls=[l for l in trace.splitlines() if re.search(r'\bfsync\(',l)];rs=[l for l in trace.splitlines() if re.search(r'\brename(?:at2|at)?\(',l)];network=[l for l in trace.splitlines() if re.search(r'\b(?:socket|connect|sendto|recvfrom)\(',l)]
 assert len(calls)==syncs and all(l.rstrip().endswith('= 0') for l in calls),(label,len(calls),syncs)
 assert len(rs)==renames,(label,len(rs),renames);assert not network
 if verb=='assess':assert snapshot(p)==before;assert not re.search(r'read\([^\n]*(?:disk.raw|disk-1.vmdk)',trace)
 if verb in ['assess','attest-published','acknowledge-published']:
  mutations=[l for l in trace.splitlines() if re.search(r'\b(?:write|writev|pwrite64|unlink|unlinkat|ftruncate|fallocate|mkdir|mkdirat)\(',l) and str(p) in l];assert not mutations,(label,mutations)
 if verb=='attest-published':assert snapshot(p)==before
 results.append({'history':mode,'verb':verb,'fsync_calls':len(calls),'rename_calls':len(rs),'network_calls':0,'trace_sha256':hashlib.sha256(trace.encode()).hexdigest(),'exit':code,'success':data['operation_succeeded']});shutil.rmtree(p)
for mode,verb,points in [('written','acknowledge-published',[1,5,6,7,9,10]),('published','attest-published',[4,7,8]),('staged','reconcile-intent',[5,6,7,8])]:
 for point in points:
  for sig,want in [('SIGINT',130),('SIGTERM',143)]:
   label=f'signal-{mode}-{point}-{sig}';p=prepare(mode,parent,label);code,data,trace=call(p,verb,label,f'fsync:signal={sig}:delay_exit=50ms:when={point}');assert code in [0,want],(label,code,data)
   assert data['status']==('completed' if code==0 else 'cancelled') and data['operation_succeeded']==(code==0);signals.append({'history':mode,'signal':sig,'fsync_boundary':point,'exit':code,'data':data['data']});shutil.rmtree(p)
for mode,verb,point,want in [('written','acknowledge-published',1,1),('written','acknowledge-published',10,3),('published','attest-published',8,3),('staged','reconcile-intent',10,3)]:
 label=f'fault-{mode}-{point}';p=prepare(mode,parent,label);code,data,trace=call(p,verb,label,f'fsync:error=EIO:when={point}');assert code in [0,want],(label,code,data);faults.append({'history':mode,'fsync_boundary':point,'exit':code,'data':data['data']});shutil.rmtree(p)
for point,following in [(1,'acknowledge-published'),(10,'attest-published')]:
 label=f'kill-{point}';p=prepare('written',parent,label);code,data,trace=call(p,'acknowledge-published',label,f'fsync:signal=SIGKILL:when={point}');assert code!=0 and 'SIGKILL' in trace
 out=subprocess.run([cli,'recover','assess',p/'recover.json','--json'],capture_output=True,check=True,timeout=30);state=json.loads(out.stdout)['data']['publication']['output'];assert state['state']==('publish_intent' if point==1 else 'published'),state
 out=subprocess.run([cli,'recover',following,p/'recover.json','--json'],capture_output=True,check=True,timeout=30);assert json.loads(out.stdout)['operation_succeeded'];kills.append({'fsync_boundary':point,'state_after_kill':state,'explicit_following_operation':following,'following_success':True});shutil.rmtree(p)
p=prepare('written',parent,'delivery');code,data,trace=call(p,'acknowledge-published','delivery',full=True);assert code==1 and data['status']=='output_failed' and data['operation_succeeded'];delivery=data;shutil.rmtree(p)
# Independent source/output assessment and true long-verification signals.
for fault in ['source_transaction','missing_output']:
 p=prepare('published',parent,'assessment-'+fault)
 if fault=='source_transaction':(p/'jobs'/('txn-'+('11'*16)+'.json')).write_bytes(b'partial')
 else:(p/'jobs'/('output-'+('17'*16)+'.json')).unlink()
 out=subprocess.run([cli,'recover','assess',p/'recover.json','--json'],capture_output=True,timeout=30)
 data=json.loads(out.stdout if out.returncode==0 else out.stderr)
 if fault=='source_transaction':assert out.returncode==0 and data['data']['source']['pending_transaction']
 else:assert out.returncode==1 and data['data']['source'] is not None and data['data']['publication'] is None
 assessments.append({'case':fault,'exit':out.returncode,'data':data['data']});shutil.rmtree(p)
import time
for sig,want in [(signal.SIGINT,130),(signal.SIGTERM,143),(signal.SIGKILL,None)]:
 label='verification-'+str(sig);p=prepare('written',parent,label,4096);trace=private/(label+'.strace')
 with (private/(label+'.stdout')).open('wb') as out,(private/(label+'.stderr')).open('wb') as err:
  proc=subprocess.Popen(['strace','-f','-yy','-o',str(trace),'-e','trace=read,pread64',cli,'recover','acknowledge-published',p/'recover.json','--json'],stdout=out,stderr=err,start_new_session=True)
  until=time.monotonic()+20;pid=None
  while time.monotonic()<until:
   text=trace.read_text() if trace.exists() else ''
   match=re.search(r'(?m)^(\d+)\s+read\([^\n]*disk.raw',text)
   if match:pid=int(match[1]);break
   assert proc.poll() is None,'child exited before verification marker'
   time.sleep(.001)
  assert pid is not None,'verification marker timeout'
  os.kill(pid,signal.SIGSTOP)
  busy=subprocess.run([cli,'recover','assess',p/'recover.json','--json'],capture_output=True,timeout=10);assert busy.returncode==1
  os.kill(pid,sig)
  if sig!=signal.SIGKILL:os.kill(pid,signal.SIGCONT)
  code=proc.wait(timeout=30)
 if want:
  assert code==want,(label,code)
  data=json.loads((private/(label+'.stderr')).read_text());assert data['data']['logical_bytes_verified']<256<<20 and not data['data']['bundle_synced']
  verification.append({'signal':sig.name,'exit':code,'store_busy_while_stopped':True,'data':data['data']})
 else:
  assert code!=0
  observation=subprocess.run([cli,'recover','assess',p/'recover.json','--json'],capture_output=True,check=True,timeout=30);state=json.loads(observation.stdout)['data']['publication']['output'];assert state['state']=='publish_intent' and state['pending_transaction']
  following=subprocess.run([cli,'recover','acknowledge-published',p/'recover.json','--json'],capture_output=True,check=True,timeout=30);assert json.loads(following.stdout)['operation_succeeded']
  verification.append({'signal':sig.name,'state_after_kill':state,'explicit_following_success':True,'store_busy_while_stopped':True})
 shutil.rmtree(p)
for injection,label in [('openat:signal=SIGKILL:when=1','kill-startup'),('renameat:signal=SIGKILL:when=1','kill-before-promotion')]:
 p=prepare('written',parent,label);code,data,trace=call(p,'acknowledge-published',label,injection);assert code!=0
 observation=subprocess.run([cli,'recover','assess',p/'recover.json','--json'],capture_output=True,check=True,timeout=30);state=json.loads(observation.stdout)['data']['publication']['output'];assert state['state']=='publish_intent' and state['pending_transaction']
 following=subprocess.run([cli,'recover','acknowledge-published',p/'recover.json','--json'],capture_output=True,check=True,timeout=30);assert json.loads(following.stdout)['operation_succeeded']
 kills.append({'boundary':label,'state_after_kill':state,'explicit_following_success':True});shutil.rmtree(p)
write(report/'verification-signals.json',verification);write(report/'independent-assessment.json',assessments)
write(report/'syscall-audit.json',results);write(report/'signal-cases.json',signals);write(report/'io-fault-cases.json',faults);write(report/'process-loss.json',kills);write(report/'output-failure.json',delivery)
print('Audits:',len(results),'signals:',len(signals),'I/O faults:',len(faults),'process loss:',len(kills),'output failure: passed',flush=True)
assert not list(parent.iterdir())
