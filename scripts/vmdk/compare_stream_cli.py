#!/usr/bin/env python3
"""Qualify CLI stream conversion against the synthetic R5.10/R5.11 QEMU corpus.

Generate fixtures with compare_stream_envelope.py and the archived pre-R5.12
CLI (that historical gate intentionally expects rejection). No private images.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for key in ['cli', 'directory', 'report']:
        p.add_argument('--' + key, type=Path, required=True)
    args = p.parse_args()
    cases = []
    for image in sorted(args.directory.glob('*.vmdk')):
        prior = sha(image)
        def run(command, extra=()):
            return subprocess.run([str(args.cli), command, str(image), *map(str, extra),
                                   '--format', 'vmdk', '--json'], capture_output=True, text=True, timeout=120)
        inspected = run('inspect')
        if image.stem == 'unaligned':
            assert inspected.returncode != 0
            cases.append(dict(case=image.stem, unsupported_rejected=True))
            continue
        assert inspected.returncode == 0, inspected.stderr
        output = args.directory / (image.stem + '-cli.raw')
        assert not output.exists()
        plan = run('plan', [output, '--backend', 'auto'])
        assert plan.returncode == 0 and not output.exists(), plan.stderr
        copied = run('copy', [output, '--verify', '--backend', 'auto', '--block-size', '65537', '--workers', '4'])
        assert copied.returncode == 0, copied.stderr
        raw = args.directory / (image.stem.removesuffix('-footer') + '.raw')
        assert sha(output) == sha(raw)
        verified = run('verify', [output, '--block-size', '131071'])
        assert verified.returncode == 0, verified.stderr
        qemu = subprocess.run(['qemu-img', 'compare', '-f', 'vmdk', '-F', 'raw', str(image), str(output)],
                              capture_output=True, text=True, timeout=120)
        assert qemu.returncode == 0, qemu.stderr
        assert sha(image) == prior
        report = json.loads(copied.stdout)
        cases.append(dict(case=image.stem, logical_bytes=report['logical_bytes'],
                          backend=report['backend'], authored_raw_matched=True, qemu_matched=True,
                          image_sha256=prior, raw_sha256=sha(raw)))
        output.unlink()
    assert len(cases) == 7
    args.report.write_text(json.dumps(dict(schema=1, scope='synthetic_cli_conversion', cases=cases,
        cli_sha256=sha(args.cli), script_sha256=sha(Path(__file__))), indent=2) + '\n')
    print('Six CLI conversions match authored RAW and QEMU; unsupported capacity rejects.')


if __name__ == '__main__':
    main()
