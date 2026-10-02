#[path = "support/stream.rs"]
mod fixture;
use fixture::*;
use rvvdk_vmdk::{
    Descriptor, SparseDescriptor, SparseHeader, SparseLayerDescriptor, StreamDescriptor,
    StreamDirectory, StreamEnvelope, StreamHeader, StreamLimits, StreamMarker,
};
use std::io::{Cursor, Read, Seek, SeekFrom};

#[test]
fn descriptor_is_explicit_bounded_base_only_and_never_an_old_sparse_disk() {
    assert!(Descriptor::parse(descriptor()).is_err());
    assert!(SparseDescriptor::parse(descriptor()).is_err());
    assert!(SparseLayerDescriptor::parse(descriptor()).is_err());
    let d = StreamDescriptor::parse(descriptor()).unwrap();
    assert_eq!(d.size_bytes(), 4 << 20);
    for (from, to) in [
        ("ffffffff", "12345678"),
        ("streamOptimized", "monolithicSparse"),
        ("SPARSE", "FLAT"),
        ("version=1", "version=3"),
    ] {
        let bad = String::from_utf8(descriptor().to_vec())
            .unwrap()
            .replace(from, to);
        assert!(StreamDescriptor::parse(bad.as_bytes()).is_err(), "{from}");
    }
    let mut extra = descriptor().to_vec();
    extra.extend_from_slice(b"RW 8192 SPARSE \"second.vmdk\"\n");
    assert!(StreamDescriptor::parse(&extra).is_err());
    let mut padded = descriptor().to_vec();
    padded.resize(1024, 0);
    assert!(StreamDescriptor::parse(&padded).is_ok());
    padded[1000] = b'x';
    assert!(StreamDescriptor::parse(&padded).is_err());
}

#[test]
fn front_and_footer_envelopes_bind_capacity_without_admitting_payloads() {
    for footer in [false, true] {
        let data = image(footer);
        assert!(SparseHeader::parse(&data[..512], data.len() as u64).is_err());
        let e = StreamEnvelope::read_from(
            Cursor::new(&data),
            data.len() as u64,
            StreamLimits::default(),
        )
        .unwrap();
        assert_eq!(e.header.capacity_bytes(), 4 << 20);
        assert_eq!(e.cid, 0x12345678);
        assert_eq!(e.metadata_bytes_read, if footer { 13312 } else { 11264 });
        assert_eq!(
            e.primary_directory.offset,
            if footer { 66048 } else { 26 * 512 }
        );
        assert_eq!(e.header.directory() == StreamDirectory::Footer, footer);
    }
}

#[test]
fn unsupported_geometry_flags_state_and_overflows_fail() {
    let good = header(true);
    let size = image(true).len() as u64;
    for (at, value) in [(0, 0), (4, 1), (8, 0x30005), (8, 0x30003), (44, 1024)] {
        let mut bad = good;
        put32(&mut bad, at, value);
        assert!(
            StreamHeader::parse(&bad, size, StreamLimits::default()).is_err(),
            "{at}"
        );
    }
    for (at, value) in [
        (12, 0),
        (12, 129),
        (12, u64::MAX),
        (20, 256),
        (28, 0),
        (28, u64::MAX),
        (36, 0),
        (36, u64::MAX),
        (64, 0),
        (64, u64::MAX),
    ] {
        let mut bad = good;
        put64(&mut bad, at, value);
        assert!(
            StreamHeader::parse(&bad, size, StreamLimits::default()).is_err(),
            "{at}"
        );
    }
    for at in [72, 73, 77, 78, 511] {
        let mut bad = good;
        bad[at] ^= 0xff;
        assert!(
            StreamHeader::parse(&bad, size, StreamLimits::default()).is_err(),
            "{at}"
        );
    }
    for n in [0, 16, 511] {
        assert!(StreamHeader::parse(&good[..n], size, StreamLimits::default()).is_err());
    }
    assert!(StreamHeader::parse(&good, size - 1, StreamLimits::default()).is_err());
}

#[test]
fn front_regions_and_resource_limits_are_checked_before_reads() {
    let data = image(false);
    let size = data.len() as u64;
    for (at, value) in [(48, 1), (48, 26), (56, 1), (56, 128)] {
        let mut bad = header(false);
        put64(&mut bad, at, value);
        assert!(StreamHeader::parse(&bad, size, StreamLimits::default()).is_err());
    }
    let limits = StreamLimits::default();
    for l in [
        StreamLimits {
            capacity_bytes: 1,
            ..limits
        },
        StreamLimits {
            extent_bytes: 1,
            ..limits
        },
        StreamLimits {
            overhead_bytes: 1,
            ..limits
        },
        StreamLimits {
            directory_entries: 0,
            ..limits
        },
    ] {
        assert!(StreamHeader::parse(&data[..512], size, l).is_err());
    }
    let l = StreamLimits {
        read_bytes: 11263,
        ..limits
    };
    let mut reader = Traced {
        inner: Cursor::new(data),
        requests: vec![],
        mutate_header: false,
    };
    assert!(StreamEnvelope::read_from(&mut reader, size, l).is_err());
    assert_eq!(reader.requests, [(0, 512)]);
}

#[test]
fn footer_shape_agreement_directory_marker_and_truncation_are_required() {
    let good = image(true);
    for (at, value) in [
        (66560, 0),
        (66572, 2),
        (67072 + 4, 1),
        (67072 + 12, 0),
        (67072 + 56, 128),
        (67584, 1),
        (65536, 2),
        (65548, 1),
    ] {
        let mut bad = good.clone();
        put32(&mut bad, at, value);
        assert!(
            StreamEnvelope::read_from(Cursor::new(&bad), bad.len() as u64, StreamLimits::default())
                .is_err(),
            "{at}"
        );
    }
    for cut in [1, 512, 1024, 1536, 2560] {
        let bad = &good[..good.len() - cut];
        assert!(
            StreamEnvelope::read_from(Cursor::new(bad), bad.len() as u64, StreamLimits::default())
                .is_err()
        );
    }
    let mut bad = good.clone();
    bad[512 + descriptor().len() - 25] = 0;
    assert!(
        StreamEnvelope::read_from(Cursor::new(&bad), bad.len() as u64, StreamLimits::default())
            .is_err()
    );
    // Metadata-marker padding is ignored, unlike reserved header fields.
    let mut padded = good;
    padded[65536 + 24] = 19;
    padded[66560 + 400] = 29;
    padded[67584 + 400] = 39;
    assert!(
        StreamEnvelope::read_from(
            Cursor::new(&padded),
            padded.len() as u64,
            StreamLimits::default()
        )
        .is_ok()
    );
}

#[test]
fn markers_bound_payloads_logical_ranges_and_terminal_positions() {
    let h = StreamHeader::parse(&header(false), 1 << 20, StreamLimits::default()).unwrap();
    let l = StreamLimits::default();
    let m = StreamMarker::parse(&marker(128, 600, 0), 65536, &h, l).unwrap();
    assert!(matches!(
        m,
        StreamMarker::Grain {
            logical_offset: 65536,
            next_offset: 66560,
            ..
        }
    ));
    for prefix in [
        marker(1, 100, 0),
        marker(8192, 100, 0),
        marker(u64::MAX, 100, 0),
        marker(0, 131073, 0),
        marker(0, 0, 4),
        marker(3, 0, 1),
        marker(0, 0, 0),
    ] {
        assert!(StreamMarker::parse(&prefix, 65536, &h, l).is_err());
    }
    assert!(StreamMarker::parse(&marker(0, 600, 0), (1 << 20) - 512, &h, l).is_err());
    assert!(StreamMarker::parse(&marker(0, 1, 0), 65537, &h, l).is_err());
    assert!(StreamMarker::parse(&marker(0, 1, 0), 512, &h, l).is_err());
    assert!(matches!(
        StreamMarker::parse(&marker(4, 0, 1), 65536, &h, l).unwrap(),
        StreamMarker::GrainTable(_)
    ));
    assert!(matches!(
        StreamMarker::parse(&marker(1, 0, 2), 65536, &h, l).unwrap(),
        StreamMarker::GrainDirectory(_)
    ));
    assert_eq!(
        StreamMarker::parse(&marker(0, 0, 0), (1 << 20) - 512, &h, l).unwrap(),
        StreamMarker::End
    );
}

struct Traced {
    inner: Cursor<Vec<u8>>,
    requests: Vec<(u64, usize)>,
    mutate_header: bool,
}
impl Read for Traced {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        let at = self.inner.position();
        self.requests.push((at, b.len()));
        let n = self.inner.read(b)?;
        if self.mutate_header && at == 0 && self.requests.len() > 1 {
            b[12] ^= 1;
        }
        Ok(n)
    }
}
impl Seek for Traced {
    fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(p)
    }
}

#[test]
fn envelope_reads_only_bounded_metadata_and_rechecks_header() {
    let data = image(true);
    let size = data.len() as u64;
    let mut r = Traced {
        inner: Cursor::new(data.clone()),
        requests: vec![],
        mutate_header: false,
    };
    StreamEnvelope::read_from(&mut r, size, StreamLimits::default()).unwrap();
    assert_eq!(
        r.requests,
        [
            (0, 512),
            (512, 10240),
            (66560, 1536),
            (65536, 512),
            (0, 512)
        ]
    );
    let mut r = Traced {
        inner: Cursor::new(data),
        requests: vec![],
        mutate_header: true,
    };
    assert!(StreamEnvelope::read_from(&mut r, size, StreamLimits::default()).is_err());
}

#[test]
fn tib_capacity_does_not_trigger_directory_or_payload_scanning() {
    let mut data = vec![0; 65536 + 512 + 131072 + 1536];
    let size = data.len();
    let mut h = header(true);
    put64(&mut h, 12, (1_u64 << 40) / 512);
    data[..512].copy_from_slice(&h);
    let text = String::from_utf8(descriptor().to_vec())
        .unwrap()
        .replace("8192", "2147483648");
    data[512..512 + text.len()].copy_from_slice(text.as_bytes());
    data[65536..65552].copy_from_slice(&marker(256, 0, 2));
    data[size - 1536..size - 1520].copy_from_slice(&marker(1, 0, 3));
    put64(&mut h, 56, 129);
    data[size - 1024..size - 512].copy_from_slice(&h);
    let envelope =
        StreamEnvelope::read_from(Cursor::new(&data), size as u64, StreamLimits::default())
            .unwrap();
    assert_eq!(envelope.header.capacity_bytes(), 1 << 40);
    assert_eq!(envelope.primary_directory.length, 131072);
    assert_eq!(envelope.metadata_bytes_read, 13312);
    // Directory content is intentionally NOT qualified until the map stage.
    data[66048..66052].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(
        StreamEnvelope::read_from(Cursor::new(&data), size as u64, StreamLimits::default()).is_ok()
    );
}

#[test]
fn mismatched_capacity_short_source_and_terminal_overlaps_fail() {
    let mut data = image(true);
    let size = data.len() as u64;
    let text = String::from_utf8(descriptor().to_vec())
        .unwrap()
        .replace("8192", "4096");
    data[512..512 + text.len()].copy_from_slice(text.as_bytes());
    assert!(StreamEnvelope::read_from(Cursor::new(&data), size, StreamLimits::default()).is_err());
    assert!(
        StreamEnvelope::read_from(
            Cursor::new(image(true)),
            size + 512,
            StreamLimits::default()
        )
        .is_err()
    );
    let h = StreamHeader::parse(&header(true), size, StreamLimits::default()).unwrap();
    assert!(
        StreamMarker::parse(&marker(4, 0, 1), size - 2560, &h, StreamLimits::default()).is_err()
    );
    assert!(
        StreamMarker::parse(
            &marker(0, 1024, 0),
            size - 2048,
            &h,
            StreamLimits::default()
        )
        .is_err()
    );
}

#[test]
fn stream_only_informational_ddb_key_is_bounded_and_never_changes_geometry() {
    let text = String::from_utf8(descriptor().to_vec()).unwrap()
        + "ddb.toolsInstallType = \"synthetic\"\n";
    assert_eq!(
        StreamDescriptor::parse(text.as_bytes())
            .unwrap()
            .size_bytes(),
        4 << 20
    );
    let old = text.replace("streamOptimized", "monolithicSparse");
    assert!(SparseDescriptor::parse(old.as_bytes()).is_err());
    let unknown = text.replace("toolsInstallType", "unknown");
    assert!(StreamDescriptor::parse(unknown.as_bytes()).is_err());
    let duplicate = text.clone() + "ddb.toolsInstallType = \"duplicate\"\n";
    assert!(StreamDescriptor::parse(duplicate.as_bytes()).is_err());
}
