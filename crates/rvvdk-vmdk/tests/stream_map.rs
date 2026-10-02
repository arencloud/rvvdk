#[path = "support/stream_map.rs"]
mod fixture;
use fixture::{directory, image, put32, put64, set_gte, table};
use rvvdk_vmdk::{StreamMap, StreamMapLimits};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
fn admit(b: &[u8]) -> Result<StreamMap, rvvdk_vmdk::StreamError> {
    StreamMap::read_from(Cursor::new(b), b.len() as u64, StreamMapLimits::default())
}
fn rejects(b: &[u8]) {
    assert!(admit(b).is_err());
}
#[test]
fn sparse_dense_empty_and_multiple_tables() {
    for footer in [false, true] {
        for (n, indices) in [
            (64, vec![]),
            (64, vec![1, 63]),
            (64, (0..64).collect()),
            (1536, vec![0, 1023, 1535]),
        ] {
            let b = image(footer, n, &indices);
            let m = admit(&b).unwrap();
            assert_eq!(m.cid(), 0x12345678);
            assert_eq!(
                m.grains().iter().map(|g| g.index()).collect::<Vec<_>>(),
                indices
            );
            for i in 0..n {
                let g = m.grain(i).unwrap();
                assert_eq!(g.is_some(), indices.contains(&i));
                if let Some(g) = g {
                    assert_eq!(g.logical_offset(), i * 65536);
                    assert_eq!(g.payload().offset, g.marker_offset() + 12);
                    assert_eq!(g.payload().length, 32);
                }
            }
            assert!(m.grain(n).is_err());
            assert!(m.grain(u64::MAX).is_err());
        }
    }
}
#[test]
fn exact_resource_limits_and_each_one_short() {
    for footer in [false, true] {
        let b = image(footer, 1024, &[1, 1023]);
        let s = admit(&b).unwrap().stats();
        let exact = StreamMapLimits {
            memory_bytes: s.reserved_bytes,
            read_bytes: s.metadata_bytes_read,
            table_entries: s.table_entries_examined,
            allocated_grains: 2,
            ..Default::default()
        };
        assert!(StreamMap::read_from(Cursor::new(&b), b.len() as u64, exact).is_ok());
        for limits in [
            StreamMapLimits {
                memory_bytes: exact.memory_bytes - 1,
                ..exact
            },
            StreamMapLimits {
                read_bytes: exact.read_bytes - 1,
                ..exact
            },
            StreamMapLimits {
                table_entries: exact.table_entries - 1,
                ..exact
            },
            StreamMapLimits {
                allocated_grains: 1,
                ..exact
            },
        ] {
            assert!(StreamMap::read_from(Cursor::new(&b), b.len() as u64, limits).is_err());
        }
    }
}
#[test]
fn empty_one_tib_uses_directory_space_only() {
    let b = image(true, 1 << 24, &[]);
    let m = admit(&b).unwrap();
    assert_eq!(m.stats().allocated_grains, 0);
    assert_eq!(m.stats().table_entries_examined, 0);
    assert_eq!(m.stats().reserved_bytes, 131136);
    assert_eq!(m.header().capacity_bytes(), 1 << 40);
}
#[test]
fn table_and_directory_aliases_and_unused_slots() {
    for footer in [false, true] {
        for pointer in [1, 21, 26, 128, u32::MAX] {
            let mut b = image(footer, 1024, &[1, 1023]);
            let gd = directory(&b, footer);
            put32(&mut b, gd, pointer);
            rejects(&b);
        }
        let mut b = image(footer, 1024, &[1, 1023]);
        let gd = directory(&b, footer);
        let p = fixture::word(&b, gd);
        put32(&mut b, gd + 4, p);
        rejects(&b);
        let mut b = image(footer, 64, &[1]);
        let gd = directory(&b, footer);
        put32(&mut b, gd + 4, 32);
        rejects(&b);
        let mut b = image(footer, 64, &[1]);
        set_gte(&mut b, footer, 64, 129);
        rejects(&b);
    }
}
#[test]
fn redundant_presence_and_contents_must_agree() {
    let mut b = image(false, 64, &[1]);
    put32(&mut b, 21 * 512, 0);
    rejects(&b);
    let mut b = image(false, 64, &[1]);
    put32(&mut b, 36 * 512 + 4, 129);
    rejects(&b);
    let mut b = image(false, 64, &[1]);
    put32(&mut b, 21 * 512, 32); // primary/redundant table alias
    rejects(&b);
}
#[test]
fn grain_alias_bounds_order_lba_and_compressed_size() {
    for footer in [false, true] {
        for pointer in [1, 2, 127, 128, 130, u32::MAX] {
            let mut b = image(footer, 64, &[1, 63]);
            set_gte(&mut b, footer, 63, pointer);
            rejects(&b);
        }
        let mut b = image(footer, 64, &[1, 63]);
        set_gte(&mut b, footer, 1, 129);
        set_gte(&mut b, footer, 63, 128);
        rejects(&b);
        for lba in [0, 129, 8192, u64::MAX] {
            let mut b = image(footer, 64, &[1]);
            put64(&mut b, 65536, lba);
            rejects(&b);
        }
        for size in [0, 501, 131073, u32::MAX] {
            let mut b = image(footer, 64, &[1]);
            put32(&mut b, 65536 + 8, size);
            rejects(&b);
        }
    }
}
#[test]
fn orphan_records_and_gaps_are_rejected() {
    for footer in [false, true] {
        let mut b = image(footer, 64, &[1, 63]);
        set_gte(&mut b, footer, 1, 0);
        rejects(&b);
        let mut b = image(footer, 64, &[1, 63]);
        set_gte(&mut b, footer, 63, 0);
        rejects(&b);
        let mut b = image(footer, 64, &[1]);
        let gd = directory(&b, footer);
        put32(&mut b, gd, 0);
        if !footer {
            put32(&mut b, 21 * 512, 0);
        }
        rejects(&b);
    }
    let mut b = image(false, 64, &[1]);
    b.extend_from_slice(&[0; 512]);
    rejects(&b);
}
#[test]
fn footer_table_marker_required_and_padding_ignored() {
    for (offset, value) in [(0, 3), (8, 1), (12, 2)] {
        let mut b = image(true, 64, &[1]);
        let at = table(&b, true, 0) - 512;
        put32(&mut b, at + offset, value);
        rejects(&b);
    }
    let mut b = image(true, 64, &[1]);
    let at = table(&b, true, 0) - 512;
    b[at + 16..at + 512].fill(0xff);
    assert!(admit(&b).is_ok());
}
struct Observed {
    inner: Cursor<Vec<u8>>,
    reads: Vec<(u64, usize)>,
    mutate_table: Option<usize>,
    table_reads: usize,
    change_header: bool,
    change_length: bool,
    header_reads: usize,
}
impl Observed {
    fn new(b: Vec<u8>) -> Self {
        Self {
            inner: Cursor::new(b),
            reads: vec![],
            mutate_table: None,
            table_reads: 0,
            change_header: false,
            change_length: false,
            header_reads: 0,
        }
    }
}
impl Read for Observed {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        let at = self.inner.position();
        self.reads.push((at, b.len()));
        if at == 0 {
            self.header_reads += 1;
            if self.change_length && self.header_reads == 3 {
                self.inner.get_mut().push(0);
            }
            if self.change_header && self.header_reads == 3 {
                self.inner.get_mut()[8] ^= 1;
            }
        }
        if Some(at as usize) == self.mutate_table {
            self.table_reads += 1;
            if self.table_reads == 2 {
                // Remove a record between counting and filling (footer has no redundant copy).
                put32(self.inner.get_mut(), at as usize + 4, 0);
            }
        }
        self.inner.read(b)
    }
}
impl Seek for Observed {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.inner.seek(p)
    }
}
#[test]
fn metadata_read_accounting_excludes_every_payload_and_padding_byte() {
    for footer in [false, true] {
        let b = image(footer, 1024, &[1, 511, 1023]);
        let n = b.len() as u64;
        let mut source = Observed::new(b);
        let m = StreamMap::read_from(&mut source, n, StreamMapLimits::default()).unwrap();
        assert_eq!(
            m.stats().metadata_bytes_read,
            source.reads.iter().map(|r| r.1 as u64).sum()
        );
        for g in m.grains() {
            for &(at, len) in &source.reads {
                assert!(at + len as u64 <= g.payload().offset || at >= g.marker_offset() + 512);
            }
        }
    }
}
#[test]
fn unsafe_table_locations_and_work_limits_fail_before_table_io() {
    let mut b = image(false, 64, &[1]);
    put32(&mut b, 26 * 512, 1);
    for (b, limits) in [
        (b, StreamMapLimits::default()),
        (
            image(false, 64, &[1]),
            StreamMapLimits {
                table_entries: 0,
                ..Default::default()
            },
        ),
        (
            image(false, 64, &[1]),
            StreamMapLimits {
                read_bytes: 13000,
                ..Default::default()
            },
        ),
    ] {
        let n = b.len() as u64;
        let mut s = Observed::new(b);
        assert!(StreamMap::read_from(&mut s, n, limits).is_err());
        assert!(s.reads.iter().all(|(at, _)| *at < 32 * 512));
    }
}
#[test]
fn count_header_and_length_changes_during_acquisition_are_rejected() {
    let b = image(true, 64, &[1]);
    let t = table(&b, true, 0);
    let n = b.len() as u64;
    let mut s = Observed::new(b);
    s.mutate_table = Some(t);
    assert!(StreamMap::read_from(&mut s, n, StreamMapLimits::default()).is_err());
    let b = image(false, 64, &[1]);
    let n = b.len() as u64;
    let mut s = Observed::new(b);
    s.change_header = true;
    assert!(StreamMap::read_from(&mut s, n, StreamMapLimits::default()).is_err());
}

#[test]
fn absent_directory_groups_and_empty_present_tables() {
    for footer in [false, true] {
        let b = image(footer, 1536, &[0, 1535]);
        let map = admit(&b).unwrap();
        assert!(map.grain(700).unwrap().is_none());
        assert_eq!(map.stats().table_entries_examined, 2048);
    }
    let mut b = image(false, 64, &[]);
    put32(&mut b, 26 * 512, 32);
    put32(&mut b, 21 * 512, 36);
    assert_eq!(admit(&b).unwrap().stats().allocated_grains, 0);
}
#[test]
fn truncated_sources_and_deterministic_metadata_mutations_never_panic() {
    for footer in [false, true] {
        let b = image(footer, 1024, &[1, 1023]);
        for end in [0, 511, 65535, b.len() - 1] {
            assert!(
                StreamMap::read_from(
                    Cursor::new(&b[..end]),
                    b.len() as u64,
                    StreamMapLimits::default()
                )
                .is_err()
            );
        }
        let gd = directory(&b, footer);
        let gt = table(&b, footer, 0);
        for at in (gd..gd + 16).chain(gt..gt + 32).chain(65536..65548) {
            for mask in [1, 0x80, 0xff] {
                let mut changed = b.clone();
                changed[at] ^= mask;
                let _ = admit(&changed); // Some mutations are valid sparse-map changes.
            }
        }
    }
}

#[test]
fn front_without_redundancy_and_final_length_recheck() {
    let mut b = image(false, 64, &[1, 63]);
    put32(&mut b, 8, 0x30001);
    put64(&mut b, 48, u64::MAX); // ignored when redundancy flag is absent
    let map = admit(&b).unwrap();
    assert_eq!(map.grains().len(), 2);
    let n = b.len() as u64;
    let mut source = Observed::new(b);
    source.change_length = true;
    assert!(StreamMap::read_from(&mut source, n, StreamMapLimits::default()).is_err());
}
