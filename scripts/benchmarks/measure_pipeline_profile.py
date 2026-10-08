#!/usr/bin/env python3
"""Offline authored-fixture profiling. Runs a Rust test binary; no VMware access."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics as st
import subprocess

TEST = 'ownership::retained::output::publication::tests::benchmark::profiling::local_profile'
FIXTURES = ['stored_64', 'deflated_64', 'deflated_256']
METRICS = ['wall_ms', 'cpu_ms', 'admission_ms', 'conversion_ms', 'publication_admission_ms',
           'publication_ms', 'cleanup_ms', 'cleanup_cpu_ms']


def medians(rows):
    return {mode: {k: st.median(r[k] for r in rows if r['instrumented'] == enabled) for k in METRICS}
            for mode, enabled in [('disabled', False), ('instrumented', True)]}


def repeat_needed(runs):
    return (any(r['medians']['instrumented'][k] / r['medians']['disabled'][k] > 1.05
                for r in runs for k in ['wall_ms', 'cpu_ms'])
            or any(max(r['medians'][m][k] for r in runs)/min(r['medians'][m][k] for r in runs) > 1.05
                   for m in ['disabled', 'instrumented'] for k in METRICS))


def validate(rows, pairs, fixture):
    assert len(rows) == pairs*2
    logical = (256 if fixture == 'deflated_256' else 64) << 20
    for pair in range(pairs):
        group = rows[2*pair:2*pair+2]
        assert [r['pair'] for r in group] == [pair, pair]
        assert [r['instrumented'] for r in group] == ([False, True] if pair % 2 == 0 else [True, False])
    for r in rows:
        assert r['fixture'] == fixture and r['logical_bytes'] == logical
        assert r['oracle_passed'] and r['source_unchanged'] and r['cleanup_passed']
        assert 0 < r['output_allocated_bytes'] <= logical
        assert all(math.isfinite(r[k]) and r[k] > 0 for k in METRICS)
        if r['instrumented']:
            spans = r['regions']
            for key in ['logical_source_read', 'raw_read', 'logical_compare', 'logical_hash']:
                assert spans[key]['bytes'] == 3*logical
            for key in ['container_read', 'container_hash']:
                assert spans[key]['bytes'] == 7*r['source_bytes']
            assert spans['present_grain_read']['bytes'] == 3*(8 << 20)
            assert spans['journal_commit_including_sync']['calls'] == 9
            assert spans['native_map']['calls'] == 3
            assert all(v['calls'] > 0 and v['bytes'] >= 0 and all(math.isfinite(v[k]) and v[k] >= 0 for k in ['wall_ms', 'cpu_ms']) for v in spans.values())
            assert sum(v['wall_ms'] for v in spans.values()) <= r['wall_ms']
        else:
            assert not r['regions']


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--report', type=Path, required=True)
    p.add_argument('--parent', type=Path, required=True)
    p.add_argument('--filesystem', choices=['xfs', 'btrfs', 'tmpfs'], required=True)
    p.add_argument('--cpu', type=int, default=0)
    a = p.parse_args()
    binary = a.binary.resolve(); report = a.report.resolve(); parent = a.parent.resolve()
    report.mkdir(mode=0o700, parents=True, exist_ok=False)
    parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    fs = subprocess.check_output(['stat', '-f', '-c', '%T', str(parent)], text=True).strip()
    assert fs == a.filesystem and a.cpu in os.sched_getaffinity(0)
    flags = next(s.split(':', 1)[1].split() for s in Path('/proc/cpuinfo').read_text().splitlines() if s.startswith('flags'))
    model = next(s.split(':', 1)[1].strip() for s in Path('/proc/cpuinfo').read_text().splitlines() if s.startswith('model name'))
    environment = dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                       harness_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                       kernel=platform.release(), cpu_model=model, visible_cpus=os.cpu_count(), cpu=a.cpu,
                       filesystem=fs, sha_ni_visible='sha_ni' in flags,
                       conditions='Same release test executable, alternating disabled/instrumented order per pair. One thread and fixed CPU affinity; no isolation or cache drops. Fixture setup and independent full authored-byte oracle excluded from timing. Copy includes flush, journal region includes sync; these are inclusive categories. Production library has no clocks/counters. Thread CPU in disjoint regions, process CPU for whole operation. RSS is lifetime peak including fixture/oracle; no before/after implementation speedup.')
    (report/'environment.json').write_text(json.dumps(environment, indent=2)+'\n')
    runs = []
    for fixture in FIXTURES:
        for phase, pairs in [('initial', 4), ('repeat', 8)]:
            if phase == 'repeat' and not repeat_needed([r for r in runs if r['fixture'] == fixture]):
                continue
            for round_id in range(1, 4):
                tag = f'{fixture}-{phase}-{round_id}'
                dest = report/(tag+'.json')
                env = dict(os.environ, RVVDK_RETAINED_PARENT=str(parent), RVVDK_RETAINED_PAIRS=str(pairs),
                           RVVDK_RETAINED_REPORT=str(dest), RVVDK_PROFILE_FIXTURE=fixture)
                with (report/(tag+'.txt')).open('w') as log:
                    subprocess.run(['taskset', '-c', str(a.cpu), str(binary), '--exact', TEST,
                                    '--ignored', '--nocapture', '--test-threads=1'], env=env,
                                   stdout=log, stderr=subprocess.STDOUT, check=True)
                rows = json.loads(dest.read_text()); validate(rows, pairs, fixture)
                run = dict(tag=tag, fixture=fixture, phase=phase, round=round_id, pairs=pairs,
                           medians=medians(rows), rows=rows)
                runs.append(run)
                (report/'measurements.json').write_text(json.dumps(runs, indent=2)+'\n')
                print(json.dumps({k:v for k,v in run.items() if k!='rows'}), flush=True)
    print('Completed all required initial/repeat groups', flush=True)


if __name__ == '__main__':
    main()
