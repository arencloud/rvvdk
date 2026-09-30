//! Authored hosted-sparse layouts; generated payload is not physical storage.
use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity, Result};
use rvvdk_vmdk::{BackingError, BackingResolver};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
#[derive(Clone, Copy, Debug)]
pub enum Pattern {
    Zero,
    Contiguous,
    Reversed,
    Alternating,
    Permuted,
}
#[derive(Clone)]
pub struct Layout {
    pub capacity: u64,
    pub grain: u64,
    pub grains: u64,
    pub tables: usize,
    pub directory_bytes: usize,
    pub rgd: usize,
    pub gd: usize,
    pub rgt: usize,
    pub gt: usize,
    pub overhead: u64,
    pub text: String,
}
pub fn put32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
pub fn put64(b: &mut [u8], o: usize, v: u64) {
    b[o..o + 8].copy_from_slice(&v.to_le_bytes());
}
impl Layout {
    pub fn new(capacity: u64, grain: u64) -> Self {
        assert!(
            grain >= 8192
                && grain.is_power_of_two()
                && capacity > 0
                && capacity.is_multiple_of(grain)
        );
        let grains = capacity / grain;
        let tables = grains.div_ceil(512) as usize;
        let directory_bytes = (tables * 4).div_ceil(512) * 512;
        let rgd = 21 * 512;
        let gd = rgd + directory_bytes;
        let rgt = gd + directory_bytes;
        let gt = rgt + tables * 2048;
        let overhead = ((gt + tables * 2048) as u64).div_ceil(grain) * grain;
        let text = format!(
            "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"monolithicSparse\"\nRW {} SPARSE \"disk.vmdk\"\n",
            capacity / 512
        );
        Self {
            capacity,
            grain,
            grains,
            tables,
            directory_bytes,
            rgd,
            gd,
            rgt,
            gt,
            overhead,
            text,
        }
    }
    pub fn header(&self) -> [u8; 512] {
        let mut b = [0; 512];
        put32(&mut b, 0, 0x564d444b);
        put32(&mut b, 4, 1);
        put32(&mut b, 8, 3);
        put64(&mut b, 12, self.capacity / 512);
        put64(&mut b, 20, self.grain / 512);
        put64(&mut b, 28, 1);
        put64(&mut b, 36, 20);
        put32(&mut b, 44, 512);
        put64(&mut b, 48, (self.rgd / 512) as u64);
        put64(&mut b, 56, (self.gd / 512) as u64);
        put64(&mut b, 64, self.overhead / 512);
        b[73..77].copy_from_slice(b"\n \r\n");
        b
    }
    pub fn slot(&self, index: u64, pattern: Pattern) -> Option<u64> {
        match pattern {
            Pattern::Zero => None,
            Pattern::Contiguous => Some(index),
            Pattern::Reversed => Some(self.grains - 1 - index),
            Pattern::Alternating => {
                if index.is_multiple_of(2) {
                    Some(index)
                } else {
                    None
                }
            }
            Pattern::Permuted => {
                assert!(self.grains.is_power_of_two());
                Some((index * 8191 + 17) % self.grains)
            }
        }
    }
    pub fn metadata(&self, pattern: Pattern) -> Vec<u8> {
        let mut b = vec![0; self.overhead as usize];
        b[..512].copy_from_slice(&self.header());
        b[512..512 + self.text.len()].copy_from_slice(self.text.as_bytes());
        for i in 0..self.tables {
            put32(
                &mut b,
                self.rgd + i * 4,
                ((self.rgt + i * 2048) / 512) as u32,
            );
            put32(&mut b, self.gd + i * 4, ((self.gt + i * 2048) / 512) as u32);
        }
        for i in 0..self.grains {
            let v = self.slot(i, pattern).map_or(0, |slot| {
                u32::try_from((self.overhead + slot * self.grain) / 512).unwrap()
            });
            for start in [self.rgt, self.gt] {
                put32(&mut b, start + i as usize * 4, v);
            }
        }
        b
    }
    pub fn expected(&self, offset: u64, len: usize, pattern: Pattern) -> Vec<u8> {
        (offset..offset + len as u64)
            .map(|pos| self.slot(pos / self.grain, pattern).map_or(0, payload_byte))
            .collect()
    }
}
pub fn payload_byte(slot: u64) -> u8 {
    (slot % 251 + 1) as u8
}
pub struct Device {
    pub layout: Layout,
    pub metadata: Vec<u8>,
    pub observed_size: u64,
    pub track: bool,
    pub calls: AtomicU64,
    pub bytes: AtomicU64,
    pub ceiling: u64,
}
impl Device {
    pub fn new(layout: Layout, pattern: Pattern, track: bool) -> Self {
        let observed_size = layout.overhead + layout.capacity;
        let metadata = layout.metadata(pattern);
        Self {
            layout,
            metadata,
            observed_size,
            track,
            calls: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            ceiling: u64::MAX,
        }
    }
    pub fn reset(&self) {
        self.calls.store(0, Ordering::Relaxed);
        self.bytes.store(0, Ordering::Relaxed);
    }
}
impl BlockDevice for Device {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(self.observed_size, 512, 512).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        Ok(CopyEndpoint {
            size: self.observed_size,
            capabilities: self.capabilities(),
            identity: Some(EndpointIdentity::Memory {
                address: self as *const Self as usize,
            }),
        })
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        if self.observed_size < 512 {
            let n = self
                .observed_size
                .saturating_sub(offset)
                .min(buffer.len() as u64) as usize;
            if n > 0 {
                buffer[..n].copy_from_slice(&self.metadata[offset as usize..offset as usize + n]);
            }
            return Ok(n);
        }
        assert!(
            offset <= self.observed_size && buffer.len() as u64 <= self.observed_size - offset,
            "out-of-file request"
        );
        if self.track {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let total =
                self.bytes.fetch_add(buffer.len() as u64, Ordering::Relaxed) + buffer.len() as u64;
            assert!(total <= self.ceiling, "read payload budget escaped");
        }
        let mut pos = offset;
        let mut done = 0;
        while done < buffer.len() {
            if pos < self.layout.overhead {
                assert!(
                    pos < self.metadata.len() as u64,
                    "unexpected metadata read after header-only denial"
                );
                let n =
                    (self.metadata.len() as u64 - pos).min((buffer.len() - done) as u64) as usize;
                buffer[done..done + n]
                    .copy_from_slice(&self.metadata[pos as usize..pos as usize + n]);
                done += n;
                pos += n as u64;
            } else {
                let relative = pos - self.layout.overhead;
                let n = (self.layout.grain - relative % self.layout.grain)
                    .min((buffer.len() - done) as u64) as usize;
                buffer[done..done + n].fill(payload_byte(relative / self.layout.grain));
                done += n;
                pos += n as u64;
            }
        }
        Ok(buffer.len())
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        panic!("source write")
    }
    fn flush(&self) -> Result<()> {
        panic!("source flush")
    }
}
pub struct Resolver(pub Arc<Device>);
impl BackingResolver for Resolver {
    fn resolve(&self, name: &str) -> std::result::Result<Arc<dyn BlockDevice>, BackingError> {
        assert_eq!(name, "disk.vmdk");
        Ok(self.0.clone())
    }
}
