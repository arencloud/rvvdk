#!/usr/bin/env python3
"""Run bounded, independent libFuzzer campaigns; preserve logs and learned inputs."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tarfile
import time

TARGETS = {'descriptor': 65536, 'header': 520, 'metadata': 65536, 'chain': 520}


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def archive(directory, destination):
    # Stable archives preserve every learned unit without thousands of repo files.
    with destination.open('wb') as output, gzip.GzipFile(fileobj=output, mode='wb', mtime=0, filename='') as compressed:
        with tarfile.open(fileobj=compressed, mode='w|') as tar:
            for path in sorted(directory.rglob('*')):
                if path.is_file():
                    data = path.read_bytes()
                    item = tarfile.TarInfo(str(path.relative_to(directory)))
                    item.size = len(data); item.mode = 0o644
                    tar.addfile(item, io.BytesIO(data))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--seeds', type=Path, default=Path('fuzz/seeds'))
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seconds', type=int, default=30)
    parser.add_argument('--repeats', type=int, default=3)
    args = parser.parse_args()
    assert 1 <= args.seconds <= 3600 and 1 <= args.repeats <= 10
    args.work.mkdir(parents=True, exist_ok=False)
    args.output.mkdir(parents=True, exist_ok=False)
    runs = []
    manifest = dict(generator_sha256=sha(Path(__file__)), seconds=args.seconds, repeats=args.repeats,
                    seed_files={str(p.relative_to(args.seeds)): sha(p) for p in sorted(args.seeds.rglob('*')) if p.is_file()},
                    binaries={name: dict(path=str((args.binaries/name).resolve()), sha256=sha(args.binaries/name)) for name in TARGETS}, runs=runs)
    for repeat in range(args.repeats):
        for name, maximum in TARGETS.items():
            tag = f'{name}-{repeat+1}'
            root = args.work/tag; root.mkdir()
            shutil.copytree(args.seeds/name, root/'corpus'); (root/'artifacts').mkdir()
            cmd = ['taskset', '-c', '2-6', str((args.binaries/name).resolve()), str((root/'corpus').resolve()),
                   f'-max_total_time={args.seconds}', '-timeout=5', '-rss_limit_mb=1024', '-malloc_limit_mb=64',
                   f'-max_len={maximum}', f'-seed={5901+repeat}', '-print_final_stats=1',
                   f'-artifact_prefix={(root/"artifacts").resolve()}/']
            environment = {'ASAN_OPTIONS': 'quarantine_size_mb=64:detect_leaks=1'}
            start = time.monotonic()
            with (args.output/(tag+'.txt')).open('w') as log:
                try:
                    process = subprocess.Popen(['/usr/bin/time', '-v', '-o', str(args.output/(tag+'-resources.txt'))]+cmd,
                                               stdout=log, stderr=subprocess.STDOUT, env=os.environ|environment,
                                               start_new_session=True)
                    code = process.wait(timeout=args.seconds+60)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                    code = 124
            elapsed = time.monotonic()-start
            archive(root, args.output/(tag+'-inputs.tar.gz'))
            record = dict(tag=tag, target=name, repeat=repeat+1, command=cmd, environment=environment,
                          exit_code=code, wall_seconds=elapsed, archive_sha256=sha(args.output/(tag+'-inputs.tar.gz')),
                          inputs={str(p.relative_to(root)): dict(bytes=p.stat().st_size, sha256=sha(p)) for p in sorted(root.rglob('*')) if p.is_file()})
            text = (args.output/(tag+'.txt')).read_text()
            record['stats'] = {key: int(value) for key, value in re.findall(r'stat::(\w+):\s+(\d+)', text)}
            progress = re.findall(r'#(\d+)\s+(INITED|NEW|REDUCE|pulse|DONE)\s+cov:\s*(\d+)\s+ft:\s*(\d+)', text)
            record['feedback'] = [dict(executions=int(n), event=event, coverage=int(cov), features=int(ft)) for n,event,cov,ft in progress]
            runs.append(record)
            (args.output/'runs.json').write_text(json.dumps(manifest, indent=2)+'\n')
            print(tag, 'exit', code, record['stats'], flush=True)
            if code or any((root/'artifacts').iterdir()):
                raise SystemExit('Failure preserved. Reproduce and minimize before continuing qualification.')
            assert record['feedback'] and record['stats']['number_of_executed_units'] > 0


if __name__ == '__main__':
    main()
