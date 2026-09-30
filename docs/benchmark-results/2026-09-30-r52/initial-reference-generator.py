#!/usr/bin/env python3
"""Qualify eager sparse metadata with QEMU fixtures and independent byte reconstruction."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inspect', type=Path, required=True)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    work = args.directory.resolve()
    work.mkdir(parents=True, exist_ok=False)
    helper, cli = args.inspect.resolve(), args.cli.resolve()
    qemu = Path(shutil.which('qemu-img') or 'qemu-img').resolve()
    commands, cases = [], []

    def run(argv):
        r = subprocess.run([str(a) for a in argv], text=True, capture_output=True, timeout=60)
        commands.append(dict(command=[str(a) for a in argv], exit_code=r.returncode, stdout=r.stdout, stderr=r.stderr))
        return r

    version = run([qemu, '--version'])
    assert version.returncode == 0
    for name, kind, size in [('mono', 'monolithicSparse', 1048576), ('two-tables', 'monolithicSparse', 64*1048576), ('split', 'twoGbMaxExtentSparse', 1048576)]:
        raw = work / (name + '.raw')
        with raw.open('xb') as f:
            f.truncate(size)
            # Two allocated grains, separated by unallocated grains; include a far GT.
            for offset, value in [(0, 0x5a), (size-65536, 0xa5)]:
                f.seek(offset)
                f.write(bytes([value])*65536)
        descriptor = work / (name + '.vmdk')
        assert run([qemu, 'convert', '-f', 'raw', '-O', 'vmdk', '-o', 'subformat='+kind, raw, descriptor]).returncode == 0
        extent = work / (name + '-s001.vmdk') if kind == 'twoGbMaxExtentSparse' else descriptor
        original = sha(extent)
        r = run([helper, descriptor, '0' if kind == 'twoGbMaxExtentSparse' else '--embedded'])
        assert r.returncode == 0, r.stderr
        metadata = json.loads(r.stdout)
        assert metadata['capacity_bytes'] == size
        assert len(metadata['grain_sectors']) * metadata['grain_bytes'] == size
        # Fixture utility only: reconstruct bytes from the validated map, with no
        # production VirtualDisk implementation or parent handling implied.
        reconstructed = work / (name + '.reconstructed.raw')
        with reconstructed.open('xb') as output, extent.open('rb') as source:
            for sector in metadata['grain_sectors']:
                if sector == 0:
                    data = bytes(metadata['grain_bytes'])
                else:
                    source.seek(sector*512)
                    data = source.read(metadata['grain_bytes'])
                    assert len(data) == metadata['grain_bytes']
                output.write(data)
        assert sha(reconstructed) == sha(raw)
        assert run([qemu, 'compare', '-f', 'vmdk', '-F', 'raw', descriptor, reconstructed]).returncode == 0
        assert sha(extent) == original
        rejected = run([cli, 'inspect', descriptor, '--format', 'vmdk', '--json'])
        assert rejected.returncode != 0 and json.loads(rejected.stderr)['error']['code'] == 'vmdk'
        cases.append(dict(name=name, subformat=kind, metadata=metadata, output_sha256=sha(reconstructed),
                          source_unchanged=True, public_cli_rejects=True,
                          qualification='validated metadata reconstruction equals RAW oracle and QEMU; no production sparse reader'))
    args.report.write_text(json.dumps(dict(qemu_version=version.stdout, qemu_sha256=sha(qemu), inspect_sha256=sha(helper),
        cli_sha256=sha(cli), generator_sha256=sha(Path(__file__)), directory=str(work), cases=cases, commands=commands,
        files={p.name:dict(bytes=p.stat().st_size,sha256=sha(p)) for p in sorted(work.iterdir())}), indent=2)+'\n')
    print('3 metadata maps reconstruct the RAW oracle and agree with QEMU; public CLI sparse support remains disabled')


if __name__ == '__main__':
    main()
