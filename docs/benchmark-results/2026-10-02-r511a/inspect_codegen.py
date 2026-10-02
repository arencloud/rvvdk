"""Compare stream instruction shapes; this is not a performance proof."""
from pathlib import Path
import difflib
import json
import re
import subprocess
root = Path('target/r511a/observations')
facts = {}
for short, name in [('header', 'rvvdk_vmdk::stream::StreamHeader::parse_inner'),
                    ('marker', 'rvvdk_vmdk::stream::StreamMarker::parse')]:
    normalized = {}
    item = {}
    for variant in ['before','candidate']:
        path = Path('target/r511a/reference/stream-'+variant)
        symbols = subprocess.check_output(['nm','-C','-S',str(path)],text=True)
        symbol, = [line.split(maxsplit=3) for line in symbols.splitlines() if line.endswith(' '+name)]
        start, size = int(symbol[0],16), int(symbol[1],16)
        command = ['objdump','-d','-C','--no-show-raw-insn',f'--start-address={start}',f'--stop-address={start+size}',str(path)]
        asm = subprocess.check_output(command,text=True)
        (root/(variant+'-'+short+'.asm')).write_text(asm)
        instructions = []
        for line in asm.splitlines():
            m = re.match(r'\s*([a-f0-9]+):\s+(.*)',line)
            if not m:
                continue
            at = int(m[1],16)-start
            ins = re.sub(r'#.*','',m[2]).rstrip()
            ins = re.sub(r'0x[0-9a-f]+\(%rip\)','<global>(%rip)',ins)
            def target(match):
                absolute = int(match[1],16)
                return ('+'+hex(absolute-start) if start<=absolute<start+size else '<external>')+' <'+match[2]+'>'
            ins = re.sub(r'([a-f0-9]+) <([^>]+)>',target,ins)
            instructions.append(f'{at:x}: {ins}\n')
        normalized[variant] = instructions
        item[variant] = dict(start=hex(start),bytes=size,instruction_lines=len(instructions),command=command)
    diff = ''.join(difflib.unified_diff(normalized['before'],normalized['candidate'],fromfile='before',tofile='candidate'))
    (root/(short+'-normalized.diff')).write_text(diff)
    item['normalized_instruction_shape_equal'] = not diff
    facts[name] = item
facts['normalization'] = 'Instruction/branch addresses become offsets; named external targets retain names; RIP-relative global displacements and comments removed.'
facts['limitation'] = 'Not proof of identical data, whole-program execution or performance; layout effects are not excluded.'
(root/'codegen.json').write_text(json.dumps(facts,indent=2)+'\n')
print('Saved stream code-generation comparisons.')
