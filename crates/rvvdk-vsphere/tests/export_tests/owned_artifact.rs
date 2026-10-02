use super::owned_probe::{artifact, record, store};
use super::owned_transfer::{opts, stage};
use super::*;
use rvvdk_vsphere::{
    contract::{ExportArtifact, ValidationClaim},
    ownership::{JobState, OwnershipError},
    transfer_owned_artifact,
};
#[path = "../../../rvvdk-vmdk/tests/support/stream_disk.rs"]
mod fixture;
fn image() -> Vec<u8> {
    fixture::image(
        true,
        (30u64 << 30) / 65536,
        &[
            (0, fixture::stored(&fixture::bytes(0))),
            (7, fixture::stored(&fixture::bytes(7))),
        ],
    )
}
fn replies(data: &[u8]) -> Vec<Reply> {
    let mut r = super::owned_transfer::replies();
    r[12].binary = Some(data.to_vec());
    let digest = Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    r[13] = Reply::soap(
        "HttpNfcLeaseGetManifest",
        &format!(
            "<returnval><key>private-disk</key><disk>true</disk><size>{}</size><capacity>32212254720</capacity><checksumType>sha256</checksumType><checksum>{digest}</checksum></returnval>",
            data.len()
        ),
    );
    r
}
#[tokio::test]
async fn native_metadata_precedes_completion_and_does_not_claim_logical_equivalence() {
    let out = Output::new();
    let store = store(&out);
    let path = out.0.join("jobs");
    let data = image();
    let expected = data.clone();
    let server = Server::start_observed(
        replies(&data),
        true,
        Some(Arc::new(move |request| {
            if request.contains("<HttpNfcLeaseComplete ") {
                assert_eq!(record(&path)["record"]["state"], "complete_intent");
                let metadata = ExportArtifact::from_json(
                    &std::fs::read(stage(&path).join("manifest.json")).unwrap(),
                )
                .unwrap();
                metadata
                    .check_container(expected.len() as u64, &Sha256::digest(&expected).into())
                    .unwrap();
                assert_eq!(
                    metadata.validation_claim(),
                    ValidationClaim::ContainerDigestVerified
                );
            }
        })),
    );
    let source = explicit_source(&server);
    let report = transfer_owned_artifact(source.clone(), &credentials(), store, artifact(), opts())
        .await
        .unwrap();
    assert!(report.is_artifact_success(), "{report:?}");
    assert_eq!(report.present_grains_verified, 2);
    assert_eq!(
        report.lifecycle.recovery.as_ref().unwrap().state,
        JobState::CompletedLease
    );
    let path = stage(&out.0.join("jobs"));
    assert_eq!(std::fs::read(path.join("disk-1.vmdk")).unwrap(), data);
    let metadata =
        ExportArtifact::from_json(&std::fs::read(path.join("manifest.json")).unwrap()).unwrap();
    metadata.check_source(&source).unwrap();
    assert_eq!(metadata.id(), artifact());
    let wire = serde_json::to_string(&report).unwrap();
    assert!(
        !wire.contains("PRIVATE") && !wire.contains("sha256") && !wire.contains("lease-private")
    );
}
#[tokio::test]
async fn valid_manifest_cannot_promote_corrupt_grains_or_wrong_capacity() {
    for kind in ["payload", "capacity", "header", "metadata"] {
        let out = Output::new();
        let store = store(&out);
        let mut data = image();
        match kind {
            "payload" => data[65560] ^= 1,
            "capacity" => data = fixture::image(true, 8, &[]),
            "header" => fixture::put32(&mut data, 4, 99),
            _ => {}
        }
        let mut r = replies(&data);
        r.truncate(14);
        r.extend([abort(), logout()]);
        let path = out.0.join("jobs");
        let server = Server::start_observed(
            r,
            true,
            Some(Arc::new(move |request| {
                if kind == "metadata" && request.contains("<HttpNfcLeaseGetManifest ") {
                    std::fs::write(stage(&path).join("manifest.json"), b"sentinel").unwrap();
                }
                if request.contains("<HttpNfcLeaseAbort ") {
                    assert_eq!(record(&path)["record"]["state"], "abort_intent");
                }
            })),
        );
        let report = transfer_owned_artifact(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            opts(),
        )
        .await
        .unwrap();
        assert!(!report.is_success(), "{kind}: {report:?}");
        assert!(report.manifest_verified && report.container_readback_verified);
        assert!(!report.artifact_metadata_durable);
        assert_eq!(
            report.payload_error,
            Some(if kind == "metadata" {
                OwnershipError::Identity
            } else {
                OwnershipError::Content
            })
        );
        assert_eq!(
            report.lifecycle.recovery.unwrap().state,
            JobState::AbortedLease
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseComplete "))
        );
    }
}
#[tokio::test]
async fn admitted_metadata_does_not_resolve_lost_completion() {
    let out = Output::new();
    let store = store(&out);
    let mut r = replies(&image());
    r[16].declared = Some(r[16].body.len() + 7);
    let server = Server::start_with_nodelay(r, true);
    let report = transfer_owned_artifact(
        explicit_source(&server),
        &credentials(),
        store,
        artifact(),
        opts(),
    )
    .await
    .unwrap();
    assert!(!report.is_artifact_success());
    assert!(report.native_admission_verified && report.artifact_metadata_durable);
    assert_eq!(
        report.lifecycle.recovery.unwrap().state,
        JobState::CompleteIntent
    );
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort "))
    );
}

mod benchmark;
