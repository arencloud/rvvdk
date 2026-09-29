use rvvdk_core::{Capabilities, Error, Result, VirtualDisk};
use std::time::{Duration, Instant};

/// Confirmed logical byte equality over the complete source length. Destination
/// tail bytes are outside the comparison. This is not a snapshot or flush.
#[derive(Debug, Clone, Copy)]
pub struct VerificationReport {
    pub bytes_verified: u64,
    pub elapsed: Duration,
}

/// Reusable two-buffer verifier. Caller must stabilize contents throughout the
/// comparison; fresh endpoint checks detect observed size/access changes only.
pub struct Verifier {
    length: u64,
    source: Vec<u8>,
    destination: Vec<u8>,
}
impl Verifier {
    pub fn new(length: u64, block_size: usize, memory_budget: usize) -> Result<Self> {
        if block_size == 0 {
            return Err(Error::InvalidAlignment {
                value: 0,
                alignment: 1,
            });
        }
        let size = length.min(block_size as u64) as usize;
        let required = size.checked_mul(2).ok_or(Error::MemoryAccountingOverflow)?;
        let check = |required| {
            if required > memory_budget {
                Err(Error::MemoryBudgetExceeded {
                    phase: "verification",
                    required,
                    budget: memory_budget,
                })
            } else {
                Ok(())
            }
        };
        check(required)?;
        let buffer = || -> Result<Vec<u8>> {
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(size)
                .map_err(|_| Error::Io(std::io::ErrorKind::OutOfMemory.into()))?;
            bytes.resize(size, 0);
            Ok(bytes)
        };
        let result = Self {
            length,
            source: buffer()?,
            destination: buffer()?,
        };
        check(result.storage_bytes())?;
        Ok(result)
    }
    /// Actual Vec payload capacities; excludes allocator/container overhead.
    pub fn storage_bytes(&self) -> usize {
        self.source.capacity() + self.destination.capacity()
    }
    pub fn verify<S: VirtualDisk + ?Sized, D: VirtualDisk + ?Sized>(
        &mut self,
        source: &S,
        destination: &D,
    ) -> Result<VerificationReport> {
        self.verify_controlled(source, destination, &crate::NoCancellation, |_| {})
    }
    /// Compare at block checkpoints. Callbacks report a confirmed matching prefix,
    /// on the caller's thread, and must not panic. Cancellation does not flush.
    pub fn verify_controlled<
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
        C: crate::Cancellation,
    >(
        &mut self,
        source: &S,
        destination: &D,
        cancellation: &C,
        progress: impl Fn(u64),
    ) -> Result<VerificationReport> {
        let check = || {
            if cancellation.is_cancelled() {
                Err(Error::Cancelled)
            } else {
                Ok(())
            }
        };
        check()?;
        let started = Instant::now();
        let inspect = || -> Result<_> {
            let s = source.copy_endpoint()?;
            let d = destination.copy_endpoint()?;
            if !s.capabilities.contains(Capabilities::READ)
                || !d.capabilities.contains(Capabilities::READ)
            {
                return Err(Error::MissingCapability {
                    capability: "READ for verification",
                });
            }
            if s.size != self.length {
                return Err(Error::EndpointChanged(
                    "verification source size changed".into(),
                ));
            }
            if d.size < self.length {
                return Err(Error::OutOfBounds {
                    offset: 0,
                    length: self.length,
                    size: d.size,
                });
            }
            if s.identity.is_some() && s.identity == d.identity {
                return Err(Error::AliasedEndpoints);
            }
            source.validate_destination_identity(d)?;
            Ok((s.identity, d.identity, d.size))
        };
        let before = inspect()?;
        let mut offset = 0;
        while offset < self.length {
            check()?;
            let count = (self.length - offset).min(self.source.len() as u64) as usize;
            source.read_exact_at(offset, &mut self.source[..count])?;
            check()?;
            destination.read_exact_at(offset, &mut self.destination[..count])?;
            if self.source[..count] != self.destination[..count] {
                let index = self.source[..count]
                    .iter()
                    .zip(&self.destination[..count])
                    .position(|(a, b)| a != b)
                    .expect("unequal buffers");
                return Err(Error::VerificationMismatch {
                    offset: offset + index as u64,
                });
            }
            offset += count as u64;
            progress(offset);
            check()?;
        }
        if inspect()? != before {
            return Err(Error::EndpointChanged(
                "verification endpoint changed".into(),
            ));
        }
        check()?;
        Ok(VerificationReport {
            bytes_verified: self.length,
            elapsed: started.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rvvdk_core::{MemoryBlockDevice, RawDisk};
    #[test]
    fn bounded_verification_covers_odd_tail_and_reports_first_mismatch() {
        let source = RawDisk::new(MemoryBlockDevice::new(10003).unwrap());
        let destination = RawDisk::new(MemoryBlockDevice::new(11000).unwrap());
        let bytes = vec![0x5a; 10003];
        source.write_all_at(0, &bytes).unwrap();
        destination.write_all_at(0, &bytes).unwrap();
        let mut verifier = Verifier::new(10003, 512, 1024).unwrap();
        assert_eq!(verifier.storage_bytes(), 1024);
        assert_eq!(
            verifier
                .verify(&source, &destination)
                .unwrap()
                .bytes_verified,
            10003
        );
        destination.write_all_at(10002, &[0]).unwrap();
        assert!(matches!(
            verifier.verify(&source, &destination),
            Err(Error::VerificationMismatch { offset: 10002 })
        ));
    }
    #[test]
    fn budget_empty_and_alias_checks() {
        assert!(matches!(
            Verifier::new(4096, 4096, 8191),
            Err(Error::MemoryBudgetExceeded { .. })
        ));
        let source = RawDisk::new(MemoryBlockDevice::new(0).unwrap());
        let destination = RawDisk::new(MemoryBlockDevice::new(0).unwrap());
        let mut verifier = Verifier::new(0, 4096, 0).unwrap();
        assert_eq!(verifier.storage_bytes(), 0);
        assert_eq!(
            verifier
                .verify(&source, &destination)
                .unwrap()
                .bytes_verified,
            0
        );
        assert!(matches!(
            verifier.verify(&source, &source),
            Err(Error::AliasedEndpoints)
        ));
        assert!(Verifier::new(0, 0, 0).is_err());
    }
}
