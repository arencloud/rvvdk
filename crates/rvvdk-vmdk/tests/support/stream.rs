#![allow(dead_code)]
pub fn put32(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub fn put64(b: &mut [u8], at: usize, value: u64) {
    b[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
pub fn descriptor() -> &'static [u8] {
    b"version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"streamOptimized\"\nRW 8192 SPARSE \"self.vmdk\"\n"
}
pub fn header(footer: bool) -> [u8; 512] {
    let mut b = [0; 512];
    put32(&mut b, 0, 0x564d444b);
    put32(&mut b, 4, 3);
    put32(&mut b, 8, if footer { 0x30001 } else { 0x30003 });
    put64(&mut b, 12, 8192);
    put64(&mut b, 20, 128);
    put64(&mut b, 28, 1);
    put64(&mut b, 36, 20);
    put32(&mut b, 44, 512);
    put64(&mut b, 48, if footer { 0 } else { 21 });
    put64(&mut b, 56, if footer { u64::MAX } else { 26 });
    put64(&mut b, 64, 128);
    b[73..77].copy_from_slice(b"\n \r\n");
    b[77] = 1;
    b
}
pub fn marker(value: u64, size: u32, kind: u32) -> [u8; 16] {
    let mut b = [0; 16];
    put64(&mut b, 0, value);
    put32(&mut b, 8, size);
    put32(&mut b, 12, kind);
    b
}
pub fn image(footer: bool) -> Vec<u8> {
    let mut b = vec![0; if footer { 65536 + 2560 } else { 65536 }];
    b[..512].copy_from_slice(&header(footer));
    b[512..512 + descriptor().len()].copy_from_slice(descriptor());
    if footer {
        b[65536..65552].copy_from_slice(&marker(1, 0, 2));
        b[66560..66576].copy_from_slice(&marker(1, 0, 3));
        let mut h = header(true);
        put64(&mut h, 56, 129);
        b[67072..67584].copy_from_slice(&h);
    }
    b
}
