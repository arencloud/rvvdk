"""Offline qualification checks: an incorrect oracle must not pass."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from verify_guest_fixture import validate_map, verify


class GuestFixtureTests(unittest.TestCase):
    def setUp(self):
        self.fixture = b'A' * 512 + b'B' * 512
        self.mapping = {"schema": 1, "disk_bytes": 4096, "fixture_bytes": 1024,
                        "fixture_sha256": hashlib.sha256(self.fixture).hexdigest(),
                        "extents": [{"file_offset": 0, "disk_offset": 2048, "length": 512},
                                    {"file_offset": 512, "disk_offset": 512, "length": 512}]}

    def test_fragmented_mapping_and_corrupted_byte(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            raw = bytearray(4096)
            raw[2048:2560], raw[512:1024] = self.fixture[:512], self.fixture[512:]
            (root / 'raw').write_bytes(raw)
            (root / 'fixture').write_bytes(self.fixture)
            (root / 'map').write_text(json.dumps(self.mapping))
            result = verify(root / 'raw', root / 'map', root / 'fixture')
            self.assertEqual(result['verified_bytes'], 1024)
            self.assertFalse(result['whole_disk_equivalence'])
            raw[2300] ^= 1
            (root / 'raw').write_bytes(raw)
            with self.assertRaises(ValueError):
                verify(root / 'raw', root / 'map', root / 'fixture')

    def test_incomplete_overlapping_and_out_of_range_maps_fail(self):
        for field, value in [("file_offset", 0), ("disk_offset", 2048),
                             ("disk_offset", 4096), ("length", 0),
                             ("length", -512), ("disk_offset", True),
                             ("disk_offset", 513)]:
            mapping = copy.deepcopy(self.mapping)
            mapping['extents'][1][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                validate_map(mapping, 4096, 1024)
        mapping = copy.deepcopy(self.mapping)
        mapping['extents'].pop()
        with self.assertRaises(ValueError):
            validate_map(mapping, 4096, 1024)

    def test_wrong_fixture_digest_and_truncated_disk_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'raw').write_bytes(b'A' * 4096)
            (root / 'fixture').write_bytes(b'A' * 1024)
            (root / 'map').write_text(json.dumps(self.mapping))
            with self.assertRaises(ValueError):
                verify(root / 'raw', root / 'map', root / 'fixture')
            (root / 'raw').write_bytes(b'A' * 512)
            with self.assertRaises(ValueError):
                verify(root / 'raw', root / 'map', root / 'fixture')


if __name__ == '__main__':
    unittest.main()
