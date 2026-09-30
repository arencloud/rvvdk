#!/usr/bin/env python3
"""Compare opt-in CLI parent-chain commands with independent RAW overlays and QEMU decoding."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--cli', required=True, type=Path)
    p.add_argument('--directory', required=True, type=Path)
    p.add_argument('--report', required=True, type=Path)
    args = p.parse_args()
    work = args.directory.resolve(); work.mkdir(parents=True, exist_ok=False)
    cli = args.cli.resolve()
    qemu, io = [Path(shutil.which(n)).resolve() for n in ['qemu-img', 'qemu-io']]
    commands, cases = [], []

    def run(cmd, cwd):
        r = subprocess.run([str(v) for v in cmd], cwd=cwd, text=True, capture_output=True, timeout=180)
        commands.append(dict(command=[str(v) for v in cmd], cwd=str(cwd), exit_code=r.returncode, stdout=r.stdout, stderr=r.stderr))
        return r

    for tool in [qemu, io]:
        assert run([tool, '--version'], work).returncode == 0
    mono, split = 'monolithicSparse', 'twoGbMaxExtentSparse'
    for name, kinds, size in [('mono', [mono]*3, 1<<20), ('split', [split]*3, 1<<20),
                              ('mixed', [mono, split, mono], 4<<20), ('multi-split', [split]*3, (2<<30)+65536)]:
        root = work/name; root.mkdir()
        oracle = root/'oracle.raw'
        with oracle.open('wb') as f:
            f.truncate(size)
            for offset, length, value in [(0, 131072, 0x31), (size-65536, 65536, 0x51)]:
                f.seek(offset); f.write(bytes([value])*length)
            if size > 2<<30:
                f.seek((2<<30)-131072); f.write(bytes([0x61])*196608)
        r = run([qemu, 'convert', '-f', 'raw', '-O', 'vmdk', '-o', 'subformat='+kinds[0], oracle, 'layer0.vmdk'], root)
        assert r.returncode == 0, r.stderr
        writes = [[(32768, 512, 0x77), (65024, 1024, 0x88)], [(32768, 512, 0), (131072, 512, 0x99)]]
        if size > 2<<30:
            writes[0].append(((2<<30)-66048, 1024, 0xa1))
            writes[1].append(((2<<30)-512, 1024, 0xb2))
            # Preserve the initial partial-write reproducer separately. Fully replace
            # the second extent's grain to avoid depending on that producer COW path.
            writes[1].append((2<<30, 65536, 0xb2))
        for i in [1,2]:
            r = run([qemu, 'create', '-f', 'vmdk', '-o', 'subformat='+kinds[i], '-b', f'layer{i-1}.vmdk', '-F', 'vmdk', f'layer{i}.vmdk', str(size)], root)
            assert r.returncode == 0, r.stderr
            for offset, length, value in writes[i-1]:
                r = run([io, '-f', 'vmdk', '-c', f'write -P {value} {offset} {length}', f'layer{i}.vmdk'], root)
                assert r.returncode == 0, r.stderr
                with oracle.open('r+b') as f:
                    f.seek(offset); f.write(bytes([value])*length)
        original = {p.name: sha(p) for p in root.glob('*.vmdk')}
        reports = {}
        for command in ['inspect', 'plan', 'copy', 'verify']:
            cmd = [cli, command, 'layer2.vmdk']
            if command != 'inspect':
                cmd.append('rvddk.raw')
            cmd += ['--format', 'vmdk', '--allow-parents', '--json']
            if command in ['plan', 'copy']:
                cmd += ['--backend', 'auto', '--workers', '4', '--block-size', '65537']
            if command == 'copy':
                cmd.append('--verify')
            result = run(cmd, root)
            assert result.returncode == 0, result.stderr
            reports[command] = json.loads(result.stdout)
            if command == 'plan':
                assert not (root/'rvddk.raw').exists()
        assert reports['inspect']['source']['logical_bytes'] == size
        assert reports['inspect']['source']['vmdk']['layer_count'] == 3
        r = run([qemu, 'convert', '-f', 'vmdk', '-O', 'raw', 'layer2.vmdk', 'qemu.raw'], root)
        assert r.returncode == 0, r.stderr
        r = run([qemu, 'compare', '-f', 'vmdk', '-F', 'raw', 'layer2.vmdk', 'rvddk.raw'], root)
        assert r.returncode == 0, r.stderr
        expected = sha(oracle)
        if not expected == sha(root/'rvddk.raw') == sha(root/'qemu.raw'):
            args.report.write_text(json.dumps(dict(failed_case=name, commands=commands, oracle_sha256=expected, rvddk_sha256=sha(root/'rvddk.raw'), qemu_sha256=sha(root/'qemu.raw')), indent=2)+'\n')
            raise AssertionError('decoded bytes differ from independent oracle')
        r = run([cli, 'inspect', 'layer2.vmdk', '--format', 'vmdk', '--json'], root)
        assert r.returncode != 0
        assert original == {p.name: sha(p) for p in root.glob('*.vmdk')}
        cases.append(dict(name=name, reports=reports, raw_sha256=expected, writes=writes, source_unchanged=True, public_cli_rejects_parent=True))
    files = {str(p.relative_to(work)): dict(bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(work.rglob('*')) if p.is_file()}
    args.report.write_text(json.dumps(dict(directory=str(work), cli_sha256=sha(cli),
        qemu_sha256=sha(qemu), qemu_io_sha256=sha(io), generator_sha256=sha(Path(__file__)),
        cases=cases, files=files, commands=commands), indent=2)+'\n')


if __name__ == '__main__':
    main()
