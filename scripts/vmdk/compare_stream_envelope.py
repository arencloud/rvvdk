#!/usr/bin/env python3
"""Offline R5.10 envelope proof; QEMU decodes fixtures, Rust admits metadata only."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def footer_form(source):
    """Author the documented footer layout using QEMU-generated compressed records."""
    h = bytearray(source[:512])
    capacity, grain = struct.unpack_from('<QQ', h, 12)
    entries = ((capacity + grain - 1)//grain + 511)//512
    gd = struct.unpack_from('<Q', h, 56)[0] * 512
    overhead = struct.unpack_from('<Q', h, 64)[0] * 512
    output = bytearray(source[:overhead])
    directory = []

    def marker(value, kind):
        return struct.pack('<QII', value, 0, kind) + bytes(496)

    for index in range(entries):
        gt = struct.unpack_from('<I', source, gd + index*4)[0] * 512
        table = bytearray(2048)
        for slot in range(512):
            sector = struct.unpack_from('<I', source, gt + slot*4)[0] if gt else 0
            if sector == 0:
                continue
            assert sector > 1
            at = sector*512
            lba, size = struct.unpack_from('<QI', source, at)
            assert lba == (index*512+slot)*grain and 0 < size <= 131072
            struct.pack_into('<I', table, slot*4, len(output)//512)
            output.extend(source[at:at+12+size])
            output.extend(bytes((-len(output)) % 512))
        output.extend(marker(4, 1))
        directory.append(len(output)//512)
        output.extend(table)
    gd_bytes = b''.join(struct.pack('<I', v) for v in directory)
    gd_bytes += bytes((-len(gd_bytes)) % 512)
    output.extend(marker(len(gd_bytes)//512, 2))
    gd_sector = len(output)//512
    output.extend(gd_bytes)
    output.extend(marker(1, 3))
    struct.pack_into('<I', h, 8, 0x30001)
    struct.pack_into('<Q', h, 48, 0)
    struct.pack_into('<Q', h, 56, gd_sector)
    output.extend(h)
    output.extend(bytes(512))
    struct.pack_into('<Q', h, 56, 2**64-1)
    output[:512] = h
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['inspect', 'cli', 'directory', 'report']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    args.directory.mkdir(parents=True, exist_ok=False)
    cases = []
    for name, size, populated in [('zero', 4<<20, False), ('data', 4<<20, True),
                                  ('two_tables', 64<<20, True), ('unaligned', (4<<20)+512, True)]:
        raw = args.directory/(name+'.raw')
        with raw.open('wb') as f:
            f.truncate(size)
            if populated:
                for offset in [65536, size-65536]:
                    f.seek(offset)
                    f.write(hashlib.shake_256(name.encode()+str(offset).encode()).digest(65536))
        front = args.directory/(name+'.vmdk')
        subprocess.run(['qemu-img','convert','-f','raw','-O','vmdk','-o','subformat=streamOptimized','-c',str(raw),str(front)], check=True)
        images = [('front', front)]
        if name != 'unaligned':
            footer = args.directory/(name+'-footer.vmdk')
            footer.write_bytes(footer_form(front.read_bytes()))
            images.append(('footer', footer))
        for profile, image in images:
            original = sha(image)
            admitted = subprocess.run([str(args.inspect),str(image)], capture_output=True, text=True, timeout=60)
            native = subprocess.run([str(args.cli),'inspect',str(image),'--format','vmdk','--json'], capture_output=True, text=True, timeout=60)
            assert native.returncode != 0 and 'version (only 1)' in native.stderr
            if name == 'unaligned':
                assert admitted.returncode != 0
                cases.append(dict(case=name,profile=profile,admitted=False,native_rejected=True))
                continue
            assert admitted.returncode == 0, admitted.stderr
            envelope = json.loads(admitted.stdout)
            assert envelope['profile'] == profile and envelope['capacity_bytes'] == size
            decoded = args.directory/(name+'-'+profile+'.decoded.raw')
            subprocess.run(['qemu-img','convert','-f','vmdk','-O','raw',str(image),str(decoded)], check=True)
            assert sha(raw) == sha(decoded) and original == sha(image)
            cases.append(dict(case=name,profile=profile,envelope=envelope,qemu_bytes_match=True,native_rejected=True,
                              image_sha256=original,fixture_source='QEMU output' if profile=='front' else 'authored footer layout, QEMU compressed records'))
    args.report.write_text(json.dumps(dict(schema=1,scope='metadata_admission_not_rust_decoding',
        qemu_version=subprocess.check_output(['qemu-img','--version'],text=True).splitlines()[0],cases=cases,
        generator_sha256=sha(Path(__file__)),helper_sha256=sha(args.inspect)),indent=2)+'\n')
    print('Six front/footer envelopes admitted, QEMU bytes match; unaligned and all public CLI stream inputs rejected.')


if __name__ == '__main__':
    main()
