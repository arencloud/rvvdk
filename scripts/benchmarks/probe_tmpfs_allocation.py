#!/usr/bin/env python3
"""Independent sparse-file read/hash/fsync allocation probe; no rvddk or VMware code."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--parent', type=Path, required=True)
    p.add_argument('--report', type=Path, required=True)
    p.add_argument('--runs', type=int, default=30)
    a = p.parse_args()
    assert 1 <= a.runs <= 100
    assert not a.report.exists()
    libc = ctypes.CDLL(None, use_errno=True)
    expected = hashlib.sha256()
    for value, count in [(b'x', 4), (b'\0', 28), (b'y', 4), (b'\0', 28)]:
        for _ in range(count):
            expected.update(value * (1 << 20))
    rows = []
    with tempfile.TemporaryDirectory(prefix='rvddk-allocation-probe-', dir=a.parent) as directory:
        for n in range(a.runs):
            path = Path(directory) / str(n)
            with path.open('wb') as f:
                f.truncate(64 << 20)
                f.write(b'x' * (4 << 20)); f.seek(32 << 20); f.write(b'y' * (4 << 20)); f.flush()
                for offset in [4, 36]:
                    assert libc.fallocate(f.fileno(), 3, ctypes.c_longlong(offset << 20), ctypes.c_longlong(28 << 20)) == 0
                os.fsync(f.fileno())
            samples = [path.stat().st_blocks * 512]
            for _ in range(4):
                with path.open('rb') as f:
                    h = hashlib.sha256()
                    while data := f.read(1 << 20):
                        h.update(data)
                    assert h.digest() == expected.digest()
                    os.fsync(f.fileno())
                samples.append(path.stat().st_blocks * 512)
            rows.append(dict(run=n, allocated_bytes=samples, oracle_passed=True))
            path.unlink()
    result = dict(kernel=platform.release(), mount=subprocess.check_output(['findmnt', '-n', '-o', 'FSTYPE,OPTIONS', '-T', str(a.parent)], text=True).strip(),
                  shmem_hugepage_policy=Path('/sys/kernel/mm/transparent_hugepage/shmem_enabled').read_text().strip(),
                  rows=rows, changed_runs=sum(len(set(r['allocated_bytes'])) > 1 for r in rows),
                  conditions='Fresh sparse 64 MiB files with 8 MiB data. Four read/hash/fsync passes after setup; no writes, mmap or rvddk code during reads. All bytes independently checked. Cause of any allocation changes is not inferred.')
    a.report.write_text(json.dumps(result, indent=2) + '\n')
    print('Probe:', len(rows), 'files;', result['changed_runs'], 'changed allocation')


if __name__ == '__main__':
    main()
