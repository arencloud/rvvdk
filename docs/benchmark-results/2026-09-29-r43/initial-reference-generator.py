#!/usr/bin/env python3
"""Generate disposable local fixtures; compare rvvdk bytes with QEMU and an oracle."""
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
    parser.add_argument('--dump', type=Path, required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    work = args.directory.resolve()
    work.mkdir(parents=True, exist_ok=False)  # Never replace an earlier experiment.
    dump = args.dump.resolve()
    qemu = Path(shutil.which('qemu-img') or 'qemu-img').resolve()
    records, cases = [], []

    def run(argv):
        result = subprocess.run([str(a) for a in argv], text=True, capture_output=True, timeout=60)
        records.append(dict(command=[str(a) for a in argv], exit_code=result.returncode,
                            stdout=result.stdout, stderr=result.stderr))
        return result

    version = run([qemu, '--version'])
    assert version.returncode == 0
    header = '# Disk DescriptorFile\nversion=1\nCID=12345678\nparentCID=ffffffff\ncreateType="{}"\n'
    data = bytes((i * 29 + i // 251) % 256 for i in range(1024 * 1024))
    (work / 'data.bin').write_bytes(data)
    (work / 'other.bin').write_bytes(data[::-1])
    layouts = [
        ('monolithic', 'monolithicFlat', 'RW 128 FLAT "data.bin" 0\n', data[:65536]),
        ('split', 'twoGbMaxExtentFlat', 'RW 5 FLAT "data.bin" 0\nRW 7 FLAT "other.bin" 0\nRW 11 FLAT "data.bin" 0\n', data[:2560] + data[::-1][:3584] + data[:5632]),
        ('custom', 'custom', 'RW 2 FLAT "data.bin" 3\nRW 4 ZERO\nRW 1 FLAT "data.bin" 0\n', data[1536:2560] + bytes(2048) + data[:512]),
    ]
    for name, kind, extents, expected in layouts:
        descriptor = work / f'{name}.vmdk'
        descriptor.write_text(header.format(kind) + extents)
        actual = work / f'{name}.rvvdk.raw'
        assert run([dump, descriptor, actual]).returncode == 0
        assert actual.read_bytes() == expected
        reference = work / f'{name}.qemu.raw'
        conversion = run([qemu, 'convert', '-f', 'vmdk', '-O', 'raw', descriptor, reference])
        if kind == 'custom':
            assert conversion.returncode != 0 and "Unsupported image type 'custom'" in conversion.stderr
            status = 'QEMU rejects custom; rvvdk matches independently constructed byte oracle only'
        else:
            assert conversion.returncode == 0 and reference.read_bytes() == expected
            assert run([qemu, 'compare', '-f', 'vmdk', '-F', 'raw', descriptor, actual]).returncode == 0
            status = 'rvvdk, QEMU and byte oracle agree'
        cases.append(dict(name=name, status=status, bytes=len(expected), output_sha256=sha(actual),
                          descriptor=descriptor.read_text(), descriptor_sha256=sha(descriptor)))
    # Also consume actual QEMU-generated hosted descriptors, without changing text.
    for kind in ['monolithicFlat', 'twoGbMaxExtentFlat']:
        descriptor = work / f'qemu-{kind}.vmdk'
        assert run([qemu, 'convert', '-f', 'raw', '-O', 'vmdk', '-o', f'subformat={kind}', work / 'data.bin', descriptor]).returncode == 0
        actual = work / f'qemu-{kind}.rvvdk.raw'
        assert run([dump, descriptor, actual]).returncode == 0
        assert actual.read_bytes() == data
        assert run([qemu, 'compare', '-f', 'vmdk', '-F', 'raw', descriptor, actual]).returncode == 0
        cases.append(dict(name=f'qemu-{kind}', status='unaltered QEMU-generated descriptor matches source RAW and QEMU compare',
                          bytes=len(data), output_sha256=sha(actual), descriptor=descriptor.read_text(), descriptor_sha256=sha(descriptor)))
    report = dict(qemu_version=version.stdout, qemu_sha256=sha(qemu), dump_sha256=sha(dump),
                  generator_sha256=sha(Path(__file__)), directory=str(work), cases=cases, commands=records,
                  files={p.name: dict(bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(work.iterdir())})
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(f'{len(cases)} fixtures passed their stated qualification; custom reference support remains unavailable')


if __name__ == '__main__':
    main()
