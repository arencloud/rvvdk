#!/usr/bin/env python3
"""Generate hosted sparse extents and qualify headers only; never claim decoded bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


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
        commands.append(dict(command=[str(a) for a in argv], exit_code=r.returncode,
                             stdout=r.stdout, stderr=r.stderr))
        return r

    version = run([qemu, '--version'])
    assert version.returncode == 0
    for name, kind, size, accepted in [
        ('monolithic-1m', 'monolithicSparse', 1048576, True),
        ('monolithic-64m', 'monolithicSparse', 67108864, True),
        ('split-1m', 'twoGbMaxExtentSparse', 1048576, True),
        ('unaligned', 'monolithicSparse', 1049088, False),
        ('stream', 'streamOptimized', 1048576, False),
    ]:
        descriptor = work / (name + '.vmdk')
        assert run([qemu, 'create', '-f', 'vmdk', '-o', 'subformat=' + kind, descriptor, size]).returncode == 0
        info = run([qemu, 'info', '-f', 'vmdk', '--output=json', descriptor])
        assert info.returncode == 0
        info = json.loads(info.stdout)
        assert info['virtual-size'] == size
        extent = work / (name + '-s001.vmdk') if kind == 'twoGbMaxExtentSparse' else descriptor
        before = sha(extent)
        result = run([helper, extent])
        record = dict(name=name, subformat=kind, extent=extent.name, file_bytes=extent.stat().st_size,
                      header_hex=extent.read_bytes()[:512].hex(), qemu_info=info, expected_admission=accepted)
        if accepted:
            assert result.returncode == 0, result.stderr
            header = json.loads(result.stdout)
            assert header['capacity_bytes'] == size
            assert header['directory_entries'] == (size // header['grain_bytes'] + 511) // 512
            assert header['overhead_bytes'] <= extent.stat().st_size
            # Record advertised space without inferring descriptor presence from
            # the filename/subformat. Header admission does not parse its contents.
            if header['descriptor'] is not None:
                region = header['descriptor']
                contents = extent.read_bytes()[region['offset']:region['offset'] + region['length']]
                assert len(contents) == region['length']
                record['descriptor_region_nonzero_bytes'] = sum(byte != 0 for byte in contents)
            record['header'] = header
            record['qualification'] = 'header admission and capacity agree; directories, grains and logical bytes unqualified'
        else:
            assert result.returncode != 0
            assert ('grain-aligned' if name == 'unaligned' else 'Unsupported') in result.stderr
            record['rejection'] = result.stderr
        assert sha(extent) == before
        rejected = run([cli, 'inspect', descriptor, '--format', 'vmdk', '--json'])
        assert rejected.returncode != 0 and json.loads(rejected.stderr)['error']['code'] == 'vmdk'
        record['public_cli_rejects'] = True
        cases.append(record)
    args.report.write_text(json.dumps(dict(qemu_version=version.stdout, qemu_sha256=sha(qemu),
        inspect_sha256=sha(helper), cli_sha256=sha(cli), generator_sha256=sha(Path(__file__)),
        directory=str(work), cases=cases, commands=commands,
        files={p.name: dict(bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(work.iterdir())}), indent=2) + '\n')
    print('3 hosted headers admitted; 2 deliberate subset rejections; all 5 remain unsupported by the public CLI')


if __name__ == '__main__':
    main()
