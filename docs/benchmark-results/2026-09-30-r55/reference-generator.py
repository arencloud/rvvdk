#!/usr/bin/env python3
"""Qualify public sparse CLI inspect/plan/copy/verify with RAW oracles and QEMU-generated images."""
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
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    work = args.directory.resolve()
    work.mkdir(parents=True, exist_ok=False)
    cli = args.cli.resolve()
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
        r = run([cli, 'inspect', descriptor, '--format', 'vmdk', '--extents', '--json'])
        assert r.returncode == 0, r.stderr
        inspected = json.loads(r.stdout)
        assert inspected['source']['logical_bytes'] == size
        assert inspected['source']['vmdk']['backing_file_count'] == (2 if name == 'multi-split' else 1)
        r = run([cli, 'plan', descriptor, output, '--format', 'vmdk', '--backend', 'auto', '--json'])
        assert r.returncode == 0 and not output.exists(), r.stderr
        planned = json.loads(r.stdout)
        assert planned['execution']['selected_backend'] == 'threaded'
        r = run([cli, 'copy', descriptor, output, '--format', 'vmdk', '--backend', 'auto', '--workers', '4', '--block-size', '65537', '--verify', '--json'])
        assert r.returncode == 0, r.stderr
        copied = json.loads(r.stdout)
        assert copied['verification']['bytes_verified'] == size
        assert copied['durability'] == 'file_and_directory_synced'
        assert sha(output) == sha(raw)
        r = run([cli, 'verify', descriptor, output, '--format', 'vmdk', '--block-size', '65537', '--json'])
        assert r.returncode == 0, r.stderr
        verified = json.loads(r.stdout)
        assert run([qemu, 'compare', '-f', 'vmdk', '-F', 'raw', descriptor, output]).returncode == 0
        assert inputs == {name:sha(work/name) for name in inputs}
        cases.append(dict(name=name, subformat=kind, inspected=inspected, planned=planned, copied=copied, verified=verified, output_sha256=sha(output), source_unchanged=True))
    args.report.write_text(json.dumps(dict(qemu_version=version.stdout, qemu_sha256=sha(qemu), cli_sha256=sha(cli), generator_sha256=sha(Path(__file__)), directory=str(work), cases=cases, commands=commands, files={p.name:dict(bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(work.iterdir())}), indent=2)+'\n')
    print('All four CLI commands pass on four QEMU fixtures; copied bytes equal RAW and QEMU, including a two-file split disk')


if __name__ == '__main__':
    main()
