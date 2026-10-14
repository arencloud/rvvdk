//! Offline qualification tool; never linked into rvddk production binaries.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{hint::black_box, time::Instant};

fn digest(backend: &str, bytes: &[u8], chunk: usize) -> [u8; 32] {
    match backend {
        "sha2" => {
            let mut h = Sha256::new();
            for piece in bytes.chunks(chunk) {
                h.update(black_box(piece));
            }
            h.finalize().into()
        }
        "ring" => {
            let mut h = ring::digest::Context::new(&ring::digest::SHA256);
            for piece in bytes.chunks(chunk) {
                h.update(black_box(piece));
            }
            h.finish().as_ref().try_into().unwrap()
        }
        _ => panic!("unknown benchmark backend"),
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
fn data(len: usize, sparse: bool) -> Vec<u8> {
    (0..len)
        .map(|i| {
            if sparse && i % (1 << 20) >= 32768 {
                0
            } else {
                ((i as u64 * 131 + (i as u64 >> 8) * 17 + 29) & 255) as u8
            }
        })
        .collect()
}
fn cpu() -> (f64, i64) {
    let mut r = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid output pointer; read only after successful getrusage.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) },
        0
    );
    // SAFETY: getrusage succeeded.
    let r = unsafe { r.assume_init() };
    (
        r.ru_utime.tv_sec as f64 * 1000.
            + r.ru_utime.tv_usec as f64 / 1000.
            + r.ru_stime.tv_sec as f64 * 1000.
            + r.ru_stime.tv_usec as f64 / 1000.,
        r.ru_maxrss,
    )
}
fn metadata() -> serde_json::Value {
    json!({"build":option_env!("RVDDK_HASH_BUILD").unwrap_or("default"),"sha":std::is_x86_feature_detected!("sha"),"sse2":std::is_x86_feature_detected!("sse2"),"ssse3":std::is_x86_feature_detected!("ssse3"),"sse41":std::is_x86_feature_detected!("sse4.1"),"avx":std::is_x86_feature_detected!("avx"),"avx2":std::is_x86_feature_detected!("avx2")})
}
fn check() {
    let vectors = [
        (
            Vec::new(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".to_vec(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".to_vec(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
        (
            vec![b'a'; 1_000_000],
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
        ),
    ];
    let mut count = 0;
    for (bytes, expected) in vectors {
        for backend in ["sha2", "ring"] {
            for chunk in [1, 55, 64, 65, 4096, 1 << 20] {
                assert_eq!(hex(&digest(backend, &bytes, chunk)), expected);
                count += 1;
            }
        }
    }
    let mut cases = Vec::new();
    for len in [
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
        (1 << 20) - 1,
        1 << 20,
        (1 << 20) + 1,
    ] {
        for sparse in [false, true] {
            let bytes = data(len, sparse);
            let expected = digest("sha2", &bytes, usize::MAX);
            for chunk in [1, 7, 63, 64, 65, 4095, 65536, 1 << 20] {
                for backend in ["sha2", "ring"] {
                    assert_eq!(digest(backend, &bytes, chunk), expected);
                    count += 1;
                }
            }
            cases.push(json!({"bytes":len,"sparse_pattern":sparse,"sha256":hex(&expected)}));
        }
    }
    println!(
        "{}",
        json!({"metadata":metadata(),"checks":count,"cases":cases})
    );
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("metadata") => println!("{}", metadata()),
        Some("check") => check(),
        Some("bench") => {
            assert_eq!(args.len(), 7);
            let size: usize = args[2].parse().unwrap();
            let chunk: usize = args[3].parse().unwrap();
            let iterations: usize = args[4].parse().unwrap();
            let pairs: usize = args[5].parse().unwrap();
            let sparse = args[6] == "sparse";
            assert!(
                (1..=256 << 20).contains(&size)
                    && chunk > 0
                    && (1..=1_000_000).contains(&iterations)
                    && (1..=64).contains(&pairs)
            );
            let bytes = data(size, sparse);
            let expected = digest("sha2", &bytes, chunk);
            assert_eq!(expected, digest("ring", &bytes, chunk));
            let mut rows = Vec::new();
            for pair in 0..pairs {
                for backend in if pair % 2 == 0 {
                    ["sha2", "ring"]
                } else {
                    ["ring", "sha2"]
                } {
                    let before = cpu();
                    let start = Instant::now();
                    for _ in 0..iterations {
                        assert_eq!(
                            black_box(digest(backend, black_box(&bytes), chunk)),
                            expected
                        );
                    }
                    let wall = start.elapsed().as_secs_f64() * 1000.;
                    let after = cpu();
                    rows.push(json!({"pair":pair,"backend":backend,"wall_ms":wall,"cpu_ms":after.0-before.0,"peak_rss_kib":after.1,"bytes_per_digest":size,"iterations":iterations,"chunk":chunk,"sparse_pattern":sparse,"sha256":hex(&expected),"oracle_passed":true}));
                }
            }
            println!("{}", json!({"metadata":metadata(),"rows":rows}));
        }
        _ => panic!("expected metadata, check, or bench SIZE CHUNK ITERATIONS PAIRS dense|sparse"),
    }
}
