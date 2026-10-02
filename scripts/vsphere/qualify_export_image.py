#!/usr/bin/env python3
"""Offline lab qualification of a completed export; all image data stays private.

QEMU is an independent reference decoder, not an rvddk production dependency.
Run only after export publication, in a private directory on a disposable runner.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

from verify_guest_fixture import verify


def sha256(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def qualify(args):
    report = json.loads(args.report.read_text())
    if (report['primary_error'] is not None or report['lease_cleanup'] != 'completed'
            or report['session_cleanup'] != 'logged_out' or not report['manifest_verified']
            or not report['artifact_published'] or report['artifact_cleanup_error'] is not None
            or report['lease_cleanup_error'] is not None or report['pagination_cleanup_error'] is not None
            or len(report['files']) != 1):
        raise ValueError('completed, verified export required')
    file = report['files'][0]
    if file['name'] != 'disk-1.vmdk':
        raise ValueError('unexpected artifact name')
    source = args.artifact / file['name']
    if source.stat().st_size != file['encoded_bytes'] or sha256(source) != file['sha256']:
        raise ValueError('artifact differs from the completed export')
    args.directory.mkdir(mode=0o700, exist_ok=False)
    raw = args.directory / 'decoded.raw'
    qemu = shutil.which('qemu-img')
    if not qemu:
        raise ValueError('qemu-img required')
    info = subprocess.run([qemu, 'info', '-f', 'vmdk', '--output=json', str(source)],
                          capture_output=True, check=True, timeout=60)
    (args.directory / 'qemu-info-private.json').write_bytes(info.stdout)
    metadata = json.loads(info.stdout)
    if metadata['format'] != 'vmdk' or metadata['virtual-size'] != report['selected_capacity_bytes']:
        raise ValueError('decoded format/capacity mismatch')
    if metadata.get('backing-filename'):
        raise ValueError('unexpected backing image')
    if args.reference_artifact is not None:
        reference = args.reference_artifact / 'disk-1.vmdk'
        prior = json.loads(args.reference_qualification.read_text())
        private = json.loads(args.reference_digests.read_text())
        if (prior['scope'] != 'offline_qemu_decode_and_mapped_guest_oracle'
                or prior['qemu_decode_exit_code'] != 0 or not prior['oracle']['matched']
                or not prior['encoded_source_unchanged']
                or prior['logical_bytes'] != metadata['virtual-size']
                or sha256(reference) != private['encoded_sha256']):
            raise ValueError('independently verified reference required')
        started = time.monotonic()
        with (args.directory / 'qemu-compare-private.log').open('xb') as log:
            comparison = subprocess.run([qemu, 'compare', '-f', 'vmdk', '-F', 'vmdk',
                                         str(reference), str(source)],
                                        stdout=log, stderr=log, timeout=600)
        if comparison.returncode != 0:
            raise ValueError('decoded logical comparison failed')
        result = {'schema': 1, 'scope': 'qemu_logical_compare_to_independently_verified_reference',
                  'qemu_compare_exit_code': 0,
                  'qemu_compare_seconds': time.monotonic() - started,
                  'logical_bytes': metadata['virtual-size'],
                  'encoded_sha256_matches_reference': file['sha256'] == private['encoded_sha256'],
                  'artifact_digest_matches_export_report': True,
                  'reference_artifact_digest_rechecked': True,
                  'oracle_via_logical_equivalence': prior['oracle'],
                  'private_image_digests_published': False, 'whole_source_equivalence': False}
        with args.public_report.open('x') as output:
            output.write(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result, indent=2))
        return
    if shutil.disk_usage(args.directory).free < 2 * 1024**3:
        raise ValueError('insufficient decode headroom')
    started = time.monotonic()
    with (args.directory / 'qemu-convert-private.log').open('xb') as log:
        process = subprocess.Popen([qemu, 'convert', '-f', 'vmdk', '-O', 'raw', '-S', '4096',
                                    str(source), str(raw)], stdout=log, stderr=log)
        try:
            while process.poll() is None:
                if time.monotonic() - started > 600 or shutil.disk_usage(args.directory).free < 1024**3:
                    raise ValueError('decode time/space limit')
                time.sleep(.25)
            if process.returncode != 0:
                raise ValueError('independent decode failed')
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    decode_seconds = time.monotonic() - started
    oracle = verify(raw, args.map, args.fixture)
    digest = sha256(raw)
    unchanged = sha256(source) == file['sha256']
    if not unchanged:
        raise ValueError('source changed during qualification')
    native = subprocess.run([str(args.cli.resolve()), 'inspect', str(source), '--format', 'vmdk', '--json'],
                            capture_output=True, timeout=60)
    (args.directory / 'native-private.stdout').write_bytes(native.stdout)
    (args.directory / 'native-private.stderr').write_bytes(native.stderr)
    (args.directory / 'private-digests.json').write_text(json.dumps({'raw_sha256': digest,
                                                                  'encoded_sha256': file['sha256']}) + '\n')
    result = {'schema': 1, 'scope': 'offline_qemu_decode_and_mapped_guest_oracle',
              'qemu_decode_exit_code': 0, 'qemu_decode_seconds': decode_seconds,
              'format': metadata['format'], 'logical_bytes': raw.stat().st_size,
              'raw_allocated_bytes': raw.stat().st_blocks * 512,
              'oracle': oracle, 'encoded_source_unchanged': unchanged,
              'native_cli_exit_code': native.returncode,
              'private_image_digests_published': False,
              'whole_source_equivalence': False}
    with args.public_report.open('x') as output:
        output.write(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['report', 'artifact', 'map', 'fixture', 'directory', 'cli', 'public-report']:
        parser.add_argument('--' + name, type=Path, required=True)
    for name in ['reference-artifact', 'reference-digests', 'reference-qualification']:
        parser.add_argument('--' + name, type=Path)
    args = parser.parse_args()
    references = [args.reference_artifact, args.reference_digests, args.reference_qualification]
    if any(references) and not all(references):
        parser.error('all three reference arguments are required together')
    try:
        qualify(args)
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
        parser.exit(1, 'offline qualification failed; inspect private runner diagnostics\n')


if __name__ == '__main__':
    main()
