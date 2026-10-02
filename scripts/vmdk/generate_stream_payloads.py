#!/usr/bin/env python3
"""Generate synthetic decoder fixtures with the independent system zlib encoder."""
import hashlib
import json
from pathlib import Path
import zlib

root = Path('crates/rvvdk-vmdk/tests/fixtures/stream')
root.mkdir(parents=True, exist_ok=True)
pattern = bytes(((i*17+29) ^ (i>>7)) & 255 for i in range(65536))
files = {}
for name, raw, window in [('zero.zlib',bytes(65536),15),('pattern.zlib',pattern,15),('pattern.gzip',pattern,31),('oversized.zlib',bytes(1<<20),15)]:
    encoder = zlib.compressobj(6,zlib.DEFLATED,window)
    payload = encoder.compress(raw)+encoder.flush()
    (root/name).write_bytes(payload)
    files[name] = dict(encoded_bytes=len(payload),decoded_bytes=len(raw),sha256=hashlib.sha256(payload).hexdigest())
(root/'provenance.json').write_text(json.dumps(dict(generator='scripts/vmdk/generate_stream_payloads.py',zlib_version=zlib.ZLIB_RUNTIME_VERSION,scope='synthetic only',files=files),indent=2)+'\n')
