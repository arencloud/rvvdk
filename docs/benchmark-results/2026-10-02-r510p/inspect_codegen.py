"""Retain bounded disassembly and compare instruction shapes, not performance."""
from pathlib import Path
import difflib,json,re,subprocess
root=Path('target/r510p/observations')
binaries={
 'original':Path('target/r510-reference/descriptor-baseline'),
 'before':Path('target/r510p/reference/descriptor-before'),
 'candidate':Path('target/r510p/reference/descriptor-candidate'),
}
name='rvvdk_vmdk::descriptor::Descriptor::parse_with_limits'
normalized={};facts={}
for variant,path in binaries.items():
 symbols=subprocess.check_output(['nm','-C','-S',str(path)],text=True)
 found=[line.split(maxsplit=3) for line in symbols.splitlines() if line.endswith(' '+name)]
 symbol,=found;start,size=int(symbol[0],16),int(symbol[1],16)
 command=['objdump','-d','-C','--no-show-raw-insn',f'--start-address={start}',f'--stop-address={start+size}',str(path)]
 asm=subprocess.check_output(command,text=True)
 (root/(variant+'-parse.asm')).write_text(asm)
 instructions=[]
 for line in asm.splitlines():
  m=re.match(r'\s*([a-f0-9]+):\s+(.*)',line)
  if not m:continue
  at=int(m[1],16)-start;ins=re.sub(r'#.*','',m[2]).rstrip()
  ins=re.sub(r'0x[0-9a-f]+\(%rip\)','<global>(%rip)',ins)
  def target(m):
   absolute=int(m[1],16)
   return ('+'+hex(absolute-start) if start<=absolute<start+size else '<external>')+' <'+m[2]+'>'
  ins=re.sub(r'([a-f0-9]+) <([^>]+)>',target,ins)
  instructions.append(f'{at:x}: {ins}\n')
 normalized[variant]=instructions
 facts[variant]={'start':hex(start),'bytes':size,'instruction_lines':len(instructions),'general_eq_symbol_present':any(line.endswith(' rvvdk_vmdk::descriptor::eq') for line in symbols.splitlines()),'command':command,'elf_size':subprocess.check_output(['size',str(path)],text=True)}
diff=''.join(difflib.unified_diff(normalized['original'],normalized['before'],fromfile='original',tofile='before'))
(root/'parse-normalized.diff').write_text(diff)
facts['original_before_instruction_shape_equal']=not diff
facts['normalization']='function instruction/branch addresses become offsets; named external targets retain symbol names; RIP-relative global displacements and comments removed'
facts['limitation']='Not proof of identical data, whole-program execution or performance. Layout and relocation effects remain hypotheses.'
(root/'codegen.json').write_text(json.dumps(facts,indent=2)+'\n')
print('Saved codegen evidence; old/current instruction shape equal:',not diff)
