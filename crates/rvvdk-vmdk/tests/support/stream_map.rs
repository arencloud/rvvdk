//! Authored metadata only: payload bytes are deliberately not compressed data.
#![allow(dead_code)]
#[path = "stream.rs"]
mod envelope;
use envelope::{header, marker};
pub use envelope::{put32, put64};
pub fn image(footer: bool, grains: u64, populated: &[u64]) -> Vec<u8> {
    assert!(populated.windows(2).all(|w| w[0] < w[1]));
    assert!(populated.iter().all(|i| *i < grains));
    let mut h = header(footer);
    put64(&mut h, 12, grains * 128);
    let descriptor = format!(
        "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"streamOptimized\"\nRW {} SPARSE \"self.vmdk\"\n",
        grains * 128
    );
    let mut out = vec![0; 65536];
    out[512..512 + descriptor.len()].copy_from_slice(descriptor.as_bytes());
    let mut gd = vec![0; (grains.div_ceil(512) * 4).div_ceil(512) as usize * 512];
    let mut rgd = gd.clone();
    let groups: Vec<_> = populated
        .iter()
        .map(|i| i / 512)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if !footer {
        assert!(gd.len() == 512 && groups.len() <= 8);
    }
    for (n, group) in groups.iter().enumerate() {
        let mut table = [0; 2048];
        for &index in populated.iter().filter(|i| *i / 512 == *group) {
            put32(
                &mut table,
                (index % 512) as usize * 4,
                (out.len() / 512) as u32,
            );
            let mut record = [0xa5; 512];
            record[..16].copy_from_slice(&marker(index * 128, 32, 0xa5a5a5a5));
            out.extend_from_slice(&record);
        }
        if footer {
            let at = out.len();
            out.resize(at + 512, 0);
            out[at..at + 16].copy_from_slice(&marker(4, 0, 1));
            put32(&mut gd, *group as usize * 4, (out.len() / 512) as u32);
            out.extend_from_slice(&table);
        } else {
            let p = 32 + n * 8;
            let r = p + 4;
            put32(&mut gd, *group as usize * 4, p as u32);
            put32(&mut rgd, *group as usize * 4, r as u32);
            out[p * 512..p * 512 + 2048].copy_from_slice(&table);
            out[r * 512..r * 512 + 2048].copy_from_slice(&table);
        }
    }
    if footer {
        let at = out.len();
        out.resize(at + 512, 0);
        out[at..at + 16].copy_from_slice(&marker((gd.len() / 512) as u64, 0, 2));
        put64(&mut h, 56, (out.len() / 512) as u64);
        out.extend_from_slice(&gd);
        let at = out.len();
        out.resize(at + 512, 0);
        out[at..at + 16].copy_from_slice(&marker(1, 0, 3));
        out.extend_from_slice(&h);
        out.extend_from_slice(&[0; 512]);
        put64(&mut h, 56, u64::MAX);
    } else {
        out[21 * 512..22 * 512].copy_from_slice(&rgd);
        out[26 * 512..27 * 512].copy_from_slice(&gd);
    }
    out[..512].copy_from_slice(&h);
    out
}
pub fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
pub fn directory(b: &[u8], footer: bool) -> usize {
    let header = if footer { b.len() - 1024 } else { 0 };
    u64::from_le_bytes(b[header + 56..header + 64].try_into().unwrap()) as usize * 512
}
pub fn table(b: &[u8], footer: bool, group: usize) -> usize {
    word(b, directory(b, footer) + group * 4) as usize * 512
}
pub fn set_gte(b: &mut [u8], footer: bool, index: usize, value: u32) {
    let p = table(b, footer, index / 512);
    put32(b, p + index % 512 * 4, value);
    if !footer {
        let r = word(b, 21 * 512 + index / 512 * 4) as usize * 512;
        put32(b, r + index % 512 * 4, value);
    }
}
