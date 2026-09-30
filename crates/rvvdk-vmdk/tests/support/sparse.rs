// Authored from the published field layout; no producer implementation copied.
pub fn put32(b: &mut [u8], offset: usize, value: u32) {
    b[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
pub fn put64(b: &mut [u8], offset: usize, value: u64) {
    b[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
pub fn header() -> [u8; 512] {
    let mut b = [0; 512];
    put32(&mut b, 0, 0x564d444b);
    put32(&mut b, 4, 1);
    put32(&mut b, 8, 3);
    put64(&mut b, 12, 2048);
    put64(&mut b, 20, 128);
    put64(&mut b, 28, 1);
    put64(&mut b, 36, 20);
    put32(&mut b, 44, 512);
    put64(&mut b, 48, 21);
    put64(&mut b, 56, 26);
    put64(&mut b, 64, 128);
    b[73..77].copy_from_slice(b"\n \r\n");
    b
}
