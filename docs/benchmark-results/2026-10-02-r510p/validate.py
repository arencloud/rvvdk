import json,re,subprocess,time
from pathlib import Path
root=Path('target/r510p/observations')
commands=[
 ('workspace-tests',['cargo','test','--workspace']),
 ('clippy',['cargo','clippy','--workspace','--all-targets','--','-D','warnings']),
 ('fmt',['cargo','fmt','--all','--','--check']),
 ('release-helper',['cargo','build','--release','-p','rvvdk-vmdk','--example','inspect_stream_envelope']),
 ('release-cli',['cargo','build','--release','-p','rvvdk-cli']),
 ('qemu-reference',['python3','scripts/vmdk/compare_stream_envelope.py','--inspect','target/release/examples/inspect_stream_envelope','--cli','target/release/rvddk','--directory','target/r510p/reference/qemu-fixtures','--report',str(root/'qemu-reference.json')]),
 ('codegen',['python3','target/r510p/reference/inspect_codegen.py']),
]
results=[]
for name,command in commands:
 with (root/(name+'.txt')).open('w') as log:
  started=time.time();result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
 results.append({'name':name,'command':command,'exit_code':result.returncode,'start_unix':started,'end_unix':time.time()})
 (root/'validation-commands.json').write_text(json.dumps(results,indent=2)+'\n')
 print(name, 'exit',result.returncode,flush=True)
 if result.returncode:raise SystemExit(result.returncode)
rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; 0 filtered out',(root/'workspace-tests.txt').read_text())
(root/'validation.json').write_text(json.dumps({'workspace_passed':sum(int(r[0]) for r in rows),'workspace_ignored':sum(int(r[2]) for r in rows),'workspace_failed':sum(int(r[1]) for r in rows),'new_tests':0,'reason':'Inline hint preserves parser semantics; existing lexical/bounds/stream/parent cases exercise unchanged contracts.','clippy':'all targets; -D warnings; passed','fmt':'passed','qemu_cases':7,'live_access_this_step':False},indent=2)+'\n')
