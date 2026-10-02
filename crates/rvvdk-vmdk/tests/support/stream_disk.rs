#![allow(dead_code)]
#[path = "stream.rs"]
mod envelope;
use envelope::{header, marker};
pub use envelope::{put32, put64};
/// Independently authored zlib/stored-DEFLATE, including Adler-32. No production decoder/encoder reuse.
pub fn stored(raw: &[u8]) -> Vec<u8> {
    let mut b = vec![0x78, 0x01];
    let mut left = raw;
    loop {
        let n = left.len().min(65535);
        b.push(if n == left.len() { 1 } else { 0 });
        b.extend_from_slice(&(n as u16).to_le_bytes());
        b.extend_from_slice(&(!(n as u16)).to_le_bytes());
        b.extend_from_slice(&left[..n]);
        left = &left[n..];
        if left.is_empty() {
            break;
        }
    }
    let (mut a, mut c) = (1u32, 0u32);
    for &v in raw {
        a = (a + v as u32) % 65521;
        c = (c + a) % 65521;
    }
    b.extend_from_slice(&((c << 16) | a).to_be_bytes());
    b
}
pub fn bytes(index: u64) -> Vec<u8> {
    (0..65536)
        .map(|i| ((i as u64 * 17 + index * 29) ^ (i as u64 >> 7)) as u8)
        .collect()
}
pub fn image(footer: bool, grains: u64, records: &[(u64, Vec<u8>)]) -> Vec<u8> {
    assert!(records.windows(2).all(|w| w[0].0 < w[1].0));
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
    let groups: Vec<_> = records
        .iter()
        .map(|r| r.0 / 512)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for (n, &group) in groups.iter().enumerate() {
        let mut table = [0; 2048];
        for (index, payload) in records.iter().filter(|r| r.0 / 512 == group) {
            assert!(*index < grains);
            put32(
                &mut table,
                (*index % 512) as usize * 4,
                (out.len() / 512) as u32,
            );
            out.extend_from_slice(&marker(index * 128, payload.len() as u32, 0)[..12]);
            out.extend_from_slice(payload);
            out.resize(out.len().div_ceil(512) * 512, 0x5a);
        }
        if footer {
            let at = out.len();
            out.resize(at + 512, 0);
            out[at..at + 16].copy_from_slice(&marker(4, 0, 1));
            put32(&mut gd, group as usize * 4, (out.len() / 512) as u32);
            out.extend_from_slice(&table);
        } else {
            assert!(gd.len() == 512 && groups.len() <= 8);
            let p = 32 + n * 8;
            let r = p + 4;
            put32(&mut gd, group as usize * 4, p as u32);
            put32(&mut rgd, group as usize * 4, r as u32);
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
