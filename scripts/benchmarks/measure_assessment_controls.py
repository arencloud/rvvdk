#!/usr/bin/env python3
"""Bounded A/A and A/B assessment controls after an adverse startup observation."""
import argparse
import json
import os
from pathlib import Path
import shutil
import statistics as st

from measure_recovery_cli import PREFIX, command_measure, digest, invoke, load, validate_aux_rows, write


def summarize(report):
    index = load(report / 'assessment-control-measurements.json')
    assert len(index) == 6
    groups = {}
    for comparison in ['identical', 'candidate']:
        runs = [r for r in index if r['comparison'] == comparison]
        assert [r['round'] for r in runs] == [1, 2, 3]
        groups[comparison] = {b: {k: [] for k in ['wall_ms', 'cpu_ms', 'peak_rss_kib']} for b in ['reference', 'candidate']}
        for run in runs:
            assert digest(report / run['file']) == run['sha256'] and run['pairs'] == 8
            rows = load(report / run['file'])
            validate_aux_rows(rows, 8, 'existing')
            for binary in groups[comparison]:
                for metric in groups[comparison][binary]:
                    groups[comparison][binary][metric].append(st.median(r['metrics'][metric] for r in rows if r['binary'] == binary))
    result = {'timed_observations': 96, 'groups': groups}
    write(report / 'assessment-control-summary.json', result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['reference', 'candidate', 'library', 'parent', 'private', 'report']:
        parser.add_argument('--' + name, required=True, type=Path)
    a = parser.parse_args()
    for key, value in vars(a).items():
        setattr(a, key, value.resolve())
    assert not (a.report / 'assessment-control-measurements.json').exists()
    assert not list(a.parent.iterdir())
    cpu = min(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {cpu})
    handoff = a.private / 'assessment-control.handoff'
    invoke(a.library, PREFIX + 'prepare_fixture', os.environ | {
        'RVVDK_RETAINED_PARENT': str(a.parent), 'RVDDK_CLI_HISTORY': 'published',
        'RVDDK_CLI_GRAINS': '1024', 'RVDDK_CLI_HANDOFF': str(handoff)},
        a.private / 'assessment-control-prepare.txt')
    fixture = Path(handoff.read_text())
    assert fixture.parent == a.parent and fixture.name.startswith('rvddk-retained-')

    def snapshot():
        return {str(p.relative_to(fixture)): (digest(p), p.stat().st_ino, p.stat().st_size, p.stat().st_blocks)
                for p in fixture.rglob('*') if p.is_file()}

    before = snapshot()
    runs = []
    expected = None
    try:
        for comparison in ['identical', 'candidate']:
            binaries = {'reference': a.reference, 'candidate': a.reference if comparison == 'identical' else a.candidate}
            for round_ in range(1, 4):
                rows = []
                for pair in range(8):
                    for binary in (['reference', 'candidate'] if pair % 2 == 0 else ['candidate', 'reference']):
                        metrics = command_measure(binaries[binary], ['recover', 'assess', str(fixture / 'recover.json'), '--json'], a.private)
                        output = (a.private / 'command.stdout').read_text()
                        data = json.loads(output)
                        assert data['operation_succeeded'] and data['status'] == 'assessed'
                        if expected is None:
                            expected = output
                        assert output == expected
                        rows.append({'pair': pair, 'binary': binary, 'metrics': metrics, 'oracle_passed': True})
                filename = f'assessment-control-{comparison}-{round_}.json'
                write(a.report / filename, rows)
                validate_aux_rows(rows, 8, 'existing')
                runs.append({'comparison': comparison, 'round': round_, 'pairs': 8, 'file': filename,
                             'sha256': digest(a.report / filename), 'binary_sha256': {k: digest(p) for k, p in binaries.items()}})
                write(a.report / 'assessment-control-measurements.json', runs)
        assert snapshot() == before
    finally:
        shutil.rmtree(fixture)
    assert not list(a.parent.iterdir())
    write(a.report / 'assessment-control-environment.json', {
        'cpu': cpu, 'fixture_unchanged': True, 'fixture_removed': True,
        'generator_sha256': digest(Path(__file__)), 'conditions': 'Serial after main SHA-host campaign; same ordinary caches and frozen executables.'})
    summarize(a.report)
    print('96 diagnostic assessment observations validated; fixture unchanged and removed')


if __name__ == '__main__':
    main()
