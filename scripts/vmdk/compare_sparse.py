#!/usr/bin/env python3
"""Compare production SparseDisk reads with RAW oracles and QEMU-generated images."""
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dump', type=Path, required=True)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    work = args.directory.resolve()
    work.mkdir(parents=True, exist_ok=False)
    dump, cli = args.dump.resolve(), args.cli.resolve()
    qemu = Path(shutil.which('qemu-img') or 'qemu-img').resolve()
    commands, cases = [], []

    def run(argv):
        r = subprocess.run([str(a) for a in argv], text=True, capture_output=True, timeout=180)
        commands.append(dict(command=[str(a) for a in argv], exit_code=r.returncode, stdout=r.stdout, stderr=r.stderr))
        return r

    version = run([qemu, '--version'])
    assert version.returncode == 0
    for name, kind, size in [('mono', 'monolithicSparse', 1048576), ('two-tables', 'monolithicSparse', 64*1048576), ('split', 'twoGbMaxExtentSparse', 1048576), ('multi-split', 'twoGbMaxExtentSparse', 2*1024**3+65536)]:
        raw = work / (name + '.raw')
        with raw.open('xb') as f:
            f.truncate(size)
            offsets = sorted({0, size-65536, size-131072})
            for i, offset in enumerate(offsets):
                f.seek(offset)
                f.write(bytes((j*29+j//251+i*37+1) % 256 for j in range(65536)))
        descriptor = work / (name + '.vmdk')
        assert run([qemu, 'convert', '-f', 'raw', '-O', 'vmdk', '-o', 'subformat='+kind, raw, descriptor]).returncode == 0
        inputs = {p.name:sha(p) for p in work.glob(name+'*.vmdk')}
        output = work / (name + '.decoded.raw')
        r = run([dump, descriptor, '--external' if kind == 'twoGbMaxExtentSparse' else '--embedded', output])
        assert r.returncode == 0, r.stderr
        decoded = json.loads(r.stdout)
        assert decoded['size_bytes'] == size
        assert decoded['backings'] == (2 if name == 'multi-split' else 1)
        assert sha(output) == sha(raw)
        assert run([qemu, 'compare', '-f', 'vmdk', '-F', 'raw', descriptor, output]).returncode == 0
        assert inputs == {name:sha(work/name) for name in inputs}
        rejected = run([cli, 'inspect', descriptor, '--format', 'vmdk', '--json'])
        assert rejected.returncode != 0 and json.loads(rejected.stderr)['error']['code'] == 'vmdk'
        cases.append(dict(name=name, subformat=kind, decoded=decoded, output_sha256=sha(output), source_unchanged=True, public_cli_rejects=True))
    args.report.write_text(json.dumps(dict(qemu_version=version.stdout, qemu_sha256=sha(qemu), dump_sha256=sha(dump), cli_sha256=sha(cli), generator_sha256=sha(Path(__file__)), directory=str(work), cases=cases, commands=commands, files={p.name:dict(bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(work.iterdir())}), indent=2)+'\n')
    print('4 SparseDisk outputs equal RAW oracles and QEMU, including a two-file split disk; public CLI sparse support remains disabled')


if __name__ == '__main__':
    main()
