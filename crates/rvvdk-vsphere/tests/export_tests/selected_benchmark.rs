//! Opt-in, synthetic local TLS comparison. Never connects to a real VMware host.
use super::*;
use std::os::unix::fs::MetadataExt;

fn usage() -> (f64, i64) {
    let mut value = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: writable rusage storage, read only after success.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, value.as_mut_ptr()) },
        0
    );
    let value = unsafe { value.assume_init() };
    (
        value.ru_utime.tv_sec as f64
            + value.ru_utime.tv_usec as f64 / 1e6
            + value.ru_stime.tv_sec as f64
            + value.ru_stime.tv_usec as f64 / 1e6,
        value.ru_maxrss,
    )
}

#[tokio::test]
#[ignore = "opt-in synthetic performance matrix; requires a new output file"]
async fn matched_selected_export() {
    let count: usize = std::env::var("RVVDK_SELECTION_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=128).contains(&count));
    let output = std::env::var_os("RVVDK_SELECTION_REPORT").unwrap();
    let nodelay = match std::env::var("RVVDK_SELECTION_NODELAY").as_deref() {
        Ok("1") => true,
        Ok("0") | Err(_) => false,
        _ => panic!("invalid synthetic server TCP_NODELAY option"),
    };
    let mut results = Vec::new();
    for pair in 0..count {
        for explicit in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let replies = if explicit {
                explicit_replies()
            } else {
                let mut replies = selected("poweredOff");
                for reply in &mut replies {
                    reply.body = reply.body.replace("<string>ExportVm</string>", "");
                }
                replies.extend([
                    granted(),
                    ready("ENDPOINT/nfc/PRIVATE-ticket", "THUMBPRINT"),
                    data(),
                    manifest(),
                    complete(),
                    logout(),
                ]);
                replies
            };
            let server = Server::start_with_nodelay(replies, nodelay);
            let out = Output::new();
            let opts = options(&out);
            let credentials = credentials();
            let source = explicit_source(&server);
            let policy = server.policy();
            let before = usage();
            let started = std::time::Instant::now();
            let report = if explicit {
                rvvdk_vsphere::export_selected_vm(source, &credentials, opts).await
            } else {
                export_vm(policy, &credentials, opts).await
            }
            .unwrap();
            let wall_ms = started.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            assert!(report.is_success(), "{report:?}");
            assert_eq!(
                std::fs::read(out.dest().join("disk-1.vmdk")).unwrap(),
                payload().as_bytes()
            );
            let meta = std::fs::metadata(out.dest().join("disk-1.vmdk")).unwrap();
            let requests = server.requests();
            let vm_reads = requests
                .iter()
                .filter(|r| r.contains("<RetrievePropertiesEx ") && r.contains(">vm-private</obj>"))
                .count();
            assert_eq!(vm_reads, if explicit { 5 } else { 2 });
            results.push(serde_json::json!({
                "pair":pair, "explicit":explicit, "wall_ms":wall_ms,
                "cpu_ms":(after.0-before.0)*1000., "process_peak_rss_kib":after.1,
                "vm_reads":vm_reads, "requests":requests.len(),
                "encoded_bytes":meta.len(), "allocated_bytes":meta.blocks()*512,
                "byte_comparison_passed":true,
            }));
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &results).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}
