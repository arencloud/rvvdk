#!/usr/bin/env python3
"""Offline R5.12q diagnostics using frozen Rust binaries; outputs contain no paths.

Run on a quiescent retained export, with a private work directory on the output
filesystem. The report directory must be new. Raw command output stays private.
No cache eviction, governor changes, filesystem tuning or VMware actions occur.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'before', 'candidate', 'probe', 'work', 'report']:
        p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--cpu', type=int, default=0)
    a = p.parse_args()
    a.report.mkdir(exist_ok=False, parents=True)
    a.work.mkdir(exist_ok=True, parents=True, mode=0o700)
    raw = a.work / 'converted.raw'
    assert not raw.exists() and shutil.disk_usage(a.work).free > 33 * 1024**3
    source_stamp = a.source.stat()
    binaries = dict(before=a.before, candidate=a.candidate, probe=a.probe)
    hashes = {k: hashlib.sha256(v.read_bytes()).hexdigest() for k, v in binaries.items()}
    data = dict(binary_sha256=hashes, cpu=a.cpu, rows=[], reads=[], checks=[])

    def save():
        (a.report / 'measurements.json').write_text(json.dumps(data, indent=2) + '\n')

    def snapshot():
        base = Path(f'/sys/devices/system/cpu/cpu{a.cpu}/cpufreq')
        return dict(unix_seconds=time.time(), load_average=os.getloadavg(),
                    frequency={n: (base / n).read_text().strip() for n in
                               ['scaling_governor', 'scaling_cur_freq'] if (base / n).exists()},
                    cpu_ticks=[s for s in Path('/proc/stat').read_text().splitlines()
                               if s.startswith(('cpu ', f'cpu{a.cpu} '))],
                    free_bytes=shutil.disk_usage(a.work).free)

    def run(command, label, affinity=True):
        with (a.work / (label + '.stdout')).open('w') as out, (a.work / (label + '.stderr')).open('w') as err:
            metrics = a.work / (label + '.time')
            result = subprocess.run(['/usr/bin/time', '-f',
                '{"wall_seconds":%e,"user_seconds":%U,"system_seconds":%S,"max_rss_kib":%M}',
                '-o', str(metrics), *(['taskset', '-c', str(a.cpu)] if affinity else []),
                *map(str, command)], stdout=out, stderr=err, timeout=900)
        return result.returncode, json.loads(metrics.read_text().splitlines()[-1])

    def copy(group, variant, number, progress=True, trace=False, actual=None):
        assert not raw.exists()
        actual = actual or variant
        label = f'{group}-{variant}-{number}'
        command = [binaries[actual], 'copy', a.source, raw, '--format', 'vmdk',
                   '--json', '--backend', 'auto']
        if progress:
            command += ['--progress']
        if trace:
            command = ['strace', '-f', '-qq', '-c', '-w', '-e',
                       'trace=pread64,pwrite64,fallocate,fdatasync,fsync', '-o',
                       a.report / (label + '.strace'), *command]
        before = snapshot()
        code, metrics = run(command, label)
        row = dict(group=group, variant=variant, binary=actual, round=number,
                   progress=progress, traced=trace, exit_code=code, metrics=metrics,
                   before=before, after=snapshot())
        data['rows'].append(row)
        save()
        assert code == 0, label
        report = json.loads((a.work / (label + '.stdout')).read_text())
        row.update({k: report[k] for k in ['elapsed_seconds', 'logical_bytes', 'stats', 'durability', 'backend']})
        row['allocated_bytes'] = raw.stat().st_blocks * 512
        if progress:
            events = [json.loads(s) for s in (a.work / (label + '.stderr')).read_text().splitlines()]
            row['events'] = [dict(phase=e['phase'], elapsed_seconds=e['elapsed_seconds']) for e in events]
            t = {e['phase']: e['elapsed_seconds'] for e in events}
            row['phases_seconds'] = dict(
                admission_plan=t['copy_started'] - t['planning'],
                transfer=t['copy_flushing'] - t['copy_started'],
                engine_flush=t['copy_flushed'] - t['copy_flushing'],
                revalidation=t['file_sync'] - t['copy_flushed'],
                file_sync=t['publication'] - t['file_sync'],
                publication=t['directory_sync'] - t['publication'],
                directory_sync_finish=t['completed'] - t['directory_sync'])
        frag = subprocess.run(['filefrag', str(raw)], capture_output=True, text=True, check=True)
        match = re.search(r': (\d+) extents? found', frag.stdout)
        assert match, 'filefrag summary unavailable'
        row['physical_extents'] = int(match[1])
        save()
        code, _ = run([a.candidate, 'verify', a.source, raw, '--format', 'vmdk', '--json'], label + '-verify')
        data['checks'].append(dict(run=label, case='full_native_readback', exit_code=code))
        save()
        assert code == 0
        raw.unlink()
        print(label, 'seconds', round(row['elapsed_seconds'], 3),
              'phases', row.get('phases_seconds', {}), flush=True)

    # Fixed sample count: initial three pairs plus three extra complete pairs.
    for n in range(1, 7):
        for variant in (['before', 'candidate'] if n % 2 else ['candidate', 'before']):
            copy('policy', variant, n)
    for group in ['identical', 'progress_control']:
        for n in range(1, 4):
            for variant in (['a', 'b'] if n % 2 else ['b', 'a']):
                copy(group, variant, n, progress=group == 'identical' or variant == 'b', actual='candidate')
    # Tracing alters scheduling; these runs are diagnostics, not speed benchmarks.
    for n in range(1, 3):
        for variant in (['before', 'candidate'] if n % 2 else ['candidate', 'before']):
            copy('syscalls', variant, n, trace=True)
    for n in range(1, 7):
        label = f'read-{n}'
        before = snapshot()
        code, metrics = run([a.probe, a.source, 'sequential'], label)
        row = dict(round=n, exit_code=code, metrics=metrics, before=before, after=snapshot())
        data['reads'].append(row)
        save()
        assert code == 0
        row.update(json.loads((a.work / (label + '.stdout')).read_text()))
        save()
        print(label, 'seconds', row['read_seconds'], flush=True)
    end = a.source.stat()
    assert (source_stamp.st_ino, source_stamp.st_size, source_stamp.st_mtime_ns) == (end.st_ino, end.st_size, end.st_mtime_ns)
    assert hashes == {k: hashlib.sha256(v.read_bytes()).hexdigest() for k, v in binaries.items()}
    data['source_stamp_unchanged'] = True
    data['binaries_unchanged'] = True
    save()


if __name__ == '__main__':
    main()
