#!/usr/bin/env python3
"""Compare native logical reads with independently authored RAW/QEMU fixtures.

Generate the corpus first with compare_stream_envelope.py. Only synthetic image
identities belong in this report; private images use sanitized live reports.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    for key in ['probe','cli','directory','report']:
        p.add_argument('--'+key,type=Path,required=True)
    args=p.parse_args()
    cases=[]
    for image in sorted(args.directory.glob('*.vmdk')):
        native=subprocess.run([str(args.cli),'inspect',str(image),'--format','vmdk','--json'],capture_output=True,text=True,timeout=60)
        assert native.returncode!=0 and 'version (only 1)' in native.stderr
        raw=args.directory/(image.stem.removesuffix('-footer')+'.raw')
        prior=sha(image)
        result=subprocess.run([str(args.probe),str(image),'compare',str(raw)],capture_output=True,text=True,timeout=60)
        if image.stem=='unaligned':
            assert result.returncode!=0
            cases.append(dict(case=image.stem,unsupported_rejected=True,cli_rejected=True))
            continue
        assert result.returncode==0,result.stderr
        report=json.loads(result.stdout)
        assert report['reference_matched'] and report['logical_bytes']==raw.stat().st_size
        assert sha(image)==prior
        cases.append(dict(case=image.stem,report=report,cli_rejected=True,image_sha256=prior,raw_sha256=sha(raw)))
    assert len(cases)==7
    args.report.write_text(json.dumps(dict(schema=1,scope='synthetic_native_logical_bytes',cases=cases,helper_sha256=sha(args.probe),script_sha256=sha(Path(__file__))),indent=2)+'\n')
    print('Six complete native logical images match authored RAW; unsupported capacity and all public CLI inputs reject.')


if __name__=='__main__':
    main()
