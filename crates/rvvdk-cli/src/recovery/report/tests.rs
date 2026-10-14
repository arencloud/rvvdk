use super::*;
use rvvdk_vsphere::ownership::{OutputLocation, OutputRecovery, OutputState};
fn success() -> PublicationAcknowledgmentRecoveryReport {
    PublicationAcknowledgmentRecoveryReport {
        logical_bytes_verified: 512,
        content_verified: true,
        bundle_synced: true,
        intent_synced: true,
        directories_synced: true,
        transaction_synced: true,
        transaction_promoted: true,
        acknowledgment_durable: true,
        bindings_rechecked: true,
        primary_error: None,
        assessment_error: None,
        recovery: Some(PublicationRecovery {
            output: OutputRecovery {
                state: OutputState::Published,
                sequence: 8,
                pending_transaction: false,
            },
            staged: OutputLocation::Absent,
            published: OutputLocation::Owned,
        }),
    }
}
#[test]
fn classification_requires_native_success_and_retains_failures() {
    let mut e = Envelope::new(Some(Operation::Acknowledge));
    assert_eq!(complete(success(), &mut e), Status::Completed);
    for flag in 0..8 {
        let mut r = success();
        *match flag {
            0 => &mut r.content_verified,
            1 => &mut r.bundle_synced,
            2 => &mut r.intent_synced,
            3 => &mut r.directories_synced,
            4 => &mut r.transaction_synced,
            5 => &mut r.transaction_promoted,
            6 => &mut r.acknowledgment_durable,
            _ => &mut r.bindings_rechecked,
        } = false;
        assert_eq!(complete(r, &mut e), Status::Uncertain);
        assert!(!e.operation_succeeded);
    }
    for (error, want) in [
        (OwnershipError::Cancelled, Status::Cancelled),
        (OwnershipError::Deadline, Status::Failed),
        (OwnershipError::Io, Status::Failed),
        (OwnershipError::Uncertain, Status::Uncertain),
        (OwnershipError::Identity, Status::Failed),
    ] {
        let mut r = success();
        r.primary_error = Some(error);
        assert_eq!(complete(r, &mut e), want);
        assert!(e.data["acknowledgment_durable"].as_bool().unwrap());
    }
    let mut r = success();
    r.primary_error = Some(OwnershipError::Cancelled);
    r.assessment_error = Some(OwnershipError::Io);
    assert_eq!(complete(r, &mut e), Status::Uncertain);
    let mut r = success();
    r.recovery = None;
    r.primary_error = Some(OwnershipError::Cancelled);
    assert_eq!(complete(r, &mut e), Status::Uncertain);
    for kind in 0..4 {
        let mut r = success();
        let p = r.recovery.as_mut().unwrap();
        match kind {
            0 => p.output.sequence = 7,
            1 => p.output.pending_transaction = true,
            2 => p.output.state = OutputState::PublishIntent,
            _ => p.published = OutputLocation::ForeignOrInvalid,
        };
        assert_eq!(complete(r, &mut e), Status::Uncertain);
    }
    assert_eq!(
        errors([OwnershipError::Cancelled, OwnershipError::Io]),
        Some(Status::Failed)
    );
    assert_eq!(
        errors([OwnershipError::Io, OwnershipError::Uncertain]),
        Some(Status::Uncertain)
    );
}
struct Fails {
    remaining: usize,
    flush: bool,
}
impl Write for Fails {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        }
        let n = b.len().min(self.remaining);
        self.remaining -= n;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.flush {
            Err(std::io::ErrorKind::BrokenPipe.into())
        } else {
            Ok(())
        }
    }
}
#[test]
fn output_failure_retains_completed_report_and_never_returns_success() {
    let mut sample = Envelope::new(Some(Operation::Acknowledge));
    let s = complete(success(), &mut sample);
    let mut out = Vec::new();
    assert_eq!(sample.emit(s, true, &mut out, &mut Vec::new()), 0);
    for limit in 0..out.len() {
        let mut e = Envelope::new(Some(Operation::Acknowledge));
        let s = complete(success(), &mut e);
        let mut err = Vec::new();
        assert_eq!(
            e.emit(
                s,
                true,
                &mut Fails {
                    remaining: limit,
                    flush: false
                },
                &mut err
            ),
            1
        );
        let v: Value = serde_json::from_slice(&err).unwrap();
        assert_eq!(v["operation_succeeded"], true);
        assert_eq!(v["operation_status"], "completed");
        assert_eq!(v["status"], "output_failed");
        assert_eq!(v["data"]["transaction_promoted"], true);
    }
    let mut e = Envelope::new(Some(Operation::Acknowledge));
    let s = complete(success(), &mut e);
    assert_eq!(
        e.emit(
            s,
            false,
            &mut Fails {
                remaining: usize::MAX,
                flush: true
            },
            &mut Fails {
                remaining: 0,
                flush: true
            }
        ),
        1
    );
}
#[test]
fn all_reports_use_explicit_sanitized_field_sets() {
    let r = success();
    let fields = r.data();
    assert_eq!(fields.as_object().unwrap().len(), 12);
    assert!(fields.get("raw_sha256").is_none());
    assert!(fields.get("output_id").is_none());
    assert_eq!(
        PublicationAttestationReport::default()
            .data()
            .as_object()
            .unwrap()
            .len(),
        9
    );
    assert_eq!(
        ReconciliationReport::default()
            .data()
            .as_object()
            .unwrap()
            .len(),
        11
    );
}
