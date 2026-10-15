#!/usr/bin/env python3
"""Run frozen candidate invariants on authored fixtures in private parents."""
import argparse
import os
from pathlib import Path
import subprocess

from measure_recovery_cli import PREFIX, digest, invoke, write


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['tests', 'cli', 'disk-parent', 'tmpfs-parent', 'report']:
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--full-library', action='store_true')
    a = parser.parse_args()
    for key, value in vars(a).items():
        if isinstance(value, Path):
            setattr(a, key, value.resolve())
    a.report.mkdir(parents=True, exist_ok=True)
    assert a.disk_parent.stat().st_dev != a.tmpfs_parent.stat().st_dev
    parents = [a.disk_parent, a.tmpfs_parent]
    assert all(not list(p.iterdir()) for p in parents)
    env = os.environ | {'RVDDK_RECOVERY_CLI': str(a.cli), 'TMPDIR': str(a.tmpfs_parent)}
    files = []
    if a.full_library:
        log = a.report / 'candidate-library.txt'
        with log.open('w') as output:
            subprocess.run([a.tests, '--test-threads=1'],
                           env=env | {'RVVDK_RETAINED_PARENT': str(a.disk_parent)},
                           stdout=output, stderr=subprocess.STDOUT, check=True, timeout=900)
        files.append(log)
        assert all(not list(p.iterdir()) for p in parents)
    for label, parent in [('disk', a.disk_parent), ('tmpfs', a.tmpfs_parent)]:
        for case in ['executable_histories', 'executable_exclusions', 'executable_reconciliation_candidates']:
            log = a.report / (label + '-' + case + '.txt')
            invoke(a.tests, PREFIX + case, env | {'RVVDK_RETAINED_PARENT': str(parent)}, log)
            files.append(log)
            assert all(not list(p.iterdir()) for p in parents)
    write(a.report / 'qualification.json', {
        'full_library': a.full_library, 'cli_fixture_groups': 6,
        'separate_filesystems': True, 'fixture_parents_empty': True,
        'binary_sha256': {'tests': digest(a.tests), 'cli': digest(a.cli)},
        'logs_sha256': {p.name: digest(p) for p in files},
        'scope': 'Existing fault/process-loss/control/identity/digest/sparse/persistence checks; '
                 'authored local fixtures only. Loopback enabled inside isolated network namespace.'})
    print('Candidate library/CLI fixture qualification and teardown passed', flush=True)


if __name__ == '__main__':
    main()
