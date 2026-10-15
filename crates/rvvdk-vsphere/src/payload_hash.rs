//! Digest-equivalent streaming hashing for retained payload verification only.
//!
//! Keep RustCrypto's SHA-NI path where available. The x86-64 software fallback
//! uses ring's runtime dispatch; other architectures retain the existing backend.
//! Selection does not change readback, chunk size, cancellation or persistence.
use sha2::Digest;

enum Backend {
    Sha2(sha2::Sha256),
    #[cfg(target_arch = "x86_64")]
    Ring(ring::digest::Context),
}

pub(crate) struct Sha256(Backend);

#[cfg(target_arch = "x86_64")]
fn has_sha2_acceleration() -> bool {
    std::is_x86_feature_detected!("sha")
        && std::is_x86_feature_detected!("sse2")
        && std::is_x86_feature_detected!("ssse3")
        && std::is_x86_feature_detected!("sse4.1")
}

impl Sha256 {
    pub(crate) fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        if !has_sha2_acceleration() {
            return Self(Backend::Ring(ring::digest::Context::new(
                &ring::digest::SHA256,
            )));
        }
        Self(Backend::Sha2(sha2::Sha256::new()))
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) {
        match &mut self.0 {
            Backend::Sha2(hash) => hash.update(bytes),
            #[cfg(target_arch = "x86_64")]
            Backend::Ring(hash) => hash.update(bytes),
        }
    }

    pub(crate) fn finalize(self) -> [u8; 32] {
        match self.0 {
            Backend::Sha2(hash) => hash.finalize().into(),
            #[cfg(target_arch = "x86_64")]
            Backend::Ring(hash) => hash.finish().as_ref().try_into().expect("SHA-256 length"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contexts() -> Vec<Sha256> {
        let hashes = vec![Sha256::new(), Sha256(Backend::Sha2(sha2::Sha256::new()))];
        #[cfg(target_arch = "x86_64")]
        let hashes = {
            let mut hashes = hashes;
            hashes.push(Sha256(Backend::Ring(ring::digest::Context::new(
                &ring::digest::SHA256,
            ))));
            hashes
        };
        hashes
    }

    #[test]
    fn known_vectors_include_empty_input_and_padding() {
        for (bytes, expected) in [
            (
                &b""[..],
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                &b"abc"[..],
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                &b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"[..],
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
        ] {
            for mut hash in contexts() {
                for byte in bytes.chunks(1) {
                    hash.update(byte);
                }
                assert_eq!(crate::transport::hex_string(&hash.finalize()), expected);
            }
        }
    }

    #[test]
    fn dense_and_zero_streams_match_at_block_and_chunk_boundaries() {
        for size in [
            0,
            1,
            55,
            56,
            63,
            64,
            65,
            119,
            120,
            127,
            128,
            129,
            4095,
            4096,
            4097,
            1 << 20,
        ] {
            for zeros in [false, true] {
                let bytes: Vec<_> = (0..size)
                    .map(|i| if zeros { 0 } else { (i * 17 + i / 256) as u8 })
                    .collect();
                let expected: [u8; 32] = sha2::Sha256::digest(&bytes).into();
                for chunk in [1, 63, 64, 65, 4096, 1 << 20] {
                    for mut hash in contexts() {
                        for part in bytes.chunks(chunk) {
                            hash.update(part);
                        }
                        assert_eq!(hash.finalize(), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn runtime_selection_preserves_the_hardware_backend() {
        let hash = Sha256::new();
        #[cfg(target_arch = "x86_64")]
        assert_eq!(matches!(hash.0, Backend::Sha2(_)), has_sha2_acceleration());
        #[cfg(not(target_arch = "x86_64"))]
        assert!(matches!(hash.0, Backend::Sha2(_)));
    }
}
