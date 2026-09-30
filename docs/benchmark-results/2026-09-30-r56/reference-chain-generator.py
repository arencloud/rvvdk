#!/usr/bin/env python3
"""Qualify metadata-only admission of unmodified QEMU hosted-sparse chains."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--helper', required=True, type=Path)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--directory', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    root = args.directory.resolve()
    root.mkdir(parents=True, exist_ok=False)
    helper, cli = args.helper.resolve(), args.cli.resolve()
    qemu = Path(shutil.which('qemu-img')).resolve()
    commands, cases = [], []

    def run(cmd, cwd):
        result = subprocess.run([str(v) for v in cmd], cwd=cwd, text=True, capture_output=True, timeout=60)
        commands.append(dict(command=[str(v) for v in cmd], cwd=str(cwd), exit_code=result.returncode,
                             stdout=result.stdout, stderr=result.stderr))
        return result

    assert run([qemu, '--version'], root).returncode == 0
    mono, split = 'monolithicSparse', 'twoGbMaxExtentSparse'
    for label, kinds in [('base', [mono]), ('mono', [mono]*3), ('split', [split]*3), ('mixed', [mono, split, mono])]:
        work = root / label
        work.mkdir()
        for i, kind in enumerate(kinds):
            cmd = [qemu, 'create', '-f', 'vmdk', '-o', 'subformat=' + kind]
            if i:
                cmd += ['-b', f'layer{i-1}.vmdk', '-F', 'vmdk']
            cmd += [f'layer{i}.vmdk', '1M']
            result = run(cmd, work)
            assert result.returncode == 0, result.stderr
        leaf = f'layer{len(kinds)-1}.vmdk'
        original = {p.name: sha(p) for p in work.iterdir()}
        info = run([qemu, 'info', '--backing-chain', '--output=json', leaf], work)
        assert info.returncode == 0, info.stderr
        reference = json.loads(info.stdout)
        result = run([helper, leaf], work)
        assert result.returncode == 0, result.stderr
        actual = json.loads(result.stdout)
        assert len(actual['layers']) == len(reference) == len(kinds)
        for index, layer in enumerate(actual['layers']):
            source = work / f'layer{len(kinds)-index-1}.vmdk'
            with source.open('rb') as stream:
                prefix = stream.read(512)
                if prefix[:4] == b'KDMV':
                    offset = int.from_bytes(prefix[28:36], 'little') * 512
                    size = int.from_bytes(prefix[36:44], 'little') * 512
                    assert size <= 1024**2
                    stream.seek(offset)
                    text = stream.read(size).rstrip(b'\0').decode()
                else:
                    stream.seek(0)
                    text = stream.read(1024**2).decode()
            cid = int(re.search(r'^CID=([0-9a-fA-F]+)', text, re.M)[1], 16)
            parent = int(re.search(r'^parentCID=([0-9a-fA-F]+)', text, re.M)[1], 16)
            assert layer['cid'] == cid and layer['parent_cid'] == (None if parent == 0xffffffff else parent)
            assert layer['capacity_bytes'] == reference[index]['virtual-size'] == 1024**2
        cli_result = run([cli, 'inspect', leaf, '--format', 'vmdk', '--json'], work)
        assert (cli_result.returncode == 0) == (len(kinds) == 1)
        assert original == {p.name: sha(p) for p in work.iterdir()}
        cases.append(dict(name=label, source_unchanged=True, admitted=actual,
                          qemu_info=reference, public_cli_rejects_parent=len(kinds)>1))
    files = {str(p.relative_to(root)): dict(bytes=p.stat().st_size, sha256=sha(p))
             for p in sorted(root.rglob('*')) if p.is_file()}
    args.report.write_text(json.dumps(dict(directory=str(root), helper_sha256=sha(helper),
        cli_sha256=sha(cli), qemu_sha256=sha(qemu), generator_sha256=sha(Path(__file__)),
        cases=cases, files=files, commands=commands), indent=2) + '\n')


if __name__ == '__main__':
    main()
