use std::sync::Mutex;

use rvvdk_core::{Capabilities, DiskGeometry, Error, Extent, ExtentKind, Result, VirtualDisk};
use rvvdk_datamover::{CopyOptions, DataMover};

struct ExtentDisk {
    data: Mutex<Vec<u8>>,
    geometry: DiskGeometry,
    capabilities: Capabilities,
    extents: Vec<Extent>,
}

impl ExtentDisk {
    fn new(size: usize, capabilities: Capabilities, extents: Vec<Extent>) -> Self {
        Self {
            data: Mutex::new(vec![0_u8; size]),
            geometry: DiskGeometry::new(size as u64, 512, 4096).unwrap(),
            capabilities,
            extents,
        }
    }
}

impl VirtualDisk for ExtentDisk {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let start = usize::try_from(offset).unwrap();

        let end = start + buffer.len();

        let data = self.data.lock().unwrap();

        buffer.copy_from_slice(&data[start..end]);

        Ok(buffer.len())
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }

        let start = usize::try_from(offset).unwrap();

        let end = start + buffer.len();

        let mut data = self.data.lock().unwrap();

        data[start..end].copy_from_slice(buffer);

        Ok(buffer.len())
    }

    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities.contains(Capabilities::WRITE_ZERO) {
            return Err(Error::Unsupported);
        }

        let start = usize::try_from(offset).unwrap();

        let length = usize::try_from(length).unwrap();

        let end = start + length;

        let mut data = self.data.lock().unwrap();

        data[start..end].fill(0);

        Ok(())
    }

    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities.contains(Capabilities::DISCARD) {
            return Err(Error::Unsupported);
        }

        let start = usize::try_from(offset).unwrap();

        let length = usize::try_from(length).unwrap();

        let end = start + length;

        let mut data = self.data.lock().unwrap();

        data[start..end].fill(0);

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        if offset != 0 || length != self.geometry.size() {
            return Err(Error::Unsupported);
        }

        Ok(self.extents.clone())
    }
}

#[test]
fn processes_data_zero_and_hole_extents() {
    let extents = vec![
        Extent::new(0, 1024, ExtentKind::Data).unwrap(),
        Extent::new(1024, 1024, ExtentKind::Zero).unwrap(),
        Extent::new(2048, 1024, ExtentKind::Hole).unwrap(),
        Extent::new(3072, 1024, ExtentKind::Data).unwrap(),
    ];

    let source = ExtentDisk::new(4096, Capabilities::READ, extents);

    {
        let mut data = source.data.lock().unwrap();

        data[0..1024].fill(0xaa);
        data[3072..4096].fill(0xbb);
    }

    let destination = ExtentDisk::new(
        4096,
        Capabilities::READ
            | Capabilities::WRITE
            | Capabilities::FLUSH
            | Capabilities::WRITE_ZERO
            | Capabilities::DISCARD,
        Vec::new(),
    );

    {
        let mut data = destination.data.lock().unwrap();

        data.fill(0xff);
    }

    let mover = DataMover::new(CopyOptions::new(512).unwrap());

    let stats = mover.copy(&source, &destination).unwrap();

    let data = destination.data.lock().unwrap();

    assert!(data[0..1024].iter().all(|value| *value == 0xaa));

    assert!(data[1024..2048].iter().all(|value| *value == 0));

    assert!(data[2048..3072].iter().all(|value| *value == 0));

    assert!(data[3072..4096].iter().all(|value| *value == 0xbb));

    assert_eq!(stats.bytes_read(), 2048,);

    assert_eq!(stats.bytes_written(), 2048,);

    assert_eq!(stats.bytes_zeroed(), 1024,);

    assert_eq!(stats.bytes_discarded(), 1024,);

    assert_eq!(stats.extents_processed(), 4,);
}

#[test]
fn zero_extent_falls_back_to_writes() {
    let extents = vec![Extent::new(0, 4096, ExtentKind::Zero).unwrap()];

    let source = ExtentDisk::new(4096, Capabilities::READ, extents);

    let destination = ExtentDisk::new(
        4096,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
        Vec::new(),
    );

    {
        let mut data = destination.data.lock().unwrap();

        data.fill(0xff);
    }

    let mover = DataMover::new(CopyOptions::new(512).unwrap());

    let stats = mover.copy(&source, &destination).unwrap();

    let data = destination.data.lock().unwrap();

    assert!(data.iter().all(|value| *value == 0));

    assert_eq!(stats.bytes_read(), 0,);

    assert_eq!(stats.bytes_written(), 4096,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_copied(), 8,);

    assert_eq!(stats.extents_processed(), 1,);
}

#[test]
fn hole_extent_uses_write_zero_when_discard_is_unavailable() {
    let extents = vec![Extent::new(0, 4096, ExtentKind::Hole).unwrap()];

    let source = ExtentDisk::new(4096, Capabilities::READ, extents);

    let destination = ExtentDisk::new(
        4096,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH | Capabilities::WRITE_ZERO,
        Vec::new(),
    );

    {
        let mut data = destination.data.lock().unwrap();

        data.fill(0xff);
    }

    let mover = DataMover::new(CopyOptions::new(512).unwrap());

    let stats = mover.copy(&source, &destination).unwrap();

    let data = destination.data.lock().unwrap();

    assert!(data.iter().all(|value| *value == 0));

    assert_eq!(stats.bytes_read(), 0,);

    assert_eq!(stats.bytes_written(), 0,);

    assert_eq!(stats.bytes_zeroed(), 4096,);

    assert_eq!(stats.bytes_discarded(), 0,);
}

#[test]
fn hole_extent_falls_back_to_zero_writes() {
    let extents = vec![Extent::new(0, 4096, ExtentKind::Hole).unwrap()];

    let source = ExtentDisk::new(4096, Capabilities::READ, extents);

    let destination = ExtentDisk::new(
        4096,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
        Vec::new(),
    );

    {
        let mut data = destination.data.lock().unwrap();

        data.fill(0xff);
    }

    let mover = DataMover::new(CopyOptions::new(1024).unwrap());

    let stats = mover.copy(&source, &destination).unwrap();

    let data = destination.data.lock().unwrap();

    assert!(data.iter().all(|value| *value == 0));

    assert_eq!(stats.bytes_read(), 0,);

    assert_eq!(stats.bytes_written(), 4096,);

    assert_eq!(stats.bytes_zeroed(), 0,);

    assert_eq!(stats.bytes_discarded(), 0,);

    assert_eq!(stats.blocks_copied(), 4,);
}

#[test]
fn rejects_extent_map_with_gap() {
    let extents = vec![
        Extent::new(0, 1024, ExtentKind::Data).unwrap(),
        Extent::new(2048, 2048, ExtentKind::Data).unwrap(),
    ];

    let source = ExtentDisk::new(4096, Capabilities::READ, extents);

    let destination = ExtentDisk::new(
        4096,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
        Vec::new(),
    );

    let mover = DataMover::new(CopyOptions::default());

    let result = mover.copy(&source, &destination);

    assert!(matches!(result, Err(Error::CorruptMetadata(_))));
}

#[test]
fn rejects_overlapping_extent_map() {
    let extents = vec![
        Extent::new(0, 2048, ExtentKind::Data).unwrap(),
        Extent::new(1024, 3072, ExtentKind::Zero).unwrap(),
    ];

    let source = ExtentDisk::new(4096, Capabilities::READ, extents);

    let destination = ExtentDisk::new(
        4096,
        Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
        Vec::new(),
    );

    let mover = DataMover::new(CopyOptions::default());

    let result = mover.copy(&source, &destination);

    assert!(matches!(result, Err(Error::CorruptMetadata(_))));
}
