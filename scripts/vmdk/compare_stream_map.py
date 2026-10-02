#!/usr/bin/env python3
"""Independently enumerate synthetic QEMU/authored grain maps; no payload decoding in Rust.

Run compare_stream_envelope.py first to create and decode the fixture corpus.
Only synthetic fixtures belong in this report; --records exposes grain locations.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def records(data):
    capacity, grain = struct.unpack_from('<QQ', data, 12)
    gd, = struct.unpack_from('<Q', data, 56)
    if gd == 2**64-1:
        gd, = struct.unpack_from('<Q', data, len(data)-1024+56)
    result = []
    for group in range(((capacity//grain)+511)//512):
        table, = struct.unpack_from('<I', data, gd*512+group*4)
        if not table:
            continue
        for slot in range(512):
            sector, = struct.unpack_from('<I', data, table*512+slot*4)
            if not sector:
                continue
            lba, size = struct.unpack_from('<QI', data, sector*512)
            assert lba == (group*512+slot)*grain
            result.append([group*512+slot, sector, size])
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for key in ['inspect','directory','report']:
        p.add_argument('--'+key,type=Path,required=True)
    args = p.parse_args()
    cases = []
    for path in sorted(args.directory.glob('*.vmdk')):
        proc = subprocess.run([str(args.inspect),str(path),'--records'],capture_output=True,text=True,timeout=60)
        if path.stem == 'unaligned':
            assert proc.returncode != 0
            cases.append(dict(case=path.stem, admitted=False))
            continue
        assert proc.returncode == 0, proc.stderr
        actual = json.loads(proc.stdout)
        expected = records(path.read_bytes())
        assert actual.pop('records') == expected
        assert actual['payload_validated'] is False
        cases.append(dict(case=path.stem,summary=actual,records=expected,image_sha256=digest(path),independent_map_match=True))
    assert len(cases) == 7
    args.report.write_text(json.dumps(dict(schema=1,scope='synthetic_grain_index_only',cases=cases,helper_sha256=digest(args.inspect),script_sha256=digest(Path(__file__))),indent=2)+'\n')
    print('Six independent grain maps match; unsupported unaligned image rejected.')


if __name__ == '__main__':
    main()
