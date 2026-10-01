#!/usr/bin/env python3
"""Offline only: compare a known guest file with mapped sectors in decoded RAW.

No VMware/SSH access, mounting, credentials, or guest/image mutations.
The private map must come from independent guest extent/device mapping, not from
searching the exported image. This verifies mapped fixture bytes, not a whole VM.
"""
import argparse
import hashlib
import json
from pathlib import Path

MAX_FIXTURE = 64 * 1024 * 1024
MAX_DISK = 1024**4
CHUNK = 1024 * 1024


def validate_map(mapping, disk_bytes, fixture_bytes):
    if mapping.get("schema") != 1:
        raise ValueError("unsupported map schema")
    if not 0 < disk_bytes <= MAX_DISK or not 0 < fixture_bytes <= MAX_FIXTURE:
        raise ValueError("size limit")
    if mapping.get("disk_bytes") != disk_bytes or mapping.get("fixture_bytes") != fixture_bytes:
        raise ValueError("reference size mismatch")
    extents = mapping.get("extents")
    if not isinstance(extents, list) or not 0 < len(extents) <= 1024:
        raise ValueError("extent count")
    covered = 0
    physical = []
    for extent in extents:
        values = [extent.get(k) for k in ("file_offset", "disk_offset", "length")]
        if any(type(v) is not int or v < 0 or v % 512 for v in values):
            raise ValueError("invalid aligned extent")
        file_offset, disk_offset, length = values
        if file_offset != covered or length == 0 or disk_offset + length > disk_bytes:
            raise ValueError("extent gap, overlap, or disk bound")
        covered += length
        if covered > fixture_bytes:
            raise ValueError("fixture bound")
        physical.append((disk_offset, disk_offset + length))
    if covered != fixture_bytes:
        raise ValueError("incomplete fixture map")
    physical.sort()
    if any(right[0] < left[1] for left, right in zip(physical, physical[1:])):
        raise ValueError("physical overlap")
    return extents


def verify(raw_path, map_path, fixture_path):
    with map_path.open("rb") as source:
        encoded_map = source.read(CHUNK + 1)
    if len(encoded_map) > CHUNK:
        raise ValueError("map size limit")
    mapping = json.loads(encoded_map)
    with raw_path.open("rb") as raw, fixture_path.open("rb") as fixture:
        import os
        disk_bytes = os.fstat(raw.fileno()).st_size
        fixture_bytes = os.fstat(fixture.fileno()).st_size
        extents = validate_map(mapping, disk_bytes, fixture_bytes)
        digest = hashlib.sha256()
        for extent in extents:
            raw.seek(extent["disk_offset"])
            fixture.seek(extent["file_offset"])
            remaining = extent["length"]
            while remaining:
                count = min(CHUNK, remaining)
                expected, actual = fixture.read(count), raw.read(count)
                if len(expected) != count or len(actual) != count or expected != actual:
                    raise ValueError("mapped guest bytes differ or are truncated")
                digest.update(expected)
                remaining -= count
        if digest.hexdigest() != mapping.get("fixture_sha256"):
            raise ValueError("fixture digest mismatch")
    return {"schema": 1, "scope": "independent_mapped_guest_fixture", "matched": True,
            "verified_bytes": fixture_bytes, "disk_logical_bytes": disk_bytes,
            "extent_count": len(extents), "whole_disk_equivalence": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("raw", type=Path)
    parser.add_argument("map", type=Path)
    parser.add_argument("fixture", type=Path)
    args = parser.parse_args()
    try:
        result = verify(args.raw, args.map, args.fixture)
    except (OSError, ValueError, TypeError, AttributeError):
        parser.exit(1, "fixture verification failed; inputs remain private\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
