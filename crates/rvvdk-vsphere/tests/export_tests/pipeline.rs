use super::owned_artifact::{fixture, replies};
use super::owned_probe::{artifact, record, store};
use super::*;
use rvvdk_vsphere::{
    PipelineOptions, PipelinePhase,
    contract::{EndpointIdentity, PinProvenance, SourceSelection},
    ownership::{JobStore, OutputId, OutputState, OwnershipError, PublicationDirectory},
    run_export_pipeline,
};
use std::os::unix::fs::{DirBuilderExt, FileExt};
fn oid() -> OutputId {
    OutputId::new([73; 16]).unwrap()
}
fn destination(t: &Output) -> PublicationDirectory {
    let p = t.0.join("published");
    std::fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
    PublicationDirectory::open(&p).unwrap()
}
fn data() -> Vec<u8> {
    fixture::image(
        true,
        128,
        &[
            (0, fixture::stored(&fixture::bytes(0))),
            (7, fixture::stored(&fixture::bytes(7))),
        ],
    )
}
fn responses() -> Vec<Reply> {
    replies(&data())
        .into_iter()
        .map(|mut r| {
            r.body = r
                .body
                .replace("32212254720", "8388608")
                .replace("31457280", "8192");
            r
        })
        .collect()
}
fn source(s: &Server) -> SourceSelection {
    SourceSelection::new(
        EndpointIdentity::pinned(&s.endpoint, &s.pin, PinProvenance::TrustOnFirstUse).unwrap(),
        "vm-private",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "PRIVATE-path",
        8 << 20,
    )
    .unwrap()
}
fn verify(path: &std::path::Path) {
    let f = std::fs::File::open(path).unwrap();
    let mut b = vec![0; 65536];
    for i in 0..128 {
        f.read_exact_at(&mut b, i * 65536).unwrap();
        if i == 0 || i == 7 {
            assert_eq!(b, fixture::bytes(i));
        } else {
            assert!(b.iter().all(|&x| x == 0));
        }
    }
}
#[tokio::test]
async fn pipeline_publishes_then_separate_explicit_cleanup_preserves_tombstones() {
    let t = Output::new();
    let store = store(&t);
    let dest = destination(&t);
    let server = Server::start_with_nodelay(responses(), true);
    let s = source(&server);
    let report = run_export_pipeline(
        s.clone(),
        &credentials(),
        store,
        dest,
        artifact(),
        oid(),
        PipelineOptions::default(),
    )
    .await;
    assert!(report.is_success(), "{report:?}");
    verify(
        &t.0.join("published")
            .join(oid().bundle_name())
            .join("disk.raw"),
    );
    assert_eq!(
        record(&t.0.join("jobs"))["record"]["state"],
        "completed_lease"
    );
    let wire = serde_json::to_string(&report).unwrap();
    assert!(
        !wire.contains("PRIVATE") && !wire.contains("sha256") && !wire.contains("lease-private")
    );
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort ") || r.contains("<ShutdownGuest "))
    );
    let mut store = JobStore::open(&t.0.join("jobs")).unwrap();
    let dest = PublicationDirectory::open(&t.0.join("published")).unwrap();
    store
        .cleanup_output(oid(), artifact(), &s, Some(&dest), Default::default())
        .unwrap();
    let r = store.recover(artifact(), &s).unwrap();
    store.cleanup_local(artifact(), r.operation, &s).unwrap();
    assert_eq!(std::fs::read_dir(t.0.join("jobs")).unwrap().count(), 2);
}
#[tokio::test]
async fn pipeline_preflight_collision_and_precancel_do_not_contact_host() {
    for cancel in [false, true] {
        let t = Output::new();
        let store = store(&t);
        let dest = destination(&t);
        let server = Server::start(vec![]);
        let mut options = PipelineOptions::default();
        if cancel {
            options.transfer.cancellation.cancel();
        } else {
            std::fs::write(t.0.join("published").join(oid().bundle_name()), b"foreign").unwrap();
        }
        let r = run_export_pipeline(
            source(&server),
            &credentials(),
            store,
            dest,
            artifact(),
            oid(),
            options.clone(),
        )
        .await;
        assert!(!r.is_success());
        assert_eq!(r.phase, PipelinePhase::Preflight);
        assert!(server.requests().is_empty());
        assert_eq!(std::fs::read_dir(t.0.join("jobs")).unwrap().count(), 0);
        options.local_timeout = Duration::ZERO;
        let r = run_export_pipeline(
            source(&server),
            &credentials(),
            JobStore::open(&t.0.join("jobs")).unwrap(),
            PublicationDirectory::open(&t.0.join("published")).unwrap(),
            artifact(),
            oid(),
            options,
        )
        .await;
        assert_eq!(r.remote_error, Some(Error::InvalidInput));
        assert!(server.requests().is_empty());
    }
}
#[tokio::test]
async fn pipeline_stops_after_uncertain_completion_without_abort_or_local_copy() {
    let t = Output::new();
    let mut r = responses();
    r[16] = Reply::fault("RuntimeFault");
    let server = Server::start_with_nodelay(r, true);
    let report = run_export_pipeline(
        source(&server),
        &credentials(),
        store(&t),
        destination(&t),
        artifact(),
        oid(),
        PipelineOptions::default(),
    )
    .await;
    assert!(!report.is_success());
    assert_eq!(report.phase, PipelinePhase::Export);
    assert!(report.conversion.is_none());
    assert!(report.publication.is_none());
    assert_eq!(
        record(&t.0.join("jobs"))["record"]["state"],
        "complete_intent"
    );
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort "))
    );
    assert_eq!(std::fs::read_dir(t.0.join("published")).unwrap().count(), 0);
}
#[tokio::test]
async fn pipeline_local_failures_retain_completed_source_and_do_not_delete_foreign_output() {
    for kind in [
        "budget",
        "cancel_after_export",
        "collision_after_export",
        "local_deadline",
    ] {
        let t = Output::new();
        let store = store(&t);
        let dest = destination(&t);
        let mut options = PipelineOptions::default();
        if kind == "budget" {
            options.copy = options.copy.with_memory_budget(0);
        }
        if kind == "local_deadline" {
            options.local_timeout = Duration::from_nanos(1);
        }
        let token = options.transfer.cancellation.clone();
        let path = t.0.join("published").join(oid().bundle_name());
        let p = path.clone();
        let server = Server::start_observed(
            responses(),
            true,
            Some(Arc::new(move |request| {
                if request.contains("<Logout ") {
                    if kind == "cancel_after_export" {
                        token.cancel();
                    }
                    if kind == "collision_after_export" {
                        std::fs::write(&p, b"foreign").unwrap();
                    }
                }
            })),
        );
        let s = source(&server);
        let r = run_export_pipeline(
            s.clone(),
            &credentials(),
            store,
            dest,
            artifact(),
            oid(),
            options,
        )
        .await;
        assert!(!r.is_success(), "{kind}");
        assert_eq!(
            record(&t.0.join("jobs"))["record"]["state"],
            "completed_lease"
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|v| v.contains("<HttpNfcLeaseAbort "))
        );
        if kind == "collision_after_export" {
            assert_eq!(r.phase, PipelinePhase::Publication);
            assert_eq!(
                r.publication.unwrap().primary_error,
                Some(OwnershipError::Exists)
            );
            assert_eq!(std::fs::read(path).unwrap(), b"foreign");
            assert_eq!(
                JobStore::open(&t.0.join("jobs"))
                    .unwrap()
                    .assess_output(oid(), artifact(), &s)
                    .unwrap()
                    .state,
                OutputState::Verified
            );
        } else {
            assert_eq!(r.phase, PipelinePhase::Conversion);
            assert!(r.publication.is_none());
        }
    }
}
#[tokio::test]
async fn pipeline_store_anchor_never_adopts_a_replacement_path_between_phases() {
    let t = Output::new();
    let store = store(&t);
    let dest = destination(&t);
    let root = t.0.clone();
    let server = Server::start_observed(
        responses(),
        true,
        Some(Arc::new(move |r| {
            if r.contains("<Logout ") {
                std::fs::rename(root.join("jobs"), root.join("original-jobs")).unwrap();
                std::fs::DirBuilder::new()
                    .mode(0o700)
                    .create(root.join("jobs"))
                    .unwrap();
                std::fs::write(root.join("jobs/foreign"), b"keep").unwrap();
            }
        })),
    );
    let report = run_export_pipeline(
        source(&server),
        &credentials(),
        store,
        dest,
        artifact(),
        oid(),
        PipelineOptions::default(),
    )
    .await;
    assert!(report.is_success(), "{report:?}");
    assert_eq!(std::fs::read_dir(t.0.join("jobs")).unwrap().count(), 1);
    assert_eq!(std::fs::read(t.0.join("jobs/foreign")).unwrap(), b"keep");
    assert_eq!(
        record(&t.0.join("original-jobs"))["record"]["state"],
        "completed_lease"
    );
    verify(
        &t.0.join("published")
            .join(oid().bundle_name())
            .join("disk.raw"),
    );
}
